# Monolith Stage Tuning And Reward Convergence Goal

Status: complete

Implementation complete: true

Goal name: `run_policy_correction_phase_03_monolith_stage_tuning_reward_convergence`

Master Goal: `run_persistence_facility_monolith_policy_correction_master`

Master Phase: P03

Created: 2026-07-28

Direct predecessors:
- P00 compendium transaction rollback — Complete / High
- P01 combat-eligible certified power — Complete / High
- P02 checkpoint-owned Floor anchor — Complete / High

## Objective

Monolith remanifestation stage를 map difficulty의 typed authority로 저장하고 role+stage target/stat/reward를
한 resolver와 one-pass rounding으로 합성해 stage 3의 실제 적 능력치를 stage 2보다 높인다. Unity에는 exact
multiplier 대신 qualitative reward tier만 투영한다.

## Approved Target

- Difficulty가 typed monolith stage 2/3을 저장하고 monolith ledger와 mismatch면 fail closed한다.
- Stage multiplier는 target power와 enemy max-health/attack/defense에 적용하고 generated-wave budget에는
  적용하지 않는다.
- Live 결과는 stage 2 health 165%, attack/defense 145%; stage 3 health 207%, attack/defense 182%다.
- Role+stage percentage는 base stat/원본 reward quantity에 각각 한 번 합성·반올림한다.
- Reward는 nearest rounding/minimum-one을 한 번만 적용한다.
- Unity는 exact reward multiplier를 읽거나 표시하지 않고 stage + qualitative `Enhanced`/`FinalEnhanced`
  tier를 표시한다.
- P02 Core payload V2/envelope V3 checkpoint/continue/rollback에서 typed stage가 보존된다.

## In Scope

- map difficulty typed stage schema, manifestation/conversion과 ledger cross-validation
- combat preview/battle setup stat resolver와 unchanged generated-wave budget
- combat reward one-pass role+stage resolver/rounding
- Core DTO exact percent 제거 및 qualitative tier 추가
- server serialized pass-through, Unity reader/presentation/tests/canonical
- checkpoint/continue/rollback mismatch and strict legacy rejection

## Out Of Scope

- monolith price/use/attempt/residual/exchange 정책
- encounter wave composition/reward pool identity 재밸런싱
- hidden boss/P06 content
- P00-P02 정책 재설계나 과거 save migration
- 범위 밖 dirty 변경 정리

## Dirty Worktree Gate

Core monolith/node/combat/behavior, live RON/canonical과 Unity reader/presentation/tests는 기존 campaign 변경과
겹친다. 수정 전 current source와 scoped diff를 읽고 사용자 변경을 보존한다. reset/restore/stash/clean,
repository-wide format과 무관 Unity asset 변경을 하지 않는다. 안전하게 병합할 수 없으면 중단한다.

## Implementation Plan

1. current monolith conversion, ledger, difficulty resolver, preview stat/wave, reward rounding과 Unity consumer를
   end-to-end 추적한다.
2. stage 2/3 stat equality, double-rounding quantity, exact-percent projection의 focused baseline/defect를 고정한다.
3. typed stage를 difficulty에 추가하고 하나의 role+stage tuning resolver로 target/stat/reward를 합성한다.
4. wave budget은 role tuning만 사용하고 reward는 original quantity one-pass로 계산한다.
5. Core DTO/Unity reader/presentation을 qualitative tier로 breaking 교체하고 legacy field를 제거한다.
6. checkpoint/continue/mismatch, focused Core/RON/server/Unity와 broad 검증 후 독립 completion review를 수행한다.

## Test Requirements

- fixed fixture stage 2/3 exact target/HP/attack/defense와 stage 3 strictly greater
- stage 2/3 generated wave budget/composition equality
- small quantity one-pass rounding edge case와 minimum-one
- ledger/difficulty stage mismatch rejection
- checkpoint/continue/rollback typed stage round-trip
- exact `reward_multiplier_percent` DTO/reader/mock/fixture rejection 또는 부재
- qualitative reward tier Unity read/presentation
- Core/RON/server/Unity focused/broad, workspace all-targets, scoped legacy/diff/status

## Completion Conditions

- Master P03 user-visible exit gate와 모든 test requirement가 current evidence로 입증된다.
- Target/stat/reward가 같은 typed stage authority를 쓰고 wave budget은 stage와 분리된다.
- Exact reward percent/legacy save field와 compatibility fallback이 남지 않는다.
- Completion review가 `Complete / High`다.

## Completion Record

- Completed: 2026-07-30
- Independent review: `COMPLETION_REVIEW.md`
- Verdict: `Complete / High`
- Typed stage authority, one-pass role+stage tuning/reward, qualitative Unity projection,
  checkpoint/continue/rollback preservation을 current source와 focused/broad test로 재검증했다.

## Stop Conditions

- 새 stage 수치, reward rounding 또는 qualitative label 정책이 필요함
- target dirty 변경과 안전하게 병합할 수 없음
- 필수 Unity project/test runner에 접근할 수 없음
- P02 schema를 재설계해야만 typed stage를 보존할 수 있음
