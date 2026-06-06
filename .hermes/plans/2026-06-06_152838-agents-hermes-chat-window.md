# Agents Hermes Chat Window Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Build the first working Agents module screen in Zoid 25: a Hermes chat window with user and agent avatars/profile pictures, message bubbles, streaming responses, and a composer so Ziad can chat with Hermes without using the terminal.

**Architecture:** Keep the active Zoid 25 scaffold clean, but replace the blank canvas with an Agents/Hermes workspace. The frontend should own chat state and UI; the Tauri backend should proxy requests to the local Hermes API Server so the API key never lives in browser-rendered React code. Hermes integration should use the documented OpenAI-compatible endpoint `POST /v1/chat/completions` on the local gateway API server, default `http://127.0.0.1:8642/v1`, with SSE streaming.

**Tech Stack:** Tauri 2, Rust backend commands, React + TypeScript, Vite, CSS using current Kujoyama editorial tokens, Hermes Agent gateway API server.

---

## Screenshot / visual target

Reference image: `/Users/ziadnasreldin/Library/Application Support/Hermes/composer-images/composer_2026-06-06_12-22-59-742_f5a265.png`

Visible elements to translate, not copy blindly:

- left app navigation with an Agents area and selected `HERMES-AGENT` row;
- top status bar showing operator/local context and Hermes online state;
- central large chat stage with blue/green washed architectural background;
- message bubbles placed spatially, user message on the right with user avatar;
- agent identity/avatar visible on the left;
- composer pinned to bottom with an input and send action;
- stats/status strip below the chat area;
- profile/avatar treatment should feel like messaging someone in WhatsApp, but still inside the Kujoyama editorial OS system.

## Current context

- Project root: `/Users/ziadnasreldin/Zoid`.
- Current app is a clean Zoid 25 scaffold, not the old Zoid AI OS UI.
- Current files are minimal:
  - `src/App.tsx`
  - `src/App.css`
  - `src/main.tsx`
  - `src/scaffold.test.ts`
  - `src/vite-env.d.ts`
  - `src-tauri/src/lib.rs`
- Design system already exists:
  - `PRODUCT.md`
  - `DESIGN.md`
  - `tokens.json`
  - `tailwind.theme.json`
- Keep the Villa Kujoyama-inspired system: black/white, committed cobalt blue `#3558A2`, hard rules, monospaced display typography, list rows over cards, colored status dots with text labels.
- Do not resurrect old archived product UI from `legacy/`.

## Hermes integration notes from docs

Use the official Hermes Agent API Server path:

- Enable gateway API server separately if needed:
  - `hermes config set API_SERVER_ENABLED true`
  - `hermes config set API_SERVER_KEY <secret>`
  - restart gateway: `hermes gateway stop && hermes gateway`
- Default local API: `http://127.0.0.1:8642/v1`
- Health check: `GET http://127.0.0.1:8642/health`
- Models check: `GET http://127.0.0.1:8642/v1/models` with `Authorization: Bearer <key>`
- Chat endpoint: `POST /v1/chat/completions`
- Response supports SSE streaming.
- Important runtime rule: tool calls run on the API-server host. For this local macOS app, that should be the same Mac if using `127.0.0.1`.

## Assumptions for v1

1. The first working slice targets one agent only: `Hermes`.
2. User identity is local static profile data for now: `Ziad` with a configurable initials/photo avatar.
3. Agent identity is local static profile data for now: `Hermes` with a generated/lettermark avatar.
4. Real custom profile-picture upload can come after v1 if needed; v1 should still support avatar image URLs or local asset paths in the data model.
5. Hermes API key must be read by the Tauri backend from environment/config, not hardcoded in frontend source.
6. If the Hermes gateway/API server is not configured yet, the UI should show an actionable offline/blocked state rather than pretending chat works.
7. No terminal embedding in v1. This is a chat client, not a terminal replacement for every slash command yet.

---

## Proposed data model

Create frontend types in `src/agents/types.ts`:

```ts
export type AgentId = "hermes";
export type ChatRole = "user" | "assistant" | "system";
export type AgentConnectionState = "checking" | "online" | "offline" | "error";

export type ChatParticipant = {
  id: "ziad" | AgentId;
  displayName: string;
  handle: string;
  avatarUrl?: string;
  initials: string;
  presence: "online" | "offline" | "thinking";
};

export type ChatMessage = {
  id: string;
  role: Exclude<ChatRole, "system">;
  participantId: ChatParticipant["id"];
  content: string;
  createdAt: string;
  status: "sending" | "streaming" | "sent" | "error";
  error?: string;
};
```

Create Tauri command response types in `src/agents/hermesClient.ts`:

```ts
export type HermesHealth = {
  ok: boolean;
  status: "online" | "offline" | "unauthorized" | "error";
  message: string;
  model?: string;
};
```

Backend Rust types should mirror only the API payloads needed by the commands. Do not overbuild persistence yet.

---

## Step-by-step implementation plan

### Task 1: Add live-state verification gate before coding

**Objective:** Confirm the working tree and scaffold state before changing the Agents module.

**Files:**
- Read: `git status --short`
- Read: `src/App.tsx`
- Read: `src/App.css`
- Read: `src-tauri/src/lib.rs`
- Read: `DESIGN.md`

**Steps:**
1. Run `git status --short` and note that many old files are deleted because they were archived earlier.
2. Confirm active source still has only the clean scaffold files plus design docs.
3. Confirm no live `src-tauri/migrations/*.sql` files exist.
4. Do not restore anything from `legacy/` unless explicitly needed for reference.

**Expected:** Implementation starts from the clean Zoid 25 scaffold, not stale archived UI.

---

### Task 2: Add Hermes chat frontend types

**Objective:** Create typed foundations for agent participants, messages, and connection state.

**Files:**
- Create: `src/agents/types.ts`
- Test: `src/agents/types.test.ts`

**Steps:**
1. Create `src/agents/types.ts` with the types from the proposed data model.
2. Add a tiny test in `src/agents/types.test.ts` that creates a valid user message and Hermes assistant message.
3. Update `package.json` test script later to run all `src/**/*.test.ts` files, or import this test from an index test runner.

**Acceptance:** TypeScript can compile participant/message objects without widening role/status strings incorrectly.

---

### Task 3: Create static participant/avatar seeds

**Objective:** Make user and Hermes profiles first-class UI objects.

**Files:**
- Create: `src/agents/participants.ts`
- Test: `src/agents/participants.test.ts`

**Implementation shape:**

```ts
import type { ChatParticipant } from "./types";

export const userParticipant: ChatParticipant = {
  id: "ziad",
  displayName: "Ziad Salah",
  handle: "operator/local",
  initials: "ZS",
  presence: "online",
};

export const hermesParticipant: ChatParticipant = {
  id: "hermes",
  displayName: "Hermes",
  handle: "hermes-agent",
  initials: "HA",
  presence: "offline",
};
```

**Avatar requirement:**
- If `avatarUrl` exists, render the image.
- If no `avatarUrl`, render initials in a Kujoyama-style avatar frame.
- Keep user avatar and Hermes avatar visually distinct.

**Acceptance:** Tests verify display names, handles, initials, and ids.

---

### Task 4: Add a reusable Avatar component

**Objective:** Implement profile-picture UI for both user and agent.

**Files:**
- Create: `src/agents/Avatar.tsx`
- Create: `src/agents/Avatar.test.ts`
- Modify: `src/App.css` or later split `src/agents/agents.css`

**Behavior:**
- Props: `participant`, `size?: "sm" | "md" | "lg"`, `showPresence?: boolean`.
- Use `<img>` when `avatarUrl` exists.
- Use initials fallback when no image exists.
- Add accessible label: `aria-label="Ziad Salah avatar"` or `aria-label="Hermes avatar"`.
- Presence dot must have text nearby when used in status bars; avatar dot alone can be decorative if a text status exists.

**Visual direction:**
- User avatar: white/black frame, optional warm accent.
- Hermes avatar: cobalt blue field with white `HA` or glyph.
- Shape can be square with slight radius or circular; choose one and keep consistent. Since user explicitly wants profile pictures like messaging apps, use circular avatars, but frame them with hard black/blue editorial borders.

---

### Task 5: Add a MessageBubble component

**Objective:** Render chat messages with WhatsApp-like identity clarity while preserving Zoid 25 style.

**Files:**
- Create: `src/agents/MessageBubble.tsx`
- Create: `src/agents/MessageBubble.test.ts`
- Modify: `src/App.css` or `src/agents/agents.css`

**Behavior:**
- User messages align right, with user avatar on the right.
- Hermes messages align left, with Hermes avatar on the left.
- Message metadata includes participant display name and time.
- Message statuses:
  - `sending`: show `SENDING` pill.
  - `streaming`: show `HERMES WRITING` or animated restrained marker.
  - `sent`: no noisy status unless needed.
  - `error`: show `FAILED` and retry affordance later.

**Visual direction:**
- User bubble: white paper, black 1px border, right aligned.
- Hermes bubble: blue-soft or white with blue edge/frame, left aligned.
- Avoid rounded bubbly SaaS/chat UI. Use modest radius or square editorial speech slabs.
- Keep text readable, 15-16px body, max width around 62ch.

---

### Task 6: Add the chat composer component

**Objective:** Create the bottom input where the user talks to Hermes.

**Files:**
- Create: `src/agents/ChatComposer.tsx`
- Create: `src/agents/ChatComposer.test.ts`
- Modify: styles

**Behavior:**
- Multiline textarea.
- Placeholder: `Message Hermes...`
- Enter sends, Shift+Enter inserts a newline.
- Send button disabled when empty or when currently submitting if no queue behavior is implemented.
- Button label: `SEND MESSAGE` or `SEND`.
- Attach/drop-image visual affordance can be present but disabled/clearly not active in v1.

**Acceptance tests:**
- Typing updates value.
- Empty send does nothing.
- Enter invokes `onSend` with trimmed message.
- Shift+Enter keeps newline.

---

### Task 7: Add the Agents/Hermes screen layout

**Objective:** Replace blank canvas with the first Agents module screen.

**Files:**
- Create: `src/agents/AgentsHermesScreen.tsx`
- Modify: `src/App.tsx`
- Modify: `src/App.css` or create/import `src/agents/agents.css`
- Test: `src/agents/AgentsHermesScreen.test.ts`

**Layout:**
- Keep current blue rail and editorial sidebar.
- Mark `Agents` nav item as active instead of `Today`.
- Main stage should include:
  1. top status bar: `Operator / local` left, `Hermes ONLINE/OFFLINE` right;
  2. chat stage with blue-washed architectural panel background, but not so dark that text fails contrast;
  3. message stream area;
  4. pinned composer along bottom;
  5. lower stats strip with `Messages`, `Model`, `Last active`, `Session` placeholders, clearly local/empty if no data yet.

**Initial demo state:**
- Show a small Hermes greeting bubble, e.g. `Hermes is ready when the local API server is online.`
- Do not fake a successful live response.
- If no gateway is connected, show offline status and disable send with instructions.

---

### Task 8: Add Hermes backend config and health check command

**Objective:** Let the frontend know whether Hermes API Server is reachable without exposing secrets.

**Files:**
- Modify: `src-tauri/Cargo.toml` if HTTP client dependencies are missing.
- Modify: `src-tauri/src/lib.rs`
- Create: `src/agents/hermesClient.ts`
- Test: Rust unit tests if practical.

**Backend command:**
- `check_hermes_health() -> Result<HermesHealth, String>`

**Backend behavior:**
- Read API base URL from env var, default `http://127.0.0.1:8642`.
- Read API key from env var such as `ZOID_HERMES_API_KEY` or `API_SERVER_KEY`.
- Call `/health` without auth first.
- If API key exists, call `/v1/models` with `Authorization: Bearer ...` to validate auth.
- Return statuses:
  - `online`
  - `offline`
  - `unauthorized`
  - `error`

**Security:**
- Never render or log the API key.
- Redact endpoint errors if they include headers.

**Frontend behavior:**
- `getHermesHealth()` invokes the Tauri command.
- UI shows precise offline setup text if needed:
  - `Hermes API server is offline. Enable API_SERVER_ENABLED and restart hermes gateway.`

---

### Task 9: Add send-message backend command, non-streaming first

**Objective:** Get one full request/response working before streaming.

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/agents/hermesClient.ts`
- Modify: `src/agents/AgentsHermesScreen.tsx`

**Backend command:**
- `send_hermes_message(messages: Vec<HermesChatMessage>) -> Result<HermesChatResponse, String>`

**API request:**
- `POST {base}/v1/chat/completions`
- Headers:
  - `Authorization: Bearer <key>`
  - `Content-Type: application/json`
- Body:

```json
{
  "model": "hermes-agent",
  "messages": [
    { "role": "user", "content": "hello" }
  ],
  "stream": false
}
```

**Acceptance:**
- If Hermes API server is configured, user sends a message and receives a real Hermes reply in the chat window.
- If it is not configured, message moves to error/offline state with a clear setup note.

---

### Task 10: Upgrade response handling to streaming SSE

**Objective:** Make Hermes replies feel live and agent-like.

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/agents/hermesClient.ts`
- Modify: `src/agents/AgentsHermesScreen.tsx`

**Recommended approach:**
- Use Tauri events for streaming chunks:
  - frontend starts command `stream_hermes_message(requestId, messages)`;
  - backend sends events:
    - `hermes://chunk`
    - `hermes://done`
    - `hermes://error`
  - frontend appends chunks into the active assistant message.

**Acceptance:**
- During streaming, Hermes avatar/message shows `streaming` status.
- Text appears incrementally.
- Final message status becomes `sent`.
- Errors end the stream and preserve partial text with an error note.

---

### Task 11: Add local session state for v1

**Objective:** Preserve messages during one app runtime and prepare for later persistence.

**Files:**
- Create: `src/agents/useHermesChat.ts`
- Test: `src/agents/useHermesChat.test.ts` or pure reducer tests in `src/agents/hermesChatReducer.test.ts`

**Approach:**
- Use a reducer for message list and connection state:
  - `checkHealthStart`
  - `checkHealthSuccess`
  - `sendUserMessage`
  - `startAssistantMessage`
  - `appendAssistantChunk`
  - `finishAssistantMessage`
  - `failMessage`
- Keep persistence out of v1 unless the user asks.

**Acceptance:** Reducer tests cover sending, streaming, completion, and failure.

---

### Task 12: Add avatar/profile-picture configuration boundary

**Objective:** Make it easy to replace initials with real photos later.

**Files:**
- Modify: `src/agents/participants.ts`
- Possibly create: `src/agents/profileAssets.ts`

**Plan:**
- Keep default participants static.
- Add optional `avatarUrl` fields.
- For local image assets later, place app-owned static files under `src/assets/avatars/`.
- Do not request or use private user photos automatically.

**Acceptance:** UI renders correctly with and without image URLs.

---

### Task 13: Update tests and test runner

**Objective:** Ensure all frontend smoke/unit tests run consistently.

**Files:**
- Modify: `package.json`
- Modify: `src/scaffold.test.ts` or replace with `src/app.test.ts`

**Approach:**
- Keep tests lightweight because the current app has no Vitest dependency.
- Option A: keep `tsx` tests that assert static source contracts.
- Option B: add Vitest + React Testing Library if real interaction tests are needed.

**Recommendation:** For this first slice, add Vitest and React Testing Library only if the implementation needs DOM interaction tests for composer Enter/Shift+Enter. Otherwise use pure reducer tests and source smoke tests to avoid dependency creep.

**Required assertions:**
- Agents nav active state exists.
- `Hermes` participant exists.
- `Ziad` participant exists.
- `aria-label="Primary navigation"` still exists.
- `aria-label="Hermes chat"` or equivalent exists.
- Composer has accessible label.

---

### Task 14: Visual polish pass against the design system

**Objective:** Match the screenshot direction while staying Kujoyama/Zoid 25.

**Files:**
- Modify: `src/App.css` and/or `src/agents/agents.css`
- Read: `DESIGN.md`

**Checklist:**
- Committed blue remains architectural, not decorative.
- Message bubbles have clear sender identity and avatar.
- User and Hermes bubbles are visually distinct.
- Composer is pinned to bottom and keyboard-friendly.
- Focus states are visible.
- Offline/unauthorized states are truthful.
- No fake successful messages.
- No old dashboard/card grid UI.
- No tiny unreadable text in main chat.
- No color-only status.

---

### Task 15: Verification and critique gate

**Objective:** Prove the chat window works and passes the required review gate.

**Commands:**

```bash
npm run test
npm run build
npm run tauri:build
```

**Manual/API verification:**
1. If Hermes API server is configured:
   - start/confirm gateway;
   - run health check;
   - send `hey, what day is it?` from the Zoid 25 chat UI;
   - verify a real Hermes response appears.
2. If API server is not configured:
   - verify UI shows offline setup state;
   - verify send is disabled or fails truthfully;
   - do not claim real chat works.
3. Launch packaged app:
   - open `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`;
   - confirm app process starts;
   - close/kill test app process.

**Critique gate:**
- Create handoff:
  - `.hermes/reviews/agents-hermes-chat-window/handoff.md`
- Run separate critique agent.
- Fix all Required fixes.
- Re-review until verdict is `APPROVED`.

---

## Files likely to change

Frontend:
- `src/App.tsx`
- `src/App.css`
- `src/scaffold.test.ts`
- `src/agents/types.ts`
- `src/agents/participants.ts`
- `src/agents/Avatar.tsx`
- `src/agents/MessageBubble.tsx`
- `src/agents/ChatComposer.tsx`
- `src/agents/AgentsHermesScreen.tsx`
- `src/agents/hermesClient.ts`
- `src/agents/useHermesChat.ts` or `src/agents/hermesChatReducer.ts`
- `src/agents/*.test.ts`

Backend/native:
- `src-tauri/src/lib.rs`
- `src-tauri/Cargo.toml`
- possibly `src-tauri/src/hermes_api.rs` if the backend code becomes too large for `lib.rs`

Docs/review:
- `.hermes/reviews/agents-hermes-chat-window/handoff.md`
- `.hermes/reviews/agents-hermes-chat-window/critique-report.md`

Maybe changed, only if needed:
- `package.json`
- `package-lock.json`

## Tests / validation matrix

### Static/unit
- TypeScript compile via `npm run build`.
- Frontend smoke tests via `npm run test:frontend`.
- Rust compile/tests via `npm run test:rust`.
- Reducer tests for send/stream/fail message states.
- Composer tests for Enter and Shift+Enter behavior, if DOM test tooling is added.

### Hermes integration
- `GET /health` offline/online classification.
- `GET /v1/models` unauthorized/key-valid classification.
- `POST /v1/chat/completions` non-streaming response.
- SSE streaming chunk handling.

### UX/accessibility
- Keyboard focus on nav, composer, send button.
- Screen-reader labels for chat region, message list, composer, avatars.
- Status labels appear next to dots.
- Color contrast for bubbles, composer, and blue stage.
- Reduced motion respected.

### Production/macOS
- `npm run tauri:build`.
- Launch packaged `.app`.
- If Hermes API server is online, send one real message from packaged app.
- If offline, packaged app shows truthful setup state.

## Risks and tradeoffs

- **Hermes API server may not be enabled yet:** Plan includes offline setup/status rather than blocking UI work.
- **Secret handling:** API key must stay in Tauri/Rust or environment. Do not put it in React state, localStorage, or checked-in files.
- **Streaming complexity:** Build non-streaming first, then SSE. This avoids debugging UI, auth, and streaming at the same time.
- **Tool-call visibility:** Hermes may use tools during a response. v1 can show final text only; a later step can add tool-call timeline/events if the API exposes them cleanly.
- **Session continuity:** OpenAI-compatible request mode may not automatically behave like the existing terminal session. v1 should treat the chat as a new Hermes API-server conversation unless docs/API provide a supported session-id mechanism.
- **Avatars:** Use initials/defaults first. Do not assume or scrape user photo. Add explicit local profile image support later.
- **Design risk:** WhatsApp-like chat bubbles can drift into generic rounded chat UI. Keep hard rules, blue rail, editorial spacing, and restrained bubble geometry.

## Open questions before implementation

1. Should the first implementation require a real configured Hermes API Server, or is it acceptable to first ship a truthful offline UI and wire the API after you provide/confirm the local API key setup?
2. Do you want the user avatar to be initials-only for now, or should we add a local image file path you provide?
3. Should conversations persist after app restart in v1, or is runtime-only chat acceptable for the first slice?

## Definition of done

The feature is complete only when:

- The Agents nav opens a Hermes chat window.
- User and Hermes avatars/profile pictures render beside messages.
- The composer can send a message.
- If Hermes API Server is configured, a real Hermes reply appears in the UI.
- If Hermes API Server is not configured, the UI clearly says so and does not fake success.
- Tests/build/package verification pass.
- Packaged macOS app smoke test passes.
- `.hermes/reviews/agents-hermes-chat-window/critique-report.md` says `APPROVED` after required fixes are resolved.
