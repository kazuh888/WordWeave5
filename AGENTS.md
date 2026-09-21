# WordWeave5 development

## Scope and safety

- Respond in concise Japanese, である調. Distinguish evidence, inference, and unverified behavior.
- Preserve existing edits, teaching materials, study records, recordings, and ink. Inspect `git status --short` before editing; never reset unrelated changes.
- AI uses ChatGPT-authenticated Codex app-server only; no API-key fallback. Preserve direct Volta shim execution and system/user PATH discovery. Display returned model/effort, never guessed values.
- Within the application's teaching-material workflow, require a proposal, visible diff, and learner approval before registration. Append preserves existing content and review state; correction is separate. This is not a requirement to seek approval again for already requested code/doc edits. Never use real learning data for disposable tests.

## Smallest sufficient context

- Identify the requested outcome, scope, and acceptance evidence. Ask only when missing information materially changes implementation, safety, or acceptance. Do not require a questionnaire for an already clear request.
- At the start of a development task, define its completion boundary and consult `docs/process/improvement/kpi.md` for usage recording. Record available start/end counters and coverage; report missing measurements rather than estimates. This is task-time work, not background automation.
- Apply low-risk efficiency improvements during work and proactively report evidence-backed opportunities. Seek approval before changing harness/settings, model/effort, or verification standards; GitHub trial setup is a separate scoped task, not implied permission to publish.
- For “continue”, resume the single known unfinished task. If several plausible tasks remain, ask which one; do not restart historical work.
- For resumption/status, consult `tasks/current.md`; otherwise locate the relevant implementation with targeted searches. `CODEX_START.md` is an index, not a command to read every historic handoff/upgrade/log.
- Use applicable skills as required by the host; select narrowly and read selected instructions fully. Do not add overlapping generic workflows, review panels, or planning documents merely because this is software work. These instructions cannot disable host-required skills.
- Save lengthy diagnostic output to a task-specific log and report status, counts, and relevant failures. Do not repeatedly dump whole files/logs or relay full transcripts between agents.

## Delegation

- The parent model/effort is the user's choice; do not change it automatically.
- Delegate only an independent, bounded task with useful parallel work for the parent or a concrete independent-check benefit. Default to one helper, no recursive delegation or merge/report-only agents. More helpers need a task-specific justification.
- For bounded read-only lookup, use `ww-scout` (gpt-5.6-terra, medium). If that role is unavailable, an equivalent read-only task with explicit Terra/medium is allowed; report model unavailability rather than silently escalating. Complex implementation/risk decisions stay with the parent.
- Use fresh context (`fork_turns="none"` where supported): pass the question, relevant paths, constraints, and expected evidence only. A helper reads this shared file and only task-relevant skills. Do not blindly fork the conversation. Trust scoped evidence; re-open it only for integration or a specific unresolved concern.

## Verification and stopping

- For a behavior change, add/run a focused regression test where feasible, then relevant integration checks. Data persistence, cancellation, authentication, or broad UI changes warrant focused independent review.
- For Windows application delivery, run `cargo test --all-targets --locked` and `cargo build --release --locked --bin wordweave5`. Explicit user/CI checks take precedence. Docs/config-only work requires appropriate syntax/link/config checks, not an unrelated application rebuild.
- Repeat passed checks only after relevant changes, failures, or unresolved risks. Never substitute old logs for new verification. Native IME, microphone, pen, TTS, and real Codex acceptance remain separate from mock tests.
- Stop when scoped acceptance checks pass or a genuine blocker needs the user. Report changes, checks/results, and remaining uncertainty once. Update a short current-state entry when a task changes it; do not refresh every historical handoff.
