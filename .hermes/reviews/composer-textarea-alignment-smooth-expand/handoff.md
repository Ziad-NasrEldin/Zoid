# Feature Handoff: Composer textarea alignment and smooth expansion

## Original request

in zoid 25
## Page Feedback: /
**Output Detail:** Detailed
**Viewport:** 1698×1009

### 1. <AgentsHermesScreen> <ChatComposer2> textarea
**Location:** .chat-composer > .composer-input-column > .composer-input-wrap > textarea
**Source:** src/vendor/agentation-fixed.mjs:7238:17
**React:** <AgentsHermesScreen> <ChatComposer2>
**Position:** 498px, 882px (1050×52px)
**Context:** [before: "Message Hermes"]
**Feedback:** by default this field is not aligned with the buttons on the right and left of it, i want it to allign please , only expand it whenever the user types in more than one line so it naturally ahs to expand
also make the expanding animation smooth and premium

## Implementation summary

- Made the Hermes composer textarea default to the shared `--composer-control-size` token (`44px`) so its top/bottom/height align with the `+` and `SEND/LOCKED` buttons.
- Changed the textarea to expand from the bottom only when content needs more than one line, capped at `132px`.
- Added a premium height transition (`220ms cubic-bezier(0.22, 1, 0.36, 1)`) and disabled manual resize so the field stays aligned unless content naturally expands it.
- Switched resize sync to `useLayoutEffect` with transition-safe measurement so shrinking back to one line returns to button height.
- Removed the older per-keystroke typing ring/glow so typing one line does not visually pulse or look “stuck typing”; only height changes animate.

## Changed files

- `src/agents/ChatComposer.tsx`: composer height constants and transition-safe auto-height sync.
- `src/App.css`: default textarea height matches buttons; smooth height transition; no manual resize/per-keystroke typing glow.
- `src/scaffold.test.ts`: updated guard strings for aligned default height, smooth multiline expansion, and absence of per-keystroke typing animation.

## How to test

- Open `http://127.0.0.1:1420/`, go to Agents.
- Verify `.composer-attach`, `.composer-send`, and `.chat-composer textarea` all render at 44px height with matching top/bottom when the composer is empty or has one line.
- Type two lines; the textarea should smoothly grow taller while the bottom stays aligned to the buttons.

## Tests run

- Browser DOM geometry on `http://127.0.0.1:1420/` Agents screen: PASS.
  - Empty/one-line textarea: attach/send/textarea all `top=458`, `height=44`, `bottom=502`.
  - One-line typed text: no animation (`animationName=none`), textarea remains `44px` and aligned.
  - Two-line textarea: `height=57`, `bottom=502`; attach/send remain `height=44`, `bottom=502`; textarea transition is `height 0.22s cubic-bezier(0.22, 1, 0.36, 1)`.
- `npm exec vite -- build`: PASS. Vite production frontend build completed.
- `npx tauri build --config '{"build":{"beforeBuildCommand":""}}'`: PASS after the Vite build, produced `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed app refreshed: PASS. Replaced `/Applications/Zoid 25.app`, relaunched it, and verified running process `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- `npm run build`: BLOCKED by unrelated pre-existing TypeScript errors in `src/ui/GlobalDropdown.behavior.test.tsx` (DOM lib type mismatches), not by the composer files.
- `npm run test:frontend`: previously failed due unrelated Code workspace assertion: `Code workspace must remove the useless repository status panel: code-workspace-feedback` in `src/scaffold.test.ts:140`.

## Git info

- Branch: current working tree in `/Users/ziadnasreldin/Zoid`.
- Commit SHA, if committed: not committed.
- Isolated diff command: `git diff -- src/agents/ChatComposer.tsx src/App.css src/scaffold.test.ts`

## Frontend/backend/database notes

- Frontend routes/components: Hermes Agents screen composer only.
- Backend endpoints/services: not touched.
- Database tables/migrations: not touched.

## Scope Boundary / Dirty Working Tree Handling

Intended fix files only:

- `src/agents/ChatComposer.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

The repository already has many unrelated modified/untracked files and review folders. They were not cleaned, reverted, or included in this approval claim. Review should judge the isolated composer diff only.

## Reviewer focus areas

- Confirm the textarea default one-line height is exactly aligned with adjacent composer controls.
- Confirm multiline content expands naturally and caps at the existing max height.
- Confirm deleting/shrinking back to one line returns to the 44px aligned state.
- Confirm no per-keystroke typing ring/glow remains.
- Confirm the CSS transition is smooth and scoped to textarea height.

## Fix cycle notes

Updated after removing per-keystroke typing animation and refreshing the installed app.
