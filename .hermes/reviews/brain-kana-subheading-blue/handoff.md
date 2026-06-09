# Feature Handoff: Brain kana subheading blue

## Original request

Page Feedback: / — paragraph "記憶" in `body > div#root > main.zoid25-shell > section.brain-workspace-shell > header.brain-hero > div > p.kana-line` is still gray. Make the Brain subheading blue too, like the rest of the pages.

## Implementation summary

- Fixed the Brain hero kana selector specificity so the earlier `.brain-hero p:not(.eyebrow)` muted paragraph color no longer overrides `.kana-line`.
- Kept normal Brain body/helper paragraphs muted through the existing `.brain-hero p:not(.kana-line)` rule.
- Added a source guard requiring the Brain-specific `p.kana-line` blue rule.

## Changed files

- `src/App.css`: changed `.brain-hero .kana-line` to `.brain-hero p.kana-line` so it wins over the generic Brain paragraph selector.
- `src/scaffold.test.ts`: extended the all-page kana blue guard to include Brain.

## How to test

- `npm run test:frontend && npm run build`
- `npm run tauri:build`
- Browser DOM check on the Brain page: `.brain-hero .kana-line` computed color should be `rgb(53, 88, 162)`.
- Installed app from `/Applications/Zoid 25.app` should show Brain workspace with the `記憶` subheading in blue.

## Tests run

- `npm run test:frontend && npm run build`: PASS.
- Browser computed-style check on Brain page at `http://127.0.0.1:1420/`: PASS, `記憶` color returned `rgb(53, 88, 162)` and browser console had no JS errors.
- `npm run tauri:build`: PASS, with existing Rust dead-code warnings for `apply_profile_runtime_args` and `prompt_with_enabled_profile_context`.
- Replaced `/Applications/Zoid 25.app`, relaunched `/Applications/Zoid 25.app/Contents/MacOS/zoid`, clicked Brain in the native app, and captured `/tmp/zoid-brain-kana-native-after-click2.png`: PASS, Brain workspace visible and `記憶` appears blue.

## Git info

- Branch: main
- Commit SHA, if committed: not committed
- Diff base, if known: working tree has substantial pre-existing unrelated dirty/untracked Zoid work; intended change is scoped to `src/App.css`, `src/scaffold.test.ts`, and this review folder.

## Frontend/backend/database notes

- Frontend routes/components: Brain workspace hero CSS only.
- Backend endpoints/services: none.
- Database tables/migrations: none.

## Reviewer focus areas

- Confirm the CSS specificity fix addresses the exact reported `p.kana-line` under `.brain-hero`.
- Confirm body/helper text in the Brain hero remains muted and only the kana subheading turns blue.
- Confirm the source guard protects against this regression.

## Fix cycle notes

Initial review request.
