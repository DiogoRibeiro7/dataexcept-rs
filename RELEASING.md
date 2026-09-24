# Releasing

## crates.io Trusted Publishing

The crate uses crates.io Trusted Publishing through GitHub Actions.

Configure the trusted publisher on crates.io with:

- **GitHub owner:** `DiogoRibeiro7`
- **Repository:** `dataexcept-rs`
- **Workflow:** `release.yml`
- **Environment:** leave empty

The workflow lives at `.github/workflows/release.yml` and requests
`id-token: write`. Authentication is performed with
`rust-lang/crates-io-auth-action@v1`, which provides a short-lived crates.io
token to `cargo publish`.

No long-lived `CARGO_REGISTRY_TOKEN` GitHub secret is required for normal
releases after Trusted Publishing is configured.

## Publish a release

1. Prepare and merge the version/changelog PR into `main`.
2. Open **Actions → Release → Run workflow**.
3. Enter the exact crate version from `Cargo.toml`, without the `v` prefix.
4. Run the workflow.

The workflow:

- verifies the requested version matches `Cargo.toml`;
- runs formatting, Clippy, tests, and documentation;
- lists the packaged files;
- runs `cargo publish --dry-run`;
- authenticates to crates.io through GitHub OIDC;
- publishes with `cargo publish`;
- creates the matching annotated Git tag;
- creates a GitHub prerelease.

The tag and GitHub release are only created after crates.io publication
succeeds.

Do not publish from an unmerged feature branch or with a version that differs
from `Cargo.toml`.

## SemVer compatibility

CI includes a `cargo-semver-checks` gate. Alpha versions are allowed to evolve
with intentional breaking changes. Beginning with beta releases, CI compares the
current public API against the latest `v*` Git tag and checks all crate
features for semantic-version compatibility.

Before preparing `0.1.0-beta.1`, make sure the latest alpha release has a
matching Git tag because that tag becomes the beta baseline.

Before publishing the first beta:

- review [BETA_MIGRATION.md](BETA_MIGRATION.md);
- confirm [PROTOCOL_STABILITY.md](PROTOCOL_STABILITY.md) still matches the
  canonical envelope schema;
- run the full CI suite with the beta version in `Cargo.toml`;
- require the SemVer compatibility job to pass against the latest release tag;
- ensure the changelog documents every public API migration.
