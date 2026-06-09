# Hermes topbar control spacing

## Scope
User said the repository dropdown, Files button, and CLI online/status control were cramped and needed appropriate spacing.

## Changes
- `src/agents/AgentsHermesScreen.tsx`
  - Added Japanese status text beside the Hermes CLI state using `CONNECTION_STATE_JAPANESE`.
  - Status now renders as a two-part label, e.g. `Hermes CLI ERROR` + `エラー`.
- `src/App.css`
  - Converted the sumi-e Hermes topbar into an actual grid for this layout.
  - Moved the status/repository/files control strip onto its own full-width second row instead of squeezing it into the title row.
  - Increased control spacing from 8px to 14px and gave the strip explicit roomy columns:
    - connection: min 220px flexible
    - repository: min 320px flexible
    - files: min 148px
  - Increased controls from 36px to 44px high.
  - Added padding/gap for the repository control and Files button so bilingual labels do not feel compressed.
  - Added an agents-scoped responsive override so global `.file-manager-toggle-button span { display: none; }` does not hide `Files / 書類` below 1180px.
- `src/scaffold.test.ts`
  - Updated source guards for the new spacious topbar row, 44px controls, Japanese status label, responsive file-label visibility, and wider Files button.

## Validation
- `npm run build` passed.
- `npm run test:frontend` passed.
- Browser computed layout on Agents page:
  - status: 255×44
  - repository: 354×44
  - files: 148×44
  - gaps: 14px and 14px
  - strip: 784×44
  - text includes `HERMES CLI ERROR / エラー`, `REPOSITORY / 接続 / Unlinked / 未接続`, and `FILES / 書類`.

## Required review fix completed
- First critique requested keeping Files labels visible at `max-width: 1180px` despite the global hide rule.
- Added `.agents-sumi-e .file-manager-toggle-button span { display: inline; }` in the agents responsive block and guarded it in `src/scaffold.test.ts`.

## Review focus
Confirm the three topbar controls are no longer cramped, spacing/height/columns are appropriate, bilingual labels remain visible including responsive Files labels, and the layout does not regress earlier compactness/alignment requirements.
