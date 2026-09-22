//! Typed schema and tabular-operation errors.

use std::{error::Error, fmt};

use serde_json::json;

use crate::{DataError, FailureMetadata};

const MODULE: &str = "dataexcept::schema";

/// Raised when a column has an unexpected data type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DtypeMismatchError {
    /// Column name.
    pub column: String,
    /// Allowed data types.
    pub expected: Vec<String>,
    /// Observed data type.
    pub found: String,
}

impl DtypeMismatchError {
    /// Creates a data-type mismatch error.
    #[must_use]
    pub fn new<I, S>(column: impl Into<String>, expected: I, found: impl Into<String>) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            column: column.into(),
            expected: expected.into_iter().map(Into::into).collect(),
            found: found.into(),
        }
    }
}

impl fmt::Display for DtypeMismatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Column '{}' has dtype {}; expected {}",
            self.column,
            self.found,
            self.expected.join(", ")
        )
    }
}

impl Error for DtypeMismatchError {}

impl From<DtypeMismatchError> for DataError {
    fn from(value: DtypeMismatchError) -> Self {
        let message = value.to_string();

        Self::new("DtypeMismatchError", message)
            .with_module(MODULE)
            .with_attribute("column", json!(value.column))
            .with_attribute("expected", json!(value.expected))
            .with_attribute("found", json!(value.found))
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when indices are misaligned for a tabular operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexAlignmentError {
    /// Optional details about the misalignment.
    pub details: Option<String>,
}

impl IndexAlignmentError {
    /// Creates an index-alignment error.
    #[must_use]
    pub fn new(details: Option<String>) -> Self {
        Self { details }
    }
}

impl fmt::Display for IndexAlignmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DataFrame indices are misaligned")?;
        if let Some(details) = &self.details {
            write!(formatter, ": {details}")?;
        }
        Ok(())
    }
}

impl Error for IndexAlignmentError {}

impl From<IndexAlignmentError> for DataError {
    fn from(value: IndexAlignmentError) -> Self {
        let message = value.to_string();
        let mut error = Self::new("IndexAlignmentError", message)
            .with_module(MODULE)
            .with_failure(FailureMetadata::unknown());

        if let Some(details) = value.details {
            error = error.with_attribute("details", json!(details));
        }

        error
    }
}

/// Raised when a merge fails because the left and right keys do not align.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeKeyError {
    /// Keys supplied by the left-hand table.
    pub left_keys: Vec<String>,
    /// Keys supplied by the right-hand table.
    pub right_keys: Vec<String>,
}

impl MergeKeyError {
    /// Creates a merge-key error.
    #[must_use]
    pub fn new<L, R, LS, RS>(left_keys: L, right_keys: R) -> Self
    where
        L: IntoIterator<Item = LS>,
        R: IntoIterator<Item = RS>,
        LS: Into<String>,
        RS: Into<String>,
    {
        Self {
            left_keys: left_keys.into_iter().map(Into::into).collect(),
            right_keys: right_keys.into_iter().map(Into::into).collect(),
        }
    }
}

impl fmt::Display for MergeKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Failed to merge on keys {:?} and {:?}",
            self.left_keys, self.right_keys
        )
    }
}

impl Error for MergeKeyError {}

impl From<MergeKeyError> for DataError {
    fn from(value: MergeKeyError) -> Self {
        let message = value.to_string();

        Self::new("MergeKeyError", message)
            .with_module(MODULE)
            .with_attribute("left_keys", json!(value.left_keys))
            .with_attribute("right_keys", json!(value.right_keys))
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when applying a schema evolution step fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaEvolutionError {
    /// Target schema version.
    pub schema_version: String,
    /// Optional reason for the failure.
    pub reason: Option<String>,
}

impl SchemaEvolutionError {
    /// Creates a schema-evolution error.
    #[must_use]
    pub fn new(schema_version: impl Into<String>, reason: Option<String>) -> Self {
        Self {
            schema_version: schema_version.into(),
            reason,
        }
    }
}

impl fmt::Display for SchemaEvolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Schema evolution to {} failed",
            self.schema_version
        )?;
        if let Some(reason) = &self.reason {
            write!(formatter, ": {reason}")?;
        }
        Ok(())
    }
}

impl Error for SchemaEvolutionError {}

impl From<SchemaEvolutionError> for DataError {
    fn from(value: SchemaEvolutionError) -> Self {
        let message = value.to_string();
        let mut error = Self::new("SchemaEvolutionError", message)
            .with_module(MODULE)
            .with_attribute("schema_version", json!(value.schema_version))
            .with_failure(FailureMetadata::unknown());

        if let Some(reason) = value.reason {
            error = error.with_attribute("reason", json!(reason));
        }

        error
    }
}

#[cfg(test)]
mod tests {
    use super::{DtypeMismatchError, IndexAlignmentError, MergeKeyError, SchemaEvolutionError};
    use crate::DataError;

    #[test]
    fn dtype_mismatch_preserves_expected_types() {
        let error = DtypeMismatchError::new("revenue", ["float64", "int64"], "object");
        assert_eq!(
            error.to_string(),
            "Column 'revenue' has dtype object; expected float64, int64"
        );

        let envelope = DataError::from(error).to_envelope();
        assert_eq!(envelope.error_type, "DtypeMismatchError");
    }

    #[test]
    fn index_alignment_includes_details() {
        let error = IndexAlignmentError::new(Some("left index has 20 extra rows".to_owned()));
        assert_eq!(
            error.to_string(),
            "DataFrame indices are misaligned: left index has 20 extra rows"
        );
    }

    #[test]
    fn merge_key_error_preserves_both_key_sets() {
        let error = MergeKeyError::new(["customer_id"], ["cust_id"]);
        let envelope = DataError::from(error).to_envelope();
        let attributes = envelope.attributes.expect("attributes should exist");

        assert_eq!(attributes["left_keys"][0], "customer_id");
        assert_eq!(attributes["right_keys"][0], "cust_id");
    }

    #[test]
    fn schema_evolution_carries_reason() {
        let error =
            SchemaEvolutionError::new("v2.1", Some("incompatible column type change".to_owned()));

        assert_eq!(
            error.to_string(),
            "Schema evolution to v2.1 failed: incompatible column type change"
        );
    }
}
