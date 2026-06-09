# Zoid Domain Context

## Glossary

### Brain Note

A Brain Note is Zoid's local mirror of a human-written note from a source such as Apple Notes. The synced human title/body can round-trip with Apple Notes, while extracted summaries, task candidates, clarifying sessions, agent briefs, embeddings, and work status remain Zoid-owned metadata unless the user explicitly writes them back.

### Task Candidate

A Task Candidate is a possible actionable item extracted from a Brain Note. It is not real work by default; it must be reviewed, clarified, merged, ignored, converted, or sent into an agent-prep session before Zoid treats it as executable work.

### Apple Notes Brain

Apple Notes Brain is a built-in Zoid Brain source. In v1, only a dedicated user-approved Apple Notes folder named `Zoid Brain` receives 2-way sync; other selected Apple Notes folders are read-only imports by default. Sync must preserve data, avoid automatic hard deletes, and never let agents directly modify Apple Notes without explicit user action.

### Automation

A Zoid automation is a live Hermes-managed cron job or watcher. The Automations workspace should not include general background processes, arbitrary local scripts, agent chats, repositories, or future non-Hermes routines unless they are explicitly represented as Hermes automations/watchers.

### Watcher

A watcher is an automation that monitors state or output over time and reports or reacts when its configured condition changes. Watchers are shown alongside cron jobs in the Automations workspace when Hermes can list their live status.

## Global UI Rules

- Dropdowns in Zoid 25 must use the shared `GlobalDropdown` component (`src/ui/GlobalDropdown.tsx`) and the `.zoid-dropdown*` design-system styles from `src/App.css`; do not add new native `<select>` controls or one-off dropdown styling.

## Launch Rules

- When relaunching the installed app, always target Zoid 25 explicitly: use `/Applications/Zoid 25.app` or bundle id `com.mavoid.zoid25`. Do not use generic `open -a Zoid`, because it may launch the older `/Applications/Zoid.app` bundle. The executable/process name can still appear as `zoid`, especially for dev/debug builds.
