# Beta Migration Guide

This guide describes the expected migration path from the public alpha releases
to `0.1.0-beta.1`.

## Scope of the beta transition

The beta line is where the Rust public API begins receiving stronger SemVer
protection. The language-neutral envelope protocol is versioned independently
and remains governed by its own schema compatibility rules.

Before adopting the beta, update alpha-era call sites that still depend on
earlier public shapes.

## Boundary context options

Boundary option structs use fluent setters rather than public field literals.

Use:

```rust
use dataexcept::BrokerContextOptions;

let options = BrokerContextOptions::default()
    .component("billing")
    .correlation_id("corr-9")
    .partition(3)
    .offset(1042)
    .consumer_group("billing")
    .message_id("msg-7");
```

Do not rely on constructing the struct by naming its fields.

The same rule applies to `OrchestratorContextOptions`.

## W3C trace context

Parsed `W3CTraceContext` values are immutable through the public API. Read
validated state through accessors such as:

```rust
let trace_id = trace.trace_id();
let parent_id = trace.parent_id();
let traceparent = trace.traceparent();
```

This prevents callers from mutating a valid parsed trace into an invalid one.

## Extensible enums

Several public enums are `#[non_exhaustive]`, including boundary and
validation error enums. Downstream exhaustive matches must include a wildcard
arm.

This is deliberate: adding future variants should not force a breaking crate
release.

## Consuming envelope JSON

Use `EnvelopeNode` for cross-language payloads. Do not deserialize arbitrary
transport payloads directly into `ErrorEnvelope`.

`ErrorEnvelope` is the Rust producer model for ordinary exception records.
`EnvelopeNode` represents the full protocol union:

- ordinary exception records;
- cycle records;
- truncation markers.

Use typed accessors such as `kind()`, `error_type()`, `failure()`,
`cause()`, `context()`, and `exceptions()` for routine inspection.

## Optional integrations

The `tracing`, `opentelemetry`, and `sentry` integrations remain
feature-gated. The crate does not own subscriber, provider, exporter, or SDK
initialization.

Applications remain responsible for runtime observability configuration.

## MSRV

The minimum supported Rust version remains Rust 1.85. CI checks the full feature
set on that toolchain.

## Before moving to beta

A beta release should satisfy all of the following:

- CI is green on stable and Rust 1.85;
- all feature combinations compile;
- packaged crate validation succeeds;
- canonical Python envelope fixtures parse in Rust;
- future additive 1.x fields remain readable;
- `cargo-semver-checks` passes against the latest release tag;
- the changelog contains explicit migration notes for any remaining API change.

Once `0.1.0-beta.1` is published, avoid intentional breaking Rust API changes
unless the beta version is advanced accordingly and the migration is documented.
