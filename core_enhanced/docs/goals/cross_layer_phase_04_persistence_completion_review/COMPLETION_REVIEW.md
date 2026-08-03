# Cross-Layer Phase 04 Persistence Completion Review

Review date: 2026-07-14

Review procedure: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction: none

## Item Audit Table

| Phase item | Current integrated result | Verdict | Fit | Correction |
| --- | --- | --- | --- | --- |
| G17 identity lifetime | checkpoint-owned streams rewind while global VN/gameplay occurrence identities remain independent | Complete | High | None |
| G23 reward/growth | one staged typed grant executor and live policy-owned XP/level/tier semantics | Complete | High | None |
| G16 record removal | no gameplay battle history; battle-keyed traces are external and non-fatal | Complete | High | None |
| G29 global compendium | actual-spawn consuming union outside run/checkpoint rollback | Complete | High | None |
| G21 Safe Zone return | all-pending atomic delivery on planning completion only | Complete | High | None |
| Server/Unity integration | generic strict pass-through/profile union and presentation-only readers | Complete | High | None |

## Persistence And Identity Review

- Checkpoint restore rewinds run-owned UUID streams and state while preserving only the explicitly
  global VN namespace. It does not preserve a whole manager or accidentally advance gameplay IDs.
- Compendium state lives at the Core root and merges monotonically. Run/checkpoint/battle-entry loads
  cannot replace or erase it; server actor recreation also merges rather than replaces.
- Pending research remains ordinary inventory state and follows checkpoint semantics. Delivery is a
  transaction at planning return, not a global-profile operation. SavePoint captures the delivered
  value, preventing duplicate load delivery.
- Simulation delivery identity is independent from observation-log insertion. Optional trace names
  use actual battle UUID and are never read back as gameplay state.

## Result And Transition Review

- One staged combat result composes employee condition, survival XP, authored reward effects,
  run-local research, node consumption, compendium discovery, and final transition without reviving
  battle history.
- Compendium union occurs only after a consuming transition succeeds. Retry and retreat skip it;
  fatal/final consuming failure retains it despite RunFailed.
- Research delivery qualifies only when the final visible transition is `NodeCompleted` or
  `FloorAdvanced`. Retry, retreat, checkpoint load, reconnect, no-recovery failure, RunComplete, and
  RunFailed leave the pending queue untouched.
- Event-started combat composes Phase 3 ownership: battle completion cannot grant until the Event
  itself resolves. Gate arrival composes G14's floor transition and grants before its new planning
  result is published.

## External Contract Review

- Core root compendium projection and planning-completion delivery facts have separate strict DTOs.
  Neither is inferred from event history, inventory differences, preview, or local catalog state.
- game_server preserves typed Core results/snapshots and result-before-snapshot order. Its only
  compendium state is a player-global monotonic profile used when recreating Core.
- Unity's compendium and delivery readers reject unknown/missing/duplicate data. The client displays
  the command result once through the trailing snapshot and contains no grant/discovery authority.
- Canonical Core/server/Unity documents and the current non-combat probe describe the current shapes
  and lifetimes; historical audit prose remains evidence only.

## Validation Review

- Independent R04 Core focused reruns: identity 1, reward/growth 1, trace failure 1, compendium 3,
  research 19; all passed.
- Independent server focused reruns: compendium actor recreation 1 and research delivery ordering 1;
  both passed.
- Independent Unity combined strict contract rerun: 9/9.
- Current broad evidence on unchanged production code: Core 746, server 30, Unity EditMode 211,
  Unity PlayMode 52 passed with four explicit environment-dependent live-server skips.
- Formatting, checks, generated fixture equality, relevant ignored-test, legacy/fallback, current
  canonical, and scoped diff audits passed. Zero-test binary/example targets are not counted.

## Legacy, Fallback, And Cross-Component Audit

- No player battle-record state/query/DTO, abnormality-keyed diagnostic store, compendium rollback
  copy, replacement import, category delivery allowlist/hook, entry delivery DTO, Unity grant /
  discovery calculation, server JSON repair, or relevant compatibility alias remains.
- G16 and G29 are complementary: deleting historical combat payloads leaves a small ID-set profile,
  not a renamed history store.
- G17 and G29 lifetimes agree: checkpoint rewind is explicit and cannot erase global knowledge.
- G23 and G21 share the fragment inventory executor/policy but not authority; reward completion may
  queue research, while only the final planning transition delivers it.
- G29 and G21 both compose node consumption but cannot partially synchronize because compendium is
  an infallible validated set union after successful completion and delivery is prevalidated in the
  staged result/inventory.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: Phase 4 removes an obsolete persistence domain, separates global/run/diagnostic lifetimes,
  keeps rewards and delivery staged at their owning transitions, and exposes only strict typed facts
  to generic transport and presentation-only clients. Future profile storage, reward content, and
  node categories extend these seams without compatibility migrations.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

None inside Phase 4.

## Policy Questions

None.

## Review Conclusion

R04 is Complete / High with no immediate correction. Phase 4 may close and Phase 5 may begin with
the persistence, identity, result, global-profile, and planning-return boundaries above fixed.
