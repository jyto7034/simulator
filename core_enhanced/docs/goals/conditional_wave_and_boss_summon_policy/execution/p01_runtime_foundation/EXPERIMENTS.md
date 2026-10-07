# P01 Runtime Foundation Experiments

## 2026-08-16 — GEX-00 intake

- Exact selected member: `GS-CWBS-P2G50-002 / GOAL-CWBS-P01-RUNTIME-FOUNDATION`.
- Phase/node: `PHASE-CWBS-01-RUNTIME-FOUNDATION / AG-CWBS-01-RUNTIME-FOUNDATION`.
- Direct predecessor: none.
- Applicable integration acceptance: generic `IVA-CWBS-01`, `IVA-CWBS-08`, Phase-local `IVA-CWBS-09`.
- Upstream structural validation: P2G-40 and P2G-50 validators passed with zero failures.
- Existing exact execution document set: none; new document set initialized.
- Current tree, dirty baseline and code locations: pending.
- Initial `game_core`/`game_server` full baseline: pending and must precede production mutation.
- Production mutation: none.

## 2026-08-16 — current tree and initial baseline

- HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Rust/Cargo: `1.91.1`; `Cargo.lock` SHA-256:
  `e1597f3369caa9637fb7e41c6ca5230b0954926610bc661933123a446ba17836`.
- Scoped files are pre-existing dirty current actual. Representative frozen pre-mutation hashes:
  - `battle/core/build.rs`: `1125b384...`
  - `battle/core/sim.rs`: `af5130fc...`
  - `battle/scenario.rs`: `a33d434b...`
  - `world/state.rs`: `d012e043...`
  - `world/combat.rs`: `23fa46f0...`
  - `world/node_flow.rs`: `2a1cef20...`
  - `player_game_actor/mod.rs`: `b24dd94e...`
  - `player_game_actor/handlers.rs`: `24f796d0...`
  - `active_run_store.rs`: `f43c3fc5...`
- `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_core`:
  passed; unit/integration/doc test binaries collectively ran 1,080 tests, 0 failed.
- `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_server`:
  passed; 72 tests, 0 failed.
- GEX-00 conclusion: exact target, authorization, dependency, current tree, dirty isolation and runner are usable; GEX-10 begins.

## 2026-08-16 — GEX-10 implementation and focused validation

- Added a faction-neutral shared Spawn Kernel with typed Wave/Summon provenance, stable request IDs, canonical placement
  order, whole-batch preflight, value-bearing occupancy policy, clone-backed rollback and terminal exact-once settlement.
- Added trigger `register -> bind -> settle`, summoner-removal-first cancellation, process-local ordered/idempotent compendium
  candidate ledger. Existing Wave/compendium consumers were not cut over in P01.
- Added host-authoritative disconnect auto-Pause. Active-battle detach no longer schedules actor TTL; reconnect projects the
  paused current snapshot, and only an explicit revision-authorized Resume restarts playback. Nonbattle TTL is unchanged.
- No checkpoint/store schema was changed; active battle and Kernel ledgers remain outside `RunCheckpointPayload`.
- Iteration evidence:
  - first Kernel run exposed noncanonical retry identity after placement sorting; request identity was canonicalized at all
    reserve/bind/cancel boundaries.
  - rollback assertion was narrowed to battle mutation state because the terminal exact-once ledger is intentionally retained.
  - no production partial mutation from these failed test iterations remained.
- `cargo test -p game_core spawn_kernel`: 7 passed, 0 failed.
- `cargo test -p game_core disconnect_auto_pause_is_idempotent_and_requires_explicit_resume`: 1 passed, 0 failed.
- `cargo test -p game_server detached_live_battle_has_no_ttl_and_reconnects_paused`: 1 passed, 0 failed.
- `cargo test -p game_server detached_actor_stops_after_ttl_and_deregisters`: 1 passed, 0 failed.
- `cargo test -p game_core current_v4_checkpoint_document_has_exact_flat_semantic_field_set_and_values`: 1 passed,
  0 failed.
- `cargo test -p game_core load_run_checkpoint_restores_visible_state_and_anchor_but_not_load_count_or_peak`: 1 passed,
  0 failed.
- `git diff --check` on the five modified tracked P01 files: passed.
- GEX-10 result: implementation complete for P01; fresh read-only GEX-20 begins.

## 2026-08-16 — GEX-20 pass 1

- Audit Target Record matched all five GEX-10 hashes and `git diff --check` remained clean.
- Verdict: `Not Complete`.
- Findings: unrestricted batch closure did not enforce one commit per placement; bound same-ID changed request and duplicate
  candidate ID changed payload could bypass exact identity comparison.
- No production mutation occurred during the audit. Active goal-run returned P01 to an in-scope correction GEX-10.

## 2026-08-16 — correction GEX-10

- Replaced the unrestricted whole-batch closure with a Kernel-controlled canonical per-placement `FnMut` loop. Exactly one
  distinct `UnitInstanceId` is required per placement; any later error or duplicate identity restores the pre-batch Core.
- An already-bound trigger now reserves/compares the complete canonical request before idempotent success.
- Candidate duplicate identity now returns idempotent false only for the same complete payload and reports an identity conflict
  for a changed payload.
- Added regressions for rollback after an earlier successful placement, duplicate returned unit identity, trigger changed-payload
  re-bind and candidate changed-payload reuse.
- `cargo test -p game_core spawn_kernel`: 9 passed, 0 failed.
- `git diff --check` on the P01 target files: passed.
- Only `battle/core/spawn_kernel.rs` changed from audit pass 1; other focused P01 evidence remains current and reusable.
- Correction GEX-10 result: complete; new target hash frozen in `PLAN.md`; fresh GEX-20 pass 2 begins.

## 2026-08-16 — GEX-20 pass 2

- All three pass-1 findings were directly verified as closed.
- Verdict: `Not Complete` because the shared occupancy enum made `SameFaction` fail to include a `Summoner` occupant and
  allowed the authoring wildcard `Any` to appear as a runtime occupant kind.
- No production mutation occurred during the audit. P01 returned to correction GEX-10; P02 remains ineligible.

## 2026-08-16 — typed occupancy correction GEX-10

- Split actual runtime occupant kinds from policy allowance classes; actual occupants cannot carry wildcard `Any`.
- `AllowOccupied([SameFaction])` now accepts a summoner occupant, while `AllowOccupied([Summoner])` remains narrow and does
  not accept other same-faction occupants.
- Added focused positive/negative coverage.
- `cargo test -p game_core spawn_kernel`: 10 passed, 0 failed.
- `git diff --check` on the P01 target files: passed.
- Correction result: complete; `battle/core/mod.rs` re-export and Kernel hashes frozen in `PLAN.md`; GEX-20 pass 3 begins.

## 2026-08-16 — GEX-20 pass 3

- Audit target matched all frozen hashes and `git diff --check` passed.
- Both predecessor finding sets were directly rechecked as closed.
- Verdict: `Complete / High`; focused P01 evidence `15 passed / 0 failed`; P02 became eligible.
