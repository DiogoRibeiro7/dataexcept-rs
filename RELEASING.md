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
