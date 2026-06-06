# Feature Handoff: Agents session metrics and repository link

## Original request

"I want you to implement a couple of additions in agents page
In the bottom of the page there are 4 sections (messages, bridge, operator, session)
I want you to change all of them except the session name 
Add the context used metric , number of chat compressions, repository that this session is assigned to( add an option to link a repository to a chat so the chat knows where are we gonna work)
Codex usage 5h and weekly data, time elapsed after prompt was given to the model, ai model used(gpt5.5 for example),"

## Implementation summary

- Replaced the old Agents bottom metrics (`Messages`, `Bridge`, `Operator`, `Session`) with four bottom sections:
  - `Context used` plus `Compressions`
  - linked `Repository`
  - `Codex usage` plus `Model`
  - preserved `Session` plus prompt `Elapsed`
- Added a repository linking control in the Agents topbar so a chat/session can be assigned to a repository path.
- Wired the linked repository into the real Hermes CLI send path:
  - frontend passes `linkedRepository` into `sendHermesCliMessage`;
  - Tauri command accepts `linked_repository` / `linkedRepository`;
  - backend validates the path exists and is a directory;
  - Hermes CLI command runs with `current_dir` set to the linked repository.
- Added prompt elapsed tracking: elapsed is `idle` before a prompt, live-updates while Hermes is responding, and stores the final elapsed duration after completion/failure.
- Added lightweight context-used estimation based on current message content against a 200k-token context budget.
- Kept Hermes integration as CLI/terminal bridge; no API server strings or endpoints were introduced.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`: Added session metric helpers/state, repository link state/control, prompt elapsed tracking, and new four-section stats footer. Passes linked repository into Hermes send path.
- `src/agents/hermesClient.ts`: Adds optional `linkedRepository` argument to the Tauri invoke payload.
- `src-tauri/src/lib.rs`: Adds linked repository validation and applies it as Hermes CLI command working directory; adds Rust coverage for validation and fake-Hermes workdir behavior.
- `src/App.css`: Added repository link control/topbar styling and preserved a four-column stats strip.
- `src/scaffold.test.ts`: Added contract checks for new Agents metrics, old metric replacement, and linked repository pass-through/backend application.

## How to test

- `npm run test`
- `npm run build`
- `npm run tauri:build`
- Run dev server with `npm run dev`, open `http://127.0.0.1:1420/`, click Agents, link a repository, and verify the topbar/footbar update.
- In native/Tauri, send after linking a repository; backend should run Hermes CLI from that linked repo directory.

## Tests run

- `npm run test:frontend`: PASS — scaffold tests and code workspace tests passed.
- `npm run test`: PASS — frontend tests plus Rust tests passed; Rust result `4 passed`.
- `npm run build`: PASS — TypeScript and Vite production build succeeded.
- `npm run tauri:build`: PASS — release app bundle built at `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Packaged app launch: PASS — launched bundle and verified running process `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app/Contents/MacOS/zoid`.
- Browser smoke on `http://127.0.0.1:1420/`: PASS — clicked Agents, verified no console errors, linked `/Users/ziadnasreldin/Zoid`, and footer DOM updated to include `Repository: /Users/ziadnasreldin/Zoid`.
- Rust fake-Hermes workdir test: PASS — `hermes_cli_message_runs_inside_linked_repository` creates a fake Hermes CLI and proves the command executes with `$PWD` equal to the linked repository.

## Git info

- Branch: `main`
- Commit SHA: not committed in this handoff.
- Note: working tree already contains broader dirty/untracked Code workspace changes (`package.json`, `src/App.tsx`, `src/codeWorkspaceFlow*`, `.hermes/reviews/code-workspace-e2e-flow/`) that are outside this Agents metrics slice. This review should focus on the Agents metric/repository files listed above plus any direct interaction with the existing app shell.

## Frontend/backend/database notes

- Frontend routes/components: Agents screen only, rendered from `AgentsHermesScreen`.
- Backend endpoints/services: Tauri command `send_hermes_cli_message` now receives optional linked repository and applies it as CLI working directory after validation.
- Database tables/migrations: none.

## Reviewer focus areas

- Confirm footer still has exactly four sections while containing all requested data.
- Confirm repository link control is user-reachable/clickable and updates the chat session repository metric.
- Confirm linked repository is not UI-only: it is passed to Tauri and used as Hermes CLI `current_dir` after safe directory validation.
- Confirm old bottom metrics (`Messages`, `Bridge`, `Operator`) are replaced and `Session` is preserved.
- Confirm elapsed timer and model/Codex usage copy are reasonable UI placeholders without claiming unavailable real telemetry.
- Confirm no Hermes API server assumptions were reintroduced.

## Fix cycle notes

- First critique returned `REQUEST_CHANGES` on R1: repository linking was UI-only.
- Fix: wired repository through frontend invoke payload to backend Tauri command and applied it as `Command::current_dir` after validating it exists and is a directory.
- Added frontend source-contract coverage and Rust fake-Hermes workdir coverage proving the linked repository affects command execution.
- Re-ran `npm run test`, `npm run build`, `npm run tauri:build`, browser smoke, and packaged app launch after the fix.
