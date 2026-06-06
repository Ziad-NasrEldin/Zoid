# Zoid Module Docs

This directory groups module-specific planning artifacts so each product area keeps its PRD, implementation tracker, Stitch AI prompt, and prompt parts together.

## Modules

### `files-workspace/`
Files module / Finder-like local filesystem workspace.
- PRD
- Implementation tracker
- Stitch AI screen design prompt

### `autonomous-content-automation/`
Autonomous content automation module.
- PRD
- Implementation tracker
- Stitch AI full screen prompt
- Split Stitch AI prompt parts under `stitch-ai-prompt-parts/`

### `notes-workspace/`
Notes Workspace full product scope.
- PRD
- Implementation tracker
- No Stitch AI prompt generated per user request

## Conventions
- Put new module-specific docs under `Docs/modules/<module-slug>/`.
- Keep shared/global docs in `Docs/` or the existing shared folders such as `designer-screen-reference/`, `adr/`, `release/`, and `spikes/`.
- Use date-prefixed filenames for durable planning artifacts.
