//! Typed data-ingestion and validation errors.
//!
//! These types mirror the stable concepts exposed by the Python `DataExcept`
//! project while using ordinary Rust structs and `std::error::Error`.
//!
//! Conversion into [`DataError`] preserves the language-neutral envelope type
//! names and public attributes used by the Python implementation.

#![allow(clippy::module_name_repetitions)]

use std::{error::Error, fmt};

use serde_json::{Value, json};

use crate::{DataError, FailureMetadata, redaction::redact_if_url};

const MODULE: &str = "dataexcept::data";

/// Raised when loading data from a source fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLoadingError {
    /// File path, URL, or other source description.
    pub source: String,
    /// Human-readable description of the underlying failure.
    pub original: String,
}

impl DataLoadingError {
    /// Creates a data-loading error.
    #[must_use]
    pub fn new(source: impl Into<String>, original: impl Into<String>) -> Self {
        let source = source.into();

        Self {
            source: redact_if_url(&source, true),
            original: original.into(),
        }
    }

    /// Creates a data-loading error from an existing Rust error.
    #[must_use]
    pub fn from_error(source: impl Into<String>, error: &(impl Error + ?Sized)) -> Self {
        Self::new(source, error.to_string())
    }
}

impl fmt::Display for DataLoadingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Failed to load data from '{}': {}",
            self.source, self.original
        )
    }
}

impl Error for DataLoadingError {}

impl From<DataLoadingError> for DataError {
    fn from(value: DataLoadingError) -> Self {
        let message = value.to_string();

        Self::new("DataLoadingError", message)
            .with_module(MODULE)
            .with_attribute("source", json!(value.source))
            .with_attribute("original", json!(value.original))
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when input data has an unexpected format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataFormatError {
    /// Allowed format identifiers.
    pub expected_formats: Vec<String>,
    /// Format that was actually observed.
    pub found_format: String,
}

impl DataFormatError {
    /// Creates a data-format error.
    #[must_use]
    pub fn new<I, S>(expected_formats: I, found_format: impl Into<String>) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            expected_formats: expected_formats.into_iter().map(Into::into).collect(),
            found_format: found_format.into(),
        }
    }
}

impl fmt::Display for DataFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Expected data format {}; got {}",
            self.expected_formats.join(", "),
            self.found_format
        )
    }
}

impl Error for DataFormatError {}

impl From<DataFormatError> for DataError {
    fn from(value: DataFormatError) -> Self {
        let message = value.to_string();

        Self::new("DataFormatError", message)
            .with_module(MODULE)
            .with_attribute("expected_formats", json!(value.expected_formats))
            .with_attribute("found_format", json!(value.found_format))
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when a field or value fails a validation rule.
#[derive(Debug, Clone, PartialEq)]
pub struct DataValidationError {
    /// Name of the invalid field.
    pub field: String,
    /// Invalid value in a JSON-safe representation.
    pub value: Value,
    message: String,
}

impl DataValidationError {
    /// Creates a validation error using the standard DataExcept message.
    #[must_use]
    pub fn new(field: impl Into<String>, value: Value) -> Self {
        let field = field.into();
        let message = format!("Invalid value for '{field}': {value}");

        Self {
            field,
            value,
            message,
        }
    }

    /// Creates a validation error with a caller-supplied message.
    #[must_use]
    pub fn with_message(
        field: impl Into<String>,
        value: Value,
        message: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            value,
            message: message.into(),
        }
    }

    /// Returns the rendered validation message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for DataValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for DataValidationError {}

impl From<DataValidationError> for DataError {
    fn from(value: DataValidationError) -> Self {
        Self::new("DataValidationError", value.message)
            .with_module(MODULE)
            .with_attribute("field", json!(value.field))
            .with_attribute("value", value.value)
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when a required feature or value is missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingDataError {
    /// Name of the missing feature.
    pub feature: String,
    message: String,
}

impl MissingDataError {
    /// Creates a missing-data error using the standard DataExcept message.
    #[must_use]
    pub fn new(feature: impl Into<String>) -> Self {
        let feature = feature.into();
        let message = format!("Missing required feature: '{feature}'");

        Self { feature, message }
    }

    /// Creates a missing-data error with a caller-supplied message.
    #[must_use]
    pub fn with_message(feature: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            feature: feature.into(),
            message: message.into(),
        }
    }

    /// Returns the rendered missing-data message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for MissingDataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for MissingDataError {}

impl From<MissingDataError> for DataError {
    fn from(value: MissingDataError) -> Self {
        Self::new("MissingDataError", value.message)
            .with_module(MODULE)
            .with_attribute("feature", json!(value.feature))
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when an observed schema differs from the expected schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaMismatchError {
    /// Expected schema description.
    pub expected: String,
    /// Observed schema description.
    pub found: String,
}

impl SchemaMismatchError {
    /// Creates a schema-mismatch error.
    #[must_use]
    pub fn new(expected: impl Into<String>, found: impl Into<String>) -> Self {
        Self {
            expected: expected.into(),
            found: found.into(),
        }
    }
}

impl fmt::Display for SchemaMismatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Schema mismatch. Expected: {}, Found: {}",
            self.expected, self.found
        )
    }
}

impl Error for SchemaMismatchError {}

impl From<SchemaMismatchError> for DataError {
    fn from(value: SchemaMismatchError) -> Self {
        let message = value.to_string();

        Self::new("SchemaMismatchError", message)
            .with_module(MODULE)
            .with_attribute("expected", json!(value.expected))
            .with_attribute("found", json!(value.found))
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when a required tabular column is missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingColumnError {
    /// Missing column name.
    pub column: String,
    /// Optional DataFrame or table name.
    pub dataframe: Option<String>,
}

impl MissingColumnError {
    /// Creates a missing-column error.
    #[must_use]
    pub fn new(column: impl Into<String>, dataframe: Option<impl Into<String>>) -> Self {
        Self {
            column: column.into(),
            dataframe: dataframe.map(Into::into),
        }
    }
}

impl fmt::Display for MissingColumnError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Missing required column '{}'", self.column)?;
        if let Some(dataframe) = &self.dataframe {
            write!(formatter, " in DataFrame '{dataframe}'")?;
        }
        Ok(())
    }
}

impl Error for MissingColumnError {}

impl From<MissingColumnError> for DataError {
    fn from(value: MissingColumnError) -> Self {
        let message = value.to_string();
        let mut error = Self::new("MissingColumnError", message)
            .with_module(MODULE)
            .with_attribute("column", json!(value.column))
            .with_failure(FailureMetadata::unknown());

        if let Some(dataframe) = value.dataframe {
            error = error.with_attribute("dataframe", json!(dataframe));
        }

        error
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        DataFormatError, DataLoadingError, DataValidationError, MissingColumnError,
        MissingDataError, SchemaMismatchError,
    };
    use crate::{DataError, FailureKind};

    #[test]
    fn loading_error_redacts_source_credentials() {
        let error = DataLoadingError::new(
            "https://user:secret@example.com/data.csv?token=SECRETVALUE",
            "request failed",
        );

        assert!(!error.source.contains("secret"));
        assert!(!error.source.contains("SECRETVALUE"));
        assert!(error.source.contains("example.com"));
        assert!(error.source.contains("/data.csv"));
    }

    #[test]
    fn format_error_preserves_expected_formats() {
        let error = DataFormatError::new(["csv", "parquet"], "xml");

        assert_eq!(
            error.to_string(),
            "Expected data format csv, parquet; got xml"
        );

        let structured = DataError::from(error);
        let envelope = structured.to_envelope();

        assert_eq!(envelope.error_type, "DataFormatError");
        assert_eq!(
            envelope.attributes.expect("attributes should exist")["expected_formats"],
            json!(["csv", "parquet"])
        );
    }

    #[test]
    fn validation_error_matches_python_envelope_names() {
        let error = DataValidationError::new("age", json!(-1));
        let structured = DataError::from(error);
        let envelope = structured.to_envelope();

        assert_eq!(envelope.error_type, "DataValidationError");
        assert_eq!(envelope.module, "dataexcept::data");
        assert_eq!(
            envelope.attributes.expect("attributes should exist")["field"],
            "age"
        );
        assert_eq!(
            envelope.failure.expect("failure metadata should exist").kind,
            FailureKind::Unknown
        );
    }

    #[test]
    fn missing_data_has_standard_message() {
        let error = MissingDataError::new("income");
        assert_eq!(error.to_string(), "Missing required feature: 'income'");
    }

    #[test]
    fn schema_mismatch_has_stable_attributes() {
        let error = SchemaMismatchError::new("id: integer", "id: string");
        let envelope = DataError::from(error).to_envelope();
        let attributes = envelope.attributes.expect("attributes should exist");

        assert_eq!(attributes["expected"], "id: integer");
        assert_eq!(attributes["found"], "id: string");
    }

    #[test]
    fn missing_column_carries_optional_dataframe() {
        let error = MissingColumnError::new("customer_id", Some("sales"));
        assert_eq!(
            error.to_string(),
            "Missing required column 'customer_id' in DataFrame 'sales'"
        );

        let envelope = DataError::from(error).to_envelope();
        assert_eq!(
            envelope.attributes.expect("attributes should exist")["dataframe"],
            "sales"
        );
    }
}
