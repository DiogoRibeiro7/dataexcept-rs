//! Typed message-broker operation errors.

use std::{error::Error, fmt};

use serde_json::json;

use crate::{DataError, FailureMetadata, redaction::redact_if_url};

const MODULE: &str = "dataexcept::broker";

/// Coordinates identifying a broker message or stream position.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MessagePosition {
    /// Topic, queue, or subject.
    pub topic: String,
    /// Optional partition identifier.
    pub partition: Option<i32>,
    /// Optional message offset.
    pub offset: Option<i64>,
    /// Optional consumer group.
    pub consumer_group: Option<String>,
}

impl MessagePosition {
    /// Creates a position with the required topic.
    #[must_use]
    pub fn new(topic: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            ..Self::default()
        }
    }

    /// Adds a partition identifier.
    #[must_use]
    pub const fn with_partition(mut self, partition: i32) -> Self {
        self.partition = Some(partition);
        self
    }

    /// Adds an offset.
    #[must_use]
    pub const fn with_offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Adds a consumer group.
    #[must_use]
    pub fn with_consumer_group(mut self, consumer_group: impl Into<String>) -> Self {
        self.consumer_group = Some(consumer_group.into());
        self
    }

    fn describe(&self) -> String {
        let mut parts = vec![format!("topic '{}'", self.topic)];
        if let Some(partition) = self.partition {
            parts.push(format!("partition {partition}"));
        }
        if let Some(offset) = self.offset {
            parts.push(format!("offset {offset}"));
        }
        if let Some(group) = &self.consumer_group {
            parts.push(format!("group '{group}'"));
        }
        parts.join(", ")
    }
}

fn broker_suffix(broker: Option<&str>) -> String {
    broker.map_or_else(String::new, |broker| format!(" on broker '{broker}'"))
}

fn add_position_attributes(mut error: DataError, position: MessagePosition) -> DataError {
    error = error.with_attribute("topic", json!(position.topic));
    if let Some(partition) = position.partition {
        error = error.with_attribute("partition", json!(partition));
    }
    if let Some(offset) = position.offset {
        error = error.with_attribute("offset", json!(offset));
    }
    if let Some(consumer_group) = position.consumer_group {
        error = error.with_attribute("consumer_group", json!(consumer_group));
    }
    error
}

/// Raised when a connection to a message broker cannot be established.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokerConnectionError {
    /// Broker bootstrap address, cluster name, or URL.
    pub broker: String,
    /// Optional textual cause.
    pub cause: Option<String>,
    message: String,
}

impl BrokerConnectionError {
    /// Creates a broker-connection error.
    #[must_use]
    pub fn new(broker: impl Into<String>, message: Option<String>, cause: Option<String>) -> Self {
        let broker = broker.into();
        let broker = redact_if_url(&broker, true);
        let mut default = format!("Failed to connect to message broker '{broker}'");
        if let Some(cause) = &cause {
            default.push_str(": ");
            default.push_str(cause);
        }

        Self {
            broker,
            cause,
            message: message.unwrap_or(default),
        }
    }
}

impl fmt::Display for BrokerConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for BrokerConnectionError {}

impl From<BrokerConnectionError> for DataError {
    fn from(value: BrokerConnectionError) -> Self {
        let mut error = Self::new("BrokerConnectionError", value.message)
            .with_module(MODULE)
            .with_attribute("broker", json!(value.broker))
            .with_failure(FailureMetadata::unknown());

        if let Some(cause) = value.cause {
            error = error.with_attribute("cause", json!(cause));
        }
        error
    }
}

/// Raised when a broker operation exceeds its time limit.
#[derive(Debug, Clone, PartialEq)]
pub struct BrokerTimeoutError {
    /// Broker being waited on.
    pub broker: String,
    /// Optional operation name.
    pub operation: Option<String>,
    /// Optional timeout limit in seconds.
    pub timeout_seconds: Option<f64>,
    /// Optional textual cause.
    pub cause: Option<String>,
    message: String,
}

impl BrokerTimeoutError {
    /// Creates a broker-timeout error.
    #[must_use]
    pub fn new(
        broker: impl Into<String>,
        operation: Option<String>,
        timeout_seconds: Option<f64>,
        message: Option<String>,
        cause: Option<String>,
    ) -> Self {
        let broker = broker.into();
        let broker = redact_if_url(&broker, true);
        let attempted = operation.as_ref().map_or_else(
            || "Broker operation".to_owned(),
            |operation| format!("Broker operation '{operation}'"),
        );
        let mut default = format!("{attempted} timed out");
        if let Some(timeout_seconds) = timeout_seconds {
            default.push_str(" after ");
            default.push_str(&timeout_seconds.to_string());
            default.push('s');
        }
        default.push_str(&broker_suffix(Some(&broker)));
        if let Some(cause) = &cause {
            default.push_str(": ");
            default.push_str(cause);
        }

        Self {
            broker,
            operation,
            timeout_seconds,
            cause,
            message: message.unwrap_or(default),
        }
    }
}

impl fmt::Display for BrokerTimeoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for BrokerTimeoutError {}

impl From<BrokerTimeoutError> for DataError {
    fn from(value: BrokerTimeoutError) -> Self {
        let mut error = Self::new("BrokerTimeoutError", value.message)
            .with_module(MODULE)
            .with_attribute("broker", json!(value.broker))
            .with_failure(FailureMetadata::unknown());

        if let Some(operation) = value.operation {
            error = error.with_attribute("operation", json!(operation));
        }
        if let Some(timeout_seconds) = value.timeout_seconds {
            error = error.with_attribute("timeout_seconds", json!(timeout_seconds));
        }
        if let Some(cause) = value.cause {
            error = error.with_attribute("cause", json!(cause));
        }
        error
    }
}

/// Raised when publishing a message fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessagePublishError {
    /// Message coordinates.
    pub position: MessagePosition,
    /// Optional broker.
    pub broker: Option<String>,
    /// Optional textual cause.
    pub cause: Option<String>,
    message: String,
}

impl MessagePublishError {
    /// Creates a publish error.
    #[must_use]
    pub fn new(
        position: MessagePosition,
        broker: Option<String>,
        message: Option<String>,
        cause: Option<String>,
    ) -> Self {
        let broker = broker.map(|broker| redact_if_url(&broker, true));
        let mut default = format!(
            "Failed to publish to {}{}",
            position.describe(),
            broker_suffix(broker.as_deref())
        );
        if let Some(cause) = &cause {
            default.push_str(": ");
            default.push_str(cause);
        }

        Self {
            position,
            broker,
            cause,
            message: message.unwrap_or(default),
        }
    }
}

impl fmt::Display for MessagePublishError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for MessagePublishError {}

impl From<MessagePublishError> for DataError {
    fn from(value: MessagePublishError) -> Self {
        let mut error = add_position_attributes(
            Self::new("MessagePublishError", value.message)
                .with_module(MODULE)
                .with_failure(FailureMetadata::unknown()),
            value.position,
        );
        if let Some(broker) = value.broker {
            error = error.with_attribute("broker", json!(broker));
        }
        if let Some(cause) = value.cause {
            error = error.with_attribute("cause", json!(cause));
        }
        error
    }
}

/// Raised when consuming a message fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageConsumeError {
    /// Message coordinates.
    pub position: MessagePosition,
    /// Optional broker.
    pub broker: Option<String>,
    /// Optional textual cause.
    pub cause: Option<String>,
    message: String,
}

impl MessageConsumeError {
    /// Creates a consume error.
    #[must_use]
    pub fn new(
        position: MessagePosition,
        broker: Option<String>,
        message: Option<String>,
        cause: Option<String>,
    ) -> Self {
        let broker = broker.map(|broker| redact_if_url(&broker, true));
        let mut default = format!(
            "Failed to consume from {}{}",
            position.describe(),
            broker_suffix(broker.as_deref())
        );
        if let Some(cause) = &cause {
            default.push_str(": ");
            default.push_str(cause);
        }

        Self {
            position,
            broker,
            cause,
            message: message.unwrap_or(default),
        }
    }
}

impl fmt::Display for MessageConsumeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for MessageConsumeError {}

impl From<MessageConsumeError> for DataError {
    fn from(value: MessageConsumeError) -> Self {
        let mut error = add_position_attributes(
            Self::new("MessageConsumeError", value.message)
                .with_module(MODULE)
                .with_failure(FailureMetadata::unknown()),
            value.position,
        );
        if let Some(broker) = value.broker {
            error = error.with_attribute("broker", json!(broker));
        }
        if let Some(cause) = value.cause {
            error = error.with_attribute("cause", json!(cause));
        }
        error
    }
}

/// Raised when acknowledging or committing a message fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageAcknowledgementError {
    /// Message coordinates.
    pub position: MessagePosition,
    /// Optional broker.
    pub broker: Option<String>,
    /// Optional textual cause.
    pub cause: Option<String>,
    message: String,
}

impl MessageAcknowledgementError {
    /// Creates an acknowledgement error.
    #[must_use]
    pub fn new(
        position: MessagePosition,
        broker: Option<String>,
        message: Option<String>,
        cause: Option<String>,
    ) -> Self {
        let broker = broker.map(|broker| redact_if_url(&broker, true));
        let mut default = format!(
            "Failed to acknowledge {}{}",
            position.describe(),
            broker_suffix(broker.as_deref())
        );
        if let Some(cause) = &cause {
            default.push_str(": ");
            default.push_str(cause);
        }

        Self {
            position,
            broker,
            cause,
            message: message.unwrap_or(default),
        }
    }
}

impl fmt::Display for MessageAcknowledgementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for MessageAcknowledgementError {}

impl From<MessageAcknowledgementError> for DataError {
    fn from(value: MessageAcknowledgementError) -> Self {
        let mut error = add_position_attributes(
            Self::new("MessageAcknowledgementError", value.message)
                .with_module(MODULE)
                .with_failure(FailureMetadata::unknown()),
            value.position,
        );
        if let Some(broker) = value.broker {
            error = error.with_attribute("broker", json!(broker));
        }
        if let Some(cause) = value.cause {
            error = error.with_attribute("cause", json!(cause));
        }
        error
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BrokerConnectionError, BrokerTimeoutError, MessageAcknowledgementError,
        MessageConsumeError, MessagePosition, MessagePublishError,
    };
    use crate::DataError;

    #[test]
    fn connection_error_redacts_url_credentials() {
        let error = BrokerConnectionError::new(
            "kafka://user:secret@broker.internal:9092",
            None,
            Some("connection refused".to_owned()),
        );

        assert!(!error.broker.contains("secret"));
        assert_eq!(
            error.to_string(),
            "Failed to connect to message broker 'kafka://***:***@broker.internal:9092': connection refused"
        );
    }

    #[test]
    fn timeout_error_preserves_operation_and_limit() {
        let error = BrokerTimeoutError::new(
            "broker-a",
            Some("publish".to_owned()),
            Some(5.0),
            None,
            None,
        );

        assert_eq!(
            error.to_string(),
            "Broker operation 'publish' timed out after 5s on broker 'broker-a'"
        );
    }

    #[test]
    fn publish_error_carries_topic_partition_and_broker() {
        let position = MessagePosition::new("orders").with_partition(3);
        let error = MessagePublishError::new(
            position,
            Some("kafka://broker.internal:9092".to_owned()),
            None,
            None,
        );

        let envelope = DataError::from(error).to_envelope();
        let attributes = envelope.attributes.expect("attributes should exist");

        assert_eq!(attributes["topic"], "orders");
        assert_eq!(attributes["partition"], 3);
        assert_eq!(attributes["broker"], "kafka://broker.internal:9092");
    }

    #[test]
    fn consume_error_carries_full_position() {
        let position = MessagePosition::new("orders")
            .with_partition(3)
            .with_offset(1042)
            .with_consumer_group("billing");
        let error = MessageConsumeError::new(position, None, None, None);

        assert_eq!(
            error.to_string(),
            "Failed to consume from topic 'orders', partition 3, offset 1042, group 'billing'"
        );
    }

    #[test]
    fn acknowledgement_error_remains_distinct_from_consume() {
        let position = MessagePosition::new("orders")
            .with_partition(3)
            .with_offset(1042);
        let error = MessageAcknowledgementError::new(position, None, None, None);
        let envelope = DataError::from(error).to_envelope();

        assert_eq!(envelope.error_type, "MessageAcknowledgementError");
        assert_eq!(
            envelope.attributes.expect("attributes should exist")["offset"],
            1042
        );
    }
}
