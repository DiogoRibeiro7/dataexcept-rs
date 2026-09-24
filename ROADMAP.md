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

- [x] Data ingestion and validation errors
- [x] Schema and transformation errors
- [x] Database and network errors
- [x] Message-broker errors
- [x] ML training, inference, and evaluation errors
- [x] Conversion helpers for common Rust ecosystem error types

## 0.3 — Observability

- [x] Product-neutral operation context
- [x] W3C Trace Context support
- [x] Optional `tracing` integration
- [x] Optional OpenTelemetry integration
- [x] Optional Sentry integration
- [x] HTTP, worker, broker, and orchestrator boundary adapters

## 0.4 — Alpha hardening

- [x] Enforce Rust 1.85 MSRV in CI
- [x] Validate optional features independently
- [x] Validate packaged crate contents and build
- [x] Add runnable examples for core and observability workflows
- [x] Audit public API and documentation before the next prerelease
- [x] Add semantic-version compatibility checks before beta

## 0.5 — Protocol consumption

- [x] Expose envelope schema ID and version constants
- [x] Parse the full envelope union: exception, cycle, and truncation nodes
- [x] Validate canonical Python reference fixtures in Rust
- [x] Add typed accessors for common transport inspection patterns
- [ ] Add compatibility tests against future 1.x schema additions
- [ ] Prepare beta migration guide and protocol stability notes

## Compatibility policy

The Rust API follows Rust semantic versioning. The serialized envelope is a separate language-neutral contract and must remain compatible with the published DataExcept schema. Contract changes require explicit schema-version handling and cross-language tests.
