//! Cross-language compatibility tests for the DataExcept envelope contract.

use dataexcept::{DataError, FailureMetadata};
use serde_json::{Value, json};

fn schema() -> Value {
    serde_json::from_str(include_str!("../schemas/envelope-1.0.0.json"))
        .expect("canonical schema must be valid JSON")
}

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/permanent_error.json"))
        .expect("fixture must be valid JSON")
}

#[test]
fn rust_envelope_matches_cross_language_fixture() {
    let error = DataError::new("missing_column", "customer_id is required")
        .with_module("pipeline::validation")
        .with_attribute("column", json!("customer_id"))
        .with_failure(FailureMetadata::permanent());

    let emitted = serde_json::to_value(error.to_envelope()).expect("envelope must serialize");

    assert_eq!(emitted, fixture());
}

#[test]
fn rust_envelope_validates_against_canonical_schema() {
    let instance = fixture();
    let schema = schema();

    jsonschema::validate(&schema, &instance)
        .expect("Rust envelope fixture must satisfy DataExcept envelope schema 1.0.0");
}
