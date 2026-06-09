# Feature Handoff: Brain sumi-e design-system test

## Fix cycle summary

Ruthless review fixes applied and re-verified for the Brain-only Gen-M / sumi-e pilot. No global rollout was made and no external assets/artwork/Lottie/Behance/Magnific/Awwwards assets were copied.

## What changed

- Kept the Gen-M/sumi-e treatment scoped to `.brain-sumi-e` on the Brain workspace only.
- Rebuilt the hero as a flat editorial paper field: no hard card border, no conventional shadow, hairline rule only.
- Moved hero actions back into normal CSS grid flow; no absolute-positioned action stack remains.
- Moved the ink mark into its own hero grid area, reduced opacity, and kept it away from operational copy.
- Replaced self-referential design copy with product-relevant operational provenance: `Local import · conflict-aware extraction · Hermes waits for your command`.
- Removed the visible `brain-design-strip` product-surface annotation entirely.
- Fixed reference-line overflow by allowing wrapping and `overflow-wrap:anywhere`.
- Removed ineffective responsive flex behavior from the experimental hero by using real scoped grid media rules.
- De-carded/paper-flattened panels: no conventional shadows, pale hairlines, flat paper fields.
- Stopped repeated brush borders on panels; retained only a sparse placed hero mark and a faint page rule gesture.
- Added Brain-scoped editorial serif font stacks using Typekit names when available plus robust local serif fallbacks.
- Made red seal accent rare and semantic: retained only conflict/write-failed badges; generic status/chrome no longer uses red.
- Set interactive panels to `overflow: visible` to avoid clipping dropdowns/popovers/focus outlines.
- Fully scoped the older Brain fallback CSS behind `.brain-workspace-shell:not(.brain-sumi-e)` and moved needed Brain pilot structure under `.brain-sumi-e`, resolving the selector-scope blocker from the ruthless re-review.
- Added reduced-motion-safe reveal choreography for hero copy, ink mark, actions, status, and functional content.
- Tightened the final hero/status separation after visual inspection: explicit grid rows/align-content, safer reveal offset, and status margin keep actions from visually crowding the bridge status.
- Updated behavior tests so they assert operational provenance and absence of design-system annotation copy instead of locking in decorative rationale.

## Changed files

- `src/brain/BrainWorkspace.tsx`
- `src/App.css`
- `src/brain/BrainWorkspace.behavior.test.tsx`
- `.hermes/reviews/brain-sumi-e-design-test/handoff.md`

## Commands run

```bash
PATH=/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin npm run test:frontend
# PASS

PATH=/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin npm run build
# PASS; Vite emitted the existing large chunk warning only.

PATH=/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin npm run dev -- --host 127.0.0.1
# Existing preview was already using port 1420; new server attempt failed with "Port 1420 is already in use".
```

Note: the non-login tool shell did not initially expose Node on `PATH`; using the explicit Homebrew/system `PATH` ran the required npm commands successfully.

## Browser / visual verification

Used the already-running preview at `http://127.0.0.1:1420/`, opened Brain, visually inspected the page, and measured DOM geometry in browser console.

Desktop viewport `1280×720`:

- `.brain-ink-mark` and `.brain-hero-copy`: no intersection (`inkOverlapsCopy: false`).
- `.brain-reference-line`: no horizontal overflow (`scrollWidth === clientWidth`, `overflows: false`, `white-space: normal`).
- `.brain-hero > .brain-actions`: computed `position: static`; no overlap with copy or reference line.
- Document: no horizontal overflow (`documentElement.scrollWidth === clientWidth`).
- `.brain-design-strip`: absent.
- Brain interactive panels/bridge panels measured with `overflow: visible`.
- Visual inspection: hero reads as a quiet editorial paper field with large serif title, restrained monochrome placed ink mark, no visible text/mark collision, and functional Apple Notes bridge state directly after the hero. Red is not used decoratively in this view.

Narrow stress check by forcing the Brain workspace to `430px` width in the browser console:

- Ink mark and copy still did not intersect.
- Reference line still did not horizontally overflow.
- Actions remained `position: static` and did not overlap copy/reference line.
- No horizontal document overflow was introduced during the stress check.

## Scope boundary / remaining risks

- Brain page only. No global/root design-system rollout.
- Existing global/sidebar blue chrome remains outside this Brain-only pilot; the Brain surface itself is monochrome/serif and scoped.
- Browser preview runs outside Tauri, so the Apple Notes bridge fails closed as expected; this verified truthful unavailable copy and disabled sync behavior in browser.
