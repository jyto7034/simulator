# P01 Runtime Foundation Completion Audit

## Audit Pass 1 — 2026-08-16

- Audit target: `GOAL-CWBS-P01-RUNTIME-FOUNDATION` /
  `PHASE-CWBS-01-RUNTIME-FOUNDATION` / `AG-CWBS-01-RUNTIME-FOUNDATION`
- Upstream: `HA-CWBS-P2G60-002 -> GS-CWBS-P2G50-002`, accepted composite basis recorded in `PLAN.md`
- Pass relation: fresh read-only `GEX-20` after the initial P01 `GEX-10`; no production source/data was changed during
  this audit pass
- Primary verdict: **Not Complete**
- Return owner: P01 correction `GEX-10`, `WP-CWBS-01`

### Audit Target Record and Currentness

| Member | GEX-10 SHA-256 | Audit-start SHA-256 | Result |
| --- | --- | --- | --- |
| `battle/core/mod.rs` | `d5225335e6a296ea12ba9c03e4a6946b1a847a99b3eea1f7d55b8e2b674f37b1` | same | current |
| `battle/core/spawn_kernel.rs` | `caed1c9abdfed9f53a7fc73b1c7d0b4344cf0b823434b3f9410f553f77a3117c` | same | current |
| `world/combat.rs` | `d65a5f26f9011f88bbce71b6321623de5b60ba6eb58ecaae210bf333203d1de5` | same | current |
| `world/tests/combat.rs` | `19ea3f4ec6793a9db0f868bdc2d3c7ebe44e36fd715bfb3ab7ee0d4939df9e3e` | same | current |
| `player_game_actor/handlers.rs` | `97ebe7f2ca8d5fee19faec12672ea889227b8a153a893a63dad95983e148d25b` | same | current |

`git diff --check` passed for all five tracked target files. The new Kernel file and execution documents are Goal-owned;
the large pre-existing dirty diffs in the tracked files remain user-owned current actual and were not reset, stashed or
attributed to P01.

### Completion Evidence Matrix

| Completion condition | Current implementation and validation evidence | Audit result |
| --- | --- | --- |
| atomic exact-once Kernel and occupancy primitive | typed policy validation, canonical cell order, clone rollback and 7 passing Kernel tests exist | **fail**: the Kernel passes the entire placement slice to a caller closure and accepts any returned `Vec<UnitInstanceId>` as success. A caller can mutate only part of the batch, return `Ok`, and obtain a terminal `Spawned` settlement; one allocation per authored placement and ordered all-or-none commit are not enforced by the Kernel |
| trigger/removal/candidate process-local ledger | trigger and request state live in cloned `BattleCore`; removal-first cancellation and restore replay test pass | **fail**: `bind_trigger` returns `Ok(())` for an already-bound identical request ID before comparing the full request, so a changed payload under that ID is silently accepted at bind time. Candidate duplicate IDs with changed payload are also treated as an idempotent no-op rather than an identity conflict |
| auto-Pause/no TTL/explicit Resume | Core revisioned auto-Pause test, active-battle detach/reconnect test and unchanged nonbattle TTL test all pass | pass |
| process death/explicit load use | checkpoint exact-field test and successful explicit-load test pass; initial full baseline covers unchanged failure/continue/store paths; checkpoint/store source was not modified | pass |
| valid intermediate state | old consumers compile and are intentionally unchanged; dormant Kernel/candidate seam exists | **fail** because the dormant seam does not yet enforce its own batch and identity invariants |

### Boundary Review

- Runtime/state: active Kernel and candidate state is process-local in `BattleCore`; disconnect state is process-local in the
  actor/Core. No durable battle state was introduced.
- Data/schema/store: no RON, checkpoint DTO, `RunCheckpointPayload`, active-run store or profile-store schema changed.
- Transport: reconnect emits immutable setup plus current paused update; explicit Resume uses existing battle authority.
- Consumers/legacy: old Wave and compendium consumers remain active as required. Concrete Wave/Summon cutover remains P02/P03.
- Compatibility: adding the dormant field does not change serialized SavePoint shape; focused compilation and checkpoint
  field-set evidence are non-zero and current.
- Omitted/inaccessible evidence: no external/deployment evidence is applicable. Reviewer designation is not independently
  required by the Goal or risk record; the audit is separated by pass and artifact as permitted by the pipeline.

### Exact Findings and Next Step

1. Replace the unrestricted batch closure with a Kernel-controlled deterministic per-placement commit contract, collecting
   exactly one unit ID per placement and rolling back the entire BattleCore on the first error.
2. Make trigger re-bind compare the complete canonical request before returning idempotent success.
3. Make compendium candidate idempotence accept only the same ID plus same payload; reject conflicting identity reuse.
4. Add focused regression tests for partial commit impossibility and both identity conflicts, rerun impacted evidence, freeze a
   new Audit Target Record, then perform a new fresh GEX-20 pass without deleting this verdict.

P02 is not eligible while this verdict is current.

## Audit Pass 2 — 2026-08-16

- Predecessor verdict: Audit Pass 1 `Not Complete`
- Audit target: same P01 Goal/Phase after correction GEX-10
- Pass relation: fresh read-only GEX-20; no production source/data changed during this audit
- Primary verdict: **Not Complete**
- Return owner: P01 correction `GEX-10`, `WP-CWBS-01`

### Audit Target Record and Currentness

Audit Pass 1's four unchanged members retained the same hashes. `battle/core/spawn_kernel.rs` changed only by the recorded
correction and was frozen at
`8e910be16c397d002ce05a91fc7ea519b0d39a5827c853f697f9c31a710aee31`. Audit-start comparison matched all five current
hashes in `PLAN.md`; `git diff --check` passed.

### Pass-1 Finding Closure

| Prior finding | Current direct evidence | Result |
| --- | --- | --- |
| unrestricted batch commit | Kernel-owned per-placement loop, distinct returned IDs, whole-Core rollback; earlier-placement and duplicate-ID tests | closed |
| changed-payload trigger re-bind | canonical request is reserved/compared before idempotent success; focused test | closed |
| changed-payload candidate reuse | complete candidate payload comparison and typed identity conflict; focused test | closed |

### New Completion Finding

The explicit SP-02 rule says `SameFaction` includes the summoner. Current `SpawnPlacement.occupants` uses the same
`SpawnOccupancyClass` enum as the authoring allow-list and `allows` performs only exact membership. Therefore an occupant
classified as `Summoner` is rejected by `AllowOccupied([SameFaction])`, contradicting the policy. The shared enum also permits
`Any` as an actual occupant kind even though `Any` is an allow-list wildcard rather than a runtime occupant classification.

This is a typed occupancy primitive correctness gap, so the atomic occupancy completion condition and valid intermediate-state
condition fail. Auto-Pause/session, checkpoint/load, identity and batch corrections remain passing and current.

### Required Correction

1. Separate runtime occupant kinds from allow-list values so runtime occupants cannot be `Any`.
2. Make the `SameFaction` allowance accept both ordinary same-faction occupants and the summoner.
3. Add focused positive/negative coverage, rerun the Kernel seam, freeze a new target, then perform fresh GEX-20 pass 3.

P02 remains ineligible.

## Audit Pass 3 — 2026-08-16

- Predecessor verdicts: Audit Pass 1 `Not Complete`, Audit Pass 2 `Not Complete`
- Audit target: `GOAL-CWBS-P01-RUNTIME-FOUNDATION` /
  `PHASE-CWBS-01-RUNTIME-FOUNDATION` at the typed-occupancy correction target
- Pass relation: fresh read-only GEX-20; no production source/data changed during this audit
- Primary verdict: **Complete / High**

### Audit Target Record and Currentness

| Member | Audit target SHA-256 | Audit-start comparison |
| --- | --- | --- |
| `battle/core/mod.rs` | `d1749eb237d38594c326bd4475ce17cd642aafdcd9dad9b9ca9de11dec0284b3` | current / exact |
| `battle/core/spawn_kernel.rs` | `15b0cbd0d0b6642d161a5dca055b9fca8488741a6944bb3b9fa5de8479a8f576` | current / exact |
| `world/combat.rs` | `d65a5f26f9011f88bbce71b6321623de5b60ba6eb58ecaae210bf333203d1de5` | current / exact |
| `world/tests/combat.rs` | `19ea3f4ec6793a9db0f868bdc2d3c7ebe44e36fd715bfb3ab7ee0d4939df9e3e` | current / exact |
| `player_game_actor/handlers.rs` | `97ebe7f2ca8d5fee19faec12672ea889227b8a153a893a63dad95983e148d25b` | current / exact |

`git diff --check` passed. Audit-document writes are outside the production Audit Target Record. Pre-existing unrelated and
overlapping user changes remain preserved and are not claimed as P01 output.

### Completion Evidence Matrix

| Completion condition | Current implementation evidence | Current validation evidence | Result |
| --- | --- | --- | --- |
| atomic deterministic exact-once Kernel/occupancy | canonical request and placement order; all-cell preflight; typed allow-list/runtime occupant split; SameFaction includes summoner; Kernel-owned one-distinct-ID-per-placement commit; clone rollback; cached terminal settlement | Kernel suite 10/10, including all-occupant rejection, 128-cell no-cap order, later-placement rollback, duplicate ID and identity conflicts | pass |
| trigger/removal/candidate process-local ledger | register/bind/settle state and request/candidate ledgers are fields of cloned `BattleCore`; canonical full identity comparison; removal can settle cancellation before allocation | restore/removal replay, trigger re-bind and candidate ordered/idempotent/conflict tests | pass |
| auto-Pause/no TTL/explicit Resume | host-only revisioned pause; actor detach stops tick and skips TTL for active battle; attach emits current paused snapshot; existing authority-gated Resume only | Core disconnect test 1/1, active-battle detach/reconnect test 1/1, unchanged nonbattle TTL test 1/1 | pass |
| process death/explicit load use | checkpoint capture has no active battle field; restore explicitly clears `active_battle`; load validates and restores transactionally before assigning incremented usage | checkpoint exact shape 1/1, explicit successful load 1/1, initial full baseline covers unchanged failed-load/continue/store paths | pass |
| valid intermediate state | old Wave/compendium consumers remain unchanged; shared Kernel and dormant process-local candidate seam compile independently; no checkpoint/data migration | focused compilation is non-zero; initial full baseline plus direct non-impact comparison; final full rerun remains assigned to P06 | pass |

Focused P01 result at this target is `15 passed / 0 failed` (Kernel 10, Core disconnect 1, server active/no-TTL 1,
server nonbattle TTL 1, checkpoint shape 1, explicit load 1). Initial pre-mutation full baseline remains
`game_core 1,080 + game_server 72 / 0 failed`; its invalidated P01 seams were rebaselined above, and the Goal contract reserves
the next integrated full run for P06.

### Boundary and Evidence Review

- Runtime/state: shared authority and ledgers are process-local; spawn provenance distinguishes Wave/Summon without
  source-specific transaction behavior.
- Data/schema/store: no live RON, DTO, SavePoint payload or store schema changed; process restart reconstructs only the last
  durable checkpoint and clears active battle.
- Transport/session: paused reconnect projection and explicit Resume use the existing battle UUID/revision contract.
- Consumer/legacy/compatibility: concrete consumers intentionally remain old until P02/P03; no fallback was added and the
  dormant seam is independently typed/tested.
- Omitted/inaccessible/unknown: no completion-blocking evidence is omitted. P06 owns cross-consumer and repository-full final
  validation; deployment, Unity and production content are non-applicable P01 exclusions.
- Reviewer: the Goal does not require a separately named independent reviewer; this audit was separated from implementation
  and both corrections by read-only passes and immutable predecessor verdicts as permitted by the pipeline.

### Result and Newly Eligible Dependency

P01 is `Complete / High` at this exact Audit Target Record. `GOAL-CWBS-P02-WAVE-CUTOVER` is now dependency-eligible for its
own GEX-00. P01 does not claim concrete Wave/Summon, terminal outcome or permanent compendium completion.
