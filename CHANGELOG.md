# Changelog

All notable changes to `dataexcept` will be documented in this file.

The project follows semantic versioning. Pre-release versions may contain API changes before the first stable release.

## [Unreleased]

### Added

- full envelope protocol reader for ordinary exception, cycle, and truncation nodes;
- public envelope schema ID and version constants;
- canonical Python reference fixtures exercised directly by Rust compatibility tests.


## [0.1.0-alpha.2] - 2026-09-23

Second public alpha focused on API hardening, release automation, and validation.

### Changed

- boundary option structs now use fluent setters instead of public field literals, allowing future options without breaking callers;
- parsed `W3CTraceContext` fields are private and exposed through read-only accessors so validated trace state cannot be mutated into an invalid form;
- extensible boundary, trace-conflict, and operation-context error enums are now `#[non_exhaustive]`;
- docs.rs builds all optional features and crate-level documentation reflects the full alpha API.

### Added

- runnable examples for core errors, trace-aware observability, and boundary contexts;
- CI validation for the Rust 1.85 MSRV, individual optional features, examples, and packaged crate contents;
- crates.io Trusted Publishing through GitHub OIDC;
- a SemVer compatibility gate that becomes mandatory for beta and stable releases.

## [0.1.0-alpha.1] - 2026-09-23

Initial public alpha release.

### Added

- structured `DataError` and language-neutral `ErrorEnvelope`;
- transient, permanent, and unknown failure classification;
- retryability and optional retry-after metadata;
- strict JSON serialization with cause, context, and grouped-error support;
- canonical DataExcept JSON Schema validation and cross-language fixtures;
- credential-bearing URL redaction;
- domain errors for ingestion, validation, schema, transformation, database, network, brokers, and machine learning;
- conversions from common Rust and `serde_json` errors;
- product-neutral `OperationContext` and structured observability events;
- W3C Trace Context parsing and propagation;
- optional `tracing`, OpenTelemetry, and Sentry integrations;
- framework-neutral HTTP, worker, broker, and orchestrator boundary adapters;
- CI checks for formatting, Clippy, tests, and documentation.

### Release status

This is an experimental alpha. Public APIs may change before the first stable release.
