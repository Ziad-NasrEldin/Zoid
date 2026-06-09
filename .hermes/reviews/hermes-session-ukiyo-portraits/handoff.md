# Feature Handoff: Hermes session ukiyo-e portraits

## Original request

implement

Context from immediately preceding user request: “can you replace the images in the session icons in the sessions rail with images from ukiyuo art https://en.wikipedia.org/wiki/Ukiyo-e i want you to get a list of 50 ukiyo characters that fit to be a profile icon for sessions and they randomize uniquely whenever you start a new session what do you think ?”

## Implementation summary

- Replaced the old historical-figure session portrait pool with 50 curated ukiyo-e/kabuki character subjects.
- Added local `/public/session-ukiyo/*.svg` portrait assets so the installed Tauri app does not depend on remote image loading.
- Each portrait entry includes a stable id, display name, local asset, visual tokens, source title, and Wikimedia Commons source URL for the public-domain ukiyo-e subject used as the character reference.
- New sessions now persist a `portraitId` and choose a unique portrait from unused ids until the 50-icon pool is exhausted, then reuse begins.
- Existing sessions without a stored portrait id are migrated on localStorage hydration to unique portrait ids where possible.
- Session rail rendering now resolves portraits by stored `portraitId`, with hash fallback for older/malformed sessions.
- Maintained the no-blur expanded portrait guard from the prior feedback fix.

Known limitation / reviewer focus: the local assets are stylized SVG profile icons based on curated public-domain ukiyo-e subject metadata, not direct raster crops from Commons. Direct bulk thumbnail download from Wikimedia hit 429/robots throttling during implementation, and Art Institute IIIF thumbnails were Cloudflare-challenged. The code stores source titles/URLs for traceability.

## Changed files

- `src/agents/sessionPortraits.ts`: new 50-item ukiyo-e subject metadata pool, stable hash fallback, and unique portrait id chooser. Assets are local stylized SVG profile icons inspired by the public-domain subjects; metadata uses `inspirationTitle`/`inspirationUrl` for traceability.
- `src/agents/AgentsHermesScreen.tsx`: adds `portraitId` to session data, assigns unique portrait ids on new session, renders portraits by stored id.
- `src/App.tsx`: accepts/migrates optional `portraitId` when hydrating persisted Hermes sessions.
- `src/scaffold.test.ts`: guards 50 ukiyo-e assets, required unique-assignment helper, no-blur portrait styling, and uniqueness-before-reuse behavior.
- `public/session-ukiyo/*.svg`: 50 local ukiyo-e-style session portrait assets.

## How to test

- Run `npm run test:frontend`.
- Run `npm run build`.
- Run `npm run tauri:build`.
- Install the bundled app to `/Applications/Zoid 25.app` and launch `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Open Agents / Hermes and create multiple new sessions. Expected: each new session gets a different ukiyo-e character icon until the 50 portrait pool is exhausted; icons are sharp/non-blurred.

## Tests run

- `npm run test:frontend`: PASS.
- `npm run build`: PASS; Vite chunk-size warning only.
- `npm run tauri:build`: PASS; existing Rust dead-code warnings for `apply_profile_runtime_args` and `prompt_with_enabled_profile_context` only.
- Installed/relaunched `/Applications/Zoid 25.app`; running process verified with `pgrep -fl "/Applications/Zoid 25.app/Contents/MacOS/zoid"`.

## Git info

- Branch: current working tree in `/Users/ziadnasreldin/Zoid`.
- Commit SHA: not committed.
- Diff base: existing repo has many unrelated dirty/untracked files; review should scope to the files listed above.

## Frontend/backend/database notes

- Frontend only.
- No backend endpoints, Tauri commands, or database migrations changed for this feature.
- Persisted data shape change is optional `HermesChatSession.portraitId`; older sessions remain accepted and are migrated in memory/on next localStorage save.

## Reviewer focus areas

- Does the implementation satisfy “50 ukiyo characters that fit to be a profile icon for sessions” as stylized local icons inspired by public-domain subjects?
- Is unique randomization/persistence correct for new sessions, restored sessions, and existing sessions?
- Are old/no-portrait sessions handled safely?
- Does the sessions rail remain sharp/no blur?
- Are tests sufficient to prevent regressions?

## Fix cycle notes

Initial review request.
