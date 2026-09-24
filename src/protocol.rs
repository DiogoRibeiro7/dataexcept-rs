//! Consumer-side types for the full language-neutral envelope protocol.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{ErrorEnvelope, FailureMetadata};

/// Version of the language-neutral `DataExcept` envelope schema.
pub const ENVELOPE_SCHEMA_VERSION: &str = "1.0.0";

/// Canonical identifier of the language-neutral `DataExcept` envelope schema.
pub const ENVELOPE_SCHEMA_ID: &str =
    "https://diogoribeiro7.github.io/DataExcept/schema/envelope-1.0.0.json";

/// A fully rendered exception record in the transport protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExceptionRecord {
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
    pub cause: Option<Box<EnvelopeNode>>,
    /// Implicit execution context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Box<EnvelopeNode>>,
    /// Members of a grouped error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exceptions: Option<Vec<EnvelopeNode>>,
}

impl From<ErrorEnvelope> for ExceptionRecord {
    fn from(value: ErrorEnvelope) -> Self {
        Self {
            error_type: value.error_type,
            module: value.module,
            message: value.message,
            attributes: value.attributes,
            failure: value.failure,
            cause: value.cause.map(|cause| Box::new(Self::from(*cause).into())),
            context: value
                .context
                .map(|context| Box::new(Self::from(*context).into())),
            exceptions: value.exceptions.map(|items| {
                items
                    .into_iter()
                    .map(|item| Self::from(item).into())
                    .collect()
            }),
        }
    }
}

/// A cycle marker emitted when an exception already exists on the active path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CycleRecord {
    /// Unqualified error type.
    #[serde(rename = "type")]
    pub error_type: String,
    /// Module or logical namespace defining the error.
    pub module: String,
    /// Human-readable error message.
    pub message: String,
    #[serde(rename = "cycle")]
    cycle: TrueMarker,
}

impl CycleRecord {
    /// Creates a cycle record.
    #[must_use]
    pub fn new(
        error_type: impl Into<String>,
        module: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            error_type: error_type.into(),
            module: module.into(),
            message: message.into(),
            cycle: TrueMarker,
        }
    }
}

/// Marker emitted when an envelope child exceeds the producer depth budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TruncationMarker {
    #[serde(rename = "truncated")]
    truncated: TrueMarker,
}

impl TruncationMarker {
    /// Creates a truncation marker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            truncated: TrueMarker,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct TrueMarker;

impl Serialize for TrueMarker {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(true)
    }
}

impl<'de> Deserialize<'de> for TrueMarker {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = bool::deserialize(deserializer)?;
        if value {
            Ok(Self)
        } else {
            Err(serde::de::Error::custom("marker value must be true"))
        }
    }
}

/// High-level kind of an envelope node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EnvelopeNodeKind {
    /// A fully rendered exception record.
    Exception,
    /// A cycle marker.
    Cycle,
    /// A depth-truncation marker.
    Truncated,
}

/// Any node permitted by the `DataExcept` envelope protocol.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum EnvelopeNode {
    /// A fully rendered exception record.
    Exception(ExceptionRecord),
    /// A cycle marker.
    Cycle(CycleRecord),
    /// A depth-truncation marker.
    Truncated(TruncationMarker),
}

impl EnvelopeNode {
    /// Returns the high-level node kind.
    #[must_use]
    pub const fn kind(&self) -> EnvelopeNodeKind {
        match self {
            Self::Exception(_) => EnvelopeNodeKind::Exception,
            Self::Cycle(_) => EnvelopeNodeKind::Cycle,
            Self::Truncated(_) => EnvelopeNodeKind::Truncated,
        }
    }

    /// Returns whether this is a full exception record.
    #[must_use]
    pub const fn is_exception(&self) -> bool {
        matches!(self, Self::Exception(_))
    }

    /// Returns whether this is a cycle marker.
    #[must_use]
    pub const fn is_cycle(&self) -> bool {
        matches!(self, Self::Cycle(_))
    }

    /// Returns the shared error type for exception and cycle nodes.
    #[must_use]
    pub fn error_type(&self) -> Option<&str> {
        match self {
            Self::Exception(record) => Some(&record.error_type),
            Self::Cycle(record) => Some(&record.error_type),
            Self::Truncated(_) => None,
        }
    }

    /// Returns the shared module for exception and cycle nodes.
    #[must_use]
    pub fn module(&self) -> Option<&str> {
        match self {
            Self::Exception(record) => Some(&record.module),
            Self::Cycle(record) => Some(&record.module),
            Self::Truncated(_) => None,
        }
    }

    /// Returns the shared message for exception and cycle nodes.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Exception(record) => Some(&record.message),
            Self::Cycle(record) => Some(&record.message),
            Self::Truncated(_) => None,
        }
    }

    /// Returns structured attributes for a full exception record.
    #[must_use]
    pub fn attributes(&self) -> Option<&BTreeMap<String, Value>> {
        self.as_exception()
            .and_then(|record| record.attributes.as_ref())
    }

    /// Returns failure metadata for a full exception record.
    #[must_use]
    pub fn failure(&self) -> Option<&FailureMetadata> {
        self.as_exception()
            .and_then(|record| record.failure.as_ref())
    }

    /// Returns the explicit cause for a full exception record.
    #[must_use]
    pub fn cause(&self) -> Option<&EnvelopeNode> {
        self.as_exception()
            .and_then(|record| record.cause.as_deref())
    }

    /// Returns the implicit context for a full exception record.
    #[must_use]
    pub fn context(&self) -> Option<&EnvelopeNode> {
        self.as_exception()
            .and_then(|record| record.context.as_deref())
    }

    /// Returns grouped child nodes for a full exception record.
    #[must_use]
    pub fn exceptions(&self) -> Option<&[EnvelopeNode]> {
        self.as_exception()
            .and_then(|record| record.exceptions.as_deref())
    }

    /// Parses a JSON envelope node.
    ///
    /// Marker nodes are exact. Ordinary exception records accept unknown
    /// fields so a 1.x consumer remains forward-compatible.
    ///
    /// # Errors
    ///
    /// Returns a JSON error when the input is invalid or has an invalid shape.
    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }

    /// Serializes the node as strict JSON.
    ///
    /// # Errors
    ///
    /// Returns a JSON error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Returns the exception record when present.
    #[must_use]
    pub const fn as_exception(&self) -> Option<&ExceptionRecord> {
        match self {
            Self::Exception(record) => Some(record),
            Self::Cycle(_) | Self::Truncated(_) => None,
        }
    }

    /// Returns the cycle record when present.
    #[must_use]
    pub const fn as_cycle(&self) -> Option<&CycleRecord> {
        match self {
            Self::Cycle(record) => Some(record),
            Self::Exception(_) | Self::Truncated(_) => None,
        }
    }

    /// Returns whether this is a truncation marker.
    #[must_use]
    pub const fn is_truncated(&self) -> bool {
        matches!(self, Self::Truncated(_))
    }
}

impl From<ErrorEnvelope> for EnvelopeNode {
    fn from(value: ErrorEnvelope) -> Self {
        Self::Exception(ExceptionRecord::from(value))
    }
}

impl From<ExceptionRecord> for EnvelopeNode {
    fn from(value: ExceptionRecord) -> Self {
        Self::Exception(value)
    }
}

impl From<CycleRecord> for EnvelopeNode {
    fn from(value: CycleRecord) -> Self {
        Self::Cycle(value)
    }
}

impl From<TruncationMarker> for EnvelopeNode {
    fn from(value: TruncationMarker) -> Self {
        Self::Truncated(value)
    }
}

impl<'de> Deserialize<'de> for EnvelopeNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let object = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("envelope node must be a JSON object"))?;

        if object.contains_key("truncated") {
            return serde_json::from_value::<TruncationMarker>(value)
                .map(Self::Truncated)
                .map_err(serde::de::Error::custom);
        }

        if object.contains_key("cycle") {
            return serde_json::from_value::<CycleRecord>(value)
                .map(Self::Cycle)
                .map_err(serde::de::Error::custom);
        }

        serde_json::from_value::<ExceptionRecord>(value)
            .map(Self::Exception)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CycleRecord, ENVELOPE_SCHEMA_ID, ENVELOPE_SCHEMA_VERSION, EnvelopeNode, EnvelopeNodeKind,
        TruncationMarker,
    };

    #[test]
    fn parses_all_protocol_node_shapes() {
        let exception = EnvelopeNode::from_json(r#"{"type":"E","module":"m","message":"x"}"#)
            .expect("exception record should parse");
        let cycle =
            EnvelopeNode::from_json(r#"{"type":"E","module":"m","message":"x","cycle":true}"#)
                .expect("cycle record should parse");
        let truncated = EnvelopeNode::from_json(r#"{"truncated":true}"#)
            .expect("truncation marker should parse");

        assert!(exception.as_exception().is_some());
        assert!(cycle.as_cycle().is_some());
        assert!(truncated.is_truncated());
    }

    #[test]
    fn exposes_typed_inspection_accessors() {
        let exception = EnvelopeNode::from_json(
            r#"{"type":"E","module":"m","message":"x","failure":{"kind":"unknown","retryable":null,"retry_after_seconds":null},"cause":{"truncated":true},"exceptions":[{"type":"Child","module":"m","message":"y"}]}"#,
        )
        .expect("exception record should parse");

        assert_eq!(exception.kind(), EnvelopeNodeKind::Exception);
        assert!(exception.is_exception());
        assert!(!exception.is_cycle());
        assert_eq!(exception.error_type(), Some("E"));
        assert_eq!(exception.module(), Some("m"));
        assert_eq!(exception.message(), Some("x"));
        assert!(exception.failure().is_some());
        assert!(exception.cause().is_some_and(EnvelopeNode::is_truncated));
        assert_eq!(exception.exceptions().map(<[EnvelopeNode]>::len), Some(1));

        let cycle = EnvelopeNode::from(CycleRecord::new("E", "m", "x"));
        assert_eq!(cycle.kind(), EnvelopeNodeKind::Cycle);
        assert!(cycle.is_cycle());
        assert_eq!(cycle.error_type(), Some("E"));
        assert!(cycle.failure().is_none());
        assert!(cycle.cause().is_none());

        let truncated = EnvelopeNode::from(TruncationMarker::new());
        assert_eq!(truncated.kind(), EnvelopeNodeKind::Truncated);
        assert_eq!(truncated.error_type(), None);
        assert_eq!(truncated.message(), None);
    }

    #[test]
    fn marker_shapes_are_strict() {
        assert!(
            EnvelopeNode::from_json(
                r#"{"type":"E","module":"m","message":"x","cycle":true,"extra":1}"#
            )
            .is_err()
        );
        assert!(EnvelopeNode::from_json(r#"{"truncated":true,"extra":1}"#).is_err());
        assert!(EnvelopeNode::from_json(r#"{"truncated":false}"#).is_err());
    }

    #[test]
    fn ordinary_records_accept_future_fields() {
        let node =
            EnvelopeNode::from_json(r#"{"type":"E","module":"m","message":"x","future":42}"#)
                .expect("unknown 1.x fields should be ignored");

        assert!(node.as_exception().is_some());
    }

    #[test]
    fn marker_nodes_round_trip() {
        let cycle = EnvelopeNode::from(CycleRecord::new("E", "m", "x"));
        let truncated = EnvelopeNode::from(TruncationMarker::new());

        assert_eq!(
            cycle.to_json().expect("cycle should serialize"),
            r#"{"type":"E","module":"m","message":"x","cycle":true}"#
        );
        assert_eq!(
            truncated.to_json().expect("marker should serialize"),
            r#"{"truncated":true}"#
        );
    }

    #[test]
    fn schema_constants_match_canonical_contract() {
        assert_eq!(ENVELOPE_SCHEMA_VERSION, "1.0.0");
        assert_eq!(
            ENVELOPE_SCHEMA_ID,
            "https://diogoribeiro7.github.io/DataExcept/schema/envelope-1.0.0.json"
        );
    }
}
