# Contributing

Contributions should preserve the distinction between Rust API design and the language-neutral DataExcept wire contract.

## Before opening a change

Use the repository issue forms for bugs, feature proposals, and usage questions when prior discussion would help. For vulnerabilities, follow [SECURITY.md](SECURITY.md) and do not disclose exploit details in a public issue.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
```

All changes should be proposed through a pull request. Keep public APIs documented, avoid unnecessary dependencies, and add tests for behavioral or serialization changes.

Changes to serialized envelope semantics require compatibility tests against the canonical DataExcept schema. Breaking Rust API changes must be documented explicitly and respect the repository's prerelease and SemVer policy.
