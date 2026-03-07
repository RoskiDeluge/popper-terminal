## 1. Implementation
- [x] 1.1 Bump the application version to `0.2.1` in package, Rust, and Tauri metadata.
- [x] 1.2 Add a CI workflow that runs on pull requests and pushes to `main`, installs Node and Rust, and validates the web and Rust builds.
- [x] 1.3 Add a release workflow that runs on version tag pushes, builds Tauri artifacts through GitHub Actions, and creates a draft GitHub release.
- [x] 1.4 Configure the release workflow so it can build the bundled Popper sidecar in CI without relying on a sibling checkout.
- [x] 1.5 Document the release prerequisites, trigger convention, and any required GitHub secrets or permissions.
- [x] 1.6 Generate draft release notes automatically and add a helper flow for preparing the next patch release safely.

## 2. Validation
- [x] 2.1 Run the local validation commands that mirror the CI checks.
- [x] 2.2 Validate the OpenSpec change with `openspec validate add-ci-release-automation --strict`.
