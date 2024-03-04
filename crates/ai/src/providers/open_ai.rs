pub mod completion;
pub mod embedding;
pub mod model;

pub use completion::*;
pub use embedding::*;
pub use model::OpenAiLanguageModel;

pub const OPEN_AI_API_URL: &'static str = "http://localhost:11434/v1";
