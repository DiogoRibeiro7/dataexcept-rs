//! Structured errors and failure observability for Rust.
//!
//! `dataexcept` is the Rust companion to the Python `DataExcept` project. The
//! crate starts with the language-neutral parts of that contract: failure
//! metadata and JSON-safe error envelopes.

pub mod broker;
mod conversions;
pub mod data;
pub mod database;
mod envelope;
mod error;
mod failure;
pub mod ml;
pub mod network;
pub mod observability;
pub mod redaction;
pub mod schema;
pub mod trace_context;
#[cfg(feature = "tracing")]
pub mod tracing_integration;
pub mod transformation;

pub use broker::{
    BrokerConnectionError, BrokerTimeoutError, MessageAcknowledgementError, MessageConsumeError,
    MessagePosition, MessagePublishError,
};
pub use data::{
    DataFormatError, DataLoadingError, DataValidationError, MissingColumnError, MissingDataError,
    SchemaMismatchError,
};
pub use database::{DatabaseConnectionError, QueryExecutionError, TransactionError};
pub use envelope::ErrorEnvelope;
pub use error::DataError;
pub use failure::{FailureKind, FailureMetadata, InvalidFailureMetadata};
pub use ml::{
    ConvergenceError, CrossValidationError, HyperparameterError, ModelEvaluationError,
    ModelInferenceError, ModelTrainingError, PredictionError, TrainingTimeoutError,
};
pub use network::{ConnectionTimeoutError, HostUnreachableError, ProtocolError};
pub use observability::{
    InvalidOperationContext, ObservabilityEvent, OperationContext, OperationContextBuilder,
};
pub use redaction::{
    fingerprint, redact_if_url, redact_secret, redact_url, redact_urls_in_text, remove_secret,
};
pub use schema::{DtypeMismatchError, IndexAlignmentError, MergeKeyError, SchemaEvolutionError};
pub use trace_context::{
    TraceContextConflict, W3CTraceContext, parse_traceparent, trace_context_from_mapping,
};
#[cfg(feature = "tracing")]
pub use tracing_integration::emit_error;
pub use transformation::{
    DataNormalizationError, DataTransformationError, FeatureEngineeringError,
};

/// Convenient result alias for operations that use [`DataError`].
pub type Result<T> = std::result::Result<T, DataError>;
