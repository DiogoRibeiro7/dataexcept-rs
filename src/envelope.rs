//! Language-neutral structured error envelopes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    FailureMetadata,
    redaction::{redact_attributes, redact_urls_in_text},
};

/// A JSON-safe representation of an error.
///
/// The shape mirrors the stable concepts in the Python `DataExcept` envelope:
/// type, module, message, public attributes, recovery metadata, cause/context
/// chains, and grouped errors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    /// Unqualified error type.
    #[serde(rename = "type")]
    pub error_type: String,
    /// Module or logical namespace defining the error.
    pub module: String,
    /// Human-readable error message.
    pub message: String,
    /// Public structured attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<BTreeMap<String, Value>>,
    /// Recovery metadata for classified failures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<FailureMetadata>,
    /// Explicitly chained cause.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Box<ErrorEnvelope>>,
    /// Implicit execution context, where a caller chooses to expose it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Box<ErrorEnvelope>>,
    /// Members of a grouped error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exceptions: Option<Vec<ErrorEnvelope>>,
}

impl ErrorEnvelope {
    /// Creates the minimal required envelope.
    #[must_use]
    pub fn new(
        error_type: impl Into<String>,
        module: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();

        Self {
            error_type: error_type.into(),
            module: module.into(),
            message: redact_urls_in_text(&message, false),
            attributes: None,
            failure: None,
            cause: None,
            context: None,
            exceptions: None,
        }
    }

    /// Attaches public structured attributes.
    #[must_use]
    pub fn with_attributes(mut self, attributes: BTreeMap<String, Value>) -> Self {
        if !attributes.is_empty() {
            self.attributes = Some(redact_attributes(attributes, false));
        }
        self
    }

    /// Attaches machine-readable recovery metadata.
    #[must_use]
    pub fn with_failure(mut self, failure: FailureMetadata) -> Self {
        self.failure = Some(failure);
        self
    }

    /// Attaches an explicit cause.
    #[must_use]
    pub fn with_cause(mut self, cause: ErrorEnvelope) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Attaches execution context.
    #[must_use]
    pub fn with_context(mut self, context: ErrorEnvelope) -> Self {
        self.context = Some(Box::new(context));
        self
    }

    /// Attaches grouped child errors.
    #[must_use]
    pub fn with_exceptions(mut self, exceptions: Vec<ErrorEnvelope>) -> Self {
        if !exceptions.is_empty() {
            self.exceptions = Some(exceptions);
        }
        self
    }

    /// Parses a normal error envelope from JSON.
    ///
    /// Use `EnvelopeNode::from_json` when the input may contain cycle or
    /// truncation protocol markers.
    ///
    /// # Errors
    ///
    /// Returns a JSON error when the input is invalid or does not match the
    /// normal error-envelope shape.
    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }

    /// Serializes the envelope as strict JSON.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serializes the envelope as pretty-printed strict JSON.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] if serialization fails.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::ErrorEnvelope;
    use crate::FailureMetadata;

    #[test]
    fn round_trips_through_json() {
        let envelope = ErrorEnvelope::new(
            "MissingColumnError",
            "dataexcept::schema",
            "missing required column customer_id",
        )
        .with_failure(FailureMetadata::permanent());

        let json = envelope.to_json().expect("envelope should serialize");
        let parsed = ErrorEnvelope::from_json(&json).expect("envelope should parse");

        assert_eq!(parsed, envelope);
    }

    #[test]
    fn serializes_required_fields_and_failure_metadata() {
        let envelope = ErrorEnvelope::new(
            "MissingColumnError",
            "dataexcept::schema",
            "missing required column customer_id",
        )
        .with_failure(FailureMetadata::permanent());

        let value = serde_json::to_value(envelope).expect("envelope should serialize");

        assert_eq!(value["type"], "MissingColumnError");
        assert_eq!(value["module"], "dataexcept::schema");
        assert_eq!(value["failure"]["kind"], "permanent");
        assert_eq!(value["failure"]["retryable"], false);
        assert!(value.get("attributes").is_none());
    }

    #[test]
    fn preserves_nested_causes() {
        let cause = ErrorEnvelope::new("IoError", "std::io", "connection refused");
        let root =
            ErrorEnvelope::new("DataLoadingError", "example", "load failed").with_cause(cause);

        let value = serde_json::to_value(root).expect("envelope should serialize");
        assert_eq!(value["cause"]["type"], "IoError");
    }

    #[test]
    fn export_redacts_url_path_and_credentials() {
        let mut attributes = BTreeMap::new();
        attributes.insert(
            "endpoint".to_owned(),
            json!("https://user:secret@example.com/webhook/path?token=SECRETVALUE"),
        );

        let envelope = ErrorEnvelope::new(
            "WebhookError",
            "example",
            "POST https://user:secret@example.com/webhook/path?token=SECRETVALUE failed",
        )
        .with_attributes(attributes);

        let value = serde_json::to_value(envelope).expect("envelope should serialize");

        assert_eq!(
            value["message"],
            "POST https://***:***@example.com/***?token=*** failed"
        );
        assert_eq!(
            value["attributes"]["endpoint"],
            "https://***:***@example.com/***?token=***"
        );
    }
}
