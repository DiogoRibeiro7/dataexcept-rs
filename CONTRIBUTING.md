# Contributing

Contributions should preserve the distinction between Rust API design and the language-neutral DataExcept wire contract.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
```

All changes should be proposed through a pull request. Keep public APIs documented, avoid unnecessary dependencies, and add tests for behavioral or serialization changes.

Changes to serialized envelope semantics require compatibility tests against the canonical DataExcept schema.
