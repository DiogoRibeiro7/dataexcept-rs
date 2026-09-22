# Roadmap

`dataexcept-rs` is the Rust companion to the Python DataExcept project. The goal is interoperability, not a mechanical Python-to-Rust port.

## 0.1 — Core contract

- [x] Failure classification: transient, permanent, unknown
- [x] Retryability and retry-after metadata
- [x] JSON-safe structured error envelope
- [x] Cause, context, and grouped-error representation
- [x] Minimal idiomatic Rust operational error type
- [x] Formatting, Clippy, tests, and documentation in CI
- [x] Validate emitted envelopes against the canonical DataExcept JSON Schema
- [x] Add cross-language fixtures shared with the Python implementation
- [x] Credential-bearing URL redaction compatible with DataExcept

## 0.2 — Domain errors

- [ ] Data ingestion and validation errors
- [ ] Schema and transformation errors
- [ ] Database and network errors
- [ ] Message-broker errors
- [ ] ML training, inference, and evaluation errors
- [ ] Conversion helpers for common Rust ecosystem error types

## 0.3 — Observability

- [ ] Product-neutral operation context
- [ ] W3C Trace Context support
- [ ] Optional `tracing` integration
- [ ] Optional OpenTelemetry integration
- [ ] Optional Sentry integration
- [ ] HTTP, worker, broker, and orchestrator boundary adapters

## Compatibility policy

The Rust API follows Rust semantic versioning. The serialized envelope is a separate language-neutral contract and must remain compatible with the published DataExcept schema. Contract changes require explicit schema-version handling and cross-language tests.
