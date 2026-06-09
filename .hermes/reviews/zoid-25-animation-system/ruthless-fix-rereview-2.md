APPROVED

Scope: re-reviewed the current working-tree diff against HEAD after the AgentMonitorPanel accessibility correction.

Findings:
- No ARIA button wrapper remains around the AgentMonitorPanel textarea/buttons. The panel root is an `article` with `role="group"`, not `role="button"`, and nested controls remain native controls.
- Keyboard-equivalent panel focus/select behavior remains: the panel root is focusable with `tabIndex={0}`, and Enter/Space on the panel itself call `onFocus(session.id)`. Key events from nested textarea/buttons are ignored by the `event.target !== event.currentTarget` guard.
- Active-run cleanup remains sound: stale finalizers now clear `activeHermesRunsRef` only when the stored `assistantId` matches the finishing run, and runtime completion is also guarded by run id.
- Permission-marker fallback remains sound: marker persistence failures are downgraded only for marker create/save errors, while path validation and real filesystem/Git errors still propagate.

Verification:
- `npm exec tsx src/agents/dashboardLayoutState.test.ts`
- `npm exec tsx src/agents/AgentsHermesScreen.file-manager.test.tsx`
- `cargo test --manifest-path src-tauri/Cargo.toml --lib -- --test-threads=1 file_manager_listing_does_not_fail_when_permission_marker_cannot_persist scan_repository_folder_does_not_fail_when_permission_marker_cannot_persist github_branch_lookup_does_not_fail_when_permission_marker_cannot_persist`
