# Impeccable re-review: Settings page

Scope: Zoid 25 Settings page.

Method:
- Loaded Impeccable docs: `/impeccable`, `/layout`, `/critique` from https://impeccable.style/docs/impeccable/ and related pages.
- Applied project `PRODUCT.md` and `DESIGN.md` as authoritative product/register context.
- Reviewed `src/App.tsx`, `src/App.css`, `src/providers/ProvidersSettings.tsx`.
- Ran `npx --yes impeccable detect --json src/App.tsx src/App.css src/providers/ProvidersSettings.tsx`.
- Visually inspected the live Settings page in browser at `http://127.0.0.1:1420/`.

## AI slop verdict

FAIL, but the foundation is recoverable.

The functional wiring is much better than the visual register. The Settings page now protects overflow and edge cases, but the Impeccable review says the page still reads as an art-directed “settings shrine” instead of a serious local product control room.

The biggest mismatch is against Zoid's own product/register files:
- PRODUCT.md: product register, local-first AI OS, dense but calm, truth before polish.
- DESIGN.md: Kujoyama Editorial OS, stark rules, one cobalt architectural blue, mono-forward hierarchy, rows over cards.
- Current Settings: sumi-e serif override, decorative ink mark, seal accents, radial washes, reveal animation, equal tab cards, overview card row before the form.

## Automated detector

Command:
`npx --yes impeccable detect --json src/App.tsx src/App.css src/providers/ProvidersSettings.tsx`

Settings-relevant output:
- `src/App.css:2408` side-tab/accent-border warning on `.settings-reference-line`.
- `src/App.css:2425-2427` active tab uses inset/black/seal accent language instead of the DESIGN.md cobalt architecture.
- `src/App.css:2460-2466` reveal animation is technically reduced-motion guarded, but conceptually decorative for a Settings surface.

Most other detector warnings were global or outside Settings.

## Nielsen heuristic scores

Scale: 1 bad, 10 strong.

1. Visibility of status: 7/10
   Save status, loaded profile, storage path, provider status, memory meter exist. They are truthful, but visually secondary behind the hero/theatre.

2. Match with real world: 6/10
   Hermes/local/provider/archive language maps well. “Soul”, seals, ink marks, and theatrical serif styling add ambiguity to high-risk config.

3. User control and freedom: 7/10
   Tabs, Save, archive restore/delete, and provider actions are present. But Save is global while the active tab changes, so scope/change state could be clearer.

4. Consistency and standards: 5/10
   The sumi-e layer violates DESIGN.md’s mono-forward, cobalt-led, product-control grammar.

5. Error prevention: 6/10
   Credentials/localStorage warnings exist. Destructive archive safety is improved. Still needs stronger hierarchy around risky provider/safety changes.

6. Recognition over recall: 5/10
   Seven tabs expose scope, but their details truncate. The user must scan/hover/scroll to understand sections.

7. Flexibility and efficiency: 5/10
   Dense controls are reachable, but the first screen delays the actual editable fields.

8. Aesthetic and minimalist design: 4/10
   Main failure. Decorative hero, ink mark, radial washes, seal accents, and repeated panels consume attention before the task.

9. Error recovery: 7/10
   Save/provider errors are explicit. Archive confirmation is solid. Inline changed-field feedback is still weak.

10. Help and documentation: 6/10
   Helper copy is abundant and truthful, but too much of it competes with the core controls.

## Impeccable /layout dimensions

Spacing: 5/10
- Many values sit near a scale, but it is not clean 4pt discipline: 7, 10, 14, 18, 22, 34, clamp gaps.
- Top-level sections are too close relative to how much the hero consumes.
- Breathing room is spent on decoration rather than task separation.

Visual hierarchy: 4/10
- The strongest object is the hero/title/ink composition, not the Settings task.
- The first editable controls are below the initial attention path.
- Active tab state uses black/seal instead of the system’s committed cobalt architecture.

Grid and structure: 6/10
- Technically structured: tablist, content panel, overview rail, responsive collapse.
- Product-structure problem: seven equal tab cards across the width create cramped/truncated navigation.
- Overview metrics behave like a dashboard card row before the user reaches the form.

Rhythm and variety: 4/10
- Too many visual languages: serif, kana, ink mark, seal square, radial washes, bordered cards, pale rules, top brush lines, dropdowns, meters.
- Rhythm is ornamental, not operational.

Density: 5/10
- Lower Identity form is mostly clean.
- Top stack is dense with low-task-value visuals.
- Settings should be dense where editing happens and calm/compact where orientation happens; current page reverses that.

## Cognitive load failures

- First-screen priority inversion: hero/profile card/sticky heading/tabs/overview precede actual editing.
- Seven equal tabs create choice overload and truncation.
- Overview cards compete with active form controls.
- Decorative sumi-e language adds semantic noise to configuration work.
- Too many bordered containers at similar weight: header, heading, nav cards, overview cards, section card, field controls.
- Truth/source signals exist but are not the dominant hierarchy.

## Priority fixes from exact Impeccable criteria

1. Remove the sumi-e theatrical Settings layer.
   Neutralize the ink mark, radial backgrounds, seal accents, serif override, and reveal choreography. Return Settings to the Kujoyama Editorial OS product grammar: mono/sans, black rules, cobalt selected states.

2. Collapse the hero into an operational header.
   Keep title, active profile, storage path/source, save status, and Save. Stop spending half the viewport on atmosphere.

3. Replace the seven equal tab cards.
   Use a left vertical ruled list or compact grouped rows. Avoid truncated descriptions. Group by task/risk: Identity, Memory, Models/Providers, Tools, Safety, Archive.

4. Move overview metrics out of the primary path.
   Put Memory/Soul/Access/Model summary into a compact right rail or below the active form, not before the fields.

5. Normalize spacing to the product scale.
   Use 8/12 for related controls, 16/24 within panels, 48+ only for major breaks. Remove odd gaps and decorative clamp theatrics unless they solve a responsive structure problem.

## Verdict

The previous pass fixed correctness and overflow edge cases. The exact Impeccable review does not approve the current visual layout direction.

Verdict: REQUIRED FIXES REMAIN.