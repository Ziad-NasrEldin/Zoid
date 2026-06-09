# Feature Handoff: Provider management settings

## Original request

Page feedback on `/` Settings main provider dropdown: user wanted a feature to add a provider using an API key, mainly Google API key for Gemma/Gemini, and explicitly asked to brainstorm/ask questions before implementing. Follow-up decisions resolved the feature shape: add a separate Providers settings section; Zoid 25 stores its own provider list and manually syncs/applies it to Hermes; support Google/Gemini presets, model dropdowns, masked/reveal keys, lightweight validation, immediate Main provider/model selectability, and defer multiple keys.

## Implementation summary

- Added a dedicated Settings tab, `04 Providers`, for Zoid-managed provider presets and validation.
- Added provider templates/model dropdown data, including Google Gemini/Gemma options.
- Added a provider form with display name, provider type, API key, default model, optional base URL/custom field support, masked key handling, reveal action, connection test, save, and manual apply/sync.
- Added frontend client wrappers for typed Tauri invokes.
- Added Rust backend commands to list/save/validate/apply/reveal managed providers.
- Provider metadata is stored in Zoid/Hermes profile-local JSON; apply/sync writes Hermes-compatible environment entries only when explicitly invoked.
- API keys are not exposed in normal list responses; reveal is an explicit command. Google validation uses the Google Generative Language models endpoint.
- Saved Zoid-managed providers are folded into Main provider/model option availability in the Settings models controls.
- Multiple API keys per provider are intentionally deferred for a later version.

## Changed files

- `src/providers/providerClient.ts`: provider types, templates/model options, and Tauri invoke wrappers.
- `src/providers/ProvidersSettings.tsx`: Providers settings UI and provider management actions.
- `src/App.tsx`: added Providers settings tab and integrated managed providers into Main provider/Main model options.
- `src/App.css`: provider-management layout/styles.
- `src/scaffold.test.ts`: regression guards for Providers UI, no native select, key masking/reveal, commands, Google template/model, and immediate selectability.
- `src-tauri/Cargo.toml`: added `reqwest` for lightweight validation.
- `src-tauri/Cargo.lock`: dependency lock update.
- `src-tauri/src/lib.rs`: managed-provider persistence, sanitization, validation, env sync, reveal/apply/list/save Tauri commands, command registration, and Rust tests.

## How to test

- `npm run test:frontend`
- `npm run build`
- `npm run test:rust`
- `npm run tauri:build`
- Replace `/Applications/Zoid 25.app` with `src-tauri/target/release/bundle/macos/Zoid 25.app` and launch `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- In the installed app, open Settings and check that tab `04 Providers` is present. Selecting it should show provider-management controls for provider type/model/API key/test/save/apply.

## Tests run

- `npm run test:frontend && npm run build && npm run test:rust`: PASS. Frontend scaffold/dropdown tests passed; Vite/TypeScript build passed; Rust tests passed with `27 passed; 0 failed`.
- `npm run tauri:build`: PASS. Built release binary and macOS app bundle at `src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed bundle copy to `/Applications/Zoid 25.app`: PASS.
- Launched installed app from `/Applications/Zoid 25.app/Contents/MacOS/zoid`: PASS. Process observed running as `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Native screenshot check: PASS for installed Settings page and visible `04 Providers` tab. Full native tab-click screenshot was partially blocked by macOS focus/Stage Manager automation stealing focus; browser/dev verification earlier selected the Providers tab and confirmed the Providers page header, but reviewer should prefer source/tests for detailed controls unless manually clicking the native tab.

## Git info

- Branch: `main`
- Commit SHA at handoff time: `424be61`
- Diff base: not isolated; repository has broad unrelated dirty/untracked Zoid work. Scope review to the changed files listed above.

## Frontend/backend/database notes

- Frontend routes/components: Settings/Profile workspace in `src/App.tsx`; new providers UI under `src/providers/`.
- Backend endpoints/services: new Tauri commands in `src-tauri/src/lib.rs`: `list_managed_providers`, `save_managed_provider`, `validate_managed_provider`, `apply_managed_provider`, `reveal_managed_provider_key`.
- Database tables/migrations: none.
- Persistence: profile-local provider metadata JSON and Hermes-compatible env sync on manual apply.

## Reviewer focus areas

- Secrets: normal list responses must not leak API keys; reveal must be explicit; env writing must reject newline injection and avoid duplicate entries.
- Provider sync truthfulness: Save should store in Zoid only; Apply/sync should be the manual Hermes-facing step.
- Google validation: endpoint and failure states should be lightweight and fail closed.
- UI consistency: model choice should use `GlobalDropdown`, not native select/manual typing.
- Immediate availability: saved managed providers should appear in Main provider/model options without requiring app restart.
- Dirty-tree scope: ignore unrelated files/reviews unless they directly affect this feature.

## Fix cycle notes

First critique returned `REQUEST_CHANGES` with two required fixes.

Fixes applied:
- Loaded managed providers in `SettingsArchive` on mount via `listManagedProviders()`, so persisted providers populate Main provider/model options even if the user opens Models before opening Providers.
- Changed `save_managed_provider_inner()` to resolve the existing provider by the derived sanitized id when `input.id` is absent, preventing duplicate records/key collisions for repeated new saves of the same provider type/id.
- Added scaffold coverage requiring the parent Settings mount load, and a Rust regression test `managed_provider_second_new_save_updates_existing_id`.

Post-fix verification:
- `npm run test:frontend && npm run build && npm run test:rust`: PASS. Rust tests now `28 passed; 0 failed`.
