// study-app/app-core/src/lib.rs

pub mod ai_bridge;
pub mod error;
pub mod extraction;
pub mod parser;

// Re-export core types for easy access by the UI layer
pub use error::ExtractionError;