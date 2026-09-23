//! Framework-neutral observability boundary adapters.
//!
//! The adapters accept plain metadata and stable operation identifiers so
//! HTTP servers, workers, brokers, and workflow engines can integrate without
//! becoming runtime dependencies of `dataexcept`.

use std::collections::BTreeMap;

use thiserror::Error;

use crate::{
    InvalidOperationContext, OperationContext, TraceContextConflict, W3CTraceContext,
    trace_context_from_mapping,
};

const REQUEST_ID_HEADERS: [&str; 2] = ["x-request-id", "request-id"];
const CORRELATION_ID_HEADERS: [&str; 2] = ["x-correlation-id", "correlation-id"];
const JOB_ID_KEYS: [&str; 3] = ["job_id", "task_id", "id"];
const WORKER_CORRELATION_KEYS: [&str; 3] = ["correlation_id", "x-correlation-id", "correlation-id"];

/// Validation failure while constructing a boundary context.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BoundaryContextError {
    /// A required text field was empty after trimming.
    #[error("{0} must not be empty")]
    EmptyField(&'static str),
    /// A text field contained a carriage return or newline.
    #[error("{0} must be single-line text")]
    MultilineField(&'static str),
    /// The HTTP method was not a valid token.
    #[error("method must be a valid HTTP method token")]
    InvalidHttpMethod,
    /// The HTTP route included a query string or fragment.
    #[error("route must be a route template without query or fragment")]
    InvalidRoute,
    /// The resulting operation context was invalid.
    #[error(transparent)]
    InvalidOperationContext(#[from] InvalidOperationContext),
    /// Incoming trace provenance conflicted with existing context.
    #[error(transparent)]
    TraceContextConflict(#[from] TraceContextConflict),
}

/// Framework-neutral HTTP request context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpContext {
    operation_context: OperationContext,
    trace_context: Option<W3CTraceContext>,
}

impl HttpContext {
    /// Returns the normalized operation context.
    #[must_use]
    pub const fn operation_context(&self) -> &OperationContext {
        &self.operation_context
    }

    /// Returns propagated W3C trace context, when present.
    #[must_use]
    pub const fn trace_context(&self) -> Option<&W3CTraceContext> {
        self.trace_context.as_ref()
    }
}

/// Worker/task operation context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerContext {
    operation_context: OperationContext,
    trace_context: Option<W3CTraceContext>,
    attempt: Option<u32>,
}

impl WorkerContext {
    /// Returns the normalized operation context.
    #[must_use]
    pub const fn operation_context(&self) -> &OperationContext {
        &self.operation_context
    }

    /// Returns propagated W3C trace context, when present.
    #[must_use]
    pub const fn trace_context(&self) -> Option<&W3CTraceContext> {
        self.trace_context.as_ref()
    }

    /// Returns the retry attempt number, when supplied.
    #[must_use]
    pub const fn attempt(&self) -> Option<u32> {
        self.attempt
    }
}

/// Stable broker boundary operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrokerOperation {
    /// Publishing a message.
    Publish,
    /// Consuming a message.
    Consume,
    /// Acknowledging a message.
    Acknowledge,
}

impl BrokerOperation {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Publish => "publish",
            Self::Consume => "consume",
            Self::Acknowledge => "acknowledge",
        }
    }
}

/// Optional metadata for a broker boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BrokerContextOptions<'a> {
    /// Optional logical component name.
    pub component: Option<&'a str>,
    /// Optional correlation identifier.
    pub correlation_id: Option<&'a str>,
    /// Optional partition number.
    pub partition: Option<u32>,
    /// Optional broker offset.
    pub offset: Option<u64>,
    /// Optional consumer-group identifier.
    pub consumer_group: Option<&'a str>,
    /// Optional message identifier.
    pub message_id: Option<&'a str>,
}

/// Broker or stream-processing boundary context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokerContext {
    operation_context: OperationContext,
    trace_context: Option<W3CTraceContext>,
    topic: String,
    partition: Option<u32>,
    offset: Option<u64>,
    consumer_group: Option<String>,
    message_id: Option<String>,
}

impl BrokerContext {
    /// Returns the normalized operation context.
    #[must_use]
    pub const fn operation_context(&self) -> &OperationContext {
        &self.operation_context
    }

    /// Returns propagated W3C trace context, when present.
    #[must_use]
    pub const fn trace_context(&self) -> Option<&W3CTraceContext> {
        self.trace_context.as_ref()
    }

    /// Returns the topic name.
    #[must_use]
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// Returns the partition, when supplied.
    #[must_use]
    pub const fn partition(&self) -> Option<u32> {
        self.partition
    }

    /// Returns the offset, when supplied.
    #[must_use]
    pub const fn offset(&self) -> Option<u64> {
        self.offset
    }

    /// Returns the consumer group, when supplied.
    #[must_use]
    pub fn consumer_group(&self) -> Option<&str> {
        self.consumer_group.as_deref()
    }

    /// Returns the message identifier, when supplied.
    #[must_use]
    pub fn message_id(&self) -> Option<&str> {
        self.message_id.as_deref()
    }
}

/// Optional metadata for an orchestrator boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OrchestratorContextOptions<'a> {
    /// Optional workflow-run identifier.
    pub run_id: Option<&'a str>,
    /// Optional step-run identifier.
    pub step_run_id: Option<&'a str>,
    /// Optional logical component name.
    pub component: Option<&'a str>,
    /// Optional correlation identifier.
    pub correlation_id: Option<&'a str>,
    /// Optional retry attempt number.
    pub attempt: Option<u32>,
}

/// Workflow/orchestrator step context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestratorContext {
    operation_context: OperationContext,
    trace_context: Option<W3CTraceContext>,
    step_run_id: Option<String>,
    attempt: Option<u32>,
}

impl OrchestratorContext {
    /// Returns the normalized operation context.
    #[must_use]
    pub const fn operation_context(&self) -> &OperationContext {
        &self.operation_context
    }

    /// Returns propagated W3C trace context, when present.
    #[must_use]
    pub const fn trace_context(&self) -> Option<&W3CTraceContext> {
        self.trace_context.as_ref()
    }

    /// Returns the step-run identifier, when supplied.
    #[must_use]
    pub fn step_run_id(&self) -> Option<&str> {
        self.step_run_id.as_deref()
    }

    /// Returns the retry attempt number, when supplied.
    #[must_use]
    pub const fn attempt(&self) -> Option<u32> {
        self.attempt
    }
}

/// Builds HTTP observability context from plain headers and a route template.
///
/// `route` should be a low-cardinality route template such as `/users/{id}`,
/// not a raw request path with query parameters.
///
/// # Errors
///
/// Returns [`BoundaryContextError`] for an invalid method, route, component,
/// or conflicting W3C trace context.
pub fn http_context_from_request(
    method: &str,
    route: Option<&str>,
    headers: &BTreeMap<String, String>,
    component: Option<&str>,
) -> Result<HttpContext, BoundaryContextError> {
    let method = normalize_http_method(method)?;
    let operation = match route {
        Some(route) => format!("{method} {}", validate_route(route)?),
        None => method,
    };

    let request_id = first_metadata_value(headers, &REQUEST_ID_HEADERS);
    let correlation_id = first_metadata_value(headers, &CORRELATION_ID_HEADERS);
    let mut builder = OperationContext::builder()
        .system("http")
        .operation(operation);

    if let Some(component) = component {
        builder = builder.component(component);
    }
    if let Some(request_id) = request_id {
        builder = builder.request_id(request_id);
    }
    if let Some(correlation_id) = correlation_id {
        builder = builder.correlation_id(correlation_id);
    }

    let operation_context = builder.build()?;
    let trace_context = trace_from_metadata(headers);
    let operation_context = correlate_trace(operation_context, trace_context.as_ref())?;

    Ok(HttpContext {
        operation_context,
        trace_context,
    })
}

/// Builds worker observability context from stable task metadata.
///
/// Explicit job and correlation identifiers take precedence over metadata
/// fallbacks. Task arguments and payloads are intentionally excluded.
///
/// # Errors
///
/// Returns [`BoundaryContextError`] for invalid task metadata or conflicting
/// W3C trace context.
pub fn worker_context_from_task(
    task_name: &str,
    metadata: &BTreeMap<String, String>,
    job_id: Option<&str>,
    correlation_id: Option<&str>,
    component: Option<&str>,
    attempt: Option<u32>,
) -> Result<WorkerContext, BoundaryContextError> {
    let task_name = single_line(task_name, "task_name")?;
    let job_id = optional_single_line(job_id, "job_id")?
        .or_else(|| first_metadata_value(metadata, &JOB_ID_KEYS));
    let correlation_id = optional_single_line(correlation_id, "correlation_id")?
        .or_else(|| first_metadata_value(metadata, &WORKER_CORRELATION_KEYS));

    let mut builder = OperationContext::builder()
        .system("worker")
        .operation(task_name);
    if let Some(component) = component {
        builder = builder.component(component);
    }
    if let Some(job_id) = job_id {
        builder = builder.job_id(job_id);
    }
    if let Some(correlation_id) = correlation_id {
        builder = builder.correlation_id(correlation_id);
    }

    let operation_context = builder.build()?;
    let trace_context = trace_from_metadata(metadata);
    let operation_context = correlate_trace(operation_context, trace_context.as_ref())?;

    Ok(WorkerContext {
        operation_context,
        trace_context,
        attempt,
    })
}

/// Builds broker observability context without capturing message payloads.
///
/// Partition, offset, consumer-group, and message identifiers remain wrapper
/// metadata instead of becoming low-cardinality operation labels.
///
/// # Errors
///
/// Returns [`BoundaryContextError`] for invalid textual metadata or conflicting
/// W3C trace context.
pub fn broker_context_from_message(
    operation: BrokerOperation,
    topic: &str,
    metadata: &BTreeMap<String, String>,
    options: BrokerContextOptions<'_>,
) -> Result<BrokerContext, BoundaryContextError> {
    let BrokerContextOptions {
        component,
        correlation_id,
        partition,
        offset,
        consumer_group,
        message_id,
    } = options;
    let topic = single_line(topic, "topic")?;
    let correlation_id = optional_single_line(correlation_id, "correlation_id")?;
    let consumer_group = optional_single_line(consumer_group, "consumer_group")?;
    let message_id = optional_single_line(message_id, "message_id")?;
    let operation_name = format!("{} {topic}", operation.as_str());

    let mut builder = OperationContext::builder()
        .system("broker")
        .operation(operation_name);
    if let Some(component) = component {
        builder = builder.component(component);
    }
    if let Some(correlation_id) = correlation_id {
        builder = builder.correlation_id(correlation_id);
    }

    let operation_context = builder.build()?;
    let trace_context = trace_from_metadata(metadata);
    let operation_context = correlate_trace(operation_context, trace_context.as_ref())?;

    Ok(BrokerContext {
        operation_context,
        trace_context,
        topic,
        partition,
        offset,
        consumer_group,
        message_id,
    })
}

/// Builds workflow-step observability context from stable orchestrator metadata.
///
/// The operation name is the stable `workflow:step` pair. Run identifiers are
/// correlation metadata and are never folded into the operation name.
///
/// # Errors
///
/// Returns [`BoundaryContextError`] for invalid textual metadata or conflicting
/// W3C trace context.
pub fn orchestrator_context_from_step(
    workflow: &str,
    step: &str,
    metadata: &BTreeMap<String, String>,
    options: OrchestratorContextOptions<'_>,
) -> Result<OrchestratorContext, BoundaryContextError> {
    let OrchestratorContextOptions {
        run_id,
        step_run_id,
        component,
        correlation_id,
        attempt,
    } = options;
    let workflow = single_line(workflow, "workflow")?;
    let step = single_line(step, "step")?;
    let run_id = optional_single_line(run_id, "run_id")?;
    let step_run_id = optional_single_line(step_run_id, "step_run_id")?;
    let correlation_id = optional_single_line(correlation_id, "correlation_id")?;

    let mut builder = OperationContext::builder()
        .system("orchestrator")
        .operation(format!("{workflow}:{step}"));
    if let Some(component) = component {
        builder = builder.component(component);
    }
    if let Some(run_id) = run_id {
        builder = builder.job_id(run_id);
    }
    if let Some(correlation_id) = correlation_id {
        builder = builder.correlation_id(correlation_id);
    }

    let operation_context = builder.build()?;
    let trace_context = trace_from_metadata(metadata);
    let operation_context = correlate_trace(operation_context, trace_context.as_ref())?;

    Ok(OrchestratorContext {
        operation_context,
        trace_context,
        step_run_id,
        attempt,
    })
}

fn normalize_http_method(method: &str) -> Result<String, BoundaryContextError> {
    let method = method.trim().to_ascii_uppercase();
    if method.is_empty() || !method.bytes().all(is_http_token_byte) {
        return Err(BoundaryContextError::InvalidHttpMethod);
    }
    Ok(method)
}

const fn is_http_token_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'A'..=b'Z'
            | b'0'..=b'9'
            | b'!'
            | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'*'
            | b'+'
            | b'-'
            | b'.'
            | b'^'
            | b'_'
            | b'`'
            | b'|'
            | b'~'
    )
}

fn validate_route(route: &str) -> Result<String, BoundaryContextError> {
    let route = single_line(route, "route")?;
    if route.contains(['?', '#']) {
        return Err(BoundaryContextError::InvalidRoute);
    }
    Ok(route)
}

fn single_line(value: &str, field: &'static str) -> Result<String, BoundaryContextError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(BoundaryContextError::EmptyField(field));
    }
    if value.contains('\r') || value.contains('\n') {
        return Err(BoundaryContextError::MultilineField(field));
    }
    Ok(normalized.to_owned())
}

fn optional_single_line(
    value: Option<&str>,
    field: &'static str,
) -> Result<Option<String>, BoundaryContextError> {
    value.map(|value| single_line(value, field)).transpose()
}

fn first_metadata_value(metadata: &BTreeMap<String, String>, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        metadata.iter().find_map(|(key, value)| {
            if key.eq_ignore_ascii_case(name) {
                safe_metadata_text(value)
            } else {
                None
            }
        })
    })
}

fn safe_metadata_text(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.contains('\r') || value.contains('\n') {
        None
    } else {
        Some(value.to_owned())
    }
}

fn trace_from_metadata(metadata: &BTreeMap<String, String>) -> Option<W3CTraceContext> {
    trace_context_from_mapping(
        metadata
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
    )
}

fn correlate_trace(
    operation_context: OperationContext,
    trace_context: Option<&W3CTraceContext>,
) -> Result<OperationContext, BoundaryContextError> {
    match trace_context {
        Some(trace_context) => Ok(trace_context.to_operation_context(Some(operation_context))?),
        None => Ok(operation_context),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        BrokerContextOptions, BrokerOperation, OrchestratorContextOptions,
        broker_context_from_message, http_context_from_request, orchestrator_context_from_step,
        worker_context_from_task,
    };

    const TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

    #[test]
    fn http_uses_route_template_and_propagates_trace() {
        let headers = BTreeMap::from([
            ("X-Request-Id".to_owned(), "req-42".to_owned()),
            ("traceparent".to_owned(), TRACEPARENT.to_owned()),
        ]);

        let context =
            http_context_from_request("post", Some("/users/{id}"), &headers, Some("accounts"))
                .expect("HTTP context should be valid");

        assert_eq!(context.operation_context().system(), Some("http"));
        assert_eq!(
            context.operation_context().operation(),
            Some("POST /users/{id}")
        );
        assert_eq!(context.operation_context().request_id(), Some("req-42"));
        assert_eq!(
            context.operation_context().trace_id(),
            Some("4bf92f3577b34da6a3ce929d0e0e4736")
        );
        assert_eq!(context.operation_context().span_id(), None);
    }

    #[test]
    fn worker_prefers_explicit_identifiers() {
        let metadata = BTreeMap::from([
            ("task_id".to_owned(), "metadata-job".to_owned()),
            ("correlation_id".to_owned(), "metadata-corr".to_owned()),
        ]);

        let context = worker_context_from_task(
            "billing.settle_invoice",
            &metadata,
            Some("job-42"),
            Some("corr-9"),
            None,
            Some(2),
        )
        .expect("worker context should be valid");

        assert_eq!(context.operation_context().job_id(), Some("job-42"));
        assert_eq!(context.operation_context().correlation_id(), Some("corr-9"));
        assert_eq!(context.attempt(), Some(2));
    }

    #[test]
    fn broker_keeps_coordinates_out_of_operation_name() {
        let metadata = BTreeMap::new();
        let context = broker_context_from_message(
            BrokerOperation::Consume,
            "orders",
            &metadata,
            BrokerContextOptions {
                component: Some("billing"),
                correlation_id: Some("corr-9"),
                partition: Some(3),
                offset: Some(1042),
                consumer_group: Some("billing"),
                message_id: Some("msg-7"),
            },
        )
        .expect("broker context should be valid");

        assert_eq!(
            context.operation_context().operation(),
            Some("consume orders")
        );
        assert_eq!(context.partition(), Some(3));
        assert_eq!(context.offset(), Some(1042));
        assert_eq!(context.consumer_group(), Some("billing"));
        assert_eq!(context.message_id(), Some("msg-7"));
    }

    #[test]
    fn orchestrator_uses_stable_workflow_step_pair() {
        let metadata = BTreeMap::new();
        let context = orchestrator_context_from_step(
            "daily_etl",
            "load_customers",
            &metadata,
            OrchestratorContextOptions {
                run_id: Some("run-42"),
                step_run_id: Some("step-run-7"),
                component: None,
                correlation_id: Some("corr-9"),
                attempt: Some(1),
            },
        )
        .expect("orchestrator context should be valid");

        assert_eq!(
            context.operation_context().operation(),
            Some("daily_etl:load_customers")
        );
        assert_eq!(context.operation_context().job_id(), Some("run-42"));
        assert_eq!(context.step_run_id(), Some("step-run-7"));
        assert_eq!(context.attempt(), Some(1));
    }

    #[test]
    fn http_rejects_raw_query_routes() {
        let result = http_context_from_request("GET", Some("/users?id=42"), &BTreeMap::new(), None);

        assert!(result.is_err());
    }
}
