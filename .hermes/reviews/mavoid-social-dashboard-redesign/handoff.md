# MaVoid Social Dashboard UI Redesign critique handoff

## Scope
The user rejected the first MaVoid Buffer dashboard visual treatment because the page was too yellow and did not match Zoid's sumi-e design system. This pass redesigns only the UI layer while preserving the previously approved backend/read-back/safety logic.

## User feedback to satisfy
- "the UI is horrible"
- "i dont like how you made everything in light yellow"
- "thats not even the design system of sumi-e"
- "please redesign the whole page again"

## Intended design direction
- Zoid sumi-e / ink command room, not generic yellow SaaS dashboard.
- White paper, black ink, gray wash, red-seal accents only.
- No warm yellow card wash, no yellow/orange rate-limit wash, no beige hero gradient.
- Make the 8:00 / 10:00 / 18:00 social automation rhythm first-class.
- Keep all Buffer posting claims truthful and fail-closed.

## Changed in this pass
- `src/social/SocialDashboard.tsx`
  - Added `social-ink-command` shell.
  - Rewrote hero copy to "Ink control room for the daily social rhythm".
  - Added first-class `social-rhythm-lane` with 08:00 creator/design agent, 10:00 daily intel to Buffer, and 18:00 evening post to Buffer.
  - Kept Refresh, Check Buffer API, Run 8:00 creator, and Pause/Resume creator handlers intact.
  - Preserved selected post, media URL, gate/retry reason, and Hermes cron panel.
- `src/App.css`
  - Replaced the previous yellow/beige social dashboard block with sumi-e social tokens mapped to existing global tokens:
    - `--social-ink: var(--sumi-ink)`
    - `--social-paper: var(--sumi-paper)`
    - `--social-soft-paper: var(--sumi-soft-paper)`
    - `--social-seal: var(--sumi-seal)`
  - Removed rejected yellowish values from the social CSS:
    - `rgba(255, 252, 242...`
    - `rgba(255, 241, 225...`
    - `rgba(236,228,209...`
  - Introduced ink-line hero, red-seal status accents, paper panels, rhythm lane, and restrained grayscale wash.
- `src/social/SocialDashboard.behavior.test.tsx`
  - Added regression checks for `social-ink-command`, `social-rhythm-lane`, 08:00/10:00/18:00 labels, "Ink control room" copy, and absence of rejected yellow washes in the social CSS.

## Verification run
- `npx tsx src/social/SocialDashboard.behavior.test.tsx` first failed on missing `social-ink-command` before implementation.
- After implementation:
  - `npx tsx src/social/socialViewModel.test.ts && npx tsx src/social/SocialDashboard.behavior.test.tsx` passed.
  - `npm run build` passed.
  - `cargo check` passed.
  - `npm run tauri:build` passed and rebuilt `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.

## Known limitations / unrelated context
- The broader repo remains dirty from other Zoid work.
- Browser navigation to Content in the running Vite page was inconclusive because the app stayed on the Code workspace during browser interaction; source/tests/build confirm the redesigned Content workspace bundle, but a target-page visual screenshot was not captured in this pass.
- Do not require fixing unrelated scaffold/Rust test-harness issues unless this redesign introduced them.

## Review request
Return APPROVED or CHANGES_REQUESTED. Required fixes should be limited to this redesign pass. Review for:
1. Whether the rejected yellow/beige social dashboard treatment is gone.
2. Whether the new page uses Zoid sumi-e tokens and visual language.
3. Whether the 8/10/18 automation rhythm is clearer.
4. Whether handlers and safety/truthfulness are preserved.
5. Whether any visual/code issue introduced by this pass blocks delivery.
