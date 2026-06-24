# Goal Completion Review Guide

This document defines a reusable review process for checking whether completed master/subgoal items were actually implemented in the long-term direction.

It is not a gameplay policy source of truth. Use it as an audit procedure when reviewing completed goal checkboxes, post-policy implementation goals, or broad refactor master goals.

## Purpose

A completion review must answer more than "is the checkbox checked?" It must verify:

- the documented policy was implemented in runtime behavior,
- the implementation matches the project's long-term refactor direction,
- no legacy fallback, compatibility layer, dual schema, or hidden source-of-truth split remains,
- tests protect user-visible behavior and external contracts,
- any remaining risk is explicitly classified for follow-up.

## Source Of Truth Order

Review evidence in this order:

1. Actual runtime code.
2. Live RON/data.
3. Unity-facing snapshot/command/WebSocket contracts.
4. Latest policy documents.
5. Goal documents and historical notes.

If a goal document says an item is complete but runtime code or live data says otherwise, the review must treat the item as incomplete until proven by code/data behavior.

## Review Inputs

For each completed master goal item, collect:

- master goal item text,
- linked subgoal directory,
- confirmed policy decisions,
- expected user-visible behavior,
- affected runtime modules,
- affected live RON/data files,
- affected Unity/server DTOs or transport messages,
- tests and validation commands that supposedly fixed the behavior.

If any of these links are missing, record the missing link as a review finding instead of guessing.

## Per-Item Review Procedure

For every completed item:

1. Restate the intended policy or refactor outcome in one paragraph.
2. Identify the expected source of truth after the implementation.
3. Read the runtime code paths that create, mutate, validate, serialize, and consume that source of truth.
4. Read the relevant live RON/data and schema/loader validation.
5. Check Unity-facing DTOs, command payloads, WebSocket messages, snapshots, and timeline/log outputs if the item touches external contracts.
6. Search for old names, old enum variants, old fields, fallback paths, ignored tests, debug-only bypasses, and compatibility shims.
7. Read the tests and decide whether they pin behavior rather than private implementation shape.
8. Run focused validation for the item.
9. Record a verdict with evidence and remaining risk.

## Long-Term Direction Review

For each completed item, also inspect whether the implementation is genuinely the long-term design rather than a local patch.

Check:

- whether the implementation reduced source-of-truth duplication,
- whether it introduced any new duplicated state that must be kept in sync,
- whether it removed obsolete code/data/tests instead of preserving them behind compatibility layers,
- whether it avoided special-case patches that only satisfy one known test,
- whether names, types, and module locations match the domain responsibility,
- whether existing functions could have been composed instead of adding a parallel feature,
- whether a simpler structure is now possible but was left unrealized,
- whether responsibilities became clearer across runtime, data loading, validation, Unity DTOs, and tests,
- whether future content/policy changes can be represented without another migration,
- whether tests protect the durable contract rather than the current helper layout.

Record:

- Long-term fit: high / medium / low.
- Improvement class: none / follow-up refactor / immediate correction recommended.
- Reason: concrete code/data/test evidence.

## Verdict Categories

Use exactly one primary verdict per item:

- Complete: implemented in code/data/contracts, covered by meaningful validation, no material legacy path remains.
- Partially complete: main path works, but an edge path, data loader, DTO, validation, or test gap remains.
- Document-only complete: goal docs mark it complete, but runtime/data behavior does not prove it.
- Policy decision needed: implementation requires a gameplay/UX/schema/balance decision not settled by current policy.
- Not applicable: the item became obsolete because a later confirmed policy removed the need.

Add a secondary long-term fit rating:

- High: design simplifies ownership and matches the expected future model.
- Medium: acceptable now, but follow-up simplification or consolidation is likely.
- Low: works by patching around the old model or leaving a confusing ownership split.

## Evidence Template

Use this shape for each reviewed item:

```md
## <Item Name>

Master item:
Subgoal:
Policy source:

Expected behavior:

Runtime evidence:
- `<file>`: ...

Data evidence:
- `<file>`: ...

External contract evidence:
- `<file or DTO>`: ...

Test evidence:
- `<test name or command>`: ...

Legacy/fallback audit:
- ...

Long-term direction review:
- Fit: high / medium / low
- Improvement class: none / follow-up refactor / immediate correction recommended
- Reason: ...

Verdict:
- Complete / Partially complete / Document-only complete / Policy decision needed / Not applicable

Remaining risk:
- ...
```

## Cross-Component Review

After all items are reviewed individually, perform a second pass across components.

Check:

- whether two completed items define competing sources of truth,
- whether one goal's DTO or schema assumptions contradict another goal,
- whether tests pass only because components are not exercised together,
- whether live RON/data still follows an older policy,
- whether Unity-facing docs/contracts still describe an older shape,
- whether debug exports/logs are intentionally debug-only or accidentally used as runtime source.

This pass should produce a short list of cross-component findings. Do not bury these inside per-item notes.

## Validation Expectations

Prefer focused validation first, broad validation last.

Focused validation should cover:

- the runtime behavior changed by the item,
- live RON loading and validation when data shape changes,
- Unity/server DTO shape when external contracts change,
- representative gameplay flow when timing/order matters.

Broad validation should include the smallest command set that exercises the affected blast radius. Record commands, failures, fixes, and final results.

## Review Output

A review goal should produce:

- an item-by-item audit table,
- detailed evidence sections for incomplete or risky items,
- a cross-component findings section,
- a list of immediate correction candidates,
- a list of follow-up refactor candidates,
- validation commands and results,
- explicit policy questions, if any.

Do not silently fix broad issues during the review unless the review goal explicitly includes implementation work. Prefer producing a precise correction plan that can become a dedicated goal.
