//! Correlates a `DataExcept` error with incoming W3C trace context.

use dataexcept::{DataError, ObservabilityEvent, OperationContext, parse_traceparent};

const TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

fn main() {
    let trace =
        parse_traceparent(TRACEPARENT, Some("vendor=value"), None).expect("valid traceparent");

    let operation = OperationContext::builder()
        .system("worker")
        .component("billing")
        .operation("billing.settle_invoice")
        .job_id("job-42")
        .correlation_id("corr-9")
        .build()
        .expect("valid operation context");

    let operation = trace
        .to_operation_context(Some(operation))
        .expect("trace identifiers should not conflict");

    let error = DataError::new("settlement_failed", "invoice settlement failed");
    let event = ObservabilityEvent::from_data_error(&error, Some(operation));

    println!(
        "{}",
        event
            .to_json()
            .expect("observability event should serialize")
    );
}
