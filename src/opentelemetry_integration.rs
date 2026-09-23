//! Optional OpenTelemetry integration.
//!
//! This module depends only on the OpenTelemetry API crate. It does not install
//! an SDK, exporter, processor, provider, or global runtime.

use opentelemetry::{KeyValue, trace::Span};

use crate::{DataError, FailureKind, OperationContext};

const ENVELOPE_SCHEMA_ID: &str =
    "https://diogoribeiro7.github.io/DataExcept/schema/envelope-1.0.0.json";

/// Converts a [`DataError`] into OpenTelemetry-compatible exception attributes.
///
/// Trace and span identifiers from [`OperationContext`] are deliberately not
/// duplicated as custom attributes because OpenTelemetry already carries those
/// identifiers in native span context.
///
/// # Errors
///
/// Returns [`serde_json::Error`] when `include_envelope` is true and the
/// canonical error envelope cannot be serialized.
pub fn exception_to_otel_attributes(
    error: &DataError,
    operation_context: Option<&OperationContext>,
    include_envelope: bool,
) -> Result<Vec<KeyValue>, serde_json::Error> {
    let envelope = error.to_envelope();
    let qualified_type = format!("{}::{}", envelope.module, envelope.error_type);

    let mut attributes = vec![
        KeyValue::new("exception.type", qualified_type),
        KeyValue::new("exception.message", envelope.message.clone()),
    ];

    if let Some(failure) = &envelope.failure {
        let kind = match failure.kind {
            FailureKind::Transient => "transient",
            FailureKind::Permanent => "permanent",
            FailureKind::Unknown => "unknown",
        };

        attributes.push(KeyValue::new("dataexcept.failure.kind", kind));

        if let Some(retryable) = failure.retryable {
            attributes.push(KeyValue::new("dataexcept.failure.retryable", retryable));
        }

        if let Some(retry_after_seconds) = failure.retry_after_seconds {
            attributes.push(KeyValue::new(
                "dataexcept.failure.retry_after_seconds",
                retry_after_seconds,
            ));
        }
    }

    if let Some(context) = operation_context {
        push_operation_attributes(&mut attributes, context);
    }

    if include_envelope {
        attributes.push(KeyValue::new(
            "dataexcept.envelope.schema",
            ENVELOPE_SCHEMA_ID,
        ));
        attributes.push(KeyValue::new("dataexcept.envelope", envelope.to_json()?));
    }

    Ok(attributes)
}

/// Records a `DataExcept` exception event on an OpenTelemetry span.
///
/// Telemetry conversion failures are swallowed so observability cannot replace
/// the application failure already being handled.
pub fn record_otel_exception<S: Span>(
    span: &mut S,
    error: &DataError,
    operation_context: Option<&OperationContext>,
    include_envelope: bool,
) {
    let Ok(attributes) = exception_to_otel_attributes(error, operation_context, include_envelope)
    else {
        return;
    };

    span.add_event("exception", attributes);
}

fn push_operation_attributes(attributes: &mut Vec<KeyValue>, context: &OperationContext) {
    push_optional(attributes, "dataexcept.operation.system", context.system());
    push_optional(
        attributes,
        "dataexcept.operation.component",
        context.component(),
    );
    push_optional(
        attributes,
        "dataexcept.operation.operation",
        context.operation(),
    );
    push_optional(
        attributes,
        "dataexcept.operation.request_id",
        context.request_id(),
    );
    push_optional(attributes, "dataexcept.operation.job_id", context.job_id());
    push_optional(
        attributes,
        "dataexcept.operation.correlation_id",
        context.correlation_id(),
    );
}

fn push_optional(attributes: &mut Vec<KeyValue>, key: &'static str, value: Option<&str>) {
    if let Some(value) = value {
        attributes.push(KeyValue::new(key, value.to_owned()));
    }
}

#[cfg(test)]
mod tests {
    use opentelemetry::Value;
    use serde_json::json;

    use super::{ENVELOPE_SCHEMA_ID, exception_to_otel_attributes};
    use crate::{DataError, FailureMetadata, OperationContext};

    #[test]
    fn emits_standard_exception_and_failure_attributes() {
        let error = DataError::new("ValidationError", "age must be positive")
            .with_module("dataexcept::validation")
            .with_failure(FailureMetadata::permanent());

        let attributes = exception_to_otel_attributes(&error, None, false)
            .expect("attributes should be created");

        assert_eq!(
            string_attribute(&attributes, "exception.type"),
            Some("dataexcept::validation::ValidationError")
        );
        assert_eq!(
            string_attribute(&attributes, "exception.message"),
            Some("age must be positive")
        );
        assert_eq!(
            string_attribute(&attributes, "dataexcept.failure.kind"),
            Some("permanent")
        );
        assert_eq!(
            bool_attribute(&attributes, "dataexcept.failure.retryable"),
            Some(false)
        );
    }

    #[test]
    fn projects_operation_context_without_trace_or_span_ids() {
        let error = DataError::new("example", "failed");
        let context = OperationContext::builder()
            .system("worker")
            .request_id("req-42")
            .trace_id("4bf92f3577b34da6a3ce929d0e0e4736")
            .span_id("00f067aa0ba902b7")
            .build()
            .expect("context should be valid");

        let attributes = exception_to_otel_attributes(&error, Some(&context), false)
            .expect("attributes should be created");

        assert_eq!(
            string_attribute(&attributes, "dataexcept.operation.system"),
            Some("worker")
        );
        assert_eq!(
            string_attribute(&attributes, "dataexcept.operation.request_id"),
            Some("req-42")
        );
        assert!(!has_attribute(&attributes, "dataexcept.operation.trace_id"));
        assert!(!has_attribute(&attributes, "dataexcept.operation.span_id"));
    }

    #[test]
    fn optional_envelope_is_strict_json_and_carries_schema_id() {
        let error = DataError::new("example", "failed").with_attribute("field", json!("age"));

        let attributes =
            exception_to_otel_attributes(&error, None, true).expect("attributes should be created");

        assert_eq!(
            string_attribute(&attributes, "dataexcept.envelope.schema"),
            Some(ENVELOPE_SCHEMA_ID)
        );

        let envelope = string_attribute(&attributes, "dataexcept.envelope")
            .expect("serialized envelope should be present");
        let value: serde_json::Value =
            serde_json::from_str(envelope).expect("envelope should be valid JSON");

        assert_eq!(value["type"], "example");
        assert_eq!(value["attributes"]["field"], "age");
    }

    fn has_attribute(attributes: &[opentelemetry::KeyValue], key: &str) -> bool {
        attributes
            .iter()
            .any(|attribute| attribute.key.as_str() == key)
    }

    fn string_attribute<'a>(
        attributes: &'a [opentelemetry::KeyValue],
        key: &str,
    ) -> Option<&'a str> {
        attributes.iter().find_map(|attribute| {
            if attribute.key.as_str() != key {
                return None;
            }
            match &attribute.value {
                Value::String(value) => Some(value.as_str()),
                _ => None,
            }
        })
    }

    fn bool_attribute(attributes: &[opentelemetry::KeyValue], key: &str) -> Option<bool> {
        attributes.iter().find_map(|attribute| {
            if attribute.key.as_str() != key {
                return None;
            }
            match attribute.value {
                Value::Bool(value) => Some(value),
                _ => None,
            }
        })
    }
}
