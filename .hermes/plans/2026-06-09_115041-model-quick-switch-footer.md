# Plan: Add quick model/reasoning button to Hermes footer model stat

## Goal

Add a small button on the right side of the footer Model stat section in the Hermes chat screen (`/`, `tauri://localhost`) so the user can quickly change the active model and reasoning effort without leaving the chat.

The target DOM from feedback is:

- `body > div#root > main.zoid25-shell > section.hermes-chat-shell > footer.chat-stats-strip > span`
- Current copy: `Model gpt-5.5 · Codex 5h / 5h / week`
- Desired behavior: small right-aligned control inside this section for model + reasoning changes.

## Current context / assumptions

- Main affected screen: `src/agents/AgentsHermesScreen.tsx`.
- Footer currently renders the model stat as plain text:
  - `src/agents/AgentsHermesScreen.tsx:1296-1301`
  - `<span><b>Model</b> {ACTIVE_MODEL} · Codex {CODEX_USAGE_TODAY} / {CODEX_USAGE_WEEKLY}</span>`
- The app already has a native command panel system:
  - `HermesCommandPanel` includes `"model"` in `src/agents/hermesCommands.ts`.
  - `COMMAND_PANEL_COPY.model` already says: `Review or change the active session model and reasoning settings.`
  - `activeCommandPanel` opens a dialog at `src/agents/AgentsHermesScreen.tsx:1318+`.
- Profile/model settings already exist on the Settings/Profile page:
  - `src/agents/hermesProfileClient.ts` defines `modelProvider`, `modelName`, `reasoningEffort`, `availableModels`.
  - `src/App.tsx:605-615` builds provider/model/reasoning dropdown options.
  - `src/App.tsx:718-725` renders Models & reasoning fields.
- Styling for the stats strip is in:
  - `src/App.css:682-687`
  - `src/App.css:1024-1041`, `1101`, `1134-1135` for the Agents sumi-e variant.
- Existing test coverage includes:
  - `src/scaffold.test.ts`, with assertions around `chat-stats-strip`.
  - `src/ui/GlobalDropdown.behavior.test.tsx` for dropdown behavior.
  - Hermes/Agents behavior tests under `src/agents/`.

Assumption to validate during implementation: the quick button should open a native Zoid model panel that can directly edit/persist the same Hermes profile settings as the Settings page, not just insert/forward a slash command.

## Proposed approach

1. Turn the Model stat span into a compact flex row with:
   - Left: current model and Codex usage text.
   - Right: small action button, e.g. `Change` / `Model` / `⚙︎`, with accessible label `Change model and reasoning`.
2. Reuse the existing native command panel infrastructure by opening `activeCommandPanel = "model"` from that button.
3. Upgrade the `"model"` command panel content from placeholder copy into a functional quick-switch control:
   - Provider dropdown.
   - Model dropdown filtered by provider.
   - Reasoning effort dropdown.
   - No separate Save/Apply button; selecting a value applies immediately.
   - Clear transient status/error text only when needed.
4. Reuse `HermesProfileSettings`, `loadHermesProfileSettings`, and `saveHermesProfileSettings` so each selection writes to the same profile backing store as the Settings page.
5. Keep the footer compact and preserve the sumi-e design: small paper/ink button, no bulky card grid, no generic SaaS styling.

## Step-by-step implementation plan

### 1. Inventory current model/profile state usage

- Inspect imports and state in `src/agents/AgentsHermesScreen.tsx`.
- Confirm whether it already loads `HermesProfileSettings`; if not, add imports from `src/agents/hermesProfileClient.ts`:
  - `loadHermesProfileSettings`
  - `saveHermesProfileSettings`
  - `defaultHermesProfileSettings`
  - type `HermesProfileSettings`
- Confirm whether `ACTIVE_MODEL` is static or derived from profile/session. If static, plan to display profile `modelName` once loaded, falling back to `ACTIVE_MODEL`.

### 2. Add local profile/model state to `AgentsHermesScreen`

Add state near existing component state:

- `const [profileSettings, setProfileSettings] = useState<HermesProfileSettings | null>(null);`
- `const [modelPanelDraft, setModelPanelDraft] = useState({ provider, model, reasoning });`
- `const [modelPanelStatus, setModelPanelStatus] = useState<string | null>(null);`
- `const [modelPanelError, setModelPanelError] = useState<string | null>(null);`
- `const [isUpdatingModelPanel, setIsUpdatingModelPanel] = useState(false);`

Load profile settings on mount using `loadHermesProfileSettings()`.

When settings load:

- Store them in `profileSettings`.
- Initialize the model panel draft from `modelProvider`, `modelName`, and `reasoningEffort`.
- If load fails, keep the chat usable and show a disabled/error state in the model panel.

### 3. Build provider/model/reasoning options inside the Hermes screen

Mirror the Settings page logic from `src/App.tsx:605-615`, but scoped locally:

- `availableModelOptions = profileSettings?.availableModels ?? defaultHermesProfileSettings.availableModels`
- `providerOptions`: include keys plus current provider.
- `modelOptions`: include models for selected provider plus current draft model.
- `reasoningOptions`: `off`, `minimal`, `low`, `medium`, `high`, `xhigh`.

When provider changes:

- Immediately update the draft provider and persist the new provider.
- If the current model is not valid for the selected provider, automatically select the first available model for that provider and persist that paired model value too.
- Do not show a separate Save/Apply button; each dropdown selection is the action.
- Show a short `Updating…` / `Updated` / error status so the user knows the change was written.

### 4. Add the footer quick button

Replace the model span in `src/agents/AgentsHermesScreen.tsx:1299` with a structured element, for example:

```tsx
<span className="chat-stats-model-section">
  <span className="chat-stats-model-copy">
    <b>Model</b> {activeModelLabel} · Codex {CODEX_USAGE_TODAY} / {CODEX_USAGE_WEEKLY}
  </span>
  <button
    aria-label="Change model and reasoning"
    className="chat-stats-model-button"
    onClick={() => setActiveCommandPanel("model")}
    title="Change model and reasoning"
    type="button"
  >
    Model
  </button>
</span>
```

Implementation details:

- Keep the button inside the third stats section so it appears on the right of that section.
- Prevent nested interactive issues by ensuring the outer element remains a non-interactive `span` or, if layout requires, switch to a `div` only if CSS/tests allow it.
- Preserve existing stat strip children count/order if tests rely on `span:nth-child(3)` and `span:nth-child(4)`.
- Use `activeModelLabel = profileSettings?.modelName || ACTIVE_MODEL`.
- Consider showing reasoning in the tooltip or button title, e.g. `Change model and reasoning: medium`.

### 5. Implement functional `activeCommandPanel === "model"` content

Inside the existing native command panel render block after the shared intro paragraph:

- Add a conditional render for `activeCommandPanel === "model"`.
- Use `GlobalDropdown` from `src/ui/GlobalDropdown.tsx` for consistency.
- Render:
  - Current active profile/storage line.
  - Provider dropdown.
  - Model dropdown.
  - Reasoning effort dropdown.
  - Status/error message.

Behavior:

- Each dropdown selection immediately builds `nextSettings = { ...profileSettings, modelProvider, modelName, reasoningEffort }` and calls `saveHermesProfileSettings(nextSettings)`.
- Provider changes must be provider-aware: if the selected provider does not support the current model, select the first available model for that provider and save both provider + model together.
- Model changes save the selected model for the active draft/provider.
- Reasoning changes save the selected reasoning effort.
- Update `profileSettings` and draft from the saved result after each successful save.
- Show `Updating…` while saving and `Updated` or `Saved to ${saved.storagePath}` after success.
- Disable only the specific dropdowns while an update is in flight if needed to avoid overlapping writes.
- If save fails, keep the previous saved settings as source of truth and show an error.

Important boundary:

- This changes profile settings for newly launched/managed Hermes sessions. If the currently running Hermes CLI process cannot hot-swap models mid-run, the UI must say so plainly, e.g. `Applies to new Zoid-launched Hermes sessions; active CLI hot-swap depends on Hermes runtime support.`
- Do not claim live runtime model has changed unless the Hermes bridge confirms it.

### 6. Style the footer button and model panel

Update `src/App.css`:

Base stats strip:

- `.chat-stats-model-section` as inline flex, align center, gap, and right-justify the button.
- `.chat-stats-model-copy` with `min-width: 0; overflow: hidden; text-overflow: ellipsis;`.
- `.chat-stats-model-button` small, high-contrast, 28-32px height, preserves current ink/paper style.

Responsive behavior:

- At desktop width, button sits at the far right of the model stat section.
- At narrow width, button remains visible and copy truncates before the button.
- Ensure it does not make the footer overflow horizontally at 1758×982 or mobile widths.

Agents sumi-e overrides:

- Add `.agents-sumi-e .chat-stats-model-button` using `var(--agents-ink-black)`, `var(--agents-paper)`, `var(--agents-pale-rule)`, serif typography, and low visual weight.
- Maintain readable bilingual-friendly spacing/lettering.

Panel styles:

- Add `.model-command-panel`, `.model-command-grid`, `.model-command-actions`, `.model-command-status` or similar.
- Reuse existing `.zoid-native-command-panel` scale; do not create a large settings-page clone.

### 7. Add/update tests

Update `src/scaffold.test.ts` to assert static structure/style expectations:

- Footer includes `chat-stats-model-section`.
- Footer includes a button/control with accessible label or title for changing model and reasoning.
- CSS includes `.chat-stats-model-button` and sumi-e override.
- Preserve existing `chat-stats-strip` sizing assertions; adjust only if the markup change makes old assumptions invalid.

Add or update a behavior test if practical, likely under `src/agents/`:

- Render `AgentsHermesScreen` with enough mocked dependencies to click the footer model button.
- Assert the model command panel opens with Provider, Model, Reasoning effort controls.
- If Tauri invoke mocking exists in the test harness, assert Save calls the profile save path with updated `modelName`/`reasoningEffort`.

If a full screen behavior test is too heavy, add focused test coverage around any extracted `ModelCommandPanel` component.

### 8. Validation commands

Run from `/Users/ziadnasreldin/Zoid`:

- `npm run test:frontend`
- `npm run build`
- If Rust/Tauri bridge code is touched, also run `npm run test:rust`.
- Start the app with the existing dev flow and visually verify in the Tauri window:
  - Footer Model stat at 1758×982 has a small right-side button.
  - Button opens the model/reasoning panel.
  - Dropdowns are usable and not clipped.
  - Save persists and updates footer label.
  - Narrow viewport/footer wrapping does not hide the button.

### 9. Required critique gate before completion

Because this is a feature change, completion requires the feature critique workflow unless explicitly waived.

Implementation agent must:

1. Create `.hermes/reviews/model-quick-switch-footer/handoff.md` with:
   - Goal and user feedback.
   - Changed files.
   - Test/build output.
   - Known boundaries, especially whether model switch is profile-only or live runtime.
2. Trigger/wait for the separate critique-agent review.
3. Fix all Required fixes.
4. Re-review until verdict is `APPROVED`.
5. Only then report the feature as complete.

## Files likely to change

- `src/agents/AgentsHermesScreen.tsx`
  - Footer Model stat markup.
  - Profile/model settings state.
  - Functional model command panel.
- `src/App.css`
  - Footer model button styling.
  - Model command panel styling.
  - Agents sumi-e overrides.
- `src/scaffold.test.ts`
  - Static CSS/markup assertions.
- Potential new or updated behavior test:
  - `src/agents/AgentsHermesScreen.model-panel.test.tsx` or similar.
- Required review handoff during implementation:
  - `.hermes/reviews/model-quick-switch-footer/handoff.md`

## Tests / validation

Minimum acceptance checks:

- The footer Model stat section still shows the active model and Codex usage.
- A small button appears on the right side of that section at 1758×982.
- Button has accessible name: `Change model and reasoning` or equivalent.
- Clicking the button opens a native Zoid model controls panel.
- Panel allows changing provider, model, and reasoning effort directly from dropdowns with no Save/Apply button.
- Provider changes are provider-aware: the model list changes with provider, and an invalid previous model is replaced with a valid provider model.
- UI clearly communicates whether changes apply to active session immediately or future/new sessions only.
- No horizontal overflow or hidden button in desktop and narrow layouts.
- `npm run test:frontend` passes.
- `npm run build` passes.
- Feature critique verdict is `APPROVED`.

## Risks, tradeoffs, and open questions

- Runtime semantics risk: existing profile settings may only affect new Zoid-launched Hermes sessions, not the already-running CLI session. The implementation must not imply hot-swapping unless Hermes supports it through the bridge.
- Duplication risk: Settings page already has model dropdown logic. Duplicating option-building in `AgentsHermesScreen` is acceptable for a small quick panel, but if it grows, extract shared helpers.
- Layout risk: current `chat-stats-strip` relies on `span:nth-child(3)` and `span:nth-child(4)`. Preserve child order and keep inner elements from breaking existing CSS/tests.
- Visual risk: the user asked for a small button, so avoid a large pill or toolbar that competes with the footer stats.
- Open question for implementation only if runtime behavior is unclear: should Save apply only to future sessions, or should Zoid also attempt a Hermes runtime model switch command/API when available?

Recommended answer: implement profile persistence now, label the scope honestly, and only add true live-session switching if an existing Hermes bridge command supports it safely.
