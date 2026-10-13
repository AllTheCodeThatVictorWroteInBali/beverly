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

    pub(crate) fn repo_id(self) -> &'static str {
        match self {
            Self::TinyLlama => "TinyLlama/TinyLlama-1.1B-Chat-v1.0",
            Self::SmolLm2_360M => "HuggingFaceTB/SmolLM2-360M-Instruct",
            Self::SmolLm2_1_7B => "HuggingFaceTB/SmolLM2-1.7B-Instruct",
        }
    }

    pub(crate) fn eos_token(self) -> &'static str {
        match self {
            Self::TinyLlama => "</s>",
            Self::SmolLm2_360M | Self::SmolLm2_1_7B => "<|im_end|>",
        }
    }

    pub(crate) fn format_prompt(self, prompt: &str) -> String {
        match self {
            Self::TinyLlama => tinyllama_prompt(&[], prompt),
            Self::SmolLm2_360M | Self::SmolLm2_1_7B => format!(
                "<|im_start|>system\nYou are a helpful AI assistant named SmolLM, trained by Hugging Face<|im_end|>\n<|im_start|>user\n{prompt}<|im_end|>\n<|im_start|>assistant\n"
            ),
        }
    }

    pub(crate) fn starts_fabricated_turn(self, generated: &str) -> bool {
        match self {
            Self::TinyLlama => generated.contains("<|user|>") || generated.contains("<|system|>"),
            Self::SmolLm2_360M | Self::SmolLm2_1_7B => {
                generated.contains("<|im_start|>user") || generated.contains("<|im_start|>system")
            }
        }
    }
}

pub fn tinyllama_prompt(history: &[(String, String)], prompt: &str) -> String {
    let mut result = "<|system|>\nYou are a helpful assistant.</s>\n".to_owned();
    for (user, assistant) in history {
        result.push_str(&format!(
            "<|user|>\n{user}</s>\n<|assistant|>\n{assistant}</s>\n"
        ));
    }
    result.push_str(&format!("<|user|>\n{prompt}</s>\n<|assistant|>\n"));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_tinyllama_prompt_is_unchanged() {
        assert_eq!(
            ChatModel::TinyLlama.format_prompt("Hello"),
            "<|system|>\nYou are a helpful assistant.</s>\n<|user|>\nHello</s>\n<|assistant|>\n"
        );
    }

    #[test]
    fn multi_turn_history_uses_expected_roles_and_stops() {
        let prompt = tinyllama_prompt(
            &[("My name is Ada".into(), "Hello Ada".into())],
            "What is my name?",
        );
        assert!(prompt.contains("<|user|>\nMy name is Ada</s>\n<|assistant|>\nHello Ada</s>\n<|user|>\nWhat is my name?</s>\n<|assistant|>\n"));
        assert_eq!(prompt.matches("<|system|>").count(), 1);
    }
}
