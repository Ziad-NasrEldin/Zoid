# Feature Handoff: Zoid home Hermes chat + Brain feedback pass

## Original request

User supplied Page Feedback for `/` in Zoid 25 and asked to fix:

1. Brain workspace does not follow the design system used across other pages.
2. Brain note `Extract tasks` does not work; investigate and fix.
3. Hermes topbar is missing the Japanese/kana text used in Automations and Settings headers.
4. Hermes header should match the rest of the pages.
5. Chat should smoothly auto-scroll down whenever a new message is sent/received so the newest message is not hidden.
6. All message bubbles need typography improvement.
7. Bubble box containers are a little too big; make them smaller and align/profile pictures adapt.
8. Composer section feels boring; enhance it.
9. Footer/session stats strip is monotone; improve it.
10. Session rail images are not visible; remove the artifact above the image and keep only the blur effect.

## Implementation summary

- Added a Hermes title kana line (`会話`) and Brain title kana line (`記憶`) to align page headers with the other Zoid 25 pages.
- Added smooth message-list auto-scroll on active session/message updates.
- Tightened message bubble padding, type size/line-height, action spacing, and medium avatar size/alignment.
- Enhanced composer and stats strip with design-system gradients, dashed framing, stronger input/send/attach states, and colored metric cells.
- Fixed session portrait layering: removed the portrait mark artifact and made the blurred portrait image visible instead of being hidden under opaque layers.
- Restyled Brain workspace using the same bordered/paper/shadow header and gradient workspace vocabulary as Automations/Settings, plus stronger panels/list hover states.
- Investigated Brain extraction: the backend only extracted lines that looked like tasks (`TODO`, checkboxes, numbered lists, imperative starts). Notes that had a valid task-like title but prose body produced no candidates, so the button looked broken. Added a fallback candidate from note title, then first meaningful body segment.
- Hardened fallback safety after critique: placeholder titles such as `Untitled`, `Untitled note`, and `New note` no longer create task candidates unless the body contains meaningful task-like prose.
- Added Rust regression tests for the meaningful-title fallback and empty/untitled no-candidate path.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`: kana header, smooth auto-scroll ref/effect, removed session portrait artifact span.
- `src/brain/BrainWorkspace.tsx`: Brain kana line and header content structure.
- `src/App.css`: scoped visual refinements for Hermes header, portrait rail, message bubbles, composer, stats footer, and Brain workspace.
- `src-tauri/src/lib.rs`: Brain extraction fallback, placeholder title guard, and regression tests.

## How to test

- `npm run build`
- `cargo test --manifest-path src-tauri/Cargo.toml brain_extraction -- --test-threads=1`
- Native closeout after critique: rebuild/reinstall/relaunch `/Applications/Zoid 25.app` and visually verify `/` in the installed app.

## Tests run

- `cargo fmt --manifest-path src-tauri/Cargo.toml && npm run build`: PASS. Frontend production build completed.
- `cargo test --manifest-path src-tauri/Cargo.toml brain_extraction -- --test-threads=1`: PASS. 3 focused Brain extraction tests passed, including `brain_extraction_does_not_create_candidate_from_empty_untitled_note` and `brain_extraction_falls_back_to_note_title_when_no_imperative_lines`. Existing Rust warnings remain about unused functions.

## Git info

- Branch: `main`
- Commit SHA: `424be61`
- Scope note: repository already has a large dirty/untracked working tree from broader Zoid work. Review this handoff as a scoped change to the four files listed above and do not treat unrelated dirty files as part of this batch.

## Frontend/backend/database notes

- Frontend routes/components: home `/` Hermes/Brain workspace UI.
- Backend: Tauri Rust Brain extraction helpers only.
- Database: no migration/schema changes.

## Reviewer focus areas

- Check that the auto-scroll effect targets the real scroll container and does not fight user scroll more than necessary.
- Check that portrait pseudo-elements no longer hide images or add the unwanted mark/artifact.
- Check CSS overrides are scoped enough for current Zoid design-system consistency.
- Check Brain fallback extraction is safe: should not create a candidate from empty/untitled notes unless meaningful body text exists.
- Check tests and build evidence.

## Fix cycle notes

- Re-review request after fixing critique R1: added `placeholder_note_title`, blocked common placeholder note titles, added no-candidate test for empty `Untitled` notes, reran frontend build and focused Brain extraction tests.
