pub mod completion;
pub mod model;

pub use completion::*;

pub use model::OllamaLanguageModel;

pub const OLLAMA_API_URL: &'static str = "http://localhost:11434/v1";
