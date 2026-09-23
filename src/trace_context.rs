//! W3C Trace Context parsing without a tracing-runtime dependency.
//!
//! The parser keeps failures correlated across execution boundaries without
//! starting spans or inventing identifiers. Incoming carrier values are
//! preserved for forwarding while parsed identifiers can enrich
//! [`OperationContext`](crate::OperationContext).

use std::collections::BTreeMap;

use thiserror::Error;

use crate::OperationContext;

const ZERO_TRACE_ID: &str = "00000000000000000000000000000000";
const ZERO_PARENT_ID: &str = "0000000000000000";

/// Parsed W3C `traceparent` plus optional propagation companions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct W3CTraceContext {
    traceparent: String,
    version: String,
    trace_id: String,
    parent_id: String,
    trace_flags: String,
    tracestate: Option<String>,
    baggage: Option<String>,
}

impl W3CTraceContext {
    /// Returns the original `traceparent` value.
    #[must_use]
    pub fn traceparent(&self) -> &str {
        &self.traceparent
    }

    /// Returns the W3C trace-context version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the trace identifier.
    #[must_use]
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Returns the incoming caller span identifier.
    #[must_use]
    pub fn parent_id(&self) -> &str {
        &self.parent_id
    }

    /// Returns the trace flags field.
    #[must_use]
    pub fn trace_flags(&self) -> &str {
        &self.trace_flags
    }

    /// Returns `tracestate` when safe to forward.
    #[must_use]
    pub fn tracestate(&self) -> Option<&str> {
        self.tracestate.as_deref()
    }

    /// Returns `baggage` when safe to forward.
    #[must_use]
    pub fn baggage(&self) -> Option<&str> {
        self.baggage.as_deref()
    }

    /// Returns whether the W3C sampled bit is set.
    #[must_use]
    pub fn sampled(&self) -> bool {
        u8::from_str_radix(&self.trace_flags, 16).is_ok_and(|flags| flags & 0x01 != 0)
    }

    /// Returns propagation fields without rewriting their received values.
    #[must_use]
    pub fn to_carrier(&self) -> BTreeMap<&'static str, String> {
        let mut carrier = BTreeMap::from([("traceparent", self.traceparent.clone())]);
        if let Some(tracestate) = &self.tracestate {
            carrier.insert("tracestate", tracestate.clone());
        }
        if let Some(baggage) = &self.baggage {
            carrier.insert("baggage", baggage.clone());
        }
        carrier
    }

    /// Correlates an operation context with this trace without inventing a span.
    ///
    /// # Errors
    ///
    /// Returns [`TraceContextConflict`] when an existing trace ID conflicts.
    pub fn to_operation_context(
        &self,
        context: Option<OperationContext>,
    ) -> Result<OperationContext, TraceContextConflict> {
        let context = context.unwrap_or_default();

        if let Some(existing) = context.trace_id() {
            if existing != self.trace_id {
                return Err(TraceContextConflict::TraceId {
                    existing: existing.to_owned(),
                    incoming: self.trace_id.clone(),
                });
            }
            return Ok(context);
        }

        Ok(context.with_trace_id_from_carrier(self.trace_id.clone()))
    }
}

/// Conflict between existing provenance and incoming trace context.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum TraceContextConflict {
    /// The operation context and incoming carrier disagree about the trace ID.
    #[error("operation context trace_id {existing} conflicts with incoming trace_id {incoming}")]
    TraceId {
        /// Existing operation-context trace ID.
        existing: String,
        /// Incoming W3C trace ID.
        incoming: String,
    },
}

/// Parses a W3C `traceparent`, returning [`None`] when invalid.
///
/// Version `00` must use the exact 55-character format. Higher versions are
/// accepted when the mandatory prefix is valid; opaque extension data remains
/// verbatim. Invalid identifiers are never replaced with generated values.
#[must_use]
pub fn parse_traceparent(
    value: &str,
    tracestate: Option<&str>,
    baggage: Option<&str>,
) -> Option<W3CTraceContext> {
    if !value.is_ascii() || value.trim() != value || value.len() < 55 {
        return None;
    }

    let bytes = value.as_bytes();
    if bytes[2] != b'-' || bytes[35] != b'-' || bytes[52] != b'-' {
        return None;
    }

    let version = &value[..2];
    let trace_id = &value[3..35];
    let parent_id = &value[36..52];
    let trace_flags = &value[53..55];

    if !mandatory_fields_are_valid(version, trace_id, parent_id, trace_flags) {
        return None;
    }
    if !version_shape_is_valid(value, version) {
        return None;
    }

    Some(W3CTraceContext {
        traceparent: value.to_owned(),
        version: version.to_owned(),
        trace_id: trace_id.to_owned(),
        parent_id: parent_id.to_owned(),
        trace_flags: trace_flags.to_owned(),
        tracestate: safe_optional_header(tracestate),
        baggage: safe_optional_header(baggage),
    })
}

/// Extracts W3C trace context from case-insensitive key-value pairs.
///
/// Invalid `tracestate` and `baggage` values are dropped without invalidating
/// a valid `traceparent`.
#[must_use]
pub fn trace_context_from_mapping<I, K, V>(carrier: I) -> Option<W3CTraceContext>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut traceparent = None;
    let mut tracestate = None;
    let mut baggage = None;

    for (key, value) in carrier {
        let key = key.as_ref();
        let value = value.as_ref();

        if key.eq_ignore_ascii_case("traceparent") {
            traceparent = Some(value.to_owned());
        } else if key.eq_ignore_ascii_case("tracestate") {
            tracestate = Some(value.to_owned());
        } else if key.eq_ignore_ascii_case("baggage") {
            baggage = Some(value.to_owned());
        }
    }

    let tracestate = safe_optional_header(tracestate.as_deref());
    let baggage = safe_optional_header(baggage.as_deref());

    parse_traceparent(
        traceparent.as_deref()?,
        tracestate.as_deref(),
        baggage.as_deref(),
    )
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn mandatory_fields_are_valid(
    version: &str,
    trace_id: &str,
    parent_id: &str,
    trace_flags: &str,
) -> bool {
    is_lower_hex(version, 2)
        && version != "ff"
        && is_lower_hex(trace_id, 32)
        && trace_id != ZERO_TRACE_ID
        && is_lower_hex(parent_id, 16)
        && parent_id != ZERO_PARENT_ID
        && is_lower_hex(trace_flags, 2)
}

fn version_shape_is_valid(value: &str, version: &str) -> bool {
    if version == "00" {
        value.len() == 55
    } else {
        value.len() == 55 || value.as_bytes().get(55) == Some(&b'-')
    }
}

fn safe_optional_header(value: Option<&str>) -> Option<String> {
    value
        .filter(|value| !value.contains('\r') && !value.contains('\n'))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::{TraceContextConflict, parse_traceparent, trace_context_from_mapping};
    use crate::OperationContext;

    const TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

    #[test]
    fn parses_valid_version_zero_traceparent() {
        let trace = parse_traceparent(TRACEPARENT, Some("vendor=value"), Some("tenant=acme"))
            .expect("traceparent should be valid");

        assert_eq!(trace.version(), "00");
        assert_eq!(trace.trace_id(), "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(trace.parent_id(), "00f067aa0ba902b7");
        assert_eq!(trace.trace_flags(), "01");
        assert!(trace.sampled());
        assert_eq!(trace.to_carrier()["tracestate"], "vendor=value");
    }

    #[test]
    fn rejects_zero_and_uppercase_identifiers() {
        assert!(
            parse_traceparent(
                "00-00000000000000000000000000000000-00f067aa0ba902b7-01",
                None,
                None
            )
            .is_none()
        );
        assert!(
            parse_traceparent(
                "00-4BF92F3577B34DA6A3CE929D0E0E4736-00f067aa0ba902b7-01",
                None,
                None
            )
            .is_none()
        );
    }

    #[test]
    fn version_zero_rejects_extensions() {
        let extended = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-extra";

        assert!(parse_traceparent(extended, None, None).is_none());
    }

    #[test]
    fn future_version_preserves_opaque_extension() {
        let extended = "01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-future";

        let trace = parse_traceparent(extended, None, None)
            .expect("future-version traceparent should preserve extension");

        assert_eq!(trace.traceparent(), extended);
        assert_eq!(trace.version(), "01");
    }

    #[test]
    fn unsafe_optional_headers_are_dropped() {
        let trace = parse_traceparent(
            TRACEPARENT,
            Some("vendor=value\r\ninjected=yes"),
            Some("tenant=acme\nadmin=true"),
        )
        .expect("traceparent should remain valid");

        assert_eq!(trace.tracestate(), None);
        assert_eq!(trace.baggage(), None);
    }

    #[test]
    fn mapping_extraction_is_case_insensitive() {
        let trace = trace_context_from_mapping([
            ("TraceParent", TRACEPARENT),
            ("TraceState", "vendor=value"),
            ("Baggage", "tenant=acme"),
        ])
        .expect("carrier should contain valid trace context");

        assert_eq!(trace.trace_id(), "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(trace.tracestate()(), Some("vendor=value"));
        assert_eq!(trace.baggage()(), Some("tenant=acme"));
    }

    #[test]
    fn operation_context_gets_trace_id_without_parent_span() {
        let trace =
            parse_traceparent(TRACEPARENT, None, None).expect("traceparent should be valid");
        let context = OperationContext::builder()
            .system("worker")
            .operation("billing.settle_invoice")
            .build()
            .expect("operation context should be valid");

        let correlated = trace
            .to_operation_context(Some(context))
            .expect("trace IDs should not conflict");

        assert_eq!(correlated.system(), Some("worker"));
        assert_eq!(
            correlated.trace_id(),
            Some("4bf92f3577b34da6a3ce929d0e0e4736")
        );
        assert_eq!(correlated.span_id(), None);
    }

    #[test]
    fn conflicting_trace_id_is_rejected() {
        let trace =
            parse_traceparent(TRACEPARENT, None, None).expect("traceparent should be valid");
        let context = OperationContext::builder()
            .trace_id("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .build()
            .expect("operation context should be valid");

        let result = trace.to_operation_context(Some(context));

        assert_eq!(
            result,
            Err(TraceContextConflict::TraceId {
                existing: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
                incoming: "4bf92f3577b34da6a3ce929d0e0e4736".to_owned(),
            })
        );
    }
}
