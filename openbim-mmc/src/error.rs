use std::io;

/// Failures opening, parsing, building, writing, or extracting an MMC archive.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MmcError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("XML error in {path}: {message}")]
    Xml { path: String, message: String },

    #[error("unsafe archive path {path:?}: {reason}")]
    UnsafeArchivePath { path: String, reason: &'static str },

    #[error("duplicate or case/normalization-colliding archive paths: {first:?} and {second:?}")]
    DuplicateArchivePath { first: String, second: String },

    #[error("the archive must contain exactly one root MultiModel.xml")]
    MissingRoot,

    #[error(
        "inconsistent ZIP metadata for {path:?}: declared {declared} uncompressed bytes, read {actual}"
    )]
    InvalidZipMetadata {
        path: String,
        declared: u64,
        actual: u64,
    },

    #[error("resource limit for {resource} exceeded: observed {actual}, maximum {maximum}")]
    LimitExceeded {
        resource: &'static str,
        actual: u64,
        maximum: u64,
    },

    #[error("unsafe extraction path {path}: {reason}")]
    UnsafeExtractionPath { path: String, reason: &'static str },

    #[error("invalid archive builder input: {0}")]
    InvalidBuilder(String),
}
