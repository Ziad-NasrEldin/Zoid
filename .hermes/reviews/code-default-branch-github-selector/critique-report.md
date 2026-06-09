# Critique Report: Code page default-branch GitHub selector re-review

Verdict: APPROVED

## Scope

Re-reviewed the Zoid 25 Code page repository-card default-branch selector after the fix cycle. Scope was limited to the previously requested fixes for the default-branch GitHub selector and its tests, per handoff instructions, despite a broad dirty tree in the checkout.

## What I inspected

- `src/code/CodeWorkspace.tsx`
- `src/code/repositoryClient.ts`
- `src/code/types.ts`
- `src/App.css`
- `src-tauri/src/lib.rs`
- `src/scaffold.test.ts`
- `package.json`
- `.hermes/reviews/code-default-branch-github-selector/handoff.md`

## Findings

### Previously required fix 1: visible status/error feedback

Status: resolved.

`CodeWorkspace` now tracks `defaultBranchStatus` and `defaultBranchError`, and renders a visible feedback panel with `role="status"` and `aria-live="polite"` whenever either value is present.

Confirmed visible messages exist for the branch list/update flow:

- Loading branch list: `Loading GitHub branches for ...`
- Branch selection ready: `Select a default branch for ...`
- Branch list failure: `Default branch selection failed: ...`
- Cancel: `Default branch edit cancelled.`
- Update in progress: `Updating GitHub default branch for ...`
- Update success: `Default branch updated to ...`
- Update failure: `Default branch update failed: ...`

The feedback is styled in `src/App.css` via `.default-branch-feedback` and `.default-branch-feedback--error`.

### Previously required fix 2: tests for feedback

Status: resolved for the scoped regression coverage.

`src/scaffold.test.ts` now asserts the presence of the feedback/status/error implementation strings including:

- `default-branch-feedback`
- `defaultBranchError`
- `Default branch update failed`
- `Default branch updated to`

The test remains mostly structural/string-based rather than a full UI interaction test, but it covers the specific regression requirements from the previous critique: the selector must be visible, GitHub-backed, not `window.prompt`, and must include visible feedback strings.

### Previously required fix 3: reproducible Rust test script

Status: resolved for Rust.

`package.json` now uses:

`cargo test --manifest-path src-tauri/Cargo.toml --lib --bins -- --test-threads=1`

I ran `npm run test:rust`; it passed:

- 20 Rust tests passed
- 0 failed
- `github_branch_lookup_uses_gh_api_and_marks_default` passed

### GitHub-backed branch list/update wiring

Status: acceptable.

Frontend:

- `listGithubBranches(repository.path)` is called when clicking Edit.
- The inline `GlobalDropdown` is populated from returned branch names.
- The current default branch is preserved as an option if the GitHub branch list omits it.
- Save calls `updateGithubDefaultBranch(repository.path, selectedBranch)`.
- On successful save, repository state is updated and edit state/options/selection are cleared.
- On failure, the selector remains available and visible error feedback is shown.

Backend:

- `list_github_branches` is registered with Tauri and calls `list_remote_branches`.
- `list_remote_branches` resolves the GitHub slug from the origin remote and calls `gh api repos/{slug}/branches --paginate --jq .[].name`.
- Branches are deduplicated, empty names are rejected, and the current default is marked with `isDefault`.
- `update_github_default_branch` remains registered and calls the existing update path.

No hardcoded branch list or secret exposure was found in the scoped implementation.

## Verification run

Commands run from `/Users/ziadnasreldin/Zoid`:

1. `npm run test:rust`
   - Result: PASS
   - Rust tests: 20 passed, 0 failed

2. `npx tsx src/ui/GlobalDropdown.behavior.test.tsx`
   - Result: PASS exit code 0
   - Output included React act-environment warnings, but no test failure.

3. `npm run test:frontend && npm run test:rust`
   - Result: FAIL before Rust tests
   - Failure:
     - `src/scaffold.test.ts:278`
     - `Error: Composer textarea needs command mode and auto-height behavior: composerHeightRef`
   - This appears unrelated to the default-branch selector scope. The default-branch scaffold assertions are earlier in the same file and were not the failing assertion. Given the explicit instruction to keep this review scoped to the default-branch selector despite the broad dirty tree, I am not treating this unrelated Composer assertion as a blocker for this feature verdict.

## Concerns / non-blocking notes

- The frontend regression coverage for default-branch feedback is structural rather than behavioral. It verifies required implementation strings, but it does not simulate Edit/list/save/failure UI state transitions. This satisfies the prior requested feedback coverage at the scaffold level, but a future dedicated component test would be stronger.
- Full `npm test` is currently not green in my checkout because of an unrelated Composer scaffold assertion. This should be addressed separately if the intended acceptance criterion is that the whole dirty tree must pass `npm test` at all times.

## Final verdict

APPROVED for the default-branch GitHub selector fix.

The prior default-branch selector review requirements are satisfied: visible feedback exists for branch list/update operations, scoped regression assertions cover the feedback strings, Rust test execution is reproducible and passing, and the selector is wired through the Tauri/GitHub branch list and default-branch update commands.
