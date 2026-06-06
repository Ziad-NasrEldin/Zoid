# Agents Workspace Screens

## Agents Dashboard
Components:
- Header: Search Runs, New Agent Run, New Agent Profile, Session Console, Settings
- Summary cards: active, blocked, failed, pending reviews
- Active runs list
- Recent run history
- Agent profiles list
- Reviewer records
- Right inspector

## New Agent Run Modal
Components:
- Agent profile selector
- Mode selector
- Working directory selector
- Prompt field
- Context attachments
- Permission preview
- Review requirement preview
Actions: Save Draft, Start Run.

## Active Runs
Components:
- Search/filter toolbar
- Run list: title, profile, linked task, workspace, status, duration, attention state
- Run inspector
Actions: Open, Open Session, Stop, Send Input, Mark Blocked, Retry, View Logs.

## Agent Run Detail
Components:
- Detail header
- Tabs: Summary, Prompt, Clean Output, Raw Logs, Status Timeline, Linked Entities, Review, Events
- Right inspector
Actions: Open Session, Retry, Duplicate, Request Review, Create Follow-up Task, Export Logs.

## Agent Profiles
Components:
- Profiles list
- Profile detail/edit form: name, command, args, working directory, env references, modes, parser, permissions
- Recent runs
Actions: Save, Test Profile, Duplicate, Disable, Delete, Set as Reviewer.

## Reviewer Agent
Components:
- Pending review queue
- Completed review records
- Required fixes list
- Blocked list
- Review inspector
Actions: New Review, Run Review, Approve, Require Fixes, Block, Attach Evidence.
