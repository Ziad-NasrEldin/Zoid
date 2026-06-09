# Critique Report: Brain kana subheading blue

## Verdict

APPROVED

## Scope reviewed

- `.hermes/reviews/brain-kana-subheading-blue/handoff.md`
- `src/App.css`
- `src/scaffold.test.ts`
- `src/brain/BrainWorkspace.tsx` only to confirm the affected DOM/classes

No app source was modified during this critique.

## Request

Fix the Brain workspace hero kana paragraph `記憶` at `.brain-hero p.kana-line` so it renders Kujo blue like other page kana subheadings, without turning normal Brain body/helper paragraphs blue.

## Findings

- `BrainWorkspace.tsx` renders the reported element as `<p className="kana-line">記憶</p>` inside `<header className="brain-hero"><div>...`, matching the reported selector.
- `src/App.css` keeps the generic muted Brain hero paragraph rule:
  - `.brain-hero p:not(.eyebrow) { ... color: var(--kujo-muted); ... }`
- The scoped fix adds/later uses:
  - `.brain-hero p.kana-line { margin: 0 0 2px; color: var(--kujo-blue); font-size: 28px; }`
- Specificity check: `.brain-hero p:not(.eyebrow)` and `.brain-hero p.kana-line` have equal effective specificity, and the `p.kana-line` rule appears later in the stylesheet, so `記憶` correctly wins with `var(--kujo-blue)` / `#3558a2` / `rgb(53, 88, 162)`.
- The later helper/body selector is `.brain-hero p:not(.kana-line)`, so it explicitly excludes the kana paragraph. It does not set `color`, but the normal hero body paragraph still remains muted via the earlier `.brain-hero p:not(.eyebrow)` rule.
- Other Brain helper/body paragraphs are still governed by muted selectors such as `.brain-bridge-error p`, `.brain-empty-state p`, `.brain-placeholder-strip p`, `.brain-panel-empty`, and muted row metadata selectors. The reviewed change does not broaden blue styling to those paragraphs.
- `src/scaffold.test.ts` extends the kana color guard to require `.brain-hero p.kana-line { margin: 0 0 2px; color: var(--kujo-blue);`, which protects the reported regression.
- The handoff accurately describes the scoped CSS specificity issue and the intended source guard.

## Verification run by reviewer

- `npm exec -- tsx src/scaffold.test.ts` — PASS

## Regression assessment

No scoped regression found. The Brain kana subheading now has enough cascade/specificity to override the muted Brain paragraph rule, while Brain body/helper paragraphs remain muted because the blue override is limited to `p.kana-line` and the body/helper rules either exclude `.kana-line` or target non-kana surfaces.

## Notes

- The repository has broad unrelated dirty/untracked work. This review intentionally assessed only the scoped Brain kana CSS/test change and handoff.
