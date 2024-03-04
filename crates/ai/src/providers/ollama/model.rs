use crate::models::AiModel;
use crate::models::{LanguageModel, TruncationDirection};
use anyhow::anyhow;
use lazy_static::lazy_static;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tiktoken_rs::{cl100k_base, CoreBPE};

lazy_static! {
    pub(crate) static ref OLLAMA_BPE_TOKENIZER: CoreBPE = cl100k_base().unwrap();
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum OllamaModel {
    #[serde(rename = "codellama:7b")]
    CodeLlamaSevenBillion,
    #[serde(rename = "codellama:13b")]
    CodeLlamaThirteenBillion,
}

impl AiModel for OllamaModel {
    fn full_name(&self) -> &'static str {
        match self {
            OllamaModel::CodeLlamaSevenBillion => "codellama:7b",
            OllamaModel::CodeLlamaThirteenBillion => "codellama:13b",
        }
    }

    fn short_name(&self) -> &'static str {
        match self {
            OllamaModel::CodeLlamaSevenBillion => "codellama-7",
            OllamaModel::CodeLlamaThirteenBillion => "codellama-13",
        }
    }

    fn cycle(&self) -> Self {
        match self {
            OllamaModel::CodeLlamaSevenBillion => OllamaModel::CodeLlamaThirteenBillion,
            OllamaModel::CodeLlamaThirteenBillion => OllamaModel::CodeLlamaSevenBillion,
        }
    }
}

#[derive(Clone)]
pub struct OllamaLanguageModel {
    name: String,
    bpe: Option<CoreBPE>,
}

impl OllamaLanguageModel {
    pub fn load(model_name: &str) -> Self {
        // Using OpenAi tokenizer with reduced capacity for now
        // The ollama tokenizer is written in python
        let bpe = tiktoken_rs::get_bpe_from_model("gpt-3.5-turbo-0613")
            .unwrap_or(OLLAMA_BPE_TOKENIZER.to_owned());
        OllamaLanguageModel {
            name: model_name.to_string(),
            bpe: Some(bpe),
        }
    }
}

impl LanguageModel for OllamaLanguageModel {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn count_tokens(&self, content: &str) -> anyhow::Result<usize> {
        if let Some(bpe) = &self.bpe {
            anyhow::Ok(bpe.encode_with_special_tokens(content).len())
        } else {
            Err(anyhow!("BPE tokenizer for Ollama model was not retrieved"))
        }
    }

    fn truncate(
        &self,
        content: &str,
        length: usize,
        direction: TruncationDirection,
    ) -> anyhow::Result<String> {
        if let Some(bpe) = &self.bpe {
            let tokens = bpe.encode_with_special_tokens(content);
            if tokens.len() > length {
                match direction {
                    TruncationDirection::End => bpe.decode(tokens[..length].to_vec()),
                    TruncationDirection::Start => bpe.decode(tokens[length..].to_vec()),
                }
            } else {
                bpe.decode(tokens)
            }
        } else {
            Err(anyhow!("BPE tokenizer for Ollama model was not retrieved"))
        }
    }

    fn capacity(&self) -> anyhow::Result<usize> {
        // Assuming the actual limit is 100,000 tokens, we use 80% of it
        let actual_limit = 100_000;
        let adjusted_limit = (actual_limit as f64 * 0.8) as usize; // 80% of the actual limit
        anyhow::Ok(adjusted_limit)
    }
}
