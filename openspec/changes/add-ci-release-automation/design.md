## Context
The repository has no existing GitHub Actions workflows. Tauri release automation requires a multi-platform build matrix and release publishing permissions. The project also depends on the external Popper shell source, which is available locally during development but must be made available explicitly in CI.

## Goals / Non-Goals
- Goals:
  - Add a repeatable CI workflow for routine validation.
  - Add a release workflow that produces Tauri artifacts and drafts a GitHub release from tagged versions.
  - Align the desktop app version with the current Popper shell version at `0.2.1`.
- Non-Goals:
  - Add code signing for macOS or Windows in this change.
  - Add a full updater integration.
  - Add end-to-end UI automation.

## Decisions
- Decision: Use GitHub Actions because the repository already lives on GitHub and Tauri documents an official GitHub release pipeline.
- Decision: Separate CI validation from release publishing so routine checks do not require release permissions.
- Decision: Trigger release automation from version tags instead of a release branch because tags map directly to versioned app artifacts.
- Decision: Check out the Popper repository as an additional repository in CI so the sidecar build path remains explicit and reproducible.
- Decision: Keep releases as draft releases so a maintainer can review release notes and attached artifacts before publishing.

## Alternatives considered
- Release branch trigger: rejected because it is less explicit than version tags and couples branch management to release cadence.
- Committing Popper source as a submodule: rejected because the current project structure intentionally keeps Popper as an external repo.
- Disabling sidecar builds in CI: rejected because release artifacts must include the bundled shell.

## Risks / Trade-offs
- Multi-platform Tauri builds may require platform-specific system packages and secrets. Mitigation: start with official Tauri GitHub workflow patterns and document required repository configuration.
- Pulling Popper from another repository in CI introduces an extra availability dependency. Mitigation: pin the checkout to an explicit ref or to the matching version tag when possible.
- Unsigned binaries may still produce platform trust warnings. Mitigation: treat signing as a later follow-up.
- Popper currently uses Unix-specific APIs and does not build on Windows. Mitigation: limit automated release targets to currently supported desktop platforms until Popper gains Windows support.

## Migration Plan
1. Approve the spec change.
2. Add CI and release workflows.
3. Bump the application version to `0.2.1`.
4. Document the release trigger and required repository settings.
5. Validate locally, then push the workflows.

## Open Questions
- Whether release tags should use `v0.2.1` or an app-prefixed tag pattern such as `app-v0.2.1`.
- Which Popper ref should be checked out in CI for release builds: a matching tag, `main`, or a configurable default.
