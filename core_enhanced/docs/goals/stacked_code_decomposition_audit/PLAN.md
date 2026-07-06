# Stacked Code Decomposition Audit

## Objective

Audit accumulated AI-written or patch-by-patch code and turn it into small, evidence-backed refactor goals that restore a single readable flow.

"Stacked code" means code that may work locally, but no longer explains the domain clearly because multiple implementation layers, temporary fixes, fallback paths, duplicated policies, or mismatched naming have accumulated without a unified ownership model.

This goal is a review and planning goal first. It should not perform broad rewrites while auditing. Implementation work should be split into follow-up goals with focused scope and validation.

## Purpose

The audit must answer:

- Which runtime flows are hard to read because behavior is spread across layers that do not share one policy?
- Where does the code force the reader to reconstruct hidden context before making a safe change?
- Which code paths are real live behavior, and which are legacy, compatibility, debug, test-only, or dead code?
- Which duplicated states or helpers are true source-of-truth duplication, and which are valid derived projections?
- Which local patches should be deleted, consolidated, renamed, or left alone?
- Which findings are safe to fix now, which require policy decisions, and which should become separate goals?

## Readable Code Standard

Readable code is code that lets the next maintainer make the correct change with minimal hidden-context reconstruction.

Do not judge readability by shortness, cleverness, file count, or visual neatness alone. Judge whether the current gameplay rule, responsibility owner, failure boundary, and change point are visible from the code structure.

Use these criteria:

- The flow reads in one direction: entry, validation/planning, mutation/commit, projection/response, and persistence if any are easy to follow.
- Names describe current responsibility, not historical reasons, previous policy, or the audit that created the name.
- Each gameplay rule has one canonical source of truth; derived snapshots, caches, DTOs, and views are visibly derived.
- Fallible planning/preflight is separated from commit/application, or the reason a commit step can fail is explicit in its name and caller ordering.
- A function stays at one abstraction level instead of mixing policy decision, mutation, DTO building, persistence, debug export, and test fixture repair.
- Legacy and compatibility paths are absent unless a current migration policy and removal condition explain why they remain.
- Tests describe durable behavior, gameplay contracts, data validation, or external DTO shape rather than only private helper structure.
- The code answers likely maintainer questions locally: what owns this state, when can this fail, what is user-visible, and what must change together?

When a finding claims a readability problem, state the specific hidden context the reader currently has to reconstruct.

## Source Of Truth Order

Review evidence in this order:

1. Actual runtime code used by current gameplay flow.
2. Live RON/data and loaders/validators.
3. Unity-facing snapshot, command, WebSocket, and save contracts.
4. Current canonical docs such as `docs/refactor_preparation_plan.md`, `docs/game_rulebook.md`, and contract docs.
5. Existing goal documents, audit notes, comments, and historical names.

If docs and code disagree, trust code/data behavior first, then update or flag the docs. If a gameplay, schema, UX, balance, reward, or migration decision is required, stop and report the policy question instead of guessing.

## Audit Scope

Use this goal for accumulated-code review across:

- gameplay flow orchestration such as run state, node entry/complete, reward, shop, combat setup, and result handling,
- live battle command/update/result boundaries,
- data loading and validation paths,
- Unity-facing DTO and server transport mapping,
- tests that preserve old behavior or assert private implementation shape,
- oversized modules where several policies are interleaved,
- helper layers whose names no longer match their real responsibility.

Out of scope unless a finding proves they affect the audited flow:

- pure balance value changes,
- new gameplay features,
- Unity visual implementation details,
- speculative future abstraction,
- broad file hierarchy reshuffling without a behavior or ownership finding.

## Stacked Code Signals

Record a finding when code shows one or more of these signals with concrete file evidence:

- The same gameplay rule is encoded in multiple places and must be changed together.
- A new branch, flag, enum variant, or fallback exists mainly to avoid removing an older path.
- A function name describes an old policy while the body implements a newer policy.
- Callers must know internal ordering, side effects, or hidden mutations to use an API safely.
- A public DTO, snapshot, or save field exposes data that is not part of the current external contract.
- Runtime code accepts invalid or legacy data and silently repairs it instead of failing at load/validation time.
- Tests pass by asserting intermediate helper behavior while the user-visible gameplay contract is unclear.
- A module mixes command parsing, policy decision, mutation, projection, serialization, and test fixture setup.
- Similar helpers exist because later changes copied an earlier shape instead of extracting the actual shared rule.
- Comments or docs explain why the code used to exist, but not why it exists in the current flow.

Do not record a finding from shape similarity alone. First confirm whether the compared paths share the same lifecycle, policy owner, timing, external contract, and validation boundary.

## Review Procedure

For each audited flow:

1. Name the current user-visible or runtime behavior in one paragraph.
2. Trace the entry point, state mutation, validation, projection/DTO, persistence if any, and tests.
3. Identify the intended canonical source of truth and every derived projection/cache/snapshot.
4. Search for old names, fallback branches, compatibility schemas, duplicate validators, ignored tests, and debug-only bypasses.
5. Classify each suspicious path as live, derived, legacy, compatibility, debug-only, test-only, or dead.
6. Record the hidden context a maintainer must reconstruct before changing the flow safely.
7. Decide whether the issue is deletion, consolidation, renaming, boundary extraction, validation hardening, test rewrite, doc update, or policy question.
8. Define the smallest follow-up goal that can improve one source-of-truth, failure boundary, naming truthfulness, or ownership boundary.
9. List focused tests or probes that would prove the behavior before and after the follow-up goal.

## Finding Categories

Use exactly one primary category per finding:

- Source-of-truth split: more than one canonical place owns the same policy or state.
- Legacy path: old behavior remains reachable or influential after the current policy moved on.
- Compatibility layer: old schema/API behavior is preserved without an explicit migration policy and removal condition.
- Responsibility pile-up: one module/function owns too many unrelated steps in the flow.
- Naming drift: names preserve an old mental model and mislead future edits.
- Failure-boundary blur: code makes it unclear which steps may fail and which state has already been committed.
- Validation gap: invalid data is accepted too late, repaired silently, or only caught by incidental runtime behavior.
- Test contract drift: tests protect helper shape, old policy, or debug output instead of durable gameplay/user-facing behavior.
- Documentation drift: docs, comments, or goal notes describe a policy that code/data no longer implements.
- No issue: the path is intentionally derived, separated by lifecycle, or different by gameplay contract.

Add a severity:

- Immediate correction: likely to cause bugs, contract mismatch, or repeated wrong edits.
- Follow-up refactor: materially improves readability/change cost but can be scheduled.
- Document only: code is acceptable, but the mental model needs clarification.
- Hold: real concern, but policy or adjacent audit evidence is missing.

## Decomposition Rules

When turning findings into refactor goals:

- Prefer deleting obsolete code before adding abstractions.
- Prefer one canonical source plus explicit derived projections over synced duplicate state.
- Prefer load-time validation failure over runtime fallback for unsupported data.
- Prefer domain names that describe the current gameplay rule, not implementation history.
- Prefer structures where a reader can see failure-prone planning before irreversible mutation.
- Prefer splitting mixed-level orchestration only when the split names real domain steps and reduces the number of facts a caller must know.
- Keep public DTO, save, live RON, and Unity-facing shape unchanged unless the goal explicitly includes that contract decision.
- Split implementation goals by source-of-truth boundary, not by file count.
- Do not introduce traits, strategy objects, adapters, or generic policy layers unless multiple current variants prove the need.
- Update tests to pin user-visible behavior, runtime contract, data validation, or external DTO shape.
- If a refactor requires a gameplay or contract decision, stop the implementation goal and record the question.

## Output Template

The audit should produce an `AUDIT_REPORT.md` or equivalent note using this shape:

```md
# <Flow Or Component> Stacked Code Audit

## Summary

- Verdict:
- Main source-of-truth owner:
- Main hidden context:
- Highest-risk finding:
- Recommended next goal:

## Findings

| ID | Category | Severity | Evidence | Recommendation |
| --- | --- | --- | --- | --- |
| F-001 | Source-of-truth split | Immediate correction | `<file>` | Create follow-up goal ... |

## Flow Trace

- Entry point:
- Planning/preflight boundary:
- Mutation owner:
- Commit/application boundary:
- Validation boundary:
- Derived projections:
- External contracts:
- Tests:

## Readability Review

- Current reader burden:
- Misleading names or abstraction levels:
- Facts that should become local/structural:
- Expected readability improvement:

## Follow-Up Goal Candidates

### <Goal Name>

Objective:

Scope:

Completion conditions:

Readable-code improvement:

Focused validation:

Policy questions:

## No-Issue Notes

- `<file>` is a derived projection, not a duplicated source, because ...
```

## Completion Conditions

- At least one accumulated-code area is audited from runtime entry point through tests/contracts.
- Each finding has concrete code/data/doc evidence, not only style preference.
- Each readability finding states what hidden context currently must be reconstructed by the reader.
- Live, derived, legacy, compatibility, debug-only, test-only, and dead paths are explicitly distinguished where relevant.
- No broad implementation rewrite is performed inside the audit unless separately approved.
- Follow-up goal candidates each reduce one source-of-truth split, responsibility pile-up, failure-boundary blur, validation gap, or naming drift.
- Follow-up goal candidates define the expected readable-code improvement, such as clearer flow order, truthful names, one canonical owner, or fewer cross-file facts required.
- Policy questions are listed instead of being silently resolved.
- Focused validation commands or probes are named for each immediate correction candidate.

## Validation Commands

This goal is primarily documentary, but use code search and focused tests to prove findings.

Suggested baseline commands:

- `cargo check -p game_core`
- focused `cargo test -p game_core <module_or_behavior> --lib -- --test-threads=1`
- focused integration tests under `tests/` when Unity-facing or gameplay-flow contracts are involved

Adjust exact commands after the audited flow is selected.

## Stop Conditions

Stop the audit and report questions if:

- current behavior cannot be identified from runtime code and live data,
- the apparent cleanup would change Unity-facing DTOs, save data, live RON schema, reward policy, failure policy, balance, or UX meaning,
- the suspected duplication may actually represent two different lifecycles or gameplay contracts,
- meaningful validation cannot be defined for the proposed follow-up goal.
