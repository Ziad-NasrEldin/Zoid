# Phase 5 — Content and OmniSocials Scope Plan

Date: 2026-06-05
Source tracker: `Docs/2026-06-01-zoid-implementation-tracker.md`
Source implementation plan: `Docs/2026-05-31-zoid-implementation-plan-v1.md`

## Goal

Complete Zoid Phase 5 as a local-first Content workspace and OmniSocials integration surface that supports a draft-first content workflow without silently scheduling or publishing anything externally.

Phase 5 proves this loop:

Content plan -> content draft -> media asset reference -> specialist review gate -> human-confirmed schedule intent -> fail-closed OmniSocials action evidence -> verification/failure history.

## Scope

Included:

- Content plans.
- Content pieces/drafts.
- Media asset references and basic platform constraints.
- Specialist design/review gates.
- Schedule intents, not real external schedule execution.
- Human confirmation requirement for scheduling/publishing actions.
- OmniSocials truthful account/status state.
- OmniSocials upload/schedule/publish command surfaces that fail closed when unconfigured and record verification/failure records.
- Content workspace UI for viewing/creating/updating plans, drafts, assets, gates, schedule intents, and verification/failure records.
- Tests and manual/local verification proving no publish/schedule happens by default.

Excluded from Phase 5:

- Real OmniSocials credential setup.
- Real external media upload.
- Real platform scheduling or publishing.
- Autonomous recurring content publishing.
- Social analytics.
- Calendar/Gmail/Business/Product workflow integration beyond existing entity-link compatibility.
- Full media processing, OCR, resizing, transcoding, or asset hosting.

## Lifecycle states

Content plans:

- `active`
- `archived`

Content pieces:

- `draft`: editable local content, not approved.
- `review_ready`: draft is ready for specialist review.
- `approved`: review gate passed, but no schedule/publish has happened.
- `scheduled`: local schedule intent exists; external scheduling is not implied.
- `blocked`: action was blocked by missing gate, missing confirmation, platform constraint, or integration state.
- `published`: reserved for a future real external provider success; Phase 5 must not create this state through fail-closed OmniSocials commands.
- `archived`

Review gates:

- `pending`
- `approved`
- `rejected`

Schedule records:

- `intent`: local schedule intent only.
- `cancelled`
- `blocked`
- `failed`
- `scheduled` / `published`: reserved for future real provider execution and must not be faked in Phase 5.

Verification records:

- `passed`: local validation or manual verification passed.
- `blocked`: action was intentionally blocked/fail-closed.
- `failed`: action attempted locally and failed safely.
- `manual`: human/manual evidence recorded.

## Action policy and confirmation boundaries

- Creating a local content plan or draft is low-risk and allowed with events.
- Adding a local media reference is allowed with validation and events.
- Creating/approving/rejecting a review gate is meaningful and must create records/events.
- Creating a schedule intent requires:
  - platform constraints pass;
  - required specialist review gate is approved;
  - an approved human confirmation decision exists;
  - no real external schedule is implied.
- Upload, schedule, and publish actions through OmniSocials require policy enforcement and must fail closed while OmniSocials is unconfigured.
- No external write can happen from Phase 5 without a separate reviewed credential/config/execution slice.

## OmniSocials integration truth states

Supported local states:

- `not_configured`: default; no credential reference exists and external actions fail closed.
- `needs_permission`: credential/config exists but permission is missing or unverified.
- `connected`: reserved for a verified future credential/config path.
- `error`: status check failed safely.
- `disabled_by_policy`: external social actions are blocked by policy.

Phase 5 default state is `not_configured`.

## Platform constraints

Initial constraints are intentionally simple:

- Instagram and TikTok schedule/publish surfaces require at least one image or video asset reference.
- LinkedIn can accept text-only drafts.
- Media asset references must be bounded metadata records, not raw media blobs.
- Unsafe/empty storage references, unsupported asset kinds, invalid metadata JSON, and secret-like metadata must be rejected.

Future dedicated media work can add full byte-size, aspect-ratio, duration, platform-specific variant, and upload preflight rules.

## Event and verification evidence

Meaningful Phase 5 actions should create durable evidence:

- plan created
- piece created/updated
- media asset referenced
- review gate created/approved/rejected
- schedule intent created/cancelled/blocked
- OmniSocials upload/schedule/publish blocked or failed

SQLite stores metadata, status, links, and verification/failure records. It must not store raw secrets or raw media blobs.

## Acceptance criteria

Phase 5 is acceptable only when:

1. A content plan can be created and listed.
2. A content piece/draft can be created, updated, and listed.
3. A media asset reference can be added and listed.
4. Platform constraints block unsupported schedule intents.
5. A pending or rejected specialist review gate blocks scheduling.
6. An approved specialist review gate plus approved human confirmation allows a local schedule intent.
7. OmniSocials default state is truthful `not_configured`.
8. OmniSocials upload/schedule/publish command surfaces fail closed and record blocked verification/failure records without external side effects.
9. Content workspace UI shows real native data or truthful empty/error/blocked states.
10. Tests cover workflow progression, fail-closed actions, review/confirmation requirements, platform constraints, and no fake connected/published state.
11. Manual/local verification documents a draft/review/schedule-intent path without publishing.
12. `npm run verify:local` passes.
13. `.hermes/reviews/phase-5-content-omnisocials/critique-report.md` says `Verdict: APPROVED`.
