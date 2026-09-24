# Security Policy

## Supported versions

Security fixes are applied to the latest published release or prerelease. Older
prerelease lines may not receive backports.

## Reporting a vulnerability

Do not report security vulnerabilities in a public issue.

Use GitHub's private vulnerability reporting flow from the repository's
**Security** tab when it is available. Include:

- the affected crate version;
- the affected feature or module;
- a minimal proof of concept or reproduction;
- the expected security impact;
- any known mitigation.

If private vulnerability reporting is unavailable, open a minimal public issue
asking for a private reporting channel without including exploit details,
credentials, secrets, or other sensitive information.

## Scope

Security reports are especially relevant for:

- secret or credential redaction failures;
- unsafe exposure of sensitive transport metadata;
- parsing behaviour that could bypass documented protocol validation;
- dependency vulnerabilities that materially affect the crate;
- release or CI configuration that could compromise published artifacts.

Please do not include live credentials, tokens, private URLs, or production data
in any report.

## Dependency and supply-chain checks

The repository runs `cargo-deny` in GitHub Actions.

Blocking checks cover:

- disallowed or unrecognized licenses;
- wildcard dependency declarations;
- unknown registries;
- unknown Git dependency sources;
- dependency policy violations.

Advisory checks run separately and are currently non-blocking so a newly
published upstream advisory does not unexpectedly block every pull request.
They remain visible in CI and should be reviewed promptly.
