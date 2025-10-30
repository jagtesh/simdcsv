# GitHub Actions Workflows

This directory contains GitHub Actions workflows for the simdcsv project.

## Workflows

### CI (`ci.yml`)

Runs on every push to `main` and on pull requests to `main`.

**Jobs:**
- **test**: Runs tests on multiple operating systems (Ubuntu, macOS, Windows) and Rust versions (stable, beta)
  - Builds the project
  - Runs all tests
  - Checks code formatting with `cargo fmt` (stable only)
  - Runs clippy linter with warnings as errors (stable only)
- **build-release**: Builds release binaries on multiple platforms and tests packaging

**Features:**
- Caching of cargo registry, index, and build artifacts for faster builds
- Cross-platform testing (Linux, macOS, Windows)
- Multi-version testing (stable and beta Rust)

### Publish (`publish.yml`)

Publishes the crate to crates.io.

**Triggers:**
- Automatically when a GitHub release is published
- Manually via workflow_dispatch (with dry-run option)

**Jobs:**
- **publish**: Publishes the crate to crates.io
  - Runs tests to ensure quality
  - Packages the crate
  - Publishes to crates.io (or dry-run if triggered manually)

**Configuration Required:**
To enable actual publishing, you need to:
1. Create a crates.io API token at https://crates.io/settings/tokens
2. Add it as a GitHub secret named `CARGO_REGISTRY_TOKEN` in the repository settings

**Usage:**
1. Create a new GitHub release with a version tag (e.g., `v0.1.0`)
2. The workflow will automatically publish the crate to crates.io
3. For testing, you can manually trigger the workflow in dry-run mode

## Testing the Workflows

### Testing CI locally
```bash
# Run the same checks that CI runs
cargo build --verbose
cargo test --verbose
cargo fmt -- --check
cargo clippy -- -D warnings
cargo build --release --verbose
cargo package --verbose
```

### Testing Publish locally
```bash
# Test packaging and dry-run publish
cargo package --verbose
cargo publish --dry-run
```

## Notes

- The publish workflow is configured with both automatic (on release) and manual trigger options
- By default, manual triggers run in dry-run mode to prevent accidental publishing
- The workflows use caching to speed up builds
- The CI workflow tests on multiple platforms to ensure cross-platform compatibility
