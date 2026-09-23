//! Optional Sentry-compatible event enrichment.
//!
//! This adapter is SDK-independent. It enriches a JSON event with structured
//! `DataExcept` metadata that can be passed into Sentry hooks by the caller.

use serde_json::{Map, Value};

use crate::{DataError, FailureKind, OperationContext};

/// Enriches a Sentry-compatible JSON event with `DataExcept` failure metadata.
///
/// The original event is left unchanged. Full error and operation context are
/// stored under `contexts`; only low-cardinality operation fields and failure
/// classification are promoted into `tags`.
#[must_use]
pub fn enrich_sentry_event(
    event: &Value,
    error: &DataError,
    operation_context: Option<&OperationContext>,
) -> Value {
    let mut enriched = event.clone();
    let object = enriched
        .as_object_mut()
        .expect("Sentry event must be a JSON object");

    let envelope = error.to_envelope();
    let envelope_value = serde_json::to_value(&envelope)
        .expect("canonical DataError envelope must serialize");

    let contexts = object
        .entry("contexts")
        .or_insert_with(|| Value::Object(Map::new()));
    if !contexts.is_object() {
        *contexts = Value::Object(Map::new());
    }
    let contexts = contexts
        .as_object_mut()
        .expect("contexts was normalized to an object");
    contexts.insert("dataexcept".to_owned(), envelope_value);

    if let Some(context) = operation_context {
        let operation = serde_json::to_value(context)
            .expect("OperationContext must serialize");
        if operation.as_object().is_some_and(|map| !map.is_empty()) {
            contexts.insert("dataexcept_operation".to_owned(), operation);
        }
    }

    let tags = object
        .entry("tags")
        .or_insert_with(|| Value::Object(Map::new()));
    if !tags.is_object() {
        *tags = Value::Object(Map::new());
    }
    let tags = tags
        .as_object_mut()
        .expect("tags was normalized to an object");

    tags.insert(
        "dataexcept.type".to_owned(),
        Value::String(envelope.error_type.clone()),
    );

    if let Some(failure) = &envelope.failure {
        let kind = match failure.kind {
            FailureKind::Transient => "transient",
            FailureKind::Permanent => "permanent",
            FailureKind::Unknown => "unknown",
        };
        tags.insert(
            "dataexcept.failure_kind".to_owned(),
            Value::String(kind.to_owned()),
        );

        let retryable = failure
            .retryable
            .map_or_else(|| "unknown".to_owned(), |value| value.to_string());
        tags.insert(
            "dataexcept.retryable".to_owned(),
            Value::String(retryable),
        );
    }

    if let Some(context) = operation_context {
        for (key, value) in context.index_fields() {
            tags.insert(
                format!("dataexcept.operation.{key}"),
                Value::String(value.to_owned()),
            );
        }
    }

    enriched
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::enrich_sentry_event;
    use crate::{DataError, FailureMetadata, OperationContext};

    #[test]
    fn enriches_without_mutating_input() {
        let event = json!({
            "message": "validation failed",
            "contexts": {"runtime": {"name": "rust"}},
            "tags": {"service": "trainer"}
        });
        let error = DataError::new("ValidationError", "age must be positive")
            .with_module("dataexcept::validation")
            .with_failure(FailureMetadata::permanent());

        let enriched = enrich_sentry_event(&event, &error, None);

        assert_eq!(event["contexts"]["runtime"]["name"], "rust");
        assert!(event["contexts"].get("dataexcept").is_none());
        assert_eq!(enriched["contexts"]["runtime"]["name"], "rust");
        assert_eq!(
            enriched["contexts"]["dataexcept"]["type"],
            "ValidationError"
        );
        assert_eq!(enriched["tags"]["service"], "trainer");
        assert_eq!(enriched["tags"]["dataexcept.failure_kind"], "permanent");
        assert_eq!(enriched["tags"]["dataexcept.retryable"], "false");
    }

    #[test]
    fn operation_context_keeps_high_cardinality_ids_out_of_tags() {
        let error = DataError::new("example", "failed");
        let context = OperationContext::builder()
            .system("worker")
            .operation("billing.settle_invoice")
            .request_id("req-42")
            .trace_id("4bf92f3577b34da6a3ce929d0e0e4736")
            .build()
            .expect("operation context should be valid");

        let enriched = enrich_sentry_event(&json!({}), &error, Some(&context));

        assert_eq!(
            enriched["contexts"]["dataexcept_operation"]["request_id"],
            "req-42"
        );
        assert_eq!(
            enriched["tags"]["dataexcept.operation.system"],
            "worker"
        );
        assert_eq!(
            enriched["tags"]["dataexcept.operation.operation"],
            "billing.settle_invoice"
        );
        assert!(enriched["tags"].get("dataexcept.operation.request_id").is_none());
        assert!(enriched["tags"].get("dataexcept.operation.trace_id").is_none());
    }

    #[test]
    fn envelope_context_is_redacted() {
        let error = DataError::new(
            "request_failed",
            "GET https://user:secret@example.com/private?token=SECRETVALUE failed",
        );

        let enriched = enrich_sentry_event(&json!({}), &error, None);
        let rendered = enriched["contexts"]["dataexcept"].to_string();

        assert!(!rendered.contains("secret"));
        assert!(!rendered.contains("SECRETVALUE"));
    }
}
