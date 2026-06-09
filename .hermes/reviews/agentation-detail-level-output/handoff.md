# Feature Handoff: Agentation detail-level output

## Original request

User reported that in Zoid 25, Agentation settings → detail level did not appear to change anything between normal/standard and forensic, and asked to make sure it works.

## Implementation summary

- Confirmed Agentation already passes `settings.outputDetail` into copied/sent output generation.
- Made the selected output detail visibly explicit in generated Agentation output with an `Output Detail` line, so Compact/Standard/Detailed/Forensic are easy to distinguish.
- Added persisted-setting validation so stale/invalid stored values fall back to Standard instead of silently behaving ambiguously.
- Added a Vite alias so Zoid 25 uses the local fixed Agentation bundle while preserving the existing `import { Agentation } from "agentation"` source API/types.
- Added scaffold regression checks for the alias and visible detail-level output marker.

## Changed files

- `vite.config.ts`: aliases `agentation` to the local patched entry.
- `src/vendor/agentation-fixed.mjs`: local Agentation 3.0.2 bundle with detail-level output marker and saved-setting sanitization.
- `src/scaffold.test.ts`: regression checks that the fixed local Agentation entry is used and that copied/sent output includes the selected detail level.

## How to test

- Open `/Applications/Zoid 25.app`.
- Use Agentation, switch Settings → Output Detail between Standard/Detailed/Forensic, then copy/send annotations.
- Expected: generated output includes `Output Detail: <selected level>` and the existing level-specific sections still differ (Forensic includes environment/full DOM/computed/accessibility data when available; Detailed includes classes/position/context; Standard remains lighter).

## Tests run

- `npm run test:frontend`: PASS
- `npm run build`: PASS
- `npm run tauri:build`: PASS
- Reinstalled/relaunched `/Applications/Zoid 25.app`: PASS, process `/Applications/Zoid 25.app/Contents/MacOS/zoid`
- Native screenshot check: PASS, Zoid 25 running with Agentation floating widget visible
- Built bundle search: PASS, production JS contains `**Output Detail:**`

## Git info

- Branch: current working tree
- Commit SHA: not committed
- Diff base: current working tree has unrelated pre-existing dirty/untracked Zoid work; review should scope only the changed files above.

## Frontend/backend/database notes

- Frontend only.
- No backend/database changes.

## Reviewer focus areas

- Verify the Vite alias is safe with TypeScript still using the package types.
- Verify copied/sent Agentation output now has an obvious detail-level difference.
- Verify this does not accidentally remove global Agentation from Zoid 25.
