# User Flow Map: Zoid Code Workspace

Date: 2026-06-06
Module: Code Workspace
Source grill: `/Users/ziadnasreldin/brainstorms/2026-06-06-zoid-code-workspace-user-flow-map.md`
Source PRD: `/Users/ziadnasreldin/Zoid/Docs/modules/code-workspace/prd.md`
Stitch project: https://stitch.withgoogle.com/projects/1952838208663206055

## 1. Flow principles

- Code Workspace is the repo/status/shipping hub.
- Normal repo click is lightweight: select repo, update inspector, stay in command center.
- Full repo detail opens through double-click, Open Details, or a focused resolution route.
- What Needs Me behaves like an inbox: every item opens the exact flow needed to resolve it.
- Launch Gate is the central shipping hub.
- Every missing Launch Gate item routes to its own resolution flow.
- Evidence flows always return to Launch Gate when launched from shipping.
- Start Agent keeps the user in Code Workspace by default, with optional Open in Agents Workspace / Split Panel.
- Every modal/subflow preserves origin context for cancel/back.
- No flow marks launch/deploy verified without required evidence.

## 2. Primary screen nodes

- Code Workspace / Repo Health Command Center
- Empty Code Workspace
- Repo Discovery / Scan Folders
- Managed Repositories List
- Right Inspector
- Repository Detail
- Run Checks
- Start Agent From Repo Modal
- Agents Workspace linked session
- Launch Gate
- Evidence Attachment / Verification
- Commit / PR Workflow
- GitHub / PR Integration
- Deployment Tracking / Actions
- Repo Settings / Fix Path
- Search / History / Archive
- Repo Handoff Export
- Native Verification / Diagnostics
- macOS Finder / Editor / Terminal external exits

## 3. First-time entry flow

```mermaid
flowchart TD
  A[Open Code Workspace] --> B{Any managed repos?}
  B -- No --> C[Empty repo command center]
  C --> D[Add Scan Folder CTA]
  D --> E[macOS folder permission picker]
  E --> F[Repo Discovery scanning state]
  F --> G[Discovery results grouped by recommendation]
  G --> H[Approve selected repos]
  H --> I[Repos Added confirmation]
  I --> J[Repo Health Command Center]
  J --> K[Newly added repos highlighted]
  K --> L[Right Inspector opens Summary for first added repo]
```

## 4. Returning entry flow

```mermaid
flowchart TD
  A[Open Code Workspace] --> B{Managed repos exist?}
  B -- Yes --> C[Repo Health Command Center]
  C --> D{Last active repo available?}
  D -- Yes --> E[Select last active repo]
  D -- No --> F[Select first Needs Me repo]
  E --> G[Right Inspector Summary]
  F --> G
```

## 5. Repo selection and navigation flow

```mermaid
flowchart TD
  A[Repo row in command center] --> B{User action}
  B -- Single click --> C[Select repo]
  C --> D[Update right inspector]
  D --> E[Stay on command center]
  B -- Double click --> F[Repository Detail]
  B -- Open Details --> F
  B -- Run Checks --> G[Run Checks flow]
  B -- Start Agent --> H[Start Agent modal]
  B -- Launch Gate --> I[Launch Gate]
  B -- View Diff --> J[Repository Detail > Diff tab]
  B -- Open Repo --> K[Open in Finder / editor / terminal]
```

## 6. Repo discovery approval flow

```mermaid
flowchart TD
  A[Scan Folders] --> B[Choose folder]
  B --> C[Grant macOS permission]
  C --> D[Scanning state]
  D --> E[Results grouped: Recommended / Needs Review / Duplicates / Ignored]
  E --> F{User action}
  F -- Approve selected --> G[Optional group/profile override]
  G --> H[Repos Added confirmation]
  H --> I[Repo Health Command Center]
  I --> J[Highlight newly added repos]
  J --> K[Inspector Summary for first added repo]
  F -- Ignore selected --> L[Move to Ignored tab]
  F -- Open in Finder --> M[Reveal local folder]
  F -- Rescan --> D
```

## 7. What Needs Me / attention inbox flow

```mermaid
flowchart TD
  A[What Needs Me item] --> B{Attention type}
  B -- Failed checks --> C[Repo Detail > Checks tab, failed command expanded]
  B -- Launch gate blocked --> D[Launch Gate, blocked item focused]
  B -- Deployment unverified --> E[Launch Gate or Deployments tab, Verify Production highlighted]
  B -- Agent waiting --> F[Linked Agents session or inline linked-run panel]
  B -- Dirty changes --> G[Repo Detail > Git Status / Diff]
  B -- Secrets/config changed --> H[Repo Detail > Diff + Danger Zone]
  B -- PR failing/awaiting review --> I[GitHub / PR Integration]
  B -- Repo path missing --> J[Repo Settings / Fix Path]
  C --> K[Resolve or downgrade item]
  D --> K
  E --> K
  F --> K
  G --> K
  H --> K
  I --> K
  J --> K
  K --> L[Return to previous context or attention inbox]
```

## 8. Run Checks flow

```mermaid
flowchart TD
  A[Run Checks from row/detail/launch gate] --> B[Run Checks screen/panel]
  B --> C[Show detected commands + source]
  C --> D[Select checks or Run Required]
  D --> E{Long/risky commands?}
  E -- Yes --> F[Confirmation]
  E -- No --> G[Run checks]
  F --> G
  G --> H[Clean progress state; output collapsed by default]
  H --> I{Result}
  I -- All pass --> J[Success]
  J --> K[Save as Launch Evidence]
  J --> L[Continue Launch Gate]
  J --> M[Start Agent anyway]
  I -- Any fail --> N[Expand failed command]
  N --> O[Start Agent to Fix]
  N --> P[Copy Output]
  N --> Q[Re-run Failed]
  N --> R[Mark Blocked]
  K --> S{Origin}
  L --> T[Return to Launch Gate with evidence]
  S -- Repo origin --> U[Stay on Checks tab/panel]
  S -- Launch origin --> T
```

## 9. Start Agent from repo flow

```mermaid
flowchart TD
  A[Start Agent from row/detail/check failure/launch gate] --> B[New Agent Session modal over Code Workspace]
  B --> C[Repo/workdir/context/rules/diff pre-attached]
  C --> D[Select agent preset + prompt]
  D --> E[Permission preview]
  E --> F[Start]
  F --> G[Stay in Code Workspace]
  G --> H[Linked run visible in Inspector > Agents]
  H --> I{Agent status}
  I -- Running --> J[Compact live status]
  I -- Waiting --> K[Appears in What Needs Me]
  I -- Done --> L[Attach output as event/evidence/handoff]
  I -- Failed --> M[Show failed run, offer reopen/retry]
  H --> N[Open in Agents Workspace]
  H --> O[Open in Split Panel]
```

## 10. Launch Gate flow

```mermaid
flowchart TD
  A[Open Launch Gate from repo/task/attention] --> B{Active gate exists?}
  B -- No --> C[Create Launch Gate setup]
  C --> D[Choose target: task/branch/PR/deployment]
  D --> E[Choose checklist template]
  E --> F[Create gate]
  F --> G[Checklist view]
  B -- Yes --> G
  G --> H[Highlight blocked/missing items]
  H --> I{Checklist item clicked}
  I -- Git state --> J[View Diff / Commit flow]
  I -- Checks --> K[Run Checks flow]
  I -- Review --> L[Start Reviewer Agent]
  I -- Push/PR --> M[Commit / PR flow]
  I -- Deploy --> N[Deployment flow]
  I -- Verify Production --> O[Evidence / Verification flow]
  G --> P{Mark Verified clicked}
  P -- Evidence complete --> Q[Verified]
  P -- Missing evidence --> R[Focus missing item]
  G --> S[Mark Failed / Blocked / Rolled Back with reason]
```

## 11. Evidence / verification flow

```mermaid
flowchart TD
  A[Missing evidence item] --> B[Evidence Attachment / Verification]
  B --> C[Choose evidence type]
  C --> D{Source available?}
  D -- Yes --> E[Pre-fill output/link/file/metadata]
  D -- No --> F[Attach file/link/output or run verification]
  E --> G[Mark Required or Supporting]
  F --> G
  G --> H[Save Evidence]
  H --> I{Satisfies gate?}
  I -- Yes --> J[Return to Launch Gate, item completed]
  I -- No --> K[Show why insufficient]
  K --> L[Mark Verified remains locked]
  L --> M[Focus missing requirement]
```

## 12. Commit / PR flow

```mermaid
flowchart TD
  A[Commit/PR action from Repo Detail or Launch Gate] --> B[Commit / PR Workflow]
  B --> C[Show branch, target branch, dirty files, risky files, diff summary]
  C --> D[Select files to stage]
  D --> E[Review generated commit message]
  E --> F[Edit message]
  F --> G[Confirm commit]
  G --> H{Create PR draft?}
  H -- No --> I[Return to origin]
  H -- Yes --> J{GitHub connected?}
  J -- No --> K[GitHub auth flow]
  K --> J
  J -- Yes --> L[Pre-fill PR title/body]
  L --> M[Confirm push/PR draft]
  M --> N[Return to Launch Gate if shipping origin; otherwise Repo Detail > PR/Git]
```

## 13. Deployment flow

```mermaid
flowchart TD
  A[Deploy action or deployment blocker] --> B[Deployment Tracking / Actions]
  B --> C[Select environment/target]
  C --> D[Review provider/command/manual instructions]
  D --> E{Automated command?}
  E -- Yes --> F[Strong confirmation]
  F --> G[Run deploy command]
  E -- No --> H[Record manual/agent-run deployment]
  G --> I{Deploy result}
  H --> I
  I -- Success --> J[Record URL/provider status/commit/timestamp]
  J --> K[Route to Verify Production / Evidence]
  I -- Failed --> L[Mark Blocked / Start Agent to Debug / Copy Logs / Roll Back]
```

## 14. GitHub auth / remote flow

```mermaid
flowchart TD
  A[Remote/PR/CI action] --> B{GitHub connected?}
  B -- Yes --> C[Continue requested action]
  B -- No --> D[Graceful Connect GitHub CTA]
  D --> E[Global GitHub auth/settings]
  E --> F{Auth result}
  F -- Success --> G[Return to exact repo tab/action]
  F -- Cancel/fail --> H[Return to previous context with local functionality]
```

## 15. Repo Settings / Fix Path flow

```mermaid
flowchart TD
  A[Settings or missing path attention item] --> B[Repo Settings]
  B --> C{Reason}
  C -- Normal settings --> D[Edit profile/group/commands/permissions/deployment/notes/rules]
  D --> E[Save]
  E --> F[Return to previous repo context]
  C -- Missing path --> G[Choose new local folder]
  G --> H{Same repo validated?}
  H -- Yes --> I[Update path]
  H -- No --> J[Warn user / choose again / override with reason]
  I --> K[Refresh repo status]
  K --> F
  B --> L[Review imported rules suggestions]
  L --> M[Approve/save selected rules]
```

## 16. Search / History / Archive flow

```mermaid
flowchart TD
  A[Open Search / History / Archive] --> B[Search/filter]
  B --> C{Result type}
  C -- Repo --> D[Select repo in command center]
  C -- File/diff --> E[Repo Detail > Diff/Changed Files]
  C -- PR --> F[GitHub / PR screen]
  C -- Launch gate --> G[Launch Gate focused item]
  C -- Evidence --> H[Evidence detail]
  C -- Agent --> I[Linked agent session]
  C -- History event --> J[Event detail with origin action]
  A --> K[Ignored/Archived management]
  K --> L[Restore repo]
  L --> M[Command center with restored repo highlighted]
```

## 17. Handoff export flow

```mermaid
flowchart TD
  A[Handoff Export from repo/launch/agent] --> B[Repo Handoff Export]
  B --> C[Preview repo state]
  C --> D[Choose include/exclude options]
  D --> E{Destination}
  E -- Copy --> F[Copy to clipboard]
  E -- Repo/docs --> G[Save handoff file]
  E -- App data --> H[Save in app data]
  E -- Attach to agent --> I[Attach to linked/new agent]
  F --> J[Return to previous context]
  G --> J
  H --> J
  I --> J
  J --> K[Record handoff event in repo history]
```

## 18. Native verification / diagnostics flow

```mermaid
flowchart TD
  A[Diagnostics from settings/verification blocker/native screen] --> B[Native Verification / Diagnostics]
  B --> C[Check Tauri/native status]
  B --> D[Check scan root permissions]
  B --> E[Check git availability]
  B --> F[Check registry persistence]
  B --> G[Check disposable test repo]
  B --> H[Check GitHub auth]
  B --> I[Check deployment provider state]
  B --> J[Check evidence storage paths]
  C --> K{All pass?}
  D --> K
  E --> K
  F --> K
  G --> K
  H --> K
  I --> K
  J --> K
  K -- Yes --> L[Return to previous verification/launch context]
  K -- No --> M[Focused fix flow]
  M --> B
```

## 19. Cancel, back, and failure behavior

```mermaid
flowchart TD
  A[Any subflow] --> B{User cancels/back or action fails}
  B -- Modal cancel --> C[Return to previous screen unchanged]
  B -- Full-screen back --> D[Return to prior tab/context]
  B -- Attention-origin flow --> E[Return to attention inbox]
  B -- Command/deploy/check failure --> F[Save failed event to repo history]
  F --> G[Offer Mark Blocked / Retry / Start Agent / Copy Logs]
  B -- Partial useful state --> H[Save draft/event, but do not complete requirement]
```

## 20. Cross-workspace Agents handoff

```mermaid
flowchart TD
  A[Code Workspace repo context] --> B[Start Agent]
  B --> C[Linked agent session created]
  C --> D[Code shows compact run status]
  D --> E{User opens agent?}
  E -- Open in Agents Workspace --> F[Agents Workspace, same repo group/session]
  E -- Open Split Panel --> G[Code + Agent split]
  E -- Stay in Code --> H[Inspector Agents tab]
  F --> I[Agent produces output]
  G --> I
  H --> I
  I --> J{Output type}
  J -- Evidence --> K[Attach to Launch Gate]
  J -- Review verdict --> L[Update Launch Gate / repo history]
  J -- Handoff --> M[Save handoff event]
  J -- Needs input --> N[What Needs Me item]
  K --> O[Return to Code repo context]
  L --> O
  M --> O
  N --> O
```

## 21. End-to-end happy paths

### 21.1 First repo onboarding

```mermaid
flowchart LR
  A[Open Code] --> B[Add Scan Folder] --> C[Approve repos] --> D[Command Center] --> E[Select repo] --> F[Run Checks] --> G[Start Agent or Launch Gate]
```

### 21.2 Fix failed checks with agent

```mermaid
flowchart LR
  A[What Needs Me: failed checks] --> B[Checks tab failed output] --> C[Start Agent to Fix] --> D[Agent run visible in Code] --> E[Re-run Failed] --> F[Save passing output] --> G[Launch Gate evidence]
```

### 21.3 Ship with evidence

```mermaid
flowchart LR
  A[Launch Gate] --> B[Resolve Git state] --> C[Run Checks] --> D[Review Agent] --> E[Commit/PR] --> F[Deploy] --> G[Verify Production] --> H[Attach Evidence] --> I[Mark Verified]
```

### 21.4 Repair missing repo path

```mermaid
flowchart LR
  A[What Needs Me: path missing] --> B[Fix Path] --> C[Choose folder] --> D[Validate repo] --> E[Refresh status] --> F[Return to Command Center]
```

## 22. Flow implementation checklist

- Preserve origin context for every modal and full-screen flow.
- Store originating route/action for Run Checks, Evidence, Commit/PR, Deployment, GitHub Auth, Settings, Diagnostics, and Agent flows.
- Ensure every attention item has a deterministic target route and focus target.
- Ensure every Launch Gate checklist item has a deterministic resolution route.
- Ensure all successful subflows update repo history.
- Ensure failed commands/deployments/verifications can become Blocked/Failed evidence but never success evidence.
- Ensure external exits to Finder/editor/terminal return focus/state cleanly when user comes back to Zoid.
- Ensure restart restores selected repo, last active route, managed registry, launch gates, evidence, linked agents, and history.
