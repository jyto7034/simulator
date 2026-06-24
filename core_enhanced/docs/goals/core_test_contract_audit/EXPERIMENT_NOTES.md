# Core Test Contract Audit Notes

## Initial Judgment

The audit should judge tests by the contract they protect, not by whether they are unit tests or integration tests.

Preferred test contract targets:

```text
player-visible behavior
-> Unity-facing DTO/transport
-> live RON/data validation
-> domain policy invariants
-> implementation helpers only when they protect one of the above
```

## Classification Labels

Use these labels during the audit:

- `strong_contract`: breaks when a player-visible, Unity-facing, live-data, or policy contract breaks.
- `useful_narrow`: covers a real contract but only a small branch or edge case.
- `implementation_shape`: mostly pins private helper structure or current implementation layout.
- `smoke_only`: useful for broad loading/compilation confidence, but weak as a contract by itself.
- `legacy_or_stale`: preserves behavior that may no longer be a current policy.
- `gap`: current policy exists but meaningful tests are missing or too weak.

## High-Risk Areas To Check First

- Battle transport tests:
  - setup/update/resync top-level server message shape
  - command result side messages
  - final battle update before combat result snapshot
  - setup-loss recovery
  - future `known_seq` rejection

- DefenseRoute gameplay tests:
  - deployment/withdraw/defeat redeploy cooldowns
  - retreat and re-entry
  - generated route continuity
  - route movement and blocking
  - ranged route attack/reposition behavior
  - attack/movement time consistency

- Skill/attack tests:
  - tile-based basic attack eligibility
  - basic projectile target-only collision
  - skill projectile directional collision
  - `TileArea`
  - `WholeFieldValidTiles`
  - target usefulness for zero damage plus hostile effects
  - skill cast interrupt/epoch

- Live data tests:
  - live abnormality roster
  - corroded employee range presets/profile roles
  - skill fragment and equipment placeholder indexes
  - live PVE encounter references
  - forbidden legacy content

## Stop-First Questions

Stop and ask the user if:

- a test appears weak because the game rule is not settled
- a test preserves old behavior that might still be intentionally supported
- improving the test would require DTO/schema/data changes
- a missing test implies a new policy decision rather than a simple coverage gap

## Follow-Up Candidate Buckets

Record follow-up work under these buckets:

- `test_strengthening`: rewrite or add tests around an existing settled contract.
- `test_deletion`: remove stale/legacy tests after confirming the contract is obsolete.
- `contract_documentation`: update docs because tests reveal the actual contract is different.
- `runtime_bug`: code appears to violate the current contract.
- `data_policy`: live RON/data coverage exposes a content/schema decision.

## 2026-06-21 Audit Judgment

Overall judgment:

- The current core/server test suite is substantially better than smoke coverage. Most recent high-risk contracts have direct tests.
- The strongest areas are live battle transport, DefenseRoute deployment/retreat/recovery, tile-based attack eligibility, projectile delivery policy, skill interrupt/stale-event handling, combat preview route/template validation, and live RON reference validation.
- The suite still has some implementation-shaped tests, especially around movement backend internals, but they are not obviously harmful because the movement backend is still a sensitive subsystem.
- I did not find ignored tests preserving stale legacy behavior in the audited scope.

Source-of-truth judgment:

- Runtime code and tests agree that battle runtime state should flow through `battle_setup_snapshot`, `battle_update`/`battle_resync`, and `combat_result` `state_snapshot` rather than trailing full snapshots after every battle command.
- Runtime code and tests agree that basic attack eligibility is tile-based, not continuous `range_units`.
- Runtime code and tests agree that movement is still continuous for movement/presentation/projectile travel, while attack eligibility is tile-based.
- Runtime code and tests agree that hard setup loss recovery is not an in-place hard resync; it recovers to `node_confirm`.
- Live RON tests agree with the current “catalog roster only” abnormality policy and forbidden legacy reward filtering.

Policy questions:

- None block this audit.
- Follow-up work can proceed without changing production code now.

## Follow-Up Recommendations

Priority 1 - `test_strengthening`: attack/movement visible timing integration

- Add a live battle test that deploys a player unit, advances a route enemy into tile range, and verifies the timeline order and battle-time consistency of:
  - `MovementSegmentStarted`
  - attack start/resolve/projectile launch/impact events
  - `HpChanged`
  - checkpoint `world_position`
- This should prove that damage cannot become visible before the movement timeline makes the target eligible from Unity's presentation perspective.
- This is the highest-value gap because it directly matches the recent “damage appears before unit is visually in range” bug class.

Priority 2 - `test_strengthening`: live corroded employee profile contracts

- Add a live RON test that pins the current profile-to-range-preset mapping:
  - `corroded_guard`, `corroded_rusher`, `corroded_bruiser`, `corroded_veteran` -> `melee_front_1`
  - `corroded_marksman` -> `ranged_center_5x5`
  - `corroded_medic` -> `ranged_center_3x3`
- The existing data tests validate the preset mechanism; this would validate the live content policy.

Priority 3 - `test_strengthening`: WebSocket contract probe in CI or Rust mirror

- The server unit tests cover DTO serialization and actor message ordering, but the real WebSocket session/probe remains outside the cargo suite.
- Either keep the Python probe as an explicit manual verification artifact or mirror the critical flow in a Rust integration test:
  - auth
  - start/select starters
  - confirm live battle
  - receive `battle_setup_snapshot`
  - receive `battle_update`
  - no trailing battle command `state_snapshot`
  - final `combat_result` snapshot after final update

Priority 4 - `test_strengthening`: enemy range-preview omission / warning-only policy

- If the policy remains “enemy range cells are not sent unless needed for visual warning”, add a DTO test to prevent accidental full enemy range spam.
- This should be a transport/DTO test, not a Unity-only convention.

Priority 5 - `test_strengthening`: AttackWindUp or presentation attack phase if adopted

- Current tests focus on attack resolution and projectile behavior.
- If Unity needs a first-class attack wind-up event, add tests around event shape, `seq`, `time_ms`, source command/cause, and ordering before damage.
- Do not add this until the event is a settled contract.

Priority 6 - `test_maintenance`: split movement backend tests by intent

- Keep backend-specific tests while direct/Rapier behavior is under active development.
- Long term, consider naming or grouping tests by:
  - public movement policy
  - backend equivalence
  - Rapier synchronization internals
- This would make it clearer which tests must survive a backend rewrite and which can change with implementation.

## Tests To Treat Carefully During Refactors

- Do not loosen server transport tests just to make a command path easier. The absence of trailing battle `state_snapshot` is now a real Unity stability contract.
- Do not remove fixed-list live skill catalog tests unless they are replaced by an equal or stronger coverage manifest.
- Do not delete movement backend tests casually. Some are implementation-shaped, but they were added around real movement regressions and still protect the active backend transition.
- Do not treat debug event log export smoke as sufficient battle behavior proof. Preserve stronger world/battle event tests as the primary contracts.

## Audit-Only Boundary

- No production code, test code, live RON/data, or external Unity docs were changed.
- This goal should finish as an audit report. Follow-up implementation should be opened as separate goals.
