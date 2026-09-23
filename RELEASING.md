# Releasing

## First crates.io alpha

The first public release is `0.1.0-alpha.1`.

The repository provides a manual GitHub Actions workflow at
`.github/workflows/release.yml`.

### One-time setup

Create a crates.io API token with permission to publish the `dataexcept`
crate, then add it to the GitHub repository as the Actions secret
`CARGO_REGISTRY_TOKEN`.

The first crates.io publication still requires this token-based bootstrap.
After the crate exists on crates.io, Trusted Publishing can be configured for
future releases.

### Publish

1. Ensure the release-preparation PR is merged into `main`.
2. Open **Actions → Release → Run workflow**.
3. Enter the crate version without the `v` prefix, for example
   `0.1.0-alpha.1`.
4. Run the workflow.

The workflow:

- verifies the version in `Cargo.toml`;
- requires the crates.io token secret;
- runs formatting, Clippy, tests, and documentation;
- lists the packaged files;
- runs `cargo publish --dry-run`;
- publishes with `cargo publish`;
- creates the matching annotated Git tag;
- creates a GitHub prerelease.

The tag and GitHub release are only created after crates.io publication
succeeds.

Do not publish from an unmerged feature branch or with a version that differs
from `Cargo.toml`.
