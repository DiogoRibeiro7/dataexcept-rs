# dataexcept-rs

[![Crates.io](https://img.shields.io/crates/v/dataexcept.svg)](https://crates.io/crates/dataexcept)
[![docs.rs](https://docs.rs/dataexcept/badge.svg)](https://docs.rs/dataexcept)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/DiogoRibeiro7/dataexcept-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/DiogoRibeiro7/dataexcept-rs/actions/workflows/ci.yml)
[![Supply Chain](https://github.com/DiogoRibeiro7/dataexcept-rs/actions/workflows/supply-chain.yml/badge.svg)](https://github.com/DiogoRibeiro7/dataexcept-rs/actions/workflows/supply-chain.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Rust implementation of the **DataExcept** structured error and failure-observability model.

The project is a companion to the Python [DataExcept](https://github.com/DiogoRibeiro7/DataExcept) package. It is not a line-by-line port. The Rust API remains idiomatic while preserving the language-neutral failure metadata and serialized error-envelope contract used across service and process boundaries.

> **Status:** public beta. The API is now under SemVer compatibility checks while the crate remains pre-1.0.

## Installation

For the beta release:

```toml
[dependencies]
dataexcept = "0.1.0-beta.2"
```

Optional observability integrations are feature-gated:

```toml
[dependencies]
dataexcept = { version = "0.1.0-beta.2", features = ["tracing", "opentelemetry", "sentry"] }
```

## Current scope

The beta includes:

- transient, permanent, and unknown failure classification;
- retryability and optional retry-after metadata;
- JSON-safe structured error envelopes with cause, context, and grouped errors;
- canonical JSON Schema validation and cross-language fixtures;
- credential-bearing URL redaction;
- idiomatic `DataError` plus conversions from common Rust errors;
- ingestion, validation, schema, transformation, database, network, broker, and ML error types;
- product-neutral operation context;
- W3C Trace Context parsing and propagation;
- optional `tracing`, OpenTelemetry, and Sentry integrations;
- framework-neutral HTTP, worker, broker, and orchestrator boundary adapters.

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

## Consuming envelope JSON

For cross-language transport boundaries, parse the full protocol with
`EnvelopeNode` rather than assuming every node is a normal exception record:

```rust
use dataexcept::EnvelopeNode;

let payload = r#"{"truncated":true}"#;
let node = EnvelopeNode::from_json(payload).expect("valid DataExcept envelope");

assert!(node.is_truncated());
assert_eq!(node.error_type(), None);
```

The protocol reader understands ordinary exception records, cycle records, and
truncation markers. Marker shapes are strict, while ordinary 1.x records ignore
unknown fields for forward compatibility. Common inspection methods expose the
node kind, identity, failure metadata, chaining, and group members without
requiring callers to destructure the transport enum.

## Runnable examples

The repository includes examples that are compiled in CI:

```bash
cargo run --example core_error
cargo run --example observability
cargo run --example boundaries
```

They cover structured error envelopes, trace-aware observability events, and
framework-neutral HTTP/broker boundary context.

## Design principles

Rust errors and wire-format errors are separate concerns. Existing domain-specific Rust enums should remain domain-specific; they can be converted to `ErrorEnvelope` values at process, API, worker, or broker boundaries.

The serialized envelope is a separate language-neutral contract. Compatibility with the canonical DataExcept JSON Schema is protected by cross-language fixtures and contract tests rather than by forcing Python implementation details into Rust.

Observability integrations do not own application runtime configuration. Subscriber setup, OpenTelemetry providers/exporters, and Sentry SDK initialization remain application concerns.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
```

See [CONTRIBUTING.md](CONTRIBUTING.md), [ROADMAP.md](ROADMAP.md), [CHANGELOG.md](CHANGELOG.md), [BETA_MIGRATION.md](BETA_MIGRATION.md), [PROTOCOL_STABILITY.md](PROTOCOL_STABILITY.md), and [CITATION.cff](CITATION.cff).

## License

MIT. See [LICENSE](LICENSE).
