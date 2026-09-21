# dataexcept-rs

[![CI](https://github.com/DiogoRibeiro7/dataexcept-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/DiogoRibeiro7/dataexcept-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Rust implementation of the **DataExcept** structured error and failure-observability model.

The project is a companion to the Python [DataExcept](https://github.com/DiogoRibeiro7/DataExcept) package. It is not intended as a line-by-line port. The Rust API should remain idiomatic while preserving the language-neutral failure metadata and serialized error-envelope contract used across service and process boundaries.

## Current scope

The first release establishes the core cross-language model:

- transient, permanent, and unknown failure classification;
- retryability and optional retry-after metadata;
- JSON-safe structured error envelopes;
- structured attributes;
- explicit cause and context chains;
- grouped errors;
- a small `DataError` type for applications that do not already have their own error enums.

Domain-specific errors, tracing, OpenTelemetry, Sentry, broker adapters, and cross-language schema fixtures are planned in the [roadmap](ROADMAP.md).

## Example

```rust
use dataexcept::{DataError, FailureMetadata};
use serde_json::json;

fn validate() -> dataexcept::Result<()> {
    let error = DataError::new(
        "missing_column",
        "customer_id is required",
    )
    .with_module("pipeline::validation")
    .with_attribute("column", json!("customer_id"))
    .with_failure(FailureMetadata::permanent());

    Err(error)
}

fn main() {
    if let Err(error) = validate() {
        let envelope = error.to_envelope();
        println!("{}", envelope.to_json_pretty().expect("valid JSON"));
    }
}
```

The resulting envelope contains stable machine-readable fields such as:

```json
{
  "type": "missing_column",
  "module": "pipeline::validation",
  "message": "customer_id is required",
  "attributes": {
    "column": "customer_id"
  },
  "failure": {
    "kind": "permanent",
    "retryable": false,
    "retry_after_seconds": null
  }
}
```

## Design principles

Rust errors and wire-format errors are separate concerns. Existing domain-specific Rust enums should remain domain-specific; they can be converted to `ErrorEnvelope` values at process, API, worker, or broker boundaries.

The serialized envelope is treated as a language-neutral contract. Compatibility with the canonical DataExcept JSON Schema will be protected with cross-language fixtures and contract tests rather than by forcing Python implementation details into Rust.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
```

See [CONTRIBUTING.md](CONTRIBUTING.md) and [ROADMAP.md](ROADMAP.md).

## License

MIT. See [LICENSE](LICENSE).
