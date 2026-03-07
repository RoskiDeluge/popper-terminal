# Change: Add CI and release automation

## Why
The project currently relies on manual local validation and manual release packaging, which makes regressions and packaging mistakes more likely. The application version is also still `0.2.0`, while the bundled Popper shell has already moved to `0.2.1`.

## What Changes
- Add GitHub Actions CI that validates the project on pull requests and pushes to `main`.
- Add GitHub Actions release automation that builds distributable artifacts and creates a draft GitHub release from a version tag.
- Bump the app version from `0.2.0` to `0.2.1` across user-facing project metadata.
- Document the required repository secrets, permissions, and release trigger flow.

## Impact
- Affected specs: `release-automation`
- Affected code: `.github/workflows/*`, `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `README.md`
