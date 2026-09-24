## Summary

Describe the change and why it is needed.

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-features`
- [ ] `cargo doc --no-deps --all-features`
- [ ] Public API changes are documented.
- [ ] Serialization/protocol changes include schema and cross-language compatibility coverage where applicable.
- [ ] Breaking changes are called out explicitly.
