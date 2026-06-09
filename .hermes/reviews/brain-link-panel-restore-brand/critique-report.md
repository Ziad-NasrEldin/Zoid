# Critique Report: Brain Apple Notes link-panel restore + layout fix

Verdict: APPROVED

## Scope reviewed

- Read handoff: `.hermes/reviews/brain-link-panel-restore-brand/handoff.md`
- Inspected current source/diff-relevant files:
  - `src/App.css`
  - `src/scaffold.test.ts`
  - `src/brain/BrainWorkspace.tsx`
  - `src/brain/BrainWorkspace.behavior.test.tsx`
- Ran focused checks:
  - `npm exec -- tsx src/scaffold.test.ts` — PASS, exit 0
  - `npm exec -- tsx src/brain/BrainWorkspace.behavior.test.tsx` — PASS, exit 0
  - Verified screenshot artifact exists: `/tmp/zoid-brain-link-panel-layout-fix.png` is a 1920x1080 PNG

## Evidence

### Layout fix addresses the screenshot complaint

`src/App.css` now gives the Brain shell explicit row ownership:

```css
.brain-workspace-shell {
  display: grid;
  grid-template-rows: auto auto auto minmax(0, 1fr);
  grid-auto-rows: auto;
  align-content: start;
  ...
}
```

This is repeated in the later refinement override as well:

```css
.brain-workspace-shell {
  grid-template-rows: auto auto auto minmax(0, 1fr);
  grid-auto-rows: auto;
  align-content: start;
  gap: 18px;
  ...
}
```

That means the hero, status/error/loading area, restored link panel/empty state row, and the main content grid are not forced into a shared constrained row. `grid-auto-rows: auto` and `align-content: start` also prevent implicit grid children from being stretched into overlapping/buried positions.

The restored panel also has its own stacking context:

```css
.brain-link-panel { position: relative; z-index: 1; ... }
```

This directly matches the intended fix and protects the panel from being visually buried behind the later Sources / Brain Inbox grid.

### DOM order is correct

`src/brain/BrainWorkspace.tsx` renders the relevant elements in this order:

1. `<header className="brain-hero">...`
2. `<p className="brain-status-line" ...>`
3. optional bridge/loading state
4. `{store ? <section className="brain-panel brain-link-panel" aria-label="Link Apple Notes folder">...`
5. optional no-source empty state
6. `{store && hasSources ? <div className="brain-ruled-grid">... Sources / Brain Inbox ...`

The `brain-link-panel` is therefore a sibling before `brain-ruled-grid`, not nested inside or behind Sources/Brain Inbox. With the CSS row fix above, it receives an auto-height row above the main Sources/Brain Inbox grid.

### Restored panel functionality and brand surface are present

`BrainWorkspace.tsx` includes:

- `Link Apple Notes folder`
- `safe read/import`
- safety copy: `Zoid only reads/imports during sync; no delete or writeback is performed here.`
- `List folders`
- `GlobalDropdown` for Apple Notes folder
- `GlobalDropdown` for sync mode
- sync-mode helper copy
- `Link selected folder`

The panel uses branded `brain-panel`, `brain-link-panel`, `brain-panel-heading`, `brain-actions`, `brain-primary-action`, and `brain-secondary-action` classes rather than an unstyled/native surface.

### Regression guard exists

`src/scaffold.test.ts` includes a direct source guard requiring both no-overlap layout invariants:

```ts
/\.brain-workspace-shell \{[^}]*grid-template-rows: auto auto auto minmax\(0, 1fr\);[^}]*grid-auto-rows: auto;[^}]*align-content: start;/
```

and:

```ts
/\.brain-link-panel \{[^}]*position: relative;[^}]*z-index: 1;/
```

The focused scaffold check passed.

### Behavior coverage exists

`src/brain/BrainWorkspace.behavior.test.tsx` asserts that:

- `.brain-link-panel` renders
- `Link Apple Notes folder` renders
- `safe read/import` renders
- safe import boundary copy renders
- `List folders` calls the folder listing flow
- sync mode dropdown renders and can select metadata tracking
- `Link selected folder` sends `{ accountName: "iCloud", folderName: "Projects", syncMode: "twoWay" }`

The focused behavior check passed.

## Notes / limitations

- I did not edit source files.
- I did not rerun the full build/Tauri build because the focused layout/source guards and behavior test are the most directly relevant checks for this critique. The handoff reports the broader build checks already passed.
- The screenshot artifact exists at `/tmp/zoid-brain-link-panel-layout-fix.png`; I verified it is a real PNG, but this review is primarily grounded in the source layout and focused tests.

## Conclusion

The follow-up layout fix is present and targeted: the Link Apple Notes / safe read-import panel now renders as a separate auto-height grid row before Sources / Brain Inbox, with stacking protection and regression coverage. The screenshot complaint that the panel was buried behind other elements is addressed by the current source.
