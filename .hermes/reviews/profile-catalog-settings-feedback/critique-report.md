# Critique Report: Profile Catalog Settings Feedback

## Verdict

APPROVED

## Summary

The current implementation satisfies the requested Settings/Profile Tools catalog behavior for this review scope. Toolsets, MCP servers, plugins, and skills are rendered as checkbox-driven catalog/list controls instead of editable text fields; skill discovery is wired to the real Hermes CLI inventory path before falling back to profile/global directory scanning; and save/load behavior maps the toggles into Hermes config shapes without replacing the broader config document.

No blocking issue was found in the scoped review of `src/App.tsx`, `src-tauri/src/lib.rs`, and the relevant source guards/tests.

## Evidence reviewed

- Read handoff: `/Users/ziadnasreldin/Zoid/.hermes/reviews/profile-catalog-settings-feedback/handoff.md`.
- Inspected `src/App.tsx` Settings > Tools rendering:
  - `renderCatalogGroup(...)` renders `profile-catalog-card`, `profile-catalog-list`, and checkbox inputs for catalog items.
  - The Tools section calls `renderCatalogGroup("Toolsets" ...)`, `renderCatalogGroup("MCP servers" ...)`, `renderCatalogGroup("Plugins" ...)`, and `renderCatalogGroup("Skills" ...)`.
  - The catalog fields are no longer routed through `renderTextField(...)`; only unrelated operations fields such as profile mode/default workdir/trusted projects remain editable text fields.
- Inspected `src-tauri/src/lib.rs` profile settings hydration/persistence:
  - `discover_hermes_skill_status_from_cli()` runs `hermes skills list --source all` with `COLUMNS=400`.
  - `parse_hermes_skill_table(...)` parses available/enabled rows and has unit coverage for enabled/disabled rows.
  - `discover_hermes_skills()` falls back to recursive skill directory scanning before using the old small static seed list.
  - `apply_real_hermes_sources(...)` hydrates `available_skills`, `available_toolsets`, `available_mcp_servers`, `available_plugins`, and current enabled lists from config-backed state.
  - `save_real_hermes_sources(...)` writes toggle state to `agent.disabled_toolsets`, `skills.disabled`, `plugins.enabled`, `plugins.disabled`, and per-MCP-server `enabled` flags while preserving unrelated YAML keys by mutating the parsed config value.
  - Existing config is backed up before save and validated after serialization/readback.
- Inspected relevant tests/source guards:
  - `src/scaffold.test.ts` includes guards requiring real Hermes skill hydration and catalog-list rendering for skills/plugins/MCP/toolsets.
  - Rust unit coverage includes `hermes_skill_table_parser_handles_real_status_rows` and `hermes_profile_settings_preserve_real_yaml_shapes`, including disabled toolsets and disabled skills persistence checks.
- Inspected the notification registration fix in scope:
  - `tauri_plugin_notification::init()` is registered in the Tauri builder.
  - `send_agent_response_email_notification` is registered in the command handler.
  - This is additive/minimal relative to catalog behavior.
- Inspected scoped git diff. The repository is broadly dirty as noted in the handoff, so this review scoped findings to catalog/settings and the minimal notification registration surface.

## Findings

No blocking findings.

## Non-blocking observations

| ID | Priority | Area | Observation |
|----|----------|------|-------------|
| I1 | Low | UX | A full 100+ skill inventory in one checkbox list may be long to scan. Consider search/filtering or grouping later, but the current list/grid catalog satisfies the user's explicit request and is not a blocker. |
| I2 | Low | Discovery | Plugin availability currently comes from configured `plugins.enabled`/`plugins.disabled` entries rather than a broader plugin directory/CLI catalog. This matches the handoff's "declared in active profile" behavior, but a future broader plugin inventory could improve parity with skills if Hermes exposes one. |
| I3 | Low | Code quality | `src-tauri/src/lib.rs` is very large and now contains many unrelated features. Splitting profile settings/catalog code into focused modules would make future reviews and maintenance easier. |

## Verification note

I did not rerun the full frontend/Rust/build/Tauri verification because the handoff reports those local verification steps already passed and this was a report-only scoped critique. Static inspection found no discrepancy with those claims in the reviewed catalog/settings code paths.

## Final approval rationale

The user-requested controls are no longer editable textareas, the skill catalog is linked to the live Hermes inventory path rather than only four seed skills, and each requested category has an enable/disable checkbox flow with persistence into the intended Hermes config fields. The implementation preserves unrelated Hermes config state and has relevant source guards/unit coverage. Therefore this feature is approved.
