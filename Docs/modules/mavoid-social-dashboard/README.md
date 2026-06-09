# MaVoid Social Dashboard Module

This module contains the formal implementation plan for building the MaVoid Buffer-backed social media management and automation dashboard inside Zoid 25.

Primary plan:
- `implementation-plan.md`

External workflow source of truth:
- `/Users/ziadnasreldin/MaVoid/social-automation-buffer`

Related workflow docs:
- `/Users/ziadnasreldin/MaVoid/social-automation-buffer/docs/buffer-social-automation-workflow.md`
- `/Users/ziadnasreldin/MaVoid/social-automation-buffer/docs/zoid-25-dashboard-requirements.md`
- `/Users/ziadnasreldin/MaVoid/social-automation-buffer/dashboard-spec/dashboard-data-model.md`

Implementation rule:
- Build read-only visibility first.
- Add safe automation controls second.
- Add scheduling/retry only after provider read-back and safety gates are implemented.
- Feature is not complete until feature-critique-workflow verdict is APPROVED.
