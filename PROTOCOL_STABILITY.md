# Protocol Stability

The serialized DataExcept envelope is a language-neutral protocol and is
versioned independently from the Rust crate.

## Current contract

- schema version: `1.0.0`
- schema ID:
  `https://diogoribeiro7.github.io/DataExcept/schema/envelope-1.0.0.json`
- dialect: JSON Schema draft 2020-12

The Rust constants `ENVELOPE_SCHEMA_VERSION` and `ENVELOPE_SCHEMA_ID`
identify this contract.

## Protocol nodes

An envelope node has one of three shapes:

1. a fully rendered exception record;
2. a cycle record;
3. a truncation marker.

Consumers should use `EnvelopeNode` when reading transport payloads.

## 1.x compatibility rules

Within envelope schema 1.x:

- new fields may be added to ordinary exception records;
- established fields do not silently change meaning, type, or nullability;
- consumers must ignore unknown ordinary-record fields;
- the same forward-compatibility rule applies recursively to nested ordinary
  exception records and structured failure metadata;
- cycle records remain exact and may contain only `type`, `module`,
  `message`, and `cycle: true`;
- truncation markers remain exact and contain only `truncated: true`.

The strict marker rules are intentional because marker identity is determined
by shape.

## Rust crate SemVer vs protocol versioning

The Rust crate version and envelope schema version solve different problems.

A crate release may change without changing the envelope schema. Conversely, a
future envelope schema version does not imply the Rust crate must share the same
version number.

Rust API compatibility is checked with crate SemVer and
`cargo-semver-checks`. Envelope compatibility is checked with the canonical
JSON Schema and cross-language fixtures.

## Producers and consumers

Rust producers generally create `ErrorEnvelope` values for ordinary exception
records.

Transport consumers should use `EnvelopeNode`, because a valid payload may
also contain cycle or truncation nodes at any recursive boundary.

## Cross-language verification

The Rust test suite includes the canonical Python fixtures for:

- ordinary exceptions;
- explicit causes;
- failure metadata;
- implicit contexts;
- nested exception groups;
- redaction;
- truncation;
- cycles.

The suite also includes a synthetic future-1.x payload with additive unknown
fields at multiple recursive levels. These tests protect the documented
forward-compatibility behavior.

## Changing the protocol

A protocol change should not be accepted by modifying Rust types alone.

A valid protocol change requires:

1. an updated canonical schema;
2. updated reference fixtures from the producer implementation;
3. Rust compatibility tests;
4. explicit schema-version handling;
5. documentation of whether old consumers remain compatible.

Breaking the documented 1.x rules requires a new major envelope schema version.
