use anyhow::{Error as E, Result};
use std::collections::HashSet;
use std::io::Write;

use candle_core::Tensor;
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::llama::{
    Cache as LlamaCache, Config, Llama, LlamaConfig, LlamaEosToks,
};
use hf_hub::Cache as HfCache;
use hf_hub::api::Progress;
use hf_hub::api::sync::{Api, ApiBuilder, ApiRepo};
use std::time::Instant;
use tokenizers::Tokenizer;

#[derive(Clone, Copy, Debug, Default)]
pub enum ChatModel {
    #[default]
    TinyLlama,
    SmolLm2_360M,
    SmolLm2_1_7B,
}

impl ChatModel {
    pub fn name(self) -> &'static str {
        match self {
            Self::TinyLlama => "TinyLlama 1.1B Chat",
            Self::SmolLm2_360M => "SmolLM2 360M Instruct",
            Self::SmolLm2_1_7B => "SmolLM2 1.7B Instruct",
        }
    }

    fn repo_id(self) -> &'static str {
        match self {
            Self::TinyLlama => "TinyLlama/TinyLlama-1.1B-Chat-v1.0",
            Self::SmolLm2_360M => "HuggingFaceTB/SmolLM2-360M-Instruct",
            Self::SmolLm2_1_7B => "HuggingFaceTB/SmolLM2-1.7B-Instruct",
        }
    }

    fn eos_token(self) -> &'static str {
        match self {
            Self::TinyLlama => "</s>",
            Self::SmolLm2_360M | Self::SmolLm2_1_7B => "<|im_end|>",
        }
    }

    fn format_prompt(self, prompt: &str) -> String {
        match self {
            Self::TinyLlama => format!(
                "<|system|>\nYou are a helpful assistant.</s>\n<|user|>\n{prompt}</s>\n<|assistant|>\n"
            ),
            Self::SmolLm2_360M | Self::SmolLm2_1_7B => format!(
                "<|im_start|>system\nYou are a helpful AI assistant named SmolLM, trained by Hugging Face<|im_end|>\n<|im_start|>user\n{prompt}<|im_end|>\n<|im_start|>assistant\n"
            ),
        }
    }

    fn starts_fabricated_turn(self, generated: &str) -> bool {
        match self {
            Self::TinyLlama => generated.contains("<|user|>") || generated.contains("<|system|>"),
            Self::SmolLm2_360M | Self::SmolLm2_1_7B => {
                generated.contains("<|im_start|>user") || generated.contains("<|im_start|>system")
            }
        }
    }
}

/// Local Llama runtime: holds the loaded weights, tokenizer, and KV cache so a
/// single instance can be reused across multiple `generate_streaming` calls.
pub struct LocalLlama {
    llama: Llama,
    cache: LlamaCache,
    tokenizer: Tokenizer,
    config: Config,
    device: candle_core::Device,
    dtype: candle_core::DType,
    model: ChatModel,
}

impl LocalLlama {
    /// Downloads (if needed) and loads the model weights, tokenizer, and config.
    #[allow(dead_code)]
    pub fn init(cpu: bool) -> Result<Self> {
        Self::init_with_model(cpu, ChatModel::default(), |_, _| {})
    }

    pub fn init_with_model(
        cpu: bool,
        model: ChatModel,
        on_load_progress: impl FnMut(String, f32),
    ) -> Result<Self> {
        let device = candle_examples::device(cpu)?;
        let dtype = candle_core::DType::F16;

        let api = ApiBuilder::from_env().with_progress(false).build()?;
        let repo = api.model(model.repo_id().to_string());

        let tokenizer_filename = cached_or_get(&repo, model, "tokenizer.json")?;
        let config_filename = cached_or_get(&repo, model, "config.json")?;

        let config: LlamaConfig = serde_json::from_slice(&std::fs::read(config_filename)?)?;
        let config = config.into_config(false);

        let filenames = hub_load_safetensors(&api, &repo, model, on_load_progress)?;
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&filenames, dtype, &device)? };

        let llama = Llama::load(vb, &config)?;
        let cache = LlamaCache::new(true, dtype, &config, &device)?;
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

        Ok(Self {
            llama,
            cache,
            tokenizer,
            config,
            device,
            dtype,
            model,
        })
    }

    /// Generates text from `prompt`, invoking `on_token` as each decoded token
    /// becomes available so a caller can stream output (e.g. over a channel).
    #[allow(dead_code)]
    pub fn generate_streaming(
        &mut self,
        prompt: &str,
        sample_len: usize,
        temperature: f64,
        on_token: impl FnMut(String),
        on_generated_token: impl FnMut(),
    ) -> Result<()> {
        self.generate_streaming_until(
            prompt,
            sample_len,
            temperature,
            on_token,
            on_generated_token,
            || false,
        )
    }

    /// Like `generate_streaming`, but ends generation early once `should_stop`
    /// returns true (checked before each token).
    pub fn generate_streaming_until(
        &mut self,
        prompt: &str,
        sample_len: usize,
        temperature: f64,
        mut on_token: impl FnMut(String),
        mut on_generated_token: impl FnMut(),
        mut should_stop: impl FnMut() -> bool,
    ) -> Result<()> {
        // Each prompt is treated as a fresh turn: reset the KV cache so the new
        // prompt's tokens don't collide with stale cache entries from the last
        // generation (that mismatch previously caused tensor broadcast errors).
        self.cache = LlamaCache::new(true, self.dtype, &self.config, &self.device)?;

        let eos_token_id = self.config.eos_token_id.clone().or_else(|| {
            self.tokenizer
                .token_to_id(self.model.eos_token())
                .map(LlamaEosToks::Single)
        });

        let formatted_prompt = self.model.format_prompt(prompt);

        let add_bos_token = matches!(self.model, ChatModel::TinyLlama);
        let mut tokens = self
            .tokenizer
            .encode(formatted_prompt, add_bos_token)
            .map_err(E::msg)?
            .get_ids()
            .to_vec();
        let mut token_streamer =
            candle_examples::token_output_stream::TokenOutputStream::new(self.tokenizer.clone());

        let mut logits_processor = LogitsProcessor::new(299792458, Some(temperature), Some(0.9));
        let mut index_pos = 0;
        let mut generated_text = String::new();
        let mut stopped_on_turn_marker = false;

        for index in 0..sample_len {
            if should_stop() {
                stopped_on_turn_marker = true;
                break;
            }

            let (context_size, context_index) = if self.cache.use_kv_cache && index > 0 {
                (1, index_pos)
            } else {
                (tokens.len(), 0)
            };

            let ctxt = &tokens[tokens.len().saturating_sub(context_size)..];
            let input = Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;
            let logits = self.llama.forward(&input, context_index, &mut self.cache)?;
            let logits = logits.squeeze(0)?;

            index_pos += ctxt.len();

            let next_token = logits_processor.sample(&logits)?;
            tokens.push(next_token);

            match eos_token_id {
                Some(LlamaEosToks::Single(eos_id)) if next_token == eos_id => break,
                Some(LlamaEosToks::Multiple(ref eos_ids)) if eos_ids.contains(&next_token) => break,
                _ => (),
            }

            on_generated_token();

            if let Some(t) = token_streamer.next_token(next_token)? {
                generated_text.push_str(&t);
                if self.model.starts_fabricated_turn(&generated_text) {
                    stopped_on_turn_marker = true;
                    break;
                }
                on_token(t);
                std::io::stdout().flush().ok();
            }
        }

        if !stopped_on_turn_marker {
            if let Some(rest) = token_streamer.decode_rest().map_err(E::msg)? {
                on_token(rest);
            }
        }

        Ok(())
    }
}

/// Downloads either a single `model.safetensors` file, or (for sharded repos)
/// every shard referenced by `model.safetensors.index.json`'s weight map,
/// mirroring `candle_examples::hub_load_safetensors` from older candle-examples
/// releases (removed upstream in 0.11).
fn hub_load_safetensors(
    api: &Api,
    repo: &ApiRepo,
    model: ChatModel,
    mut on_progress: impl FnMut(String, f32),
) -> Result<Vec<std::path::PathBuf>> {
    on_progress("Checking model files".to_string(), 0.0);

    let cache = HfCache::from_env().model(model.repo_id().to_string());
    let cached_size = |filename: &str| {
        cache
            .get(filename)
            .and_then(|path| std::fs::metadata(path).ok())
            .map(|m| m.len() as usize)
    };
    let file_size = |filename: &str| -> Result<usize> {
        match cached_size(filename) {
            Some(size) => Ok(size),
            None => Ok(api.metadata(&repo.url(filename))?.size()),
        }
    };

    let single_size = match cached_size("model.safetensors") {
        Some(size) => Some(size),
        // Offline-safe: a cached shard index means the repo is sharded, so skip the network probe.
        None if cache.get("model.safetensors.index.json").is_some() => None,
        None => api
            .metadata(&repo.url("model.safetensors"))
            .ok()
            .map(|m| m.size()),
    };

    let (files, total_bytes): (Vec<(String, usize)>, usize) = match single_size {
        Some(size) => (vec![("model.safetensors".to_string(), size)], size),
        None => {
            let index_file = cached_or_get(repo, model, "model.safetensors.index.json")?;
            let index: serde_json::Value = serde_json::from_slice(&std::fs::read(index_file)?)?;
            let weight_map = index
                .get("weight_map")
                .ok_or_else(|| E::msg("no weight_map in safetensors index file"))?
                .as_object()
                .ok_or_else(|| E::msg("weight_map in safetensors index file is not a map"))?;

            let mut safetensors_files = HashSet::new();
            for value in weight_map.values() {
                if let Some(file) = value.as_str() {
                    safetensors_files.insert(file.to_string());
                }
            }

            let files = safetensors_files
                .into_iter()
                .map(|filename| {
                    let size = file_size(&filename)?;
                    Ok((filename, size))
                })
                .collect::<Result<Vec<_>>>()?;
            let total_bytes = index
                .get("metadata")
                .and_then(|metadata| metadata.get("total_size"))
                .and_then(serde_json::Value::as_u64)
                .map(|size| size as usize)
                .unwrap_or_else(|| files.iter().map(|(_, size)| size).sum());
            (files, total_bytes)
        }
    };

    let mut completed_bytes = 0usize;
    let mut downloaded_any = false;
    let mut paths = Vec::with_capacity(files.len());

    for (filename, file_size) in files {
        let (path, downloaded) = if let Some(path) = cache.get(&filename) {
            (path, false)
        } else {
            let progress = ModelDownloadProgress::with_callback(
                filename.clone(),
                completed_bytes,
                total_bytes,
                &mut on_progress,
            );
            (repo.download_with_progress(&filename, progress)?, true)
        };
        completed_bytes = completed_bytes.saturating_add(file_size);
        if downloaded {
            downloaded_any = true;
            on_progress(
                "Downloading model weights".to_string(),
                progress_percent(completed_bytes, total_bytes),
            );
        }
        paths.push(path);
    }

    // Distinct stages let a UI tell a fresh download from files already on disk.
    if downloaded_any {
        on_progress("Download complete".to_string(), 1.0);
    } else {
        on_progress("Model files present".to_string(), 1.0);
    }
    on_progress("Initializing model".to_string(), 0.0);
    Ok(paths)
}

/// Returns the locally cached file if present, otherwise downloads it.
fn cached_or_get(repo: &ApiRepo, model: ChatModel, filename: &str) -> Result<std::path::PathBuf> {
    match HfCache::from_env()
        .model(model.repo_id().to_string())
        .get(filename)
    {
        Some(path) => Ok(path),
        None => Ok(repo.get(filename)?),
    }
}

struct ModelDownloadProgress<'a> {
    callback: Option<&'a mut dyn FnMut(String, f32)>,
    filename: String,
    base_bytes: usize,
    total_bytes: usize,
    current_bytes: usize,
    last_report: Instant,
}

impl<'a> ModelDownloadProgress<'a> {
    fn with_callback(
        filename: String,
        base_bytes: usize,
        total_bytes: usize,
        callback: &'a mut dyn FnMut(String, f32),
    ) -> Self {
        Self {
            callback: Some(callback),
            filename,
            base_bytes,
            total_bytes,
            current_bytes: 0,
            last_report: Instant::now(),
        }
    }

    fn report(&mut self, force: bool) {
        if !force && self.last_report.elapsed().as_millis() < 100 {
            return;
        }
        self.last_report = Instant::now();
        let message = format!("Downloading {}", self.filename);
        let progress = progress_percent(
            self.base_bytes.saturating_add(self.current_bytes),
            self.total_bytes,
        );
        if let Some(callback) = self.callback.as_deref_mut() {
            callback(message, progress);
        }
    }
}

impl Progress for ModelDownloadProgress<'_> {
    fn init(&mut self, size: usize, filename: &str) {
        self.filename = filename.to_string();
        self.current_bytes = 0;
        if self.total_bytes == 0 {
            self.total_bytes = size;
        }
        self.report(true);
    }

    fn update(&mut self, size: usize) {
        self.current_bytes = self.current_bytes.saturating_add(size);
        self.report(false);
    }

    fn finish(&mut self) {
        self.report(true);
    }
}

fn progress_percent(completed: usize, total: usize) -> f32 {
    if total == 0 {
        return 0.0;
    }
    (completed as f64 / total as f64).clamp(0.0, 1.0) as f32
}
