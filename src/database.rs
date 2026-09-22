//! Typed database-operation errors.

use std::{error::Error, fmt};

use serde_json::json;

use crate::{DataError, FailureMetadata, redaction::redact_url};

const MODULE: &str = "dataexcept::database";

/// Raised when connecting to a database fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConnectionError {
    /// Redacted database connection URL.
    pub db_url: String,
    message: String,
}

impl DatabaseConnectionError {
    /// Creates a database-connection error.
    #[must_use]
    pub fn new(db_url: impl Into<String>, message: Option<String>) -> Self {
        let db_url = db_url.into();
        let db_url = redact_url(&db_url, true);
        let default = format!("Failed to connect to database at '{db_url}'");

        Self {
            db_url,
            message: message.unwrap_or(default),
        }
    }

    /// Returns the rendered message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for DatabaseConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for DatabaseConnectionError {}

impl From<DatabaseConnectionError> for DataError {
    fn from(value: DatabaseConnectionError) -> Self {
        Self::new("DatabaseConnectionError", value.message)
            .with_module(MODULE)
            .with_attribute("db_url", json!(value.db_url))
            .with_failure(FailureMetadata::unknown())
    }
}

/// Raised when executing a database query fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryExecutionError {
    /// Query text that failed.
    pub query: String,
    /// Optional textual description of the underlying cause.
    pub cause: Option<String>,
}

impl QueryExecutionError {
    /// Creates a query-execution error.
    #[must_use]
    pub fn new(query: impl Into<String>, cause: Option<String>) -> Self {
        Self {
            query: query.into(),
            cause,
        }
    }
}

impl fmt::Display for QueryExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Query failed: {}", self.query)?;
        if let Some(cause) = &self.cause {
            write!(formatter, " ({cause})")?;
        }
        Ok(())
    }
}

impl Error for QueryExecutionError {}

impl From<QueryExecutionError> for DataError {
    fn from(value: QueryExecutionError) -> Self {
        let message = value.to_string();
        let mut error = Self::new("QueryExecutionError", message)
            .with_module(MODULE)
            .with_attribute("query", json!(value.query))
            .with_failure(FailureMetadata::unknown());

        if let Some(cause) = value.cause {
            error = error.with_attribute("cause", json!(cause));
        }

        error
    }
}

/// Raised when a database transaction fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionError {
    /// Optional transaction identifier.
    pub transaction_id: Option<String>,
    /// Optional textual description of the underlying cause.
    pub cause: Option<String>,
    message: String,
}

impl TransactionError {
    /// Creates a transaction error.
    #[must_use]
    pub fn new(
        transaction_id: Option<String>,
        message: Option<String>,
        cause: Option<String>,
    ) -> Self {
        let default = transaction_id.as_ref().map_or_else(
            || "Database transaction failed".to_owned(),
            |transaction_id| format!("Database transaction failed (id={transaction_id})"),
        );

        Self {
            transaction_id,
            cause,
            message: message.unwrap_or(default),
        }
    }

    /// Returns the rendered message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for TransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for TransactionError {}

impl From<TransactionError> for DataError {
    fn from(value: TransactionError) -> Self {
        let mut error = Self::new("TransactionError", value.message)
            .with_module(MODULE)
            .with_failure(FailureMetadata::unknown());

        if let Some(transaction_id) = value.transaction_id {
            error = error.with_attribute("transaction_id", json!(transaction_id));
        }
        if let Some(cause) = value.cause {
            error = error.with_attribute("cause", json!(cause));
        }

        error
    }
}

#[cfg(test)]
mod tests {
    use super::{DatabaseConnectionError, QueryExecutionError, TransactionError};
    use crate::DataError;

    #[test]
    fn connection_error_redacts_database_credentials() {
        let error = DatabaseConnectionError::new(
            "postgresql://admin:hunter2@prod-db:5432/analytics",
            None,
        );

        assert!(!error.db_url.contains("hunter2"));
        assert!(error.db_url.contains("prod-db:5432"));
        assert!(error.db_url.contains("/analytics"));

        let envelope = DataError::from(error).to_envelope();
        assert_eq!(
            envelope.attributes.expect("attributes should exist")["db_url"],
            "postgresql://***:***@prod-db:5432/***"
        );
    }

    #[test]
    fn query_error_preserves_query_and_cause() {
        let error = QueryExecutionError::new(
            "SELECT * FROM large_table",
            Some("statement timeout".to_owned()),
        );

        assert_eq!(
            error.to_string(),
            "Query failed: SELECT * FROM large_table (statement timeout)"
        );

        let envelope = DataError::from(error).to_envelope();
        let attributes = envelope.attributes.expect("attributes should exist");

        assert_eq!(attributes["query"], "SELECT * FROM large_table");
        assert_eq!(attributes["cause"], "statement timeout");
    }

    #[test]
    fn transaction_error_uses_identifier_in_default_message() {
        let error = TransactionError::new(Some("txn-42".to_owned()), None, None);

        assert_eq!(
            error.to_string(),
            "Database transaction failed (id=txn-42)"
        );
    }

    #[test]
    fn transaction_error_accepts_custom_message() {
        let error = TransactionError::new(
            None,
            Some("commit failed after retries".to_owned()),
            Some("serialization failure".to_owned()),
        );

        assert_eq!(error.to_string(), "commit failed after retries");

        let envelope = DataError::from(error).to_envelope();
        assert_eq!(
            envelope.attributes.expect("attributes should exist")["cause"],
            "serialization failure"
        );
    }
}
