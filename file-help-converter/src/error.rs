use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("I/O error occurred: {0}")]
    Io(#[from] io::Error),
    
    #[error("External crate processing error: {0}")]
    ExternalCrate(String),

    #[error("Processing failed: {0}")]
    ProcessError(String),
}
