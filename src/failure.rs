//! Machine-readable recovery metadata.

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// Classification of the underlying failure condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FailureKind {
    /// The same operation may succeed later.
    Transient,
    /// The same operation and payload are expected to fail again.
    Permanent,
    /// The library has no justified classification.
    #[default]
    Unknown,
}

/// Recovery-relevant metadata attached to an operational error.
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct FailureMetadata {
    /// Whether the failure is transient, permanent, or unclassified.
    pub kind: FailureKind,
    /// Whether retrying the same operation can succeed.
    pub retryable: Option<bool>,
    /// Backend-provided retry delay, in seconds.
    pub retry_after_seconds: Option<f64>,
}

impl FailureMetadata {
    /// Creates validated failure metadata.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidFailureMetadata`] when `retry_after_seconds` is
    /// negative, NaN, or infinite.
    pub fn new(
        kind: FailureKind,
        retryable: Option<bool>,
        retry_after_seconds: Option<f64>,
    ) -> Result<Self, InvalidFailureMetadata> {
        if let Some(seconds) = retry_after_seconds {
            if !seconds.is_finite() || seconds < 0.0 {
                return Err(InvalidFailureMetadata::InvalidRetryAfter(seconds));
            }
        }

        Ok(Self {
            kind,
            retryable,
            retry_after_seconds,
        })
    }

    /// Metadata for an unclassified failure.
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            kind: FailureKind::Unknown,
            retryable: None,
            retry_after_seconds: None,
        }
    }

    /// Metadata for a transient failure.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidFailureMetadata`] when `retry_after_seconds` is
    /// negative, NaN, or infinite.
    pub fn transient(retry_after_seconds: Option<f64>) -> Result<Self, InvalidFailureMetadata> {
        Self::new(FailureKind::Transient, Some(true), retry_after_seconds)
    }

    /// Metadata for a permanent failure.
    #[must_use]
    pub const fn permanent() -> Self {
        Self {
            kind: FailureKind::Permanent,
            retryable: Some(false),
            retry_after_seconds: None,
        }
    }
}

impl<'de> Deserialize<'de> for FailureMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawFailureMetadata {
            kind: FailureKind,
            retryable: Option<bool>,
            retry_after_seconds: Option<f64>,
        }

        let raw = RawFailureMetadata::deserialize(deserializer)?;
        Self::new(raw.kind, raw.retryable, raw.retry_after_seconds)
            .map_err(serde::de::Error::custom)
    }
}

/// Validation failure for [`FailureMetadata`].
#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum InvalidFailureMetadata {
    /// Retry delay was not finite and non-negative.
    #[error("retry_after_seconds must be finite and non-negative, got {0}")]
    InvalidRetryAfter(f64),
}

#[cfg(test)]
mod tests {
    use super::{FailureKind, FailureMetadata};

    #[test]
    fn rejects_negative_retry_delay_during_deserialization() {
        let result = serde_json::from_str::<FailureMetadata>(
            r#"{"kind":"transient","retryable":true,"retry_after_seconds":-1.0}"#,
        );

        assert!(result.is_err());
    }

    #[test]
    fn accepts_valid_retry_delay_during_deserialization() {
        let metadata = serde_json::from_str::<FailureMetadata>(
            r#"{"kind":"transient","retryable":true,"retry_after_seconds":2.5}"#,
        )
        .expect("valid failure metadata should deserialize");

        assert_eq!(metadata.kind, FailureKind::Transient);
        assert_eq!(metadata.retryable, Some(true));
        assert_eq!(metadata.retry_after_seconds, Some(2.5));
    }

    #[test]
    fn rejects_negative_retry_delay() {
        let result = FailureMetadata::new(FailureKind::Transient, Some(true), Some(-1.0));
        assert!(result.is_err());
    }

    #[test]
    fn rejects_non_finite_retry_delay() {
        let result = FailureMetadata::transient(Some(f64::INFINITY));
        assert!(result.is_err());
    }

    #[test]
    fn permanent_failures_are_not_retryable() {
        let metadata = FailureMetadata::permanent();
        assert_eq!(metadata.kind, FailureKind::Permanent);
        assert_eq!(metadata.retryable, Some(false));
    }
}
