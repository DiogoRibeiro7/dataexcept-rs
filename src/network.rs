//! Typed network-operation errors.

use std::{error::Error, fmt};

use serde_json::json;

use crate::{DataError, FailureMetadata};

const MODULE: &str = "dataexcept::network";

/// Raised when a remote host cannot be reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostUnreachableError {
    /// Host address that could not be reached.
    pub host: String,
    message: String,
}

impl HostUnreachableError {
    /// Creates a host-unreachable error.
    #[must_use]
    pub fn new(host: impl Into<String>, message: Option<String>) -> Self {
        let host = host.into();
        let default = format!("Host '{host}' is unreachable");

        Self {
            host,
            message: message.unwrap_or(default),
        }
    }

    /// Returns the rendered message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for HostUnreachableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for HostUnreachableError {}

impl From<HostUnreachableError> for DataError {
    fn from(value: HostUnreachableError) -> Self {
        Self::new("HostUnreachableError", value.message)
            .with_module(MODULE)
            .with_attribute("host", json!(value.host))
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when a network connection attempt times out.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionTimeoutError {
    /// Host address.
    pub host: String,
    /// Timeout duration in seconds.
    pub timeout: f64,
}

impl ConnectionTimeoutError {
    /// Creates a connection-timeout error.
    #[must_use]
    pub fn new(host: impl Into<String>, timeout: f64) -> Self {
        Self {
            host: host.into(),
            timeout,
        }
    }
}

impl fmt::Display for ConnectionTimeoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Connection to '{}' timed out after {} seconds",
            self.host, self.timeout
        )
    }
}

impl Error for ConnectionTimeoutError {}

impl From<ConnectionTimeoutError> for DataError {
    fn from(value: ConnectionTimeoutError) -> Self {
        let message = value.to_string();

        Self::new("ConnectionTimeoutError", message)
            .with_module(MODULE)
            .with_attribute("host", json!(value.host))
            .with_attribute("timeout", json!(value.timeout))
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when an unexpected protocol error occurs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolError {
    /// Protocol name.
    pub protocol: String,
    /// Optional additional failure details.
    pub details: Option<String>,
}

impl ProtocolError {
    /// Creates a protocol error.
    #[must_use]
    pub fn new(protocol: impl Into<String>, details: Option<String>) -> Self {
        Self {
            protocol: protocol.into(),
            details,
        }
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Protocol error in {}", self.protocol)?;
        if let Some(details) = &self.details {
            write!(formatter, ": {details}")?;
        }
        Ok(())
    }
}

impl Error for ProtocolError {}

impl From<ProtocolError> for DataError {
    fn from(value: ProtocolError) -> Self {
        let message = value.to_string();
        let mut error = Self::new("ProtocolError", message)
            .with_module(MODULE)
            .with_attribute("protocol", json!(value.protocol))
            .with_failure(FailureMetadata::unknown());

        if let Some(details) = value.details {
            error = error.with_attribute("details", json!(details));
        }

        error
    }
}

#[cfg(test)]
mod tests {
    use super::{ConnectionTimeoutError, HostUnreachableError, ProtocolError};
    use crate::DataError;

    #[test]
    fn host_unreachable_has_standard_message() {
        let error = HostUnreachableError::new("api.example.com", None);
        assert_eq!(error.to_string(), "Host 'api.example.com' is unreachable");
    }

    #[test]
    fn timeout_error_preserves_seconds() {
        let error = ConnectionTimeoutError::new("api.example.com", 30.0);
        assert_eq!(
            error.to_string(),
            "Connection to 'api.example.com' timed out after 30 seconds"
        );

        let envelope = DataError::from(error).to_envelope();
        assert_eq!(
            envelope.attributes.expect("attributes should exist")["timeout"],
            30.0
        );
    }

    #[test]
    fn protocol_error_carries_optional_details() {
        let error = ProtocolError::new("HTTP", Some("Invalid status line".to_owned()));

        assert_eq!(
            error.to_string(),
            "Protocol error in HTTP: Invalid status line"
        );

        let envelope = DataError::from(error).to_envelope();
        let attributes = envelope.attributes.expect("attributes should exist");

        assert_eq!(attributes["protocol"], "HTTP");
        assert_eq!(attributes["details"], "Invalid status line");
    }
}
