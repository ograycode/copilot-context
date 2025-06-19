# Instructions for Codex Agents

To mirror the linting and testing steps used in CI, replicate the setup from the GitHub Actions workflows.

1. Review `.github/workflows/build-test.yml` for how the stable Rust toolchain with `clippy` and `rustfmt` is installed and which commands are run.
2. For platform-specific dependencies (such as OpenSSL) see `.github/workflows/release.yml`.

Run the same commands locally to ensure your environment matches the CI configuration.
