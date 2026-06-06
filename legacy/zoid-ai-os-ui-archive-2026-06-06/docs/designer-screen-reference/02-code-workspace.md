# Code Workspace Screens

## Code Dashboard
Components:
- Header: Search Repos, Add Repo, Scan Folders, Refresh Status, Code Settings
- Summary cards: managed repos, dirty repos, open launch gates, failed checks
- Managed repositories list
- Repos needing attention
- Active code agent runs
- Recent code events
- Right inspector

## Repo Discovery
Components:
- Scan source panel
- Suggested repos list
- Bulk actions bar
- Repo preview inspector
Actions: Add Scan Folder, Run Scan, Approve Selected, Ignore Selected, Open in Files/Finder.

## Managed Repositories
Components:
- Search/filter toolbar
- Repo table: name, path, profile, branch, dirty status, checks, deployment, launch gate, activity
- Repo inspector
Actions: Open, View Status, View Diff, Run Checks, Start Agent, Launch Gate.

## Repository Detail
Components:
- Detail header: repo name, path, branch, status, primary actions
- Tabs: Overview, Git Status, Changed Files, Diff, Branches, Commits, PRs, Deployments, Launch Gate, Linked Items, History
- Right inspector

## Launch Gate
Components:
- Gate header: state, repo, product, task, commit, deployment
- Checklist: git state, typecheck, lint, tests, build, review, push, GitHub/Vercel, production verification
- Evidence panel
- Final verdict panel
- History timeline
Actions: Run Checks, Request Review, Deploy, Verify Production, Mark Verified/Failed, Roll Back.
