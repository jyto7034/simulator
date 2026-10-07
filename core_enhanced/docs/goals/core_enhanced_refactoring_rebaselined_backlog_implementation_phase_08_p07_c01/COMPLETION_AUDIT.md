# Phase 08 CRA-P07-C01 Completion Audit

Audit Mode: phase_exit
Audit Target: Phase 08 — CRA-P07-C01 Derive Tactical Tile Effects Once From Full Final State
Current Status At Audit Entry: active
Implementation Complete Claim At Audit Entry: false
Policy Source: no candidate-specific policy ID; existing tactical final-state contract plus cross-cutting
CRA-POL-006 compatibility boundary
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Repository/Commit/Dirty State: `/mnt/f/work/simulator` at
`f318414fb0156a67e04c5479000c2a2683fc0c09`; 408 expanded entries at audit entry, with the four production Core
targets already untracked before Phase mutation. Audit artifact creation produced 409 exit entries with status
SHA-256 `6bcff381a8e54bc9464907256fc3b4f7211676681988dacd304729f2599031cb`.

## Completion Evidence Matrix

| ID | 완료 조건 또는 불변식 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| C-01 | Focus/block/Rest producer와 consumer 및 final-state seam이 식별된다 | Child objective/P08-C01 | stance, position, field-work, Rest event, snapshot과 world DTO의 complete inventory | tactical 10/10, field-work 9/9, static caller/write audit | pass |
| C-02 | accepted Any worker의 교차 타일 relocation/swap이 최종 aggregate만 갖는다 | existing Any profile contract | validator가 Any에 양쪽 profile을 요구하고 moved units가 final occupancy 뒤 한 번 reconcile된다 | pre-fix 0/3 red → post-fix 3/3 green | pass |
| C-03 | FieldWorking/Ready와 Attack/Defense의 Focus/block 의미가 exact하다 | preserved invariants 1~4 | `DerivedTacticalEffects::from_state`가 full state의 닫힌 표를 소유하고 snapshot/transition이 공유한다 | work remain/exit, stance와 field completion matrix | pass |
| C-04 | Rest epoch/deadline/cadence와 stale event 의미가 exact하다 | preserved invariant 5 | old/final Rest membership 변화만 epoch/deadline을 바꾸고 exact epoch/deadline guard가 stale event를 no-op한다 | pre-fix epoch 3≠2 red → 1/1 green; stance 10/10 | pass |
| C-05 | unrelated modifier, cost/cooldown/progress/revision과 failure 의미가 보존된다 | preserved invariants 6~7 | reconciliation은 다른 modifier field를 건드리지 않고 relocation/swap/world sequencing은 유지한다 | typed modifier assertions, live tactical 2/2, field-work world 1/1 | pass |
| C-06 | live data와 DTO/wire/checkpoint/public shape가 바뀌지 않는다 | CRA-POL-006 | existing Any schema/validator, unchanged tactical DTO projection and active-battle lifetime | live RON 63/63, server all-target compile, full Core 948/948 | pass |
| C-07 | canonical과 dirty boundary가 실제 authority와 일치한다 | implementation strategy/P08-C02 | nearest runtime canonical documents one final reconciliation and moved-worker behavior; paths/status preserved | format/diff/static/hash audit | pass |
| C-08 | focused/broad validation과 Phase exit review가 통과한다 | P08-C07/P08-C08 | current dirty tree에서 producer, projection, data와 owning package를 재검증했다 | all focused pass, Core 948/948, verdict below | pass |

## Runtime And State Evidence

- Accepted tactical profile table is closed and explicit:
  - any `FieldWorking` state derives no tile effect, zero Focus contribution and block zero;
  - Ready+Attack Ground derives no tile effect and base block, while Ready+Attack Platform has block zero;
  - Ready+Defense Ground derives one `DefenseFocus`, authored Focus reduction and base+bonus block;
  - Ready+Defense Platform derives one `Rest`, zero Focus and block zero.
- `DerivedTacticalEffects::from_state` is the single derivation used both by `unit_tactical_snapshot` and mutation
  reconciliation. Snapshot no longer maintains a parallel activity/stance/tile interpretation.
- `reconcile_tactical_effects` receives the previously applied derived value after the logical state has changed. It
  changes the general Focus contribution only when old/new contributions differ, installs exact final block, clears
  non-Rest deadline and changes epoch/schedules a full interval only when Rest membership changes.
- `set_engagement_stance` updates stance/cooldown first and then reconciles once. Attack resume scheduling and
  `StanceChanged` publication order remain in the existing handler.
- `apply_tactical_position_update` now owns body, logical tile, facing and relocation cooldown only. Relocation and
  swap capture previous effects, commit all positions, defer the moved IDs while final field-work assignment/activity
  is computed, then reconcile each moved unit once. This covers workers that remain in both assignment sets.
- Ordinary deploy/withdraw/death/completion field-work reconciliation uses the non-deferred wrapper. Activity
  transition writes final `FieldWorking | Ready` plus Defense before invoking the same effect transition.
- `handle_rest_tick` remains the event consumer and still requires exact epoch, Defense+Platform+Ready and exact
  deadline. A queued event from the old epoch produced no HP/resonance/deadline change in the new regression.
- Command preflight, resource/revision atomic publication and world rollback are not redefined here. Existing
  validation/failure paths are unchanged; their common staging correction remains Phase 09 CRA-P07-C02.

## Data Evidence

- `DeploymentAffinity::Any` remains an existing serialized enum. `WeaponCombatProfile::validate_runtime_contract`
  requires both Ground Focus and Platform Rest for Any; the test profile passes the same production-valid contract.
- Current production equipment RON contains GroundOnly/PlatformOnly weapons and no live Any weapon. The correction
  therefore protects an already accepted typed capability without introducing or tuning live content.
- No equipment/run-policy/live RON file was changed by this Phase. Live RON validation executed 63 tests with zero
  failures or ignores.

## DTO/Transport/Persistence Evidence

- `UnitTacticalSnapshot` and `LiveBattleUnitTacticalStateDto` fields, enum tags, nullability and numeric types are
  unchanged. World still maps Core snapshot stance/tile/effect/cooldowns/eligibility/block/Focus/deadline/activity
  directly; no consumer recomputes the aggregate from raw events.
- `rest_epoch` remains private BattleCore scheduler bookkeeping. `next_rest_tick_ms` remains the existing optional
  projection; no new field or compatibility alias was added.
- game_server consumes the same Core battle update/checkpoint DTO. `cargo check --all-targets` confirms current shape
  compatibility but is not used as runtime-behavior proof; Core/world focused tests provide that proof.
- Tactical state is active-battle/checkpoint projection state, not durable run save state. No V3 save field, version,
  dual reader/writer, fallback or migration is needed.

## Consumer Evidence

- confirmed workspace consumers: damage calculation reads the reconciled runtime modifier; block refresh/planner
  reads runtime block capacity; Rest event runtime reads epoch/deadline/final state; world state maps the shared Core
  snapshot; game_server transports that world projection.
- canonical external consumer: Unity consumes the existing live battle tactical DTO/checkpoint boundary described by
  the battle contracts. No external field/tag/order changed.
- inaccessible/unknown consumer: deployed Unity implementation and unknown public Rust consumers were not asserted
  as inspected. CRA-POL-006 is satisfied because public/serialized types were not narrowed or changed.
- explicitly excluded consumer mutation: server/Unity mapping and external canonical edits; neither was necessary.

## Legacy/Compatibility Audit

- Stance, relocation and field-work no longer contain separate Focus arithmetic or Rest entry/exit scheduling.
- Remaining direct writes are lifecycle-specific rather than duplicate transition authorities: deploy establishes
  initial Attack block baseline; `reconcile_tactical_effects` owns effect transitions; `handle_rest_tick` advances an
  already active Rest cadence.
- The deferred field-work entry point has exactly two callers, relocation and swap, and both immediately reconcile
  every deferred moved ID from its captured previous value. The ordinary wrapper defers none.
- No ignored test, debug bypass, fallback, compatibility alias, second schema, placeholder DTO/state or live-data
  special case was introduced.

## Scoped Diff And Dirty Preservation

- Expanded status stayed at 408 entries throughout implementation with unchanged status SHA-256
  `9771d27a9a9864ded7f2b5c8a19fdc24dfa9c504419a39677c67d9fe93d08c07`; production work did not add/delete/move a
  path or change tracked/untracked classification.
- All four source targets were user-owned untracked files at activation. Baseline→final hashes are:
  - `tactical_stance.rs`: `1faca2dbc11d888619a9eb8ecbb1e60ce20fec4694c677da395b4c47f550235f` →
    `d6397d834d058f43b121f424b9289dfbde89f555292acaea23e47884e50ca908`
  - `tactical_relocation.rs`: `ed0103d839393c59c422563cfa0cc0c12d2cb28d9ffef86d413d7425f4faf48d` →
    `f61632ba763c93fb40d30a3f9aec7c6ff79a4726295d0dfbf4251afe5b2d55fb`
  - `tactical_swap.rs`: `2ed2240d4ca76fae27f55d1947975dd691e5e86149a3f017ba30f66dd9264929` →
    `96658a9fe1d98b5d69dffb9094d8c9ddf5fa40a985dda6a82e63002d9a73a7c0`
  - `field_work.rs`: `1bc178e2e4030706cd835b0f02a1c9dc8cc3abe04c7c38e25a00f62b5228a180` →
    `2229921f996084fa43d2aaffabdf4c96e5c1f7873e1af7b7f5367e17552f4a60`
- Applied hunks are limited to shared derivation/reconciliation, moved-unit deferral, four exact tests and their
  helpers. Existing tactical command, movement, field progress and user content outside these hunks was preserved.
- The canonical was pre-dirty; the Phase hunk only updates tactical stance/relocation/swap/field-work ownership.
- No reset, restore, clean, stash, deletion, move, broad formatter or adjacent repository mutation occurred.

## Canonical Alignment

- Current actual behavior: effect is derived from full final activity+stance+tile+profile and old→final applied once.
- Current canonical: Tactical Stance, Relocation, Atomic Swap and Field Work sections now state the same derivation,
  moved-worker deferral, Rest-membership epoch rule and same-effect cadence preservation.
- Approved target: Child objective and historical CRA-P07-C01 target match runtime/canonical. Goal prose is not the
  sole owner of the corrected meaning.
- CRA-POL-006 remains exact because live data and every public/serialized/persistence boundary are unchanged.

## Validation Results

All final commands ran in the current dirty worktree.

| Command | Actual result |
| --- | --- |
| `cargo test --lib game::battle::core::field_work::tests::any_` | 3 passed, 0 failed, 0 ignored, 945 filtered |
| `cargo test --lib cross_kind_relocation_into_work_invalidates_one_stale_rest_epoch` | 1 passed, 0 failed, 0 ignored, 947 filtered |
| `cargo test --lib game::battle::core::tactical_stance::tests` | 10 passed, 0 failed, 0 ignored, 938 filtered |
| `cargo test --lib game::battle::core::field_work::tests` | 9 passed, 0 failed, 0 ignored, 939 filtered |
| `cargo test --lib live_tactical_` | 2 passed, 0 failed, 0 ignored, 946 filtered |
| `cargo test --lib live_defense_stance_command_projects_cooldown_focus_pause_and_atomic_rejections` | 1 passed, 0 failed, 0 ignored, 947 filtered |
| `cargo test --lib live_freischutz_field_work_setup_progress_and_recovery_are_authoritative` | 1 passed, 0 failed, 0 ignored, 947 filtered |
| `cargo test --lib live_ron` | 63 passed, 0 failed, 0 ignored, 885 filtered |
| `cargo test --lib` | 948 passed, 0 failed, 0 ignored, 0 filtered |
| game_server `cargo check --all-targets` | compile pass; runner count not applicable |
| `cargo fmt --check` | pass; no formatter diff |
| repository `git diff --check` | pass; no tracked whitespace error |

## Baseline Failures

- Pre-test baseline was green: tactical 10/10, field-work 5/5, live tactical 2/2, live stance 1/1 and live
  field-work 1/1.
- Intentional pre-fix Any filter executed three tests and failed all three: two paths exposed invalid Rest deadline
  `Some(2500)` while FieldWorking; work exit produced general reduction `27` instead of one-Focus `17`.
- Intentional pre-fix stale Rest test executed one test and failed because epoch reached `3` instead of one
  invalidation to `2`.
- No unrelated baseline failure, ignored proof or post-fix failure remains.

## Cross-Component Findings

- No material finding. Core derivation is shared by mutation and snapshot, and world/server remain consumers rather
  than competing tactical-effect authorities.
- Phase 09 CRA-P07-C02 can now stage/commit tactical world commands against one stable effect transition; Phase 08
  did not pre-implement its cursor/reconciliation failure transaction.
- Movement backend/Boss directive ownership is untouched, so P07-C03/C04 gates remain independent.

## Long-Term Direction Review

| Candidate ID | 작업 유형 | 현재→목표 authority 결과 | 보존 불변식 결과 | 제거·통합 결과 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P07-C01 | correctness correction | tile/activity split mutations → one full-final-state derivation/reconciliation | Attack/Defense, Ready/FieldWorking, Ground/Platform, cadence, modifiers/cost/progress/shape preserved | three manual effect paths consolidated; snapshot shares derivation | red/green 4, tactical 10/10, field 9/9, Core 948/948 | pass |

- Fit: High
- Improvement class: none
- Reason: one private typed aggregate removes duplicated transition rules without adding public state or a parallel
  feature. The explicit deferred-moved-unit seam is bounded to relocation/swap and exists only so field occupancy is
  final before one application.

## Primary Verdict

- Complete
- Confidence: High

## Status Transition

- from: `active`, `Implementation Complete: false`
- to: `complete`, `Implementation Complete: true`
- next Phase activation: allowed; Master may mark Phase 08 `complete — Complete / High`, make Phase 09 ready and
  activate it only after its required-document gate.

## Remaining Risk And Required Action

- Material remaining risk: none within CRA-P07-C01 and CRA-POL-006 scope.
- Current live RON has no Any weapon, but the accepted production validator and Core live-command path are exercised
  by the exact cases; adding/tuning live Any content remains a separate data decision.
- Immediate correction: none.
- Policy question: none.
- Required action: synchronize Master registry/status and open Phase 09's document gate.
