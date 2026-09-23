//! Product-neutral observability context for failures crossing boundaries.
//!
//! Frameworks use different names for requests, tasks, jobs, workflow steps,
//! tool calls, and invocations. This module keeps the shared correlation
//! context small and independent of any observability backend.

use std::collections::BTreeMap;

use serde::Serialize;
use thiserror::Error;

use crate::{DataError, ErrorEnvelope, redaction::redact_urls_in_text};

/// Identifiers describing where an operation is executing.
///
/// System, component, and operation are intended to remain low-cardinality.
/// Request, job, correlation, trace, and span identifiers are correlation
/// values and should not normally become metric dimensions or indexed tags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct OperationContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    component: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    span_id: Option<String>,
}

impl OperationContext {
    /// Starts building a validated operation context.
    #[must_use]
    pub fn builder() -> OperationContextBuilder {
        OperationContextBuilder::default()
    }

    /// Returns the system identifier, when present.
    #[must_use]
    pub fn system(&self) -> Option<&str> {
        self.system.as_deref()
    }

    /// Returns the component identifier, when present.
    #[must_use]
    pub fn component(&self) -> Option<&str> {
        self.component.as_deref()
    }

    /// Returns the operation identifier, when present.
    #[must_use]
    pub fn operation(&self) -> Option<&str> {
        self.operation.as_deref()
    }

    /// Returns the request identifier, when present.
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    /// Returns the job identifier, when present.
    #[must_use]
    pub fn job_id(&self) -> Option<&str> {
        self.job_id.as_deref()
    }

    /// Returns the correlation identifier, when present.
    #[must_use]
    pub fn correlation_id(&self) -> Option<&str> {
        self.correlation_id.as_deref()
    }

    /// Returns the trace identifier, when present.
    #[must_use]
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    /// Returns the span identifier, when present.
    #[must_use]
    pub fn span_id(&self) -> Option<&str> {
        self.span_id.as_deref()
    }

    /// Returns only identifiers that are present.
    #[must_use]
    pub fn to_map(&self) -> BTreeMap<&'static str, &str> {
        let mut fields = BTreeMap::new();
        Self::insert_present(&mut fields, "system", self.system());
        Self::insert_present(&mut fields, "component", self.component());
        Self::insert_present(&mut fields, "operation", self.operation());
        Self::insert_present(&mut fields, "request_id", self.request_id());
        Self::insert_present(&mut fields, "job_id", self.job_id());
        Self::insert_present(&mut fields, "correlation_id", self.correlation_id());
        Self::insert_present(&mut fields, "trace_id", self.trace_id());
        Self::insert_present(&mut fields, "span_id", self.span_id());
        fields
    }

    /// Returns low-cardinality fields suitable for filtering and indexing.
    #[must_use]
    pub fn index_fields(&self) -> BTreeMap<&'static str, &str> {
        let mut fields = BTreeMap::new();
        Self::insert_present(&mut fields, "system", self.system());
        Self::insert_present(&mut fields, "component", self.component());
        Self::insert_present(&mut fields, "operation", self.operation());
        fields
    }

    /// Returns true when the context carries no identifiers.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.to_map().is_empty()
    }

    pub(crate) fn with_trace_id_from_carrier(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    fn insert_present<'a>(
        fields: &mut BTreeMap<&'static str, &'a str>,
        name: &'static str,
        value: Option<&'a str>,
    ) {
        if let Some(value) = value {
            fields.insert(name, value);
        }
    }
}

/// Builder for a validated [`OperationContext`].
#[derive(Debug, Clone, Default)]
pub struct OperationContextBuilder {
    system: Option<String>,
    component: Option<String>,
    operation: Option<String>,
    request_id: Option<String>,
    job_id: Option<String>,
    correlation_id: Option<String>,
    trace_id: Option<String>,
    span_id: Option<String>,
}

impl OperationContextBuilder {
    /// Sets the low-cardinality system identifier.
    #[must_use]
    pub fn system(mut self, value: impl Into<String>) -> Self {
        self.system = Some(value.into());
        self
    }

    /// Sets the low-cardinality component identifier.
    #[must_use]
    pub fn component(mut self, value: impl Into<String>) -> Self {
        self.component = Some(value.into());
        self
    }

    /// Sets the low-cardinality operation identifier.
    #[must_use]
    pub fn operation(mut self, value: impl Into<String>) -> Self {
        self.operation = Some(value.into());
        self
    }

    /// Sets the request identifier.
    #[must_use]
    pub fn request_id(mut self, value: impl Into<String>) -> Self {
        self.request_id = Some(value.into());
        self
    }

    /// Sets the job identifier.
    #[must_use]
    pub fn job_id(mut self, value: impl Into<String>) -> Self {
        self.job_id = Some(value.into());
        self
    }

    /// Sets the correlation identifier.
    #[must_use]
    pub fn correlation_id(mut self, value: impl Into<String>) -> Self {
        self.correlation_id = Some(value.into());
        self
    }

    /// Sets the trace identifier.
    #[must_use]
    pub fn trace_id(mut self, value: impl Into<String>) -> Self {
        self.trace_id = Some(value.into());
        self
    }

    /// Sets the span identifier.
    #[must_use]
    pub fn span_id(mut self, value: impl Into<String>) -> Self {
        self.span_id = Some(value.into());
        self
    }

    /// Validates, redacts, and creates the operation context.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidOperationContext`] when any supplied identifier is
    /// empty or contains only whitespace.
    pub fn build(self) -> Result<OperationContext, InvalidOperationContext> {
        Ok(OperationContext {
            system: sanitize("system", self.system)?,
            component: sanitize("component", self.component)?,
            operation: sanitize("operation", self.operation)?,
            request_id: sanitize("request_id", self.request_id)?,
            job_id: sanitize("job_id", self.job_id)?,
            correlation_id: sanitize("correlation_id", self.correlation_id)?,
            trace_id: sanitize("trace_id", self.trace_id)?,
            span_id: sanitize("span_id", self.span_id)?,
        })
    }
}

fn sanitize(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<String>, InvalidOperationContext> {
    value
        .map(|value| {
            if value.trim().is_empty() {
                return Err(InvalidOperationContext::EmptyField(field));
            }
            Ok(redact_urls_in_text(&value, false))
        })
        .transpose()
}

/// Validation failure for [`OperationContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidOperationContext {
    /// An explicitly supplied identifier was empty.
    #[error("{0} must not be empty")]
    EmptyField(&'static str),
}

/// Strict JSON-safe observability event shared by adapters.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ObservabilityEvent {
    event: &'static str,
    exception: ErrorEnvelope,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation: Option<OperationContext>,
}

impl ObservabilityEvent {
    /// Creates an exception observability event.
    #[must_use]
    pub fn new(exception: ErrorEnvelope, operation: Option<OperationContext>) -> Self {
        let operation = operation.filter(|context| !context.is_empty());

        Self {
            event: "exception",
            exception,
            operation,
        }
    }

    /// Creates an observability event from a [`DataError`].
    #[must_use]
    pub fn from_data_error(error: &DataError, operation: Option<OperationContext>) -> Self {
        Self::new(error.to_envelope(), operation)
    }

    /// Returns the event kind.
    #[must_use]
    pub const fn event(&self) -> &'static str {
        self.event
    }

    /// Returns the canonical exception envelope.
    #[must_use]
    pub const fn exception(&self) -> &ErrorEnvelope {
        &self.exception
    }

    /// Returns the operation context, when non-empty.
    #[must_use]
    pub const fn operation(&self) -> Option<&OperationContext> {
        self.operation.as_ref()
    }

    /// Serializes the event as strict JSON.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{InvalidOperationContext, ObservabilityEvent, OperationContext};
    use crate::DataError;

    #[test]
    fn context_serializes_only_present_fields() {
        let context = OperationContext::builder()
            .system("payments")
            .component("worker")
            .operation("settle_invoice")
            .request_id("req-42")
            .build()
            .expect("context should be valid");

        assert_eq!(context.to_map().len(), 4);
        assert_eq!(context.system(), Some("payments"));
        assert_eq!(context.request_id(), Some("req-42"));
        assert_eq!(context.trace_id(), None);
    }

    #[test]
    fn index_fields_exclude_high_cardinality_identifiers() {
        let context = OperationContext::builder()
            .system("payments")
            .component("worker")
            .operation("settle_invoice")
            .request_id("req-42")
            .correlation_id("corr-99")
            .build()
            .expect("context should be valid");

        let indexed = context.index_fields();

        assert_eq!(indexed.len(), 3);
        assert_eq!(indexed["system"], "payments");
        assert!(!indexed.contains_key("request_id"));
        assert!(!indexed.contains_key("correlation_id"));
    }

    #[test]
    fn empty_identifiers_are_rejected() {
        let result = OperationContext::builder().job_id("   ").build();

        assert_eq!(result, Err(InvalidOperationContext::EmptyField("job_id")));
    }

    #[test]
    fn url_shaped_identifiers_are_redacted() {
        let context = OperationContext::builder()
            .request_id("https://user:secret@example.com/request/123?token=SECRETVALUE")
            .build()
            .expect("context should be valid");

        assert_eq!(
            context.request_id(),
            Some("https://***:***@example.com/***?token=***")
        );
    }

    #[test]
    fn event_keeps_operation_separate_from_exception() {
        let error = DataError::new("validation_failed", "age must be positive")
            .with_attribute("field", json!("age"));
        let context = OperationContext::builder()
            .system("feature-service")
            .request_id("req-123")
            .build()
            .expect("context should be valid");
        let event = ObservabilityEvent::from_data_error(&error, Some(context));
        let value = serde_json::to_value(event).expect("event should serialize");

        assert_eq!(value["event"], "exception");
        assert_eq!(value["exception"]["type"], "validation_failed");
        assert_eq!(value["operation"]["system"], "feature-service");
        assert_eq!(value["operation"]["request_id"], "req-123");
        assert!(value["exception"].get("operation").is_none());
    }

    #[test]
    fn empty_context_is_omitted_from_event() {
        let error = DataError::new("example", "failed");
        let context = OperationContext::builder()
            .build()
            .expect("empty context is valid");
        let event = ObservabilityEvent::from_data_error(&error, Some(context));
        let value = serde_json::to_value(event).expect("event should serialize");

        assert!(value.get("operation").is_none());
    }
}
