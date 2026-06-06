# Feature review handoff: Agents Hermes chat window

## Original request
Implement the planned Agents module slice: a Hermes chat window in the clean Zoid 25 macOS app scaffold. Keep old archived UI out. User wants a clean restart with navigation/sidebar and a basic scaffold, now focused on one Agents/Hermes chat module.

## Scope implemented
- Replaced blank canvas with `AgentsHermesScreen`.
- Set sidebar `Agents` row as active.
- Added user/Hermes participants and circular avatar/profile rendering.
- Added WhatsApp-like chat layout with sender metadata, bubbles, bottom composer, status strip.
- Added composer with Enter-to-send and Shift+Enter newline behavior.
- Added Tauri backend commands:
  - `check_hermes_health`
  - `send_hermes_message`
- Backend reads local Hermes API config from env:
  - `ZOID_HERMES_API_BASE_URL` or `HERMES_API_BASE_URL`, default `http://127.0.0.1:8642`
  - `ZOID_HERMES_API_KEY` or `API_SERVER_KEY`
  - `ZOID_HERMES_MODEL`, default `hermes-agent`
- API key stays in Rust backend; React only invokes Tauri commands.
- If Hermes API is offline/unauthorized, UI says so and send is disabled.
- Changed Tauri bundle target to `app` only because prior `dmg` bundling failed while `.app` builds successfully.

## Changed files
- `src/App.tsx`
- `src/App.css`
- `src/scaffold.test.ts`
- `src/agents/types.ts`
- `src/agents/participants.ts`
- `src/agents/hermesClient.ts`
- `src/agents/Avatar.tsx`
- `src/agents/MessageBubble.tsx`
- `src/agents/ChatComposer.tsx`
- `src/agents/AgentsHermesScreen.tsx`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/lib.rs`
- `src-tauri/tauri.conf.json`

## Verification already run
- RED before implementation: `npm run test:frontend` failed with missing active Agents screen.
- GREEN:
  - `npm run test` passed.
  - `npm run build` passed as part of Tauri build.
  - `npm run tauri:build` passed after switching bundle targets to app only.
  - Packaged app launched from `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`; process observed at PID 81257, then killed.
- Hermes API server local health check:
  - `curl -fsS http://127.0.0.1:8642/health || true` returned connection failure, so real chat was not verified. UI is expected to show offline state.

## Reviewer focus
1. Does the implementation match the request and plan enough for this slice?
2. Any frontend issues in `AgentsHermesScreen` state handling, accessibility, or type safety?
3. Any Tauri command/security issues, especially secret handling and error output?
4. Is the changed bundle target acceptable, or should script/config be handled differently?
5. Identify Required fixes only for blockers to approve this feature.

## Known limitations
- Streaming SSE was not implemented; current command is non-streaming request/response with a `streaming` UI status while waiting. If streaming is a must-have for this approval, mark it required.
- Real Hermes response was not tested because the local Hermes API server was offline.
- No DOM testing library was added; tests are source-contract plus Rust unit tests.
