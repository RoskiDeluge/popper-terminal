# Releasing Popper Terminal

## Versioning
- Keep the app version aligned across `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
- Release tags use the format `vX.Y.Z`.
- Use `npm run release:prepare -- X.Y.Z` to update the version metadata before building and tagging a release.

## CI
- Pull requests to `main` and pushes to `main` run the CI workflow in `.github/workflows/ci.yml`.
- CI checks out the Popper repository into `./popper`, installs project dependencies, builds the frontend, and builds the Tauri backend.

## Release automation
- Pushing a tag such as `v0.2.1` triggers `.github/workflows/release.yml`.
- The release workflow builds Tauri artifacts for macOS Apple Silicon, macOS Intel, and Linux, then publishes the GitHub release automatically.
- macOS automation currently publishes `.app` bundles instead of `.dmg` images because the DMG bundling step has been failing in GitHub Actions after the app bundle itself is produced.
- The workflow clones `RoskiDeluge/popper` and attempts to check out a matching tag before falling back to the Popper default branch.
- The bundled Popper sidecar is built during the Tauri build through `src-tauri/build.rs` using `POPPER_PATH`.
- Windows releases are currently excluded because the Popper shell uses Unix-specific process and permission APIs and does not build successfully on Windows yet.

## Repository settings
- The release workflow requires the default `GITHUB_TOKEN` with `contents: write` permission.
- No additional secrets are required for unsigned builds.
- If code signing is added later, platform signing secrets will need to be configured before publishing signed artifacts.

## Release steps
1. Prepare the next release version with `npm run release:prepare -- X.Y.Z`.
2. Run `npm run build` and `cargo build --manifest-path src-tauri/Cargo.toml`.
3. Commit and push the release changes.
4. Create and push a fresh tag such as `v0.2.2`.
5. Wait for the release workflow to finish publishing the GitHub release and uploading the artifacts.
6. Review the generated release notes and attached artifacts in GitHub.

## Patch release guidance
- Prefer a new patch version such as `v0.2.2` over reusing an already-triggered tag.
- Reusing a tag can leave stale draft releases and ambiguous workflow history.
