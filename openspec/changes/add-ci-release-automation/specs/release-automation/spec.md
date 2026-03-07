## ADDED Requirements
### Requirement: Validate Changes in Continuous Integration
The repository MUST run automated validation on pull requests and on pushes to `main`.

#### Scenario: Pull request validation
- **WHEN** a pull request targets `main`
- **THEN** GitHub Actions runs the configured validation workflow
- **AND** the workflow installs the required Node and Rust toolchains
- **AND** the workflow fails when the application cannot be built successfully

#### Scenario: Main branch validation
- **WHEN** a commit is pushed to `main`
- **THEN** GitHub Actions runs the configured validation workflow
- **AND** the workflow verifies both the frontend project and the Tauri backend build path

### Requirement: Produce Draft Releases from Version Tags
The repository MUST support creating a draft GitHub release with bundled application artifacts from a pushed version tag.

#### Scenario: Tagged release build
- **WHEN** a maintainer pushes a version tag that matches the documented release pattern
- **THEN** GitHub Actions builds the release artifacts for the configured target platforms
- **AND** the workflow bundles the Popper sidecar into the application artifacts
- **AND** the workflow creates or updates a draft GitHub release for that version
- **AND** the draft release includes generated release notes

#### Scenario: Unsupported platform excluded
- **WHEN** the bundled Popper shell does not build on a desktop target platform
- **THEN** the release workflow excludes that platform from automated release builds
- **AND** the release documentation identifies the unsupported platform and why it is excluded

#### Scenario: Release workflow permissions
- **WHEN** the release workflow runs
- **THEN** it uses repository permissions and secrets documented by the project
- **AND** it fails with an actionable error if the required release permissions or secrets are missing

### Requirement: Keep Version Metadata Aligned for Releases
The project MUST keep user-facing application version metadata aligned across package, Rust, and Tauri configuration before a release is created.

#### Scenario: Prepare versioned release metadata
- **WHEN** a maintainer prepares version `0.2.1`
- **THEN** the version metadata in the Node package manifest, Rust crate manifest, and Tauri configuration all read `0.2.1`
- **AND** the release workflow uses that aligned version to name release artifacts

#### Scenario: Prepare next patch release safely
- **WHEN** a maintainer needs to cut the next patch release after an earlier tag already triggered automation
- **THEN** the project provides a documented preparation flow for creating a fresh patch version and tag
- **AND** the flow avoids reusing an already-triggered release tag
