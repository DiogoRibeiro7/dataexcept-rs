# Releasing

## First crates.io alpha

The first public release is `0.1.0-alpha.1`.

Before publishing:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
cargo package --list
cargo publish --dry-run
```

Then publish manually from a clean checkout of the release commit:

```bash
cargo publish
```

After crates.io accepts the package:

1. create and push the Git tag `v0.1.0-alpha.1`;
2. create the matching GitHub prerelease;
3. verify the crate page and docs.rs build;
4. configure crates.io Trusted Publishing for future releases.

Do not publish from an unmerged feature branch or with uncommitted local changes.
