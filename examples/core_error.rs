//! Builds a structured operational error and prints its transport envelope.

use dataexcept::{DataError, FailureMetadata};
use serde_json::json;

fn main() {
    let error = DataError::new("missing_column", "customer_id is required")
        .with_module("pipeline::validation")
        .with_attribute("column", json!("customer_id"))
        .with_failure(FailureMetadata::permanent());

    println!(
        "{}",
        error
            .to_envelope()
            .to_json_pretty()
            .expect("envelope should serialize")
    );
}
