# Ruthless code-review subagent launcher handoff

## Feature
Wire a scoped ruthless reviewer subagent into Zoid Agents so the user can launch an adversarial, line-by-line implementation review of code written for the current task.

## User requirement
- New subagent whose full responsibility is to review written code line by line.
- It must be ruthless and verify whether code is actually functional.
- It must flag fake/no-op/useless code instead of assuming things.
- It must ask whether questionable code should exist and distill findings.
- Nothing useless should be granted access.

## Implementation summary
- Added `src/agents/ruthlessReviewerAgent.ts`:
  - Exports `RUTHLESS_REVIEWER_TOOLSETS = ["terminal", "file"] as const`.
  - Exports `buildRuthlessReviewerPrompt(...)` with strict instructions to spawn a single leaf `delegate_task` reviewer with exactly terminal/file access.
  - Prompt forbids browser, web, memory, cronjob, messaging, design, social, further delegation, edits, commits, pushes, deploys, cron, and external side effects.
  - Prompt requires git status/diff, untracked implementation files, fake wiring/no-op checks, proof, Required vs Optional findings, and APPROVED/BLOCKED verdict.
- Updated `src/agents/AgentsHermesScreen.tsx`:
  - Imports `buildRuthlessReviewerPrompt` and `ShieldAlert`.
  - Adds `handleStartRuthlessCodeReview()` that builds the guarded prompt with selected repository/session title and sends it through the current Hermes session.
  - Adds an Agents native command panel card with visible tool/access boundaries and a `Run ruthless review` button.
- Updated `src/App.css`:
  - Adds design-system-aligned boxed styling for `.ruthless-reviewer-card` using Kujo blue, black borders, mono/display typography, and squared controls.
- Added `src/agents/ruthlessReviewerAgent.behavior.test.ts`:
  - Verifies toolsets are exactly `["terminal", "file"]`.
  - Verifies required prompt guardrails and return format.
- Updated `src/scaffold.test.ts` and `package.json` so the source guard and behavior test run in frontend tests.

## Review focus
- Confirm this actually launches a reviewer prompt through Hermes rather than a fake/no-op button.
- Confirm the subagent access boundary is genuinely minimal and not broadened by the prompt.
- Confirm the UI does not imply the reviewer directly edits/fixes code.
- Confirm repository context is passed when linked and failure/blocking language is honest when no repo is linked.
- Confirm tests are meaningful enough for the prompt/tool access contract.

## Validation already run
- `npm run test:frontend` passed.
- `npm run build` passed.
- `npm test` passed: frontend plus Rust tests, 70 passed / 1 ignored.
- `npm run tauri:build` passed and produced `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Reinstalled to `/Applications/Zoid 25.app` using `ditto` and relaunched via bundle id `com.mavoid.zoid25`.
- `pgrep -fl '/Applications/Zoid 25.app/Contents/MacOS/zoid'` returned PID 77789.
- Screenshot captured `/tmp/zoid-agents-open.png` showing Zoid 25 running on Agents/Hermes workspace.

## Known caveat
The visible reviewer card is inside the `/agents` native command panel. The screenshot proof captured Agents/Hermes workspace running after install, not the panel card opened.
