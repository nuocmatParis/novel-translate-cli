# Agent instructions

## Project status

- `main` is the current Go implementation.
- `rewrite/rust` is the incremental Rust port.
- Keep the Go implementation working until the Rust port has feature parity.

## Planning and task execution

- For roadmap work, read the local `PLAN.md`, `docs/roadmap/REVIEW_WORKFLOW.md`, and (for Phase 02–09) `docs/roadmap/PARALLEL_EXECUTION.md` plus `docs/roadmap/PARALLEL_TASKS.json`, then the active phase/task, its dependencies, and the specs, design documents, and ADRs it references. Follow the assigned role. Model A implements and a different model B independently reviews in a separate session. Obtain the current internal plan before implementing or reviewing the roadmap if any required documents are missing. Use the plan as the source for roadmap requirements.
- Use the user's current requirements and the approved specifications as the basis for behavior. The existing implementation and prototypes are not the design baseline, a reuse requirement, or evidence that a planned task is complete.
- Distinguish approved requirements, accepted technical decisions, proposals, and unverified assumptions. Draft SQL, example code, and proposed API names are not automatically implementation requirements.
- Each phase chat executes one task at a time in dependency order, or a small group of 2–3 coupled tasks recorded by the coordinator in PLAN before implementation. A chat may own a whole phase; this does not authorize coding the phase as one batch. Use a separate worktree, feature branch, build output, test database/profile, and frozen internal document copy for each chat. One designated model A coordinates shared files, migration IDs and integration, and is the only writer of the central PLAN. External dependencies must have completed independent review; internal group dependencies may be implemented together, but no member is complete until the group meets its acceptance and review requirements.
- Continue within the user's assigned scope and role without asking for confirmation after every routine step. Model A stops dependent work at the independent review handoff. Resolve ordinary implementation choices with evidence in the relevant ADR. Ask when missing information or a product requirement change needs the user's decision; continue independent authorized work without altering the candidate under review.
- Inspect and preserve existing working changes before editing. This inspection protects ongoing work and does not turn the old implementation into the design authority.
- Keep changes within the current task. Introduce libraries, public APIs, abstractions, and adapters when they have a concrete consumer in the current phase.

## Independent review

- Model A may automatically call the user-selected `gpt-6.1-sol` reviewer with `max` reasoning in a separate fresh session (`fork_turns=none` when supported). Model A must be a different model from B. Record requested settings and actual runtime identity when available; if the required reviewer is unavailable, keep `Chờ review` and report the blocker. Do not pretend self-review is an independent review.
- Model A writes implementation and tests, performs self-review, and creates a versioned handoff using the internal task template. Set the task to `Chờ review`; self-review does not authorize completion.
- Identify the reviewed candidate and its baseline. Include tracked changes, relevant untracked files, dependency/configuration inputs, and the applicable internal specification/ADR/task versions. A commit alone does not identify a dirty working tree. Preserve existing user changes and do not force-add or commit private documents to create review evidence.
- Model B reads requirements and acceptance criteria before evaluating code, tests, and A's claims. Independently run relevant checks, inspect important failure paths and test assertions, and record what was actually verified. Missing mandatory checks remain unverified.
- Model B writes separate review reports with evidence-backed findings and one verdict: `Đạt`, `Cần sửa`, or `Chưa đủ bằng chứng`. Do not manufacture findings or block acceptance on optional style preferences; any unmet mandatory acceptance criterion still blocks completion.
- Model B must not edit the reviewed source, official tests, specifications, ADRs, PLAN, phase status, or A's reports during review. Additional probes may run in an isolated temporary copy; preserve reproducible evidence for A to turn into regression tests.
- Model A addresses each finding or disputes it with specification/test evidence. Model B verifies the response and closes or reopens findings. Keep finding IDs and prior reports; changes to the candidate or its governing requirements require a new review round for affected behavior.
- Keep the reviewed candidate stable and verify its identity before the verdict. Avoid concurrent writes to shared source or status files. The phase's model A updates its own evidence and sends B's report to the coordinator; only the coordinator updates the central PLAN.
- Apply independent review to Phase 00 documentation/ADR tasks as well as code tasks, and to each phase or partial milestone handoff. If no reviewer is available, leave `Chờ review`; do not impersonate model B or bypass the gate.

## Completion and handoff

- Use the task's acceptance criteria and the phase's test matrix and exit gate. Completion requires satisfied acceptance criteria, mandatory checks, synchronized documents, and an independent `Đạt` review valid for the current candidate. Phase completion also requires B's review of the final integrated candidate, including required manual checks.
- Record actual commands, environment/toolchain, relevant test names or counts, results, manual checks, and checks not run. A successful command with zero relevant tests is not evidence of acceptance.
- Keep unverified mandatory checks and unmet dependencies visible. Do not mark a task or phase complete merely because code builds, documentation exists, or a fake-provider test passes.
- At handoff and after B's verdict, the phase model A updates its evidence and sends the task/group, candidate, next role, status, report links, and next action to the coordinator for the central `PLAN.md`. Shared ADR changes are serialized by their assigned owner. Keep versioned handoff/review reports in the internal evidence location specified by the plan.
- When resuming, reconcile the plan and evidence with the current workspace before continuing. Do not repeat completed work or skip unfinished acceptance checks.

## Required checks

Run checks appropriate to each implementation task as it progresses. For documentation-only work, verify references, consistency, and ignore rules; do not present application checks as having run.

Run these commands from the repository root before opening a Rust PR:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

## Boundaries

- Keep tests offline and deterministic. Do not call a paid model provider from automatic CI.
- Keep API keys, `.env` files, EPUBs, workspaces, and generated output out of Git.
- Documentation under `docs/` may be committed and included in a draft PR when the user approves publishing those files. Stage the approved files by path and check the diff for private data before pushing.
- Keep `PLAN.md`, `.docs/`, `docs/roadmap/evidence/`, and other private planning files and execution evidence local and excluded from Git.
- Preserve the existing workspace format and migration compatibility unless a PR explicitly documents a schema change.
- Prefer moving one domain boundary at a time from Go to Rust; do not delete the Go path until its replacement is tested. At each checkpoint, assess the related Go code using PLAN section 8. Once replacement evidence and caller checks pass, register a small cleanup task and checkpoint with one shared Go owner; independently review the deletion candidate, then commit and push promptly. Keep needed Go paths and user data intact. Completing the Rust-only transition also requires the separate final cleanup and a clean build, test, run, and package without Go on the supported targets; M9 alone does not close that goal.
- Keep `Cargo.lock` committed because this repository builds an application binary.

## Git workflow

- For user-assigned code implementation, Model A may carry out the scoped branch, commit, push, and draft PR steps below. Model B retains the independent review role.
- Use `rewrite/rust` as the base for Rust work and name the implementation branch `feature/<work-area>`. Use names that describe the work, such as `feature/project` for project setup, `feature/tui` for the terminal interface, or `feature/epub-import` for EPUB input. Reuse the same branch while its PR is open; after merge use a new PR for the next checkpoint. For work already in progress in a shared checkout, move it onto a feature branch at the agreed integrated handoff.
- For a documentation-only PR requested by the user, create a `feature/<work-area>` branch from `rewrite/rust` and open a draft PR for the approved files. Check references, consistency, and ignore rules before publishing. Apply the independent review and merge requirements to any roadmap checkpoint included in the PR.
- Each Phase 02–09 checkpoint contains 1–3 tasks and must be committed and pushed as soon as its required checks and independent integrated `Đạt` review pass; do not wait for phase completion. A documents-only ignored checkpoint has no Git commit/push. Commit the approved task or small group at that checkpoint. Review the staged diff against the candidate B checked. Stage only the approved scope and its required files. Preserve other working and staged changes. If the staged content differs in a way that affects the review, obtain review of the affected work before proceeding.
- Push the feature branch to `origin` with a normal push after the current checkpoint satisfies its required checks and integrated review. Create or update the draft PR for approved checkpoints; do not wait for all remaining phase tasks. Cross-phase consumers use the exact published head and merge SHA of a reviewed checkpoint after merge and post-merge verification. Record any existing unmerged dependency PRs and reconcile them under the merge workflow. For a Rust PR, apply the checks above to the exact proposed PR contents. Create a draft PR from the feature branch into `rewrite/rust`; after it merges, open a new PR for the next checkpoint. A PR may contain only approved checkpoints, and its entire diff must be reviewed. Keep the PR small enough to review and describe the behavior change, migration impact, and checks actually run. The coordinator automatically merges each eligible checkpoint into rewrite/rust under docs/roadmap/MERGE_WORKFLOW.md; the user authorized this on 2026-11-14. Do not wait for phase completion or request approval for each eligible merge.
- Keep private documents, evidence, and user data local. Record the branch, commit, PR link, and check results in the internal plan. If a gate fails or a Git operation cannot complete, record the current state and the specific blocker.
- Treat `README.md` as user-approved content. Do not edit its text, sections, examples, commands, images, or banner references, or replace referenced banner assets, without the user's explicit approval for the proposed change.
- When a code, command, file layout, or supported platform change calls for a README update, present the exact proposed README text and any image/banner change to the user for approval before applying it. Continue independent authorized code work while awaiting that approval.
- Branch synchronization must copy the approved README and approved banner without rewriting or expanding their content. Existing explicit approval for the same README content, banner, and synchronization remains valid; do not ask again for those approved changes.

### Commit messages

- Write commit messages, PR titles and descriptions, and merge messages in plain English. Use familiar words and short sentences. Avoid specialist terms and unexplained short forms. Keep the exact names of commands and options when readers need them.
- Use `type(scope): summary`. Choose a scope that matches the work, such as `project`, `tui`, `cli`, or `epub`. Use `feat` for a feature, `fix` for a bug fix, `build` for build changes, `ci` for checks that run automatically, `test` for tests, and `docs` for documentation.
- Start the summary with a verb. For example, use `feat(tui): show help for the current screen` or `fix(tui): return to the same item after closing help`.
- Every commit needs a body. Leave a blank line after the title, then use one short bullet for each feature, fix, or other change in the commit. Say what a person can do or see now. For build or test changes, say what was added or fixed. One clear bullet is enough for a small change.
- List only work that is in the commit. Add the reason for the change or details about existing data when the reader needs them.

### PR and merge descriptions

- Use `type(scope): summary` for the PR title and cover the whole change in that PR. Start the description with a short sentence about what changes and why.
- Under **Changes**, list each feature, fix, and other meaningful change included in the final PR. Use one plain English bullet per change and say what a person can do or see. Describe the work across all commits in the PR.
- Under **Validation**, list the checks actually run and their results. Give the command and the system used when relevant. Mark any remaining checks as pending.
- Add **Compatibility** when commands, file paths, saved data, or supported systems change. Explain what a user needs to do.
- Keep the **Changes** list and any needed compatibility details in the merge message. If the user chooses **Squash and merge**, use the PR title as the commit title.


### Merge and protected completed work

- The 2026-11-14 user authorization supersedes earlier prohibitions on automatic checkpoint merges in PLAN and frozen assignments, from the next Git operation; it does not alter acceptance criteria, dependencies, status, or an active review candidate. Follow docs/roadmap/MERGE_WORKFLOW.md: only the coordinator serializes merges, with exact base/head, integrated checks and B review, CI, race protection and post-merge verification. Report blockers to the user; never bypass protection.
- Completed, accepted code, tests and expectations, contracts, schema and behavior merged into rewrite/rust are protected: do not edit, delete, move, refactor or reformat them for new work or conflict resolution, except for the narrow test-only conflict resolution authorized in docs/roadmap/MERGE_WORKFLOW.md. File ownership, passing tests or B approval is not an exception. Scoped work may add distinct new code and read/call existing public contracts. If changing protected work outside that exception is necessary, including a fix or revert, or resolving a conflict requires a core change, stop dependent work and report the checkpoint, merge SHA, affected files/behavior, evidence, minimal proposed diff and tests; proceed only after specific user approval. Continue independent work without changing a candidate under review.
- This authorization does not approve README changes, changing main or the default branch, publishing releases, modifying real data, or relaxing gates. Post-merge failure freezes dependent consumers and is reported to the user; do not automatically fix or revert protected work. Go cleanup remains subject to parity, owner, task and review gates and does not authorize changing protected Rust work.

- After confirmed normal push with exact remote SHA and PR creation/update, before the next checkpoint, perform and record safe task-owned scratch cleanup; docs-only checkpoints do so after B acceptance and recorded handoff. Follow [checkpoint cleanup](docs/roadmap/MERGE_WORKFLOW.md#checkpoint-cleanup), including retention, ownership, evidence and Windows containment safeguards. Apply prospectively from 2026-11-17 without changing frozen candidates or historical evidence.
