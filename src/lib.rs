//! Structured errors and failure observability for Rust.
//!
//! `dataexcept` is the Rust companion to the Python `DataExcept` project. The
//! crate starts with the language-neutral parts of that contract: failure
//! metadata and JSON-safe error envelopes.

pub mod data;
mod envelope;
mod error;
mod failure;
pub mod redaction;
pub mod schema;
pub mod transformation;

pub use data::{
    DataFormatError, DataLoadingError, DataValidationError, MissingColumnError, MissingDataError,
    SchemaMismatchError,
};
pub use envelope::ErrorEnvelope;
pub use error::DataError;
pub use failure::{FailureKind, FailureMetadata, InvalidFailureMetadata};
pub use redaction::{
    fingerprint, redact_if_url, redact_secret, redact_url, redact_urls_in_text, remove_secret,
};
pub use schema::{DtypeMismatchError, IndexAlignmentError, MergeKeyError, SchemaEvolutionError};
pub use transformation::{DataNormalizationError, DataTransformationError, FeatureEngineeringError};

/// Convenient result alias for operations that use [`DataError`].
pub type Result<T> = std::result::Result<T, DataError>;
