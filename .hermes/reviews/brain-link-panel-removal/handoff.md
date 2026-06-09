# Feature Handoff: Brain link panel removal

## Original request

Page Feedback for `/` / Brain workspace: the element `body > div#root > main.zoid25-shell > section.brain-workspace-shell > section.brain-panel > div.brain-panel-heading` showed `Link Apple Notes folder` and `safe read/import`. User asked: "what is this element ??? is there something behind ? please remove it but tell me first what is it".

## What the element was

It was the heading of a Brain setup panel rendered by `BrainWorkspace.tsx` whenever the Brain store loaded. The full panel let the user list Apple Notes folders, choose a folder/sync mode, and link it as a Brain source. It was not an overlay and there was nothing hidden behind it; it was a normal visible `section.brain-panel` inserted before the no-source/source grid content.

## Implementation summary

- Removed the legacy `Link Apple Notes folder` setup panel from the Brain workspace render tree.
- Removed its local UI state, handlers, dropdown import, and list/link client imports from `BrainWorkspace.tsx`.
- Kept existing top-level Brain actions intact: `Create Zoid Brain folder` and `Sync now`.
- Updated the Brain behavior test to assert the removed panel copy (`Link Apple Notes folder`, `safe read/import`) no longer renders.
- Removed stale no-source empty-state copy that still referenced listing/linking another Apple Notes folder.

## Changed files

- `src/brain/BrainWorkspace.tsx`: removed the link/list Apple Notes panel and now-unused UI state/handlers/imports.
- `src/brain/BrainWorkspace.behavior.test.tsx`: removed interactions with the deleted panel and added absence assertions.

## Tests run

- `npm exec -- tsx src/brain/BrainWorkspace.behavior.test.tsx && npm run test:frontend && npm run build`: PASS.
- After addressing reviewer’s non-blocking stale-copy note, reran `npm exec -- tsx src/brain/BrainWorkspace.behavior.test.tsx && npm run test:frontend && npm run build`: PASS.
- Source search for `Link Apple Notes folder|safe read/import|list and link` under `src/brain`: only the regression-test absence assertions remain.

## Browser verification

- Navigated to `http://127.0.0.1:1420/`, opened Brain, and checked DOM:
  - `hasBrain: true`
  - `linkPanelExists: false`
  - `bodyTextIncludesLinkPanel: false`

## Reviewer focus areas

- Confirm the reported panel is fully removed and layout space is reclaimed.
- Confirm no accidental removal of Brain source/inbox/candidate/conflict panels when sources exist.
- Confirm no TypeScript/import dead code remains from deleted list/link UI.
- Confirm tests guard the removed visible copy.
