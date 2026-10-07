# Conditional Wave and Boss Summon — Master Goal Experiments

## 2026-08-16 — GEX-00 root intake

- User authorization: `GEXAUTH-CWBS-001`, Master Goal 전체 `goal-run`.
- Policy directory exact match:
  `core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy`.
- Current selections: `HA-CWBS-P2G60-002`, `GS-CWBS-P2G50-002`, `GOAL-CWBS-MASTER`.
- Upstream observed hashes matched the Handoff for Freeze, Phase Plan, Receipt와 Goal Set.
- `p2g40-receipt-validator/1.0.0`: passed, failures 0.
- `p2g50-goal-set-validator/1.0.0`: passed, failures 0.
- Existing execution document search for `GOAL-CWBS-MASTER`와 P01: none.
- Stable execution layout selected below the policy directory: `execution/master`, `execution/p01_*` … `execution/p06_*`.
- Dependency order selected from current Master/Phase Plan: P01 first; predecessor Phase none.
- Production mutation: none at this point.
- Game test execution: none at this point; P01 pre-mutation baseline pending.

## 2026-08-16 — P01 initial full baseline

- Identity: HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`, dirty current actual, Rust/Cargo `1.91.1`,
  `Cargo.lock` SHA-256 `e1597f3369caa9637fb7e41c6ca5230b0954926610bc661933123a446ba17836`.
- `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_core`: passed, 1,080 tests, 0 failed.
- `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_server`: passed, 72 tests, 0 failed.
- Initial full baseline result: `passed / 1,152 total / 0 failed / reusable until relevant input invalidation`.
- Existing dirty attribution: P01 reviewed paths are user-owned current actual; observed hashes for build/sim/scenario,
  world/session/savepoint and server actor/store paths match the P2G-20 source basis where recorded.
- Additional current BattleCore changes were inspected at the selected state and preserve the relevant P01 premise:
  sequential group insertion remains, generic Spawn Kernel is absent, active battle pause exists but detach auto-Pause/no-TTL is absent.
- GEX-00 result: passed; P01 GEX-10 prerequisites satisfied.

## 2026-08-16 — P01 GEX-10 result

- Shared Kernel/ledger and same-process disconnect lifecycle foundation implemented without cutting over later consumers.
- P01 focused evidence: 12 passed, 0 failed; initial full baseline remains the run baseline and is not repeated here.
- P01 production files are frozen at the hashes recorded in its `PLAN.md`; fresh GEX-20 started.

## 2026-08-16 — P01 GEX-20 pass 1 result

- Verdict: `Not Complete`; P02 remains ineligible.
- The active goal-run returned the three P01-local technical findings to correction GEX-10 without changing upstream scope.

## 2026-08-16 — P01 correction result

- All three audit-pass-1 findings were corrected in the shared Kernel and verified by 9 focused tests.
- Fresh P01 GEX-20 pass 2 started; P02 remains ineligible until its verdict is positive.

## 2026-08-16 — P01 GEX-20 pass 2 result

- Verdict: `Not Complete`; explicit `SameFaction includes summoner` typed occupancy semantics returned to correction GEX-10.

## 2026-08-16 — P01 typed occupancy correction result

- Runtime occupant and allowance types are separated; explicit SameFaction/Summoner semantics pass focused coverage.
- Fresh GEX-20 pass 3 started.

## 2026-08-16 — P01 completion

- GEX-20 Audit Pass 3 verdict: `Complete / High` at the exact P01 target hashes.
- P02 is dependency-eligible and its GEX-00 begins without repeating the initial full baseline.

## 2026-08-16 — P02 completion

- GEX-20 Audit Pass 1 verdict: `Complete / High` at the exact P02 target hashes.
- Strict Wave activation schema, ledger/Kernel consumer, bucket ordering, prospective death/group/deadline/cancel gates and
  authority-free Boss adapter were completed with focused Wave/PVE/Boss/server evidence.
- P03 became dependency-eligible without repeating the run-wide full baseline.

## 2026-08-17 — P03 completion

- GEX-20 Audit Pass 1 verdict: `Complete / High` at the exact P03 hashes recorded in its `EXPERIMENTS.md`.
- Independent summon definition/provenance/counter/batch, typed Boss Summon, deterministic placement/wait, removal/signal
  and authority-free actor behavior paths are complete.
- Final focused evidence includes summon runtime 10/0, Boss data 11/0, live RON 77/0, combat assembly 14/0 and
  Core/server check with zero warnings. The run-wide full baseline remains reserved for P06.
- P04 is dependency-eligible and begins GEX-00.

## 2026-08-17 — P04 completion and P05 selection

- P04 GEX-20 Audit Pass 1 verdict: `Complete / High` at the exact hashes recorded in its `EXPERIMENTS.md`.
- Named authored outcomes, full-bucket deterministic selection, no implicit cap/Draw, exhaustive per-outcome policy,
  transactional consumption and public/server ID/verdict projection are complete.
- Final focused evidence includes terminal group `11/0`, strict named-outcome/schema predicates `3/0`, live PVE
  reader/publication/resolution, result rollback/Retreat/BossOmen seams and Core/server check with zero warnings.
- The run-wide full baseline remains reserved for P06.
- P05 is dependency-eligible and begins GEX-00.

## 2026-08-17 — P05 completion and P06 selection

- P05 GEX-20 Audit Pass 1 verdict: `Complete / High` at the exact hashes recorded in its `EXPERIMENTS.md`.
- Successful opponent spawn candidates now use the P01 process-local ledger, flow through the P04 committed result and
  register atomically only on final Victory.
- Core/server use one `registered_ids` set and profile version 3; old four-set fields, Defeat writer and legacy union
  migration are absent. Focused evidence includes Core schema/lifetime/result seams, server store/reset/no-migration and
  paused reconnect lifecycle with zero focused failures.
- The run-wide full baseline remains reserved for P06.
- P06 is dependency-eligible and begins GEX-00.

## 2026-08-17 — P06 GEX-10 closure

- P06 froze `ATR-CWBS-P06-FINAL-001` across 25 relevant inputs after focused correction of every regression exposed by
  the final integrated commands.
- Final Core `1,131 / 0`, server `73 / 0`, combined `1,204 / 0`; pre/post 25-member SHA-256 manifest matched exactly.
- Production promotion, Unity and deployment remain untouched. P06 begins fresh read-only GEX-20; Master GEX-30 remains
  ineligible until a `Complete / High` P06 verdict.

## 2026-08-17 — P06 completion and Master audit entry

- P06 fresh GEX-20 `AUDIT-CWBS-P06-GEX20-001`: `Complete / High`; target remained current and unchanged.
- P01..P06 now all have current positive Phase audits. The active goal-run enters fresh read-only Master GEX-30 without
  repeating the frozen full suites.

## 2026-08-17 — Master GEX-30 completion

- Froze `ATR-CWBS-MASTER-FINAL-001` from the exact accepted-basis hashes, six Phase-audit evidence hashes, P06 final
  implementation manifest and integrated command/result identity.
- Compared every Phase audit to final actual. P01..P05 have only approved successor-owned deltas with current direct
  continuity evidence; P06 remains `current / unchanged` by exact 25-member comparison.
- Fresh direct review confirmed the cross-Phase spawn/settlement/terminal order, removal-first summon cancellation,
  Victory-only compendium transaction, process-local Pause/SavePoint boundary and typed nonproduction reference exclusion.
- `IVA-CWBS-01..12` all map to current durable evidence. Existing final validation remains Core `1,131 / 0`, server
  `73 / 0`, combined `1,204 / 0`; no relevant input changed, so the full commands were not repeated.
- Master audit `AUDIT-CWBS-MASTER-GEX30-001`: `Complete / High`. Deployment and production promotion remain excluded.
