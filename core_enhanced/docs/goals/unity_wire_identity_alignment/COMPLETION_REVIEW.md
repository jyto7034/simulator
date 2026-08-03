# Unity Wire Identity Alignment Completion Review

Review date: 2026-07-14

Review procedure: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

## Item Audit Table

| Finding | Current owner/result | Verdict | Fit | Correction |
| --- | --- | --- | --- | --- |
| DCA-H-09 / CPR-H-05 | exact `target_employee_uuid` sender | Complete | High | None |
| DCA-H-10 / CPR-H-06 | candidate `id`, reward option `uuid` readers | Complete | High | None |
| DCA-M-14 | typed skill catalog `range_policy` example | Complete | High | None |
| DCA-M-15 | `FloorAdvanced` completion name | Complete | High | None |
| DCA-M-16 | `primary_abnormality_id` result identity | Complete | High | None |
| DCA-L-02 | complete root example fields | Complete | High | None |
| DCA-L-03 | active modifier `name` reader | Complete | High | None |
| PD-04 shared seam | one root/result Event inner DTO | Complete | High | None |

## Runtime And External Contract Evidence

- Rust `PlayerBehavior::UseConsumableItem` and Unity `GameCommandBus` now agree exactly on
  `item_uuid + target_employee_uuid`.
- Rust `StarterEmployeeCandidate.id` and `RewardOptionSnapshotDto.uuid` are consumed directly by
  Unity. The distinct reward definition `id` remains visible but is not command identity.
- `CombatResultModel.PrimaryAbnormalityId` reads the serialized Core field and mock snapshots no
  longer advertise a competing name.
- `ActiveConsumableModifierSnapshotDto.name` is the only Unity display-name source for the active
  modifier; effect JSON remains gameplay detail, not an identity fallback.
- Root Event content and `EventState` serialize the same `EventSceneSnapshotDto`. game_server uses
  the generic typed command mapping and performs no Event-specific rewrite.

## Data Evidence

No live RON schema, value, loader, or save data changed. G02 corrected consumers and canonical
examples against existing typed runtime contracts. G03 remains the owner of the accepted Event
inner-schema cutover.

## Test Evidence

- Red Unity reproduction: 4/4 failed for exact intended mismatches.
- Focused corrected Unity: 5/5 passed.
- Core exact HQ, reward, and shared Event envelope: 3/3 passed.
- Core all-targets 723; game_server all-targets 28.
- Unity EditMode 194/194; ordinary PlayMode 51 passed/2 explicit skips; console 0.

## Legacy/Fallback Audit

- No Unity current snapshot reader accepts candidate `candidate_id`, reward `reward_id`, or active
  modifier `display_name`/effect-name fallback.
- `candidate_id` and `reward_id` remain only where they are the correct command/result field names.
- No item command alias, combat-result identity alias, server JSON repair, stale completion name,
  relevant ignored test, or mock-only current identity remains.
- Historical Unity goal notes retain earlier wrong judgments as experiment history and are not
  runtime/canonical sources.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: every identity is read once at the typed Unity boundary and forwarded unchanged; producer,
  command identity, display identity, and definition identity are no longer conflated. The shared
  Event envelope lets G03 replace one inner DTO instead of migrating root and command results twice.

## Cross-Component Findings

- Core/server/Unity agree on all seven exact fields.
- G02 does not pre-implement G03 behavior and introduces no compatibility window.
- Phase 1 capability authority and Phase 2 transaction/recovery semantics are unchanged.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

None inside G02. G03 is the required successor feature, not cleanup debt.

## Policy Questions

None.

## Review Conclusion

G02 is Complete / High with no immediate correction. G03 may begin the coordinated illustrated
employee-check Event vertical slice against the fixed shared envelope and exact wire identities.
