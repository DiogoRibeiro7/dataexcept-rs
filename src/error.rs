//! Idiomatic Rust operational error type.

use std::{collections::BTreeMap, error::Error, fmt};

use serde_json::Value;

use crate::{
    redaction::{redact_json_value, redact_urls_in_text},
    ErrorEnvelope, FailureMetadata,
};

/// A small, structured operational error suitable for application boundaries.
///
/// Applications that already have domain-specific enums do not need to replace
/// them with `DataError`; they can construct [`ErrorEnvelope`] values
/// directly. `DataError` is provided for consumers that want a ready-made
/// error type.
#[derive(Debug, Clone, PartialEq)]
pub struct DataError {
    code: String,
    message: String,
    module: String,
    attributes: BTreeMap<String, Value>,
    failure: FailureMetadata,
}

impl DataError {
    /// Creates an unclassified operational error.
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        let message = message.into();

        Self {
            code: code.into(),
            message: redact_urls_in_text(&message, true),
            module: "dataexcept".to_owned(),
            attributes: BTreeMap::new(),
            failure: FailureMetadata::unknown(),
        }
    }

    /// Sets the logical module or namespace.
    #[must_use]
    pub fn with_module(mut self, module: impl Into<String>) -> Self {
        self.module = module.into();
        self
    }

    /// Adds one structured attribute.
    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: Value) -> Self {
        self.attributes
            .insert(key.into(), redact_json_value(&value, true));
        self
    }

    /// Attaches failure metadata.
    #[must_use]
    pub fn with_failure(mut self, failure: FailureMetadata) -> Self {
        self.failure = failure;
        self
    }

    /// Returns the stable machine-readable error code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Converts the error into a transport-safe envelope.
    #[must_use]
    pub fn to_envelope(&self) -> ErrorEnvelope {
        ErrorEnvelope::new(&self.code, &self.module, &self.message)
            .with_attributes(self.attributes.clone())
            .with_failure(self.failure.clone())
    }
}

impl fmt::Display for DataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for DataError {}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::DataError;
    use crate::FailureMetadata;

    #[test]
    fn converts_to_structured_envelope() {
        let error = DataError::new("missing_column", "customer_id is required")
            .with_module("pipeline::validation")
            .with_attribute("column", json!("customer_id"))
            .with_failure(FailureMetadata::permanent());

        let envelope = error.to_envelope();

        assert_eq!(envelope.error_type, "missing_column");
        assert_eq!(envelope.module, "pipeline::validation");
        assert_eq!(
            envelope
                .attributes
                .expect("attributes should exist")
                .get("column"),
            Some(&json!("customer_id"))
        );
    }

    #[test]
    fn display_redacts_url_credentials() {
        let error = DataError::new(
            "request_failed",
            "GET https://user:secret@example.com/v1?token=SECRETVALUE failed",
        );

        let rendered = error.to_string();

        assert!(!rendered.contains("secret"));
        assert!(!rendered.contains("SECRETVALUE"));
        assert!(rendered.contains("example.com"));
        assert!(rendered.contains("/v1"));
    }

    #[test]
    fn attributes_redact_nested_url_credentials() {
        let error = DataError::new("request_failed", "request failed").with_attribute(
            "request",
            json!({
                "endpoint": "https://example.com/v1?api_key=SECRETVALUE",
            }),
        );

        let envelope = error.to_envelope();
        let attributes = envelope.attributes.expect("attributes should exist");

        assert_eq!(
            attributes["request"]["endpoint"],
            "https://example.com/***?api_key=***"
        );
    }
}
