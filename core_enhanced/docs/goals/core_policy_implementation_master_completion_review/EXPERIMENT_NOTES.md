# Experiment Notes

## Review Principles

- This is an audit-only goal. Do not implement broad fixes while reviewing.
- The audit source of truth order is runtime code, live RON/data, Unity/server contracts, latest policy docs, then goal notes.
- A completed checkbox is only a claim. Treat code/data/contract behavior as evidence.
- The audit must inspect long-term direction, not only immediate behavior.
- Findings should become precise follow-up candidates, not vague "needs cleanup" notes.

## Required Long-Term Direction Questions

Ask these for every completed item:

- Did the implementation reduce source-of-truth duplication?
- Did it introduce any new synchronized duplicate state?
- Is old behavior removed, or is it still available through a fallback, compatibility layer, dual schema, or ignored test?
- Is the implemented shape a domain model, or a patch around the old model?
- Are type names, module placement, and ownership boundaries clear?
- Can future content/policy changes use this structure without another migration?
- Do tests pin user-visible behavior, DTO shape, live RON loading, validation, or gameplay flow?

## Policy Decision Notes

If any item requires a new gameplay/UX/schema/balance decision, record:

```text
사용자와 정책 논의 필요
```

Then include:

- affected subgoal/item,
- code/data evidence,
- options,
- recommended long-term choice,
- what implementation must wait for the decision.

The review goal can complete with policy questions in the audit report. Do not block or silently decide.

## Audit Report Shape

`AUDIT_REPORT.md` should contain:

1. Executive summary.
2. Item-by-item verdict table.
3. Detailed evidence for risky/incomplete items.
4. Long-term direction findings.
5. Cross-component findings.
6. Immediate correction candidates.
7. Follow-up refactor candidates.
8. Validation commands and results.
9. Policy questions, if any.

## Initial Risks To Watch

- Master/subgoal documents may record a policy as implemented even if only the happy path was tested.
- Legacy RON loaders or validation acceptance may remain after live data migration.
- Unity-facing DTOs may still expose old fields even when core runtime moved to a new source of truth.
- Tests may have been updated to match implementation details rather than gameplay behavior.
- Debug event logs/exports may be necessary artifacts, but they must remain debug-only and not become runtime source.
- Subgoals may individually pass while cross-component contracts still drift.

## Audit Findings Notes

- `MapViewDto.available_node_ids` / `completed_node_ids` were removed, and server helper code now derives availability from `MapNodeDto.state`. However, `RunSnapshotDto.map_progression` still exposes `MapProgressionSnapshotDto.available_node_ids` and `completed_node_ids`, so the broader Unity-facing snapshot still has a duplicate progression projection.
- `Timeline naming` is not truly implemented as a rename. Current runtime code documents `Timeline` as an append-only battle event log but explicitly keeps the serialized/client-facing name for compatibility. This conflicts with the older confirmed policy unless the newer `refactor_preparation_plan.md` baseline is intended to supersede it.
- Skill targeting cleanup is complete for live skill RON and the Unity-facing skill catalog, but internal `SkillStepDef.range_units` and `SkillCastTargetingDef::FirstStepTarget` remain in non-RON/test-constructor surfaces. Treat this as a follow-up cleanup, not a current policy failure.
- Typed run snapshot migration is mostly healthy. The remaining `serde_json::Value` usage is at JSON wrapper/server transport boundaries, not inside `RunSnapshotDto` itself.
- `core_policy_validation_test_harness/POLICY_COVERAGE.md` has stale text saying final broad validation still needs to run, despite `EXPERIMENTS.md` recording broad validation success.

## Policy Question List

사용자와 정책 논의 필요:

1. Should `RunSnapshotDto.map_progression.available_node_ids` / `completed_node_ids` be removed under the already-confirmed `MapNodeDto.state` SoT policy, or retained temporarily under explicit debug-only names?
2. Should the `Timeline naming` decision still require a Rust/JSON rename, or should the policy be reconciled to keep serialized `Timeline` while documenting battle-event-log semantics?
