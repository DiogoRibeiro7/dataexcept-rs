//! Structured errors and failure observability for Rust.
//!
//! `dataexcept` is the Rust companion to the Python DataExcept project. The
//! crate starts with the language-neutral parts of that contract: failure
//! metadata and JSON-safe error envelopes.

mod envelope;
mod error;
mod failure;

pub use envelope::ErrorEnvelope;
pub use error::DataError;
pub use failure::{FailureKind, FailureMetadata, InvalidFailureMetadata};

/// Convenient result alias for operations that use [`DataError`].
pub type Result<T> = std::result::Result<T, DataError>;
