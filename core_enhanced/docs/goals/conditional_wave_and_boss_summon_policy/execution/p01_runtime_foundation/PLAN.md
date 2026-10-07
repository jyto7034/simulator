# P01 Runtime Foundation Execution Plan

- Goal ID: `GOAL-CWBS-P01-RUNTIME-FOUNDATION`
- Phase ID: `PHASE-CWBS-01-RUNTIME-FOUNDATION`
- Final Execution Node: `AG-CWBS-01-RUNTIME-FOUNDATION` (`WP-CWBS-01 + WP-CWBS-02`)
- Accepted Handoff Ref: `HA-CWBS-P2G60-002`
- Goal Set Ref: `GS-CWBS-P2G50-002`
- Run Mode / Root Target: `goal-run / GOAL-CWBS-MASTER`
- Authorization Ref: `GEXAUTH-CWBS-001`
- Current Pass: `GEX-20 pass 3 Complete / High`
- Direct Predecessor Dependency: `none`
- Execution Directory:
  `core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/execution/p01_runtime_foundation`
- Document Set: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`; `COMPLETION_AUDIT.md`는 fresh GEX-20에서 생성

## Objective and Contract-Visible Result

Wave/summon 공용 atomic Spawn Kernel과 trigger/request/candidate process-local ledger를 만들고, disconnect
auto-Pause, timeoutless same-process retention, snapshot 후 explicit Resume, process-death last-SavePoint rollback과
explicit-load-only usage를 완결한다.

Covered current policy revisions:
`PER-CWBS-SP-01-CODE-006`, `PER-CWBS-SP-02-CODE-006`, `PER-CWBS-PS-01-CODE-006`.

## Scope

### In scope

- placement preflight/commit/event ordering과 all-or-none/exact-once request identity
- value-bearing occupancy input, registration/removal primitive와 candidate/request ledger
- `BattleCore`/`ActiveBattleSession` process-local authoritative state
- server detach/attach/tick/timeout boundary와 explicit Resume
- active-run/profile store의 successful explicit-load-only usage boundary
- focused fixtures for `IVA-CWBS-01`, `IVA-CWBS-08`, Phase-local `IVA-CWBS-09`

### Out of scope

- concrete Wave/summon consumer cutover
- permanent compendium commit와 terminal outcome cutover
- active battle의 durable SavePoint 저장 또는 `RunCheckpointPayload` 전투 확장
- Unity, allied summon, production content와 deployment

## Current and Target Boundary

- Reviewed current behavior: Wave insertion is sequential, shared Kernel/ledger is absent, disconnect stops actor tick but
  does not pause Core and a 300-second TTL can remove the actor.
- Preserved actual: active battle is process memory and excluded from SavePoint.
- Target: Kernel transaction and session clock/state authority must close together without client-command pause or durable
  battle persistence.
- Old Wave/compendium consumers remain valid until later cutover; dormant candidate seam must be independently valid.

## Baseline and Validation Plan

Before production mutation, run exactly once with a goal-owned target directory:

```text
CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_core
CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_server
```

Record command, toolchain/lock/source/data/test identity, actual test counts and failures. After mutation, run focused Kernel,
ledger, occupancy, removal, detach/attach/no-TTL/Resume, restart/SavePoint/load-use tests. Reuse unaffected initial evidence;
rebaseline only invalidated seams.

## Completion Conditions

- Kernel commit is atomic, deterministic, all-or-none and exact-once with no eviction/partial allocation.
- restore is not registration; removal-first and reconnect ledger retention hold.
- disconnect pauses clock/settlement automatically, actor has no active-battle TTL, reconnect snapshot remains paused until
  explicit Resume and does not refire/reset.
- process death restores only the last completed SavePoint; active battle is absent from checkpoint payload.
- only a successful explicit load increments usage.
- old consumers and dormant candidate seam form a valid intermediate state.

## Stop Conditions

- Kernel commit and active-state writer cannot share a valid transaction/lifetime boundary.
- auto-Pause requires a client command or durable battle save.
- implementation requires policy meaning, Goal/Phase boundary, public compatibility or deployment authority change.
- existing dirty work overlaps required files and cannot be safely preserved.
- required Core/server/store consumer or validation seam is unavailable.

## Current Baseline

- Repository/worktree revision: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Toolchain/lock identity: Rust/Cargo `1.91.1`; `Cargo.lock`
  `e1597f3369caa9637fb7e41c6ca5230b0954926610bc661933123a446ba17836`
- Scoped dirty paths: BattleCore build/sim/state, world active battle/combat/checkpoint, player actor handlers/state와
  active-run store are pre-existing user changes; no clean/reset/stash performed
- Current code/data/consumer locations: `battle/core/{mod,build,sim}.rs`, `battle/scenario.rs`,
  `world/{state,combat,node_flow,snapshot}.rs`, `player_game_actor/{mod,handlers,state}.rs`, `active_run_store.rs`
- Dirty isolation evidence: P2G-20-recorded hashes match current reviewed files where recorded; additional current code was
  directly inspected and retains the selected implementation delta
- Initial full baseline: `passed / game_core 1,080 + game_server 72 / 0 failed`
- Baseline target: `/tmp/cwbs_goal`; relevant production/test/shared-boundary mutations invalidate only their impacted seams

## GEX-10 Implementation Result

- Shared foundation: `battle/core/spawn_kernel.rs`와 `BattleCore::spawn_kernel`
- Session boundary: `GameCore::auto_pause_active_battle_for_disconnect`, actor attach/detach no-TTL handling
- Added focused tests: Kernel 7, Core disconnect 1, server active-battle TTL/reconnect 1
- Reused focused regressions: server nonbattle TTL 1, checkpoint exact schema 1, explicit load usage/state 1
- Focused final result: `12 passed / 0 failed`
- Current implementation hashes:
  - `battle/core/mod.rs`: `d1749eb237d38594c326bd4475ce17cd642aafdcd9dad9b9ca9de11dec0284b3`
  - `battle/core/spawn_kernel.rs`: `15b0cbd0d0b6642d161a5dca055b9fca8488741a6944bb3b9fa5de8479a8f576`
  - `world/combat.rs`: `d65a5f26f9011f88bbce71b6321623de5b60ba6eb58ecaae210bf333203d1de5`
  - `world/tests/combat.rs`: `19ea3f4ec6793a9db0f868bdc2d3c7ebe44e36fd715bfb3ab7ee0d4939df9e3e`
  - `player_game_actor/handlers.rs`: `97ebe7f2ca8d5fee19faec12672ea889227b8a153a893a63dad95983e148d25b`
- GEX-20 pass 1 result: `Not Complete`; exact findings and return owner are preserved in `COMPLETION_AUDIT.md`.
- Correction target: Kernel-controlled one-ID-per-placement commit, full canonical re-bind identity, candidate identity conflict.
- Correction result: all three findings corrected; Kernel focused suite now `9 passed / 0 failed`; unaffected P01 evidence
  remains reusable. Production mutation is frozen for GEX-20 pass 2.
- GEX-20 pass 2 result: `Not Complete`; typed occupant kind must be separated from authoring wildcard and
  `SameFaction` must accept the summoner.
- Typed occupancy correction result: runtime `SpawnOccupantKind` is separate from allow-list values,
  `SameFaction` includes `Summoner`, and Kernel focused suite is `10 passed / 0 failed`. Target is frozen for pass 3.
- Completion Audit: `COMPLETION_AUDIT.md / Audit Pass 3 / Complete / High`; P02 dependency eligible.
