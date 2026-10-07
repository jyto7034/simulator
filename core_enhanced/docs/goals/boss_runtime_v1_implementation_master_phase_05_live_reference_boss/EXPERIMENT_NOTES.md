# Boss Runtime v1 Phase 05 — Notes

## Approved disposable-reference boundary

- The user selected option 1 and explicitly delegated the missing details because the content will be redesigned
  later.
- `suppress_little_red` is therefore production-live only as a replaceable runtime integration reference. This Goal
  must not describe the pursuit proposal as final Little Red lore or complete character design.
- Current stats, waves, rewards and `little_red_hunt_the_prey` are preserved. The only new content behavior is the
  bounded finite pursuit/rejoin sequence using already implemented vocabulary.
- Generic opponent capsule and existing public events/checkpoint are the approved presentation boundary. Dedicated
  bounty UI/reward, art, animation, VFX and Big Bad Wolf coupling are excluded.

## Direct dependencies

- Master: `../boss_runtime_v1_implementation_master/PLAN.md`
- Decision packet: `../boss_runtime_v1_implementation_master/P05_CONTENT_DECISION_PACKET.md`
- P04 completion audit:
  `../boss_runtime_v1_implementation_master_phase_04_track_and_rejoin_vertical_slice/COMPLETION_AUDIT.md`
- P04 is the direct prerequisite. P02/P03 contracts are consumed through the P04 audited handoff and current runtime
  canonical; their historical implementation logs are not reinterpreted as current source of truth.

## Dirty worktree

- Simulator start commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Exact precreation status: 261 sorted entries, SHA-256
  `832ea5972d60e3613d9dd3252dab0299853617c41f88c426f1e9f75d17ea73a8`.
- All existing modifications and untracked paths are user-owned. Required production owners must receive scoped
  diff/hash review before mutation.
- External Unity was previously observed as a separate, very dirty user-owned tree. P05 may inspect it read-only but
  must capture a fresh exact baseline before making any current-consumer claim. Unity mutation is not authorized.
- Fresh external baseline: HEAD `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`, 2154 sorted status entries,
  SHA-256 `c4c939c7b4a52c886aa00ad4c11aa68f136a72fc7fcbacc4eb28f833713e47f1`. This differs from the
  previous policy-research snapshot and is treated as user-owned concurrent work.
- On resume, the Simulator has one additional untracked `game_server/logs/app.log.2026-08-09`. Exact single-entry
  subtraction restores the P05 postcreation baseline hash. The log is unrelated, user-owned and must remain untouched.

## Structural decisions

- Live RON publication, exact PvE binding/route and feasibility validation form one atomic content migration.
- P05 may add tests and close missing strict validation needed by CRA-P01-C02/CRA-P05-C02, but it may not expand the
  runtime vocabulary or public contract.
- A content-only reference should not need DTO or save migration. Evidence contradicting this assumption is a stop
  condition, not permission to improvise P06 work.
- P06 disposition is made only after actual live path and external consumer evidence. It is not decided by the
  decision packet alone.

## Current risks to investigate

- The current validation may exercise generated previews without enumerating every manifested difficulty/setup
  combination required by P05; exact gaps must be established from source and tests.
- The exact Little Red wave and all eligible battlefield templates must provide a valid authored route after the
  minimal edit without changing support waves or encounter tuning.
- The existing automatic skill contains a step-level retarget rule independent of Boss Pattern target lock; tests
  must prove leaf skill authority does not create a second Boss graph target authority.
- External Unity's generic actor renderer must be rechecked against current source and transport docs before P06 can
  be dispositioned.

## Implemented live reference boundary

- Production now has exactly one Boss runtime graph and opt-in: `suppress_little_red` binds
  `primary_abnormality[0]` to `little_red_reference_encounter.primary_abnormality`.
- The exact primary wave alone gained `defense_main`; stats, support wave, both 5-second wave times, reward UUIDs and
  `little_red_hunt_the_prey` remain pinned by integration test.
- Actual live world tests cover success/rejoin/RouteCombat, no target, invalidation, path failure, battle-time
  timeout, Pause, reconnect, actor interruption and terminal exactly-once behavior. An actual non-opt-in live Boss
  encounter proves ordinary parity.
- Current runtime retains a dead actor's stable Boss slot identity but stops graph work. This is private state and is
  tested by stable same-time reconciliation, not by requiring identity deletion.
- Publication now exercises every live profile's three ordinary bands, four non-Monolith roles and Monolith Stage2/
  Stage3 at the five established seeds, then runs the same preview through warning and full setup feasibility.

## Confirmed publication-gate findings

- CRA-P01-C02 is still open in current source: publication validates five seeds only at baseline/no manifested
  difficulty, while runtime resolves a typed `MapNodeCombatDifficulty`.
- CRA-P05-C02 is still open in current source: publication stops at preview/threat checks; resolved spawn capacity,
  tactical/field/object references, Boss actor binding and final scenario invariants are late runtime checks.
- The shared finite matrix must have one owner. For each authored difficulty profile it comprises each authored
  ordinary band, each non-Monolith role, plus Monolith Stage2 and Stage3; the same preview instance then feeds both
  warning and setup-feasibility checks for each of the existing five seeds.
- `suppress_little_red` is a Boss encounter, so its manifested validation category must be `Boss`, not the current
  hard-coded `Combat`. Ordinary encounters retain `Combat`.
- Setup feasibility can remain pure: enemy actor UUIDs are deterministic derivatives, and an empty player group is
  sufficient. No world session, active battle, run inventory or mutable UUID allocator is needed.
- The production scenario assembler is currently private to `events/combat.rs`. Implementation must extract or
  expose one shared setup predicate without copying its field assembly into data validation; runtime defensive
  checks remain in place.

## Current external consumer finding

- Both external canonical files and the battlefield renderer have material user-owned edits, but their current
  content preserves the P05 seam: Boss/Boss uses the normal DefenseRoute setup/update flow, actor identity and HUD
  come from Core, and movement/damage/checkpoint are existing semantic inputs.
- Current `BattlefieldFieldAssembler.EnsureUnitActorVisual` maps every opponent to
  `EnsureEnemyUnitCapsule` without abnormality-specific lookup. The approved common-capsule boundary therefore
  requires no Unity mutation and does not misstate an existing Little Red asset path.
- These files are read-only evidence. Their current hashes and the 2154-entry external dirty baseline are recorded in
  `EXPERIMENTS.md`; P05 must recheck them at completion if they change again.

## Final external/P06 finding

- The three external contract/renderer evidence files are byte-identical at completion even though the wider
  2154-entry external dirty-tree hash changed concurrently. The project was inspected read-only and no Unity owner
  was mutated.
- Actual live Core serialization, reconnect, terminal and server all-target validation confirms that the common
  actor event/checkpoint path is sufficient. P06 is therefore explicitly `not_applicable_to_v1`, not silently
  skipped.

## Open policy questions

None within P05. Discovery of a required visible marker, asset, balance change, new gameplay vocabulary or public
lifecycle immediately reopens a user decision and stops the affected work.
