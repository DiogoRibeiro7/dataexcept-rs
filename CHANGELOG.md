# Changelog

All notable changes to `dataexcept` will be documented in this file.

The project follows semantic versioning. Pre-release versions may contain API changes before the first stable release.

## [Unreleased]

### Changed

- boundary option structs now use fluent setters instead of public field literals, allowing future options without breaking callers;
- parsed `W3CTraceContext` fields are private and exposed through read-only accessors so validated trace state cannot be mutated into an invalid form;
- extensible boundary, trace-conflict, and operation-context error enums are now `#[non_exhaustive]`;
- docs.rs builds all optional features and crate-level documentation reflects the full alpha API.

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
