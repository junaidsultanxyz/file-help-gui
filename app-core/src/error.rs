// app-core/src/error.rs

use thiserror::Error;

/// Top-level error type for the filesystem and extraction operations.
#[derive(Error, Debug)]
pub enum ExtractionError {
    #[error("failed to memory map the target file: {0}")]
    MmapFailed(#[source] std::io::Error),
    
    #[error("I/O error occurred: {0}")]
    IoError(#[from] std::io::Error),

    #[error("invalid UTF-8 sequence detected in source file")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("regex compilation failed: {0}")]
    RegexError(#[from] regex::Error),
}

/// Error type for the zero-cost `.study` text parser.
#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("missing required line prefix (e.g., 'T:') at line {0}")]
    MissingPrefix(usize),
    
    #[error("unexpected end of file while parsing page {0}")]
    UnexpectedEof(u32),
}