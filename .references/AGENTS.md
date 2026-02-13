# Reffy Instructions

These instructions are for AI assistants working in this project.

## TL;DR Checklist

- Decide whether Reffy ideation is needed for this request.
- If needed, read existing context in `.references/artifacts/`.
- Add/update exploratory artifacts and keep them concise.
- Run `reffy reindex` and `reffy validate` after artifact changes.
- After ideation approval, run `reffy summarize --output json` and pick only directly relevant artifacts for proposal citations.

## When To Use Reffy

Use Reffy first when the request:
- Mentions early-stage ideation, exploration, brainstorming, or raw notes
- Needs context gathering before drafting a concrete implementation plan
- Refers to "reffy", "references", "explore", "context layer", or research artifacts

## When To Skip Reffy

You can skip Reffy when the request is:
- A narrow bug fix that does not need exploratory context
- A small refactor with no requirement/design ambiguity
- A formatting, typing, or tooling-only update with clear scope

## Reffy Workflow

1. Read existing artifacts in `.references/artifacts/`.
2. Add or update artifacts to capture exploratory context.
3. Run `reffy reindex` to index newly added files into `.references/manifest.json`.
4. Run `reffy validate` to verify manifest contract compliance.

## Relationship To OpenSpec

- Reffy is the ideation/context layer.
- OpenSpec is the formal planning/spec layer.
- After ideation stabilizes, hand off to OpenSpec by following `@/openspec/AGENTS.md`.
- Do not duplicate full proposal/spec content in Reffy artifacts; summarize and link to OpenSpec outputs.

## OpenSpec Citation Rules

When an OpenSpec proposal is informed by Reffy artifacts:
- After ideation approval, run `reffy summarize --output json` to shortlist candidate artifacts.
- Include a short "Reffy References" subsection in `proposal.md` (or design notes if more appropriate).
- Cite only artifact filenames that directly informed the proposal's problem, scope, decisions, or constraints.
- Cite artifact filenames and intent, for example:
  - `testing.md` - early constraints and tradeoffs for manifest validation
- Do not include generic process artifacts or unrelated notes just because they exist.
- Keep citations at proposal/design level; task-by-task traceability is optional unless the change is high risk.
- If no Reffy artifacts informed the change, explicitly state "No Reffy references used."

### Reusable Proposal Snippet

Use this in `openspec/changes/<change-id>/proposal.md`:

```md
## Reffy References
- `artifact-name.md` - short note about how it informed this proposal
```

If none were used:

```md
## Reffy References
No Reffy references used.
```

## Artifact Conventions

- Treat `.references/` as a repository-local guidance and ideation context layer.
- Keep artifact names clear and stable.
- Prefer markdown notes for exploratory content.
- Keep manifests machine-readable and schema-compliant (version 1).
