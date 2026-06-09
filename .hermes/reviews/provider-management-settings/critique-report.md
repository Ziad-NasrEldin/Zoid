# Provider Management Settings Re-review

Verdict: APPROVED

## Scope

Re-reviewed only the previous required fixes and obvious regressions in the provider-management settings files:

- `src/App.tsx`
- `src/providers/ProvidersSettings.tsx`
- `src/providers/providerClient.ts`
- `src-tauri/src/lib.rs`
- `src/scaffold.test.ts`

## Previous blockers

### 1. Persisted managed providers were not loaded into Main provider/model until Providers tab opened

Status: RESOLVED

`SettingsArchive` now imports `listManagedProviders` and loads managed providers on mount via a dedicated `useEffect`, storing them in `managedProviders`. The Models tab builds `availableModelOptions` from both Hermes profile settings and `managedProviders`, so persisted Zoid-managed providers are available in the Main provider/Main model dropdowns without requiring the Providers tab to be opened first.

`ProvidersSettings` still refreshes and emits provider changes after saves/applies, preserving immediate selectability for newly saved providers.

### 2. Re-saving the same new provider could generate duplicate records/keychain collisions

Status: RESOLVED

`save_managed_provider_inner()` now derives a `lookup_id` from `input.id` when present, or from the sanitized `input.provider_id` when saving a new provider. It uses that lookup to find and update an existing provider record before calling `provider_from_input`, preventing repeated “new” saves of the same provider from appending duplicate records.

The added Rust regression test `managed_provider_second_new_save_updates_existing_id` verifies that two new saves for provider id `google` produce one persisted provider and update its fields.

## Regression check

No obvious regressions were found in the reviewed provider-management files. Normal list responses still use the typed client and do not expose API keys; reveal remains an explicit command. Provider/model UI continues to use `GlobalDropdown`, and Save remains distinct from Apply/sync.

## Verification run

Executed focused verification from `/Users/ziadnasreldin/Zoid`:

```bash
npm run test:frontend && npm run test:rust -- managed_provider_second_new_save_updates_existing_id
```

Result: PASS

- Frontend scaffold/dropdown tests passed.
- Focused Rust regression test passed: `1 passed; 0 failed; 27 filtered out`.

Also ran scoped whitespace/check validation:

```bash
git diff --check -- src/App.tsx src/providers/ProvidersSettings.tsx src/providers/providerClient.ts src-tauri/src/lib.rs src/scaffold.test.ts
```

Result: PASS / no output.

## Findings

No blocking findings.
