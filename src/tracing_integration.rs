//! Optional integration with the `tracing` ecosystem.
//!
//! The adapter emits a structured error event and does not install or configure
//! a subscriber. Applications retain full control over filtering, formatting,
//! layers, exporters, and runtime policy.

use crate::{DataError, OperationContext};

/// Emits a structured DataExcept error event through `tracing`.
///
/// The event includes the stable error code, rendered message, serialized
/// envelope, and low-cardinality operation fields when available. Correlation
/// identifiers remain in the serialized operation context rather than being
/// promoted to indexed fields.
///
/// This function is available only when the `tracing` feature is enabled.
pub fn emit_error(error: &DataError, operation: Option<&OperationContext>) {
    let envelope = error.to_envelope();
    let envelope_json = envelope
        .to_json()
        .unwrap_or_else(|_| "{"type":"serialization_error"}".to_owned());

    let operation_json = operation
        .and_then(|context| serde_json::to_string(context).ok())
        .unwrap_or_else(|| "{}".to_owned());

    let system = operation.and_then(OperationContext::system).unwrap_or("");
    let component = operation
        .and_then(OperationContext::component)
        .unwrap_or("");
    let operation_name = operation
        .and_then(OperationContext::operation)
        .unwrap_or("");

    tracing::event!(
        tracing::Level::ERROR,
        dataexcept.code = error.code(),
        dataexcept.message = %error,
        dataexcept.envelope = %envelope_json,
        dataexcept.operation = %operation_json,
        dataexcept.system = system,
        dataexcept.component = component,
        dataexcept.operation_name = operation_name,
        "dataexcept error"
    );
}

#[cfg(test)]
mod tests {
    use super::emit_error;
    use crate::{DataError, OperationContext};

    #[test]
    fn emit_error_accepts_context_and_no_context() {
        let error = DataError::new("example", "failed");
        let context = OperationContext::builder()
            .system("worker")
            .component("billing")
            .operation("settle_invoice")
            .build()
            .expect("operation context should be valid");

        emit_error(&error, Some(&context));
        emit_error(&error, None);
    }
}
