# P03 Summon Cutover Completion Audit

- Goal ID: `GOAL-CWBS-P03-SUMMON-CUTOVER`
- Phase ID: `PHASE-CWBS-03-SUMMON-CUTOVER`
- Audit Pass: `1`
- Audit Date: `2026-08-17`
- Audited Target: hashes in `EXPERIMENTS.md / Current GEX-10 target identities`
- Dependency: `GOAL-CWBS-P02-WAVE-CUTOVER / Complete / High / Audit Pass 1`
- Verdict: `Complete`
- Confidence: `High`
- Correction Required: `none`

## Fresh-pass method

This audit started after GEX-10 implementation and its durable record were complete. It re-read the P03 Goal boundary and
the exact `PER-CWBS-SM-01..04` / `PER-CWBS-BR-01..05` Freeze acceptance, then mapped each requirement to current
production code, live data/compiler behavior and focused test evidence. It did not infer completion from the GEX-10
summary alone and did not repeat the run-wide full baseline.

## Dependency and target integrity

- P02 remains `Complete / High`; the shared Spawn Kernel, Wave settlement and authority-free actor adapter it exposes are
  present and their focused regressions remain green.
- Current production/data hashes exactly match the frozen identities in `EXPERIMENTS.md` after the final signal-binding
  refinement and post-record checks.
- `git diff --check` passes. Existing dirty/untracked paths remain preserved current actual; no reset, restore, stash or
  unrelated cleanup occurred.
- Live `data/summoned_units/base.ron` contains zero definitions, so P03 makes no production content opt-in or rollout
  claim.

## Requirement-by-requirement audit

| Freeze acceptance | Current authoritative evidence | Audit result |
| --- | --- | --- |
| `SM-01` independent domain/provenance, no compendium/reward/authority leakage | Dedicated summoned catalog and `BattleUnitSourceIdentity::SummonedUnit`; runtime success fixture proves definition/summoner/sequence/batch; compendium matches explicitly exclude it; employee result attribution accepts only Employee; summoned actors are outside encounter authority | pass |
| `SM-02` definition stats, encounter scale exactly once, no summoner inheritance | `CompiledSummonedUnitDefinition` owns profile; commit copies it and applies only resolved encounter `BattleUnitStatScale`; 150/200/125 fixture proves 100→150 HP, 12→24 attack and rounded defense without reading summoner | pass |
| `SM-03` positive per-summoner limits and success-only battle-lifetime counters | strict positive definition/count validation; counter key is `(summoner instance, definition)`; limit checks precede placement; counter/UUID continuation advances after Kernel success only; death/despawn frees alive only; repeat/two-summoner fixture proves `alive=0,total=4` independently | pass |
| `SM-04` Persist/Despawn and unresolved cancellation ordering | batch reconciliation precedes request allocation; summoner/state validity precedes limit/space; Persist and Despawn fixture, owner-state-exit pending cancellation and zero UUID/total assertions all pass | pass |
| `BR-01` only typed BossTask Summon with four results | strict raw/compiled Summon variant owns one definition and positive count; possible-result set is exactly Spawned/LimitReached/NoSpawnSpace/Canceled and compiler requires exhaustive mappings; no SkillEffect summon path exists | pass |
| `BR-02` success-only deterministic batch/handle and clear once | deterministic successful UUID continuation, actual Kernel member IDs, pattern-activation-local handle map and exact UUID check; failure fixture keeps continuation zero; clear-once fixture remains unchanged after a second boundary; summons do not gate victory | pass |
| `BR-03` deterministic placement, closed occupancy, atomic Fail/bounded Wait/FIFO | Around uses continuous summoner coordinate to tile-center squared distance then y/x; SpawnZone preserves resolved authored order; every active occupant is classified; Kernel rejects duplicates/unwalkable/disallowed occupants; invalid set matrix, atomic failure, FIFO and equal-time space-first fixtures pass | pass |
| `BR-04` authority-free existing behavior with route and no recursion | summoned actor runtimes reuse existing control/task/query/movement code, have explicit group route, and are appended after `encounter_actor_count`; encounter phase code ignores the suffix; compiler rejects summon behavior graphs containing Summon and missing route context | pass |
| `BR-05` first-cause lifetime/signal, Despawn/Kill and bound signal contract | batch stores first removal cause; summoner removal, signal epoch and lifetime settle once; Despawn produces no death/graveyard, Kill uses normal death with no killer; bind epoch is captured at successful commit; prebind signal is nonretroactive; compiler rejects undeclared, unproduced and encounter-unbound signals | pass |

## Focused validation evidence

- summoned definition schema: `3 passed / 0 failed`
- summon compiler matrix: `4 passed / 0 failed`; full Boss data/compiler: `11 / 0`
- summon runtime integration: `10 / 0`
- Boss runtime: `22 / 0`
- Spawn Kernel: `10 / 0`
- Wave regression: `10 / 0`
- Boss setup: `3 / 0`
- compendium exclusion: `7 / 0`
- live RON publication: final `77 / 0`
- combat assembly: final `14 / 0`
- `cargo check -p game_core -p game_server`: pass, zero warnings

The exact commands, initial stale-fixture failures and corrections are in `EXPERIMENTS.md`. The inherited full baseline is
still `game_core 1,080 + game_server 72 = 1,152 / 0 failed`; per the Master validation contract it is not rerun until P06.

## Scope and forward ownership

- SavePoint remains battle-free because summon state exists only inside the active `BattleCore`; no checkpoint DTO or
  active-run payload was extended.
- Unity/presentation schema and production content promotion are outside P03. The empty opt-in source makes no released
  consumer claim. P06 owns excluded placeholder integration and final canonical/external alignment.
- Named terminal outcome selection is intentionally not implemented here; P04 owns it. P03 preserves the old terminal
  contract and proves summons themselves are not a victory gate.

## Verdict

`GOAL-CWBS-P03-SUMMON-CUTOVER` is complete at High confidence. All nine assigned Freeze acceptances have direct current
code/data evidence and focused executable coverage, no P03 stop condition is active, and no correction Goal is required.
P04 may begin GEX-00 against this exact audited target.
