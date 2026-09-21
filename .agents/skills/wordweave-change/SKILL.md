---
name: wordweave-change
description: "Plan, implement, and verify a bounded WordWeave5 Rust/egui behavior change. Use for application fixes or features in this repository; not for status questions, general explanations, or documentation-only edits."
---

# WordWeave scoped change

Follow the repository AGENTS.md. This skill adds application-specific routing, not another general development lifecycle.

## Establish a bounded change

State the intended observable result and how it will be checked, briefly. If already clear from the request, proceed without a new plan document or approval round. For a bug, distinguish expected/actual behavior and locate the owning code/test. Ask only for missing information that would change the fix or its acceptance.

Read the affected implementation and its nearest tests. Consult the relevant section of a design/upgrade document only when behavior is ambiguous. Do not load the complete version history. Preserve unrelated work.

## Choose verification by risk

| Change | Minimum useful evidence |
| --- | --- |
| Label/layout only | Review the changed UI and narrow-width behavior; actual native appearance must be marked unverified if not observed. |
| Study/chat behavior | Focused regression test for the changed state transition, then its relevant integration tests. |
| Save/delete/restore/async operations | Temporary-data tests for failure, retry/cancellation, and preservation; a focused independent check of the affected boundary. |
| Codex connection/selection | Mock protocol tests and returned-value display checks; no real paid/API-key fallback, no claim that mocks prove real authentication. |

Keep IME confirmation distinct from ordinary Enter. Preserve pending answers, context/evidence selections, original media, and review state across changes. User-approved material diffs must not become silent writes. A request to erase ambiguous data requires target/retention clarification before action.

Implement the smallest coherent fix. Use a regression test that fails for the actual defect when feasible; do not manufacture tests merely to satisfy a count. Application release checks and delegation rules are in AGENTS.md, not duplicated here.

## Finish

Report implemented scope, actual commands/results, and native checks not performed. Keep source-version evidence separate from historic logs. Do not add features, repeat passed reviews, or create report-only agents after the acceptance boundary is met.
