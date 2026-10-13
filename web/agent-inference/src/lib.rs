use std::io::Cursor;

use candle_core::{Device, Tensor, quantized::gguf_file};
use candle_transformers::{generation::LogitsProcessor, models::quantized_llama::ModelWeights};
use tokenizers::Tokenizer;
use wasm_bindgen::prelude::*;

#[path = "../../../apps/beverly-agent/src/conversation.rs"]
mod conversation;

#[wasm_bindgen]
pub struct Model {
    weights: ModelWeights,
    tokenizer: Tokenizer,
    sampler: LogitsProcessor,
    prompt: Vec<u32>,
    generated: Vec<u32>,
    position: usize,
    output: String,
    finished: bool,
    history: Vec<(String, String)>,
    pending: String,
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[wasm_bindgen]
impl Model {
    #[wasm_bindgen(constructor)]
    pub fn new(weights: &[u8], tokenizer: &[u8]) -> Result<Model, JsValue> {
        let mut reader = Cursor::new(weights);
        let content = gguf_file::Content::read(&mut reader).map_err(js_error)?;
        let metadata = &content.metadata;
        let architecture = metadata
            .get("general.architecture")
            .and_then(|value| value.to_string().ok());
        if architecture.map(String::as_str) != Some("llama")
            || metadata
                .get("llama.block_count")
                .and_then(|value| value.to_u32().ok())
                != Some(22)
            || metadata
                .get("llama.embedding_length")
                .and_then(|value| value.to_u32().ok())
                != Some(2048)
        {
            return Err(js_error(
                "The downloaded file is not the intended TinyLlama model.",
            ));
        }
        let weights =
            ModelWeights::from_gguf(content, &mut reader, &Device::Cpu).map_err(js_error)?;
        let tokenizer = Tokenizer::from_bytes(tokenizer).map_err(js_error)?;
        Ok(Self {
            weights,
            tokenizer,
            sampler: LogitsProcessor::new(299792458, Some(0.7), Some(0.9)),
            prompt: Vec::new(),
            generated: Vec::new(),
            position: 0,
            output: String::new(),
            finished: true,
            history: Vec::new(),
            pending: String::new(),
        })
    }

    pub fn start(&mut self, prompt: &str) -> Result<usize, JsValue> {
        let formatted_prompt = conversation::tinyllama_prompt(&self.history, prompt);
        self.prompt = self
            .tokenizer
            .encode(formatted_prompt, true)
            .map_err(js_error)?
            .get_ids()
            .to_vec();
        if self.prompt.is_empty() || self.prompt.len() > 1792 {
            return Err(js_error(
                "Conversation exceeds the 2048-token context. Start a new conversation.",
            ));
        }
        self.generated.clear();
        self.output.clear();
        self.position = 0;
        self.finished = false;
        self.pending = prompt.to_owned();
        self.sampler = LogitsProcessor::new(299792458, Some(0.7), Some(0.9));
        Ok(self.prompt.len())
    }

    pub fn finish_turn(&mut self) {
        if !self.pending.is_empty() {
            self.history
                .push((std::mem::take(&mut self.pending), self.output.clone()));
        }
        self.finished = true;
    }

    pub fn reset(&mut self) {
        self.history.clear();
        self.pending.clear();
        self.finished = true;
    }

    pub fn step(&mut self) -> Result<String, JsValue> {
        if self.finished {
            return Ok(serde_json::json!({"done":true,"text":self.output}).to_string());
        }
        let prompt_len = self.prompt.len();
        let input = if self.position < prompt_len {
            self.prompt[self.position..(self.position + 32).min(prompt_len)].to_vec()
        } else {
            vec![
                *self
                    .generated
                    .last()
                    .ok_or_else(|| js_error("Missing generation token"))?,
            ]
        };
        let tensor = Tensor::new(input.as_slice(), &Device::Cpu)
            .and_then(|tensor| tensor.unsqueeze(0))
            .map_err(js_error)?;
        let logits = self
            .weights
            .forward(&tensor, self.position)
            .map_err(js_error)?;
        self.position += input.len();
        if self.position < prompt_len {
            return Ok(serde_json::json!({"prefill":self.position,"total":prompt_len}).to_string());
        }
        let token = self
            .sampler
            .sample(&logits.squeeze(0).map_err(js_error)?)
            .map_err(js_error)?;
        if token == self.tokenizer.token_to_id("</s>").unwrap_or(2) {
            self.finished = true;
        } else {
            self.generated.push(token);
            let decoded = self
                .tokenizer
                .decode(&self.generated, true)
                .map_err(js_error)?;
            if let Some(marker) = ["<|user|>", "<|system|>"]
                .iter()
                .filter_map(|marker| decoded.find(marker))
                .min()
            {
                self.output = decoded[..marker].to_owned();
                self.finished = true;
            } else if !decoded.ends_with('\u{fffd}') {
                self.output = decoded;
            }
            self.finished |= self.generated.len() >= 256;
        }
        Ok(serde_json::json!({"done":self.finished,"text":self.output,"tokens":self.generated.len()}).to_string())
    }
}
