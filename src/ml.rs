//! Typed machine-learning training, evaluation, and inference errors.
//!
//! These types mirror the stable concepts exposed by the Python `DataExcept`
//! project while remaining idiomatic Rust errors.

#![allow(clippy::module_name_repetitions)]

use std::{error::Error, fmt};

use serde_json::{Value, json};

use crate::{DataError, FailureMetadata};

const MODULE: &str = "dataexcept::ml";

/// Raised when model training fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelTrainingError {
    /// Model class or identifier.
    pub model_type: String,
    /// Optional epoch index where the failure occurred.
    pub epoch: Option<u64>,
    message: String,
}

impl ModelTrainingError {
    /// Creates a model-training error.
    #[must_use]
    pub fn new(
        model_type: impl Into<String>,
        epoch: Option<u64>,
        message: Option<String>,
    ) -> Self {
        let model_type = model_type.into();
        let default = epoch.map_or_else(
            || format!("Training failed for model '{model_type}'"),
            |epoch| format!("Training failed for model '{model_type}' at epoch {epoch}"),
        );

        Self {
            model_type,
            epoch,
            message: message.unwrap_or(default),
        }
    }

    /// Returns the rendered message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ModelTrainingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ModelTrainingError {}

impl From<ModelTrainingError> for DataError {
    fn from(value: ModelTrainingError) -> Self {
        let mut error = Self::new("ModelTrainingError", value.message)
            .with_module(MODULE)
            .with_attribute("model_type", json!(value.model_type))
            .with_failure(FailureMetadata::unknown());

        if let Some(epoch) = value.epoch {
            error = error.with_attribute("epoch", json!(epoch));
        }

        error
    }
}

/// Raised when model optimization fails to converge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvergenceError {
    /// Model class or identifier.
    pub model_type: String,
    /// Number of optimization iterations attempted.
    pub iterations: u64,
    message: String,
}

impl ConvergenceError {
    /// Creates a convergence error.
    #[must_use]
    pub fn new(
        model_type: impl Into<String>,
        iterations: u64,
        message: Option<String>,
    ) -> Self {
        let model_type = model_type.into();
        let default =
            format!("Model '{model_type}' failed to converge after {iterations} iterations");

        Self {
            model_type,
            iterations,
            message: message.unwrap_or(default),
        }
    }
}

impl fmt::Display for ConvergenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ConvergenceError {}

impl From<ConvergenceError> for DataError {
    fn from(value: ConvergenceError) -> Self {
        Self::new("ConvergenceError", value.message)
            .with_module(MODULE)
            .with_attribute("model_type", json!(value.model_type))
            .with_attribute("iterations", json!(value.iterations))
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when model training exceeds its time limit.
#[derive(Debug, Clone, PartialEq)]
pub struct TrainingTimeoutError {
    /// Model class or identifier.
    pub model_type: String,
    /// Timeout limit in seconds.
    pub timeout: f64,
}

impl TrainingTimeoutError {
    /// Creates a training-timeout error.
    #[must_use]
    pub fn new(model_type: impl Into<String>, timeout: f64) -> Self {
        Self {
            model_type: model_type.into(),
            timeout,
        }
    }
}

impl fmt::Display for TrainingTimeoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Training '{}' exceeded timeout of {} seconds",
            self.model_type, self.timeout
        )
    }
}

impl Error for TrainingTimeoutError {}

impl From<TrainingTimeoutError> for DataError {
    fn from(value: TrainingTimeoutError) -> Self {
        let message = value.to_string();

        Self::new("TrainingTimeoutError", message)
            .with_module(MODULE)
            .with_attribute("model_type", json!(value.model_type))
            .with_attribute("timeout", json!(value.timeout))
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised for an invalid hyperparameter setting.
#[derive(Debug, Clone, PartialEq)]
pub struct HyperparameterError {
    /// Hyperparameter name.
    pub param: String,
    /// Invalid value in JSON-safe form.
    pub value: Value,
    message: String,
}

impl HyperparameterError {
    /// Creates a hyperparameter error.
    #[must_use]
    pub fn new(param: impl Into<String>, value: Value, message: Option<String>) -> Self {
        let param = param.into();
        let default = format!("Invalid hyperparameter '{param}': {value}");

        Self {
            param,
            value,
            message: message.unwrap_or(default),
        }
    }
}

impl fmt::Display for HyperparameterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for HyperparameterError {}

impl From<HyperparameterError> for DataError {
    fn from(value: HyperparameterError) -> Self {
        Self::new("HyperparameterError", value.message)
            .with_module(MODULE)
            .with_attribute("param", json!(value.param))
            .with_attribute("value", value.value)
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when evaluation-metric computation fails.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelEvaluationError {
    /// Metric name.
    pub metric: String,
    /// Computed or observed value.
    pub value: f64,
    message: String,
}

impl ModelEvaluationError {
    /// Creates a model-evaluation error.
    #[must_use]
    pub fn new(metric: impl Into<String>, value: f64, message: Option<String>) -> Self {
        let metric = metric.into();
        let default = format!("Failed to compute metric '{metric}', got {value}");

        Self {
            metric,
            value,
            message: message.unwrap_or(default),
        }
    }
}

impl fmt::Display for ModelEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ModelEvaluationError {}

impl From<ModelEvaluationError> for DataError {
    fn from(value: ModelEvaluationError) -> Self {
        Self::new("ModelEvaluationError", value.message)
            .with_module(MODULE)
            .with_attribute("metric", json!(value.metric))
            .with_attribute("value", json!(value.value))
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when making a prediction fails.
#[derive(Debug, Clone, PartialEq)]
pub struct PredictionError {
    /// Model class or identifier.
    pub model_type: String,
    /// Input snapshot in JSON-safe form.
    pub inputs: Value,
    message: String,
}

impl PredictionError {
    /// Creates a prediction error.
    #[must_use]
    pub fn new(model_type: impl Into<String>, inputs: Value, message: Option<String>) -> Self {
        let model_type = model_type.into();
        let default = format!("Prediction failed for model '{model_type}' with inputs {inputs}");

        Self {
            model_type,
            inputs,
            message: message.unwrap_or(default),
        }
    }
}

impl fmt::Display for PredictionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for PredictionError {}

impl From<PredictionError> for DataError {
    fn from(value: PredictionError) -> Self {
        Self::new("PredictionError", value.message)
            .with_module(MODULE)
            .with_attribute("model_type", json!(value.model_type))
            .with_attribute("inputs", value.inputs)
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when model inference fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInferenceError {
    /// Model class or identifier.
    pub model_type: String,
    /// Human-readable description of the underlying failure.
    pub original: String,
}

impl ModelInferenceError {
    /// Creates a model-inference error.
    #[must_use]
    pub fn new(model_type: impl Into<String>, original: impl Into<String>) -> Self {
        Self {
            model_type: model_type.into(),
            original: original.into(),
        }
    }

    /// Creates a model-inference error from an existing Rust error.
    #[must_use]
    pub fn from_error(model_type: impl Into<String>, error: &(impl Error + ?Sized)) -> Self {
        Self::new(model_type, error.to_string())
    }
}

impl fmt::Display for ModelInferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Inference failed for model '{}': {}",
            self.model_type, self.original
        )
    }
}

impl Error for ModelInferenceError {}

impl From<ModelInferenceError> for DataError {
    fn from(value: ModelInferenceError) -> Self {
        let message = value.to_string();

        Self::new("ModelInferenceError", message)
            .with_module(MODULE)
            .with_attribute("model_type", json!(value.model_type))
            .with_attribute("original", json!(value.original))
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when cross-validation fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossValidationError {
    /// Number of folds.
    pub folds: u32,
    /// Optional textual cause.
    pub cause: Option<String>,
}

impl CrossValidationError {
    /// Creates a cross-validation error.
    #[must_use]
    pub fn new(folds: u32, cause: Option<String>) -> Self {
        Self { folds, cause }
    }
}

impl fmt::Display for CrossValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Cross-validation failed on {} folds", self.folds)?;
        if let Some(cause) = &self.cause {
            write!(formatter, ": {cause}")?;
        }
        Ok(())
    }
}

impl Error for CrossValidationError {}

impl From<CrossValidationError> for DataError {
    fn from(value: CrossValidationError) -> Self {
        let message = value.to_string();
        let mut error = Self::new("CrossValidationError", message)
            .with_module(MODULE)
            .with_attribute("folds", json!(value.folds))
            .with_failure(FailureMetadata::unknown());

        if let Some(cause) = value.cause {
            error = error.with_attribute("cause", json!(cause));
        }

        error
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ConvergenceError, CrossValidationError, HyperparameterError, ModelEvaluationError,
        ModelInferenceError, ModelTrainingError, PredictionError, TrainingTimeoutError,
    };
    use crate::DataError;

    #[test]
    fn training_error_includes_epoch() {
        let error = ModelTrainingError::new("XGBoost", Some(12), None);

        assert_eq!(
            error.to_string(),
            "Training failed for model 'XGBoost' at epoch 12"
        );

        let envelope = DataError::from(error).to_envelope();
        let attributes = envelope.attributes.expect("attributes should exist");

        assert_eq!(attributes["model_type"], "XGBoost");
        assert_eq!(attributes["epoch"], 12);
    }

    #[test]
    fn convergence_error_preserves_iterations() {
        let error = ConvergenceError::new("LogisticRegression", 1000, None);
        let envelope = DataError::from(error).to_envelope();

        assert_eq!(envelope.error_type, "ConvergenceError");
        assert_eq!(
            envelope.attributes.expect("attributes should exist")["iterations"],
            1000
        );
    }

    #[test]
    fn training_timeout_has_stable_message() {
        let error = TrainingTimeoutError::new("RandomForest", 30.0);

        assert_eq!(
            error.to_string(),
            "Training 'RandomForest' exceeded timeout of 30 seconds"
        );
    }

    #[test]
    fn hyperparameter_error_preserves_json_value() {
        let error = HyperparameterError::new("max_depth", json!(-1), None);
        let envelope = DataError::from(error).to_envelope();
        let attributes = envelope.attributes.expect("attributes should exist");

        assert_eq!(attributes["param"], "max_depth");
        assert_eq!(attributes["value"], -1);
    }

    #[test]
    fn evaluation_error_preserves_metric_name() {
        let error = ModelEvaluationError::new("roc_auc", 0.51, None);
        let envelope = DataError::from(error).to_envelope();

        assert_eq!(
            envelope.attributes.expect("attributes should exist")["metric"],
            "roc_auc"
        );
    }

    #[test]
    fn prediction_error_carries_input_snapshot() {
        let error = PredictionError::new(
            "LinearRegression",
            json!({"age": 42, "income": 51000}),
            None,
        );
        let envelope = DataError::from(error).to_envelope();

        assert_eq!(
            envelope.attributes.expect("attributes should exist")["inputs"]["age"],
            42
        );
    }

    #[test]
    fn inference_error_preserves_original_message() {
        let error = ModelInferenceError::new("Transformer", "tensor shape mismatch");

        assert_eq!(
            error.to_string(),
            "Inference failed for model 'Transformer': tensor shape mismatch"
        );
    }

    #[test]
    fn cross_validation_error_preserves_folds_and_cause() {
        let error =
            CrossValidationError::new(5, Some("stratification impossible".to_owned()));
        let envelope = DataError::from(error).to_envelope();
        let attributes = envelope.attributes.expect("attributes should exist");

        assert_eq!(attributes["folds"], 5);
        assert_eq!(attributes["cause"], "stratification impossible");
    }
}
