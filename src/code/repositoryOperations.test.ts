import assert from "node:assert/strict";
import {
  extractRunbookUpdate,
  inferRepositoryOperationOutcome,
  mergeRunbookUpdate,
  type RepositoryOperationProfile,
} from "./repositoryOperations";

const responseWithRunbook = `Server is running at http://localhost:1420.

RUNBOOK_UPDATE
Commands:
- npm install
- npm run dev -- --host 127.0.0.1
Checks:
- curl http://127.0.0.1:1420 returned 200
Next run:
- Reuse port 1420`;

assert.equal(
  extractRunbookUpdate(responseWithRunbook),
  "Commands:\n- npm install\n- npm run dev -- --host 127.0.0.1\nChecks:\n- curl http://127.0.0.1:1420 returned 200\nNext run:\n- Reuse port 1420",
  "RUNBOOK_UPDATE content should be extracted for the next runbook",
);

assert.equal(
  inferRepositoryOperationOutcome("Blocked: missing env DATABASE_URL", "success"),
  "blocked",
  "missing env should not be marked learned/success",
);

assert.equal(
  inferRepositoryOperationOutcome("Requires confirmation before production migration.", "success"),
  "needs-user",
  "approval-gated production responses should remain needs-user",
);

assert.equal(
  inferRepositoryOperationOutcome("Build failed with error TS2322", "success"),
  "failed",
  "normal CLI responses that report failure should not increase confidence",
);

assert.equal(
  inferRepositoryOperationOutcome("Build failed, but the old server is still running at http://localhost:1420", "success"),
  "failed",
  "mixed failure plus stale running URL should not be marked learned",
);

assert.equal(
  inferRepositoryOperationOutcome("Deployment failed, but old production is deployed and verified from yesterday", "success"),
  "failed",
  "mixed deployment failure plus old deployed text should not be marked learned",
);

assert.equal(
  inferRepositoryOperationOutcome("Verified branch. Requires approval before production deploy.", "success"),
  "needs-user",
  "approval-required text should override generic verified text",
);

assert.equal(
  inferRepositoryOperationOutcome("OPERATION_OUTCOME: success\nVerified fresh localhost URL.", "needs-user"),
  "success",
  "explicit final success marker should allow learned status",
);

assert.equal(
  inferRepositoryOperationOutcome("Hermes response without explicit result", "needs-user"),
  "needs-user",
  "unstructured normal responses should remain needs-review instead of learned",
);

const profile: RepositoryOperationProfile = {
  repoId: "repo-1",
  repositoryPath: "/tmp/repo",
  action: "localhost",
  status: "running",
  confidenceScore: 0,
  runbookMarkdown: "# Existing runbook",
  updatedAt: "2026-06-09T00:00:00.000Z",
};

const merged = mergeRunbookUpdate(profile, responseWithRunbook, "2026-06-09T10:00:00.000Z");
assert.ok(merged.includes("# Existing runbook"), "existing runbook should be preserved");
assert.ok(merged.includes("## Learned update — 2026-06-09T10:00:00.000Z"), "learned update should be timestamped");
assert.ok(merged.includes("npm run dev"), "learned commands should be persisted into the runbook");

console.log("repositoryOperations tests passed");
