//! Typed transformation errors.

use std::{error::Error, fmt};

use serde_json::json;

use crate::{DataError, FailureMetadata};

const MODULE: &str = "dataexcept::transformation";

/// Raised when a named data-transformation step fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTransformationError {
    /// Transformation step name.
    pub step: String,
    /// Optional failure details.
    pub details: Option<String>,
}

impl DataTransformationError {
    /// Creates a data-transformation error.
    #[must_use]
    pub fn new(step: impl Into<String>, details: Option<String>) -> Self {
        Self {
            step: step.into(),
            details,
        }
    }
}

impl fmt::Display for DataTransformationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Data transformation '{}' failed", self.step)?;
        if let Some(details) = &self.details {
            write!(formatter, ": {details}")?;
        }
        Ok(())
    }
}

impl Error for DataTransformationError {}

impl From<DataTransformationError> for DataError {
    fn from(value: DataTransformationError) -> Self {
        let message = value.to_string();
        let mut error = Self::new("DataTransformationError", message)
            .with_module(MODULE)
            .with_attribute("step", json!(value.step))
            .with_failure(FailureMetadata::unknown());

        if let Some(details) = value.details {
            error = error.with_attribute("details", json!(details));
        }

        error
    }
}

/// Raised when a feature-engineering step fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureEngineeringError {
    /// Feature-engineering step name.
    pub step: String,
    /// Optional textual cause.
    pub cause: Option<String>,
}

impl FeatureEngineeringError {
    /// Creates a feature-engineering error.
    #[must_use]
    pub fn new(step: impl Into<String>, cause: Option<String>) -> Self {
        Self {
            step: step.into(),
            cause,
        }
    }
}

impl fmt::Display for FeatureEngineeringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Feature engineering failed at step '{}'",
            self.step
        )?;
        if let Some(cause) = &self.cause {
            write!(formatter, ": {cause}")?;
        }
        Ok(())
    }
}

impl Error for FeatureEngineeringError {}

impl From<FeatureEngineeringError> for DataError {
    fn from(value: FeatureEngineeringError) -> Self {
        let message = value.to_string();
        let mut error = Self::new("FeatureEngineeringError", message)
            .with_module(MODULE)
            .with_attribute("step", json!(value.step))
            .with_failure(FailureMetadata::unknown());

        if let Some(cause) = value.cause {
            error = error.with_attribute("cause", json!(cause));
        }

        error
    }
}

/// Raised when a normalization method fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataNormalizationError {
    /// Normalization method.
    pub method: String,
    /// Optional failure details.
    pub details: Option<String>,
}

impl DataNormalizationError {
    /// Creates a normalization error.
    #[must_use]
    pub fn new(method: impl Into<String>, details: Option<String>) -> Self {
        Self {
            method: method.into(),
            details,
        }
    }
}

impl fmt::Display for DataNormalizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Normalization using '{}' failed", self.method)?;
        if let Some(details) = &self.details {
            write!(formatter, ": {details}")?;
        }
        Ok(())
    }
}

impl Error for DataNormalizationError {}

impl From<DataNormalizationError> for DataError {
    fn from(value: DataNormalizationError) -> Self {
        let message = value.to_string();
        let mut error = Self::new("DataNormalizationError", message)
            .with_module(MODULE)
            .with_attribute("method", json!(value.method))
            .with_failure(FailureMetadata::unknown());

        if let Some(details) = value.details {
            error = error.with_attribute("details", json!(details));
        }

        error
    }
}

#[cfg(test)]
mod tests {
    use super::{DataNormalizationError, DataTransformationError, FeatureEngineeringError};
    use crate::DataError;

    #[test]
    fn transformation_error_carries_step_and_details() {
        let error = DataTransformationError::new(
            "currency_conversion",
            Some("invalid exchange rate".to_owned()),
        );

        let envelope = DataError::from(error).to_envelope();
        let attributes = envelope.attributes.expect("attributes should exist");

        assert_eq!(attributes["step"], "currency_conversion");
        assert_eq!(attributes["details"], "invalid exchange rate");
    }

    #[test]
    fn feature_engineering_error_has_stable_message() {
        let error = FeatureEngineeringError::new(
            "log_transform",
            Some("cannot take log of negative values".to_owned()),
        );

        assert_eq!(
            error.to_string(),
            "Feature engineering failed at step 'log_transform': cannot take log of negative values"
        );
    }

    #[test]
    fn normalization_error_has_method_attribute() {
        let error = DataNormalizationError::new(
            "StandardScaler",
            Some("division by zero in variance calculation".to_owned()),
        );

        let envelope = DataError::from(error).to_envelope();
        assert_eq!(
            envelope.attributes.expect("attributes should exist")["method"],
            "StandardScaler"
        );
    }
}
