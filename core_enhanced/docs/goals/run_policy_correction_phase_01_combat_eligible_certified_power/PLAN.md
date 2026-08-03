# Combat-Eligible Certified Power Correction Goal

Status: complete

Implementation complete: true

Goal name: `run_policy_correction_phase_01_combat_eligible_certified_power`

Master Goal: `run_persistence_facility_monolith_policy_correction_master`

Master Phase: P01

Created: 2026-07-28

Direct predecessor:
- `../run_policy_correction_phase_00_compendium_transaction_rollback_integrity/COMPLETION_REVIEW.md`
  — Complete / High

## Objective

Current certified combat power를 실제 combat-eligible 전체 roster의 persistent loadout만 합산하도록
수정하고, relative threat·Floor 1 initialization·combat-entry run peak가 같은 versioned formula를
사용하면서 readiness warning과 historical run peak의 비되감기 의미를 보존한다.

## Execution Sources

- Correction Master `PLAN.md` 7~13, 17~23절과 Master 실행 기록
- P00 correction completion review `Complete / High`
- 재감사 메모 P-01/R-03
- 기존 P04 power implementation Goal의 PLAN/EXPERIMENTS/EXPERIMENT_NOTES/COMPLETION_REVIEW
- `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md`
- live `game_resources/data/run/policy.ron`과 strict `RunPolicyData`

## Approved Target

- Certified roster filter는 `alive && physical_damage <= 4 && trauma < 100`이다.
- Eligible 직원의 current HP/Trauma 값과 Minor/Major damage는 coefficient가 아니라 readiness다.
- Critical/Breakdown/Dead는 current certified total에서 제외하고 warning projection은 유지한다.
- 실제 전장에 배치된 subset을 사후 재계산하지 않고 deploy 가능한 전체 roster를 합산한다.
- current relative threat, Floor 1 initialization과 combat-entry peak update가 같은 formula/version을 쓴다.
- historical run peak는 직원 손실, 상태 악화, checkpoint load/backup recovery로 감소하지 않는다.
- 의미 변경에 따라 power model version을 올리고 구 version/save를 fail closed한다. Migration, alias,
  default compatibility를 추가하지 않는다.

## In Scope

- certified roster eligibility filter와 formula 호출자
- zero-current-power relative threat/readiness 의미
- Floor 1/current threat/combat-entry run peak 일관성
- live power model version과 strict validation/canonical
- focused formula/threat/peak/RON 및 Core/server 영향 broad 검증

## Out Of Scope

- coefficient/band/role 수치 재밸런싱
- potential/capacity power의 두 번째 authority
- checkpoint-owned Floor anchor와 save payload 재설계(P02)
- monolith stage stat/reward(P03), Unity DTO/UX 변경
- 과거 save migration/dual reader와 범위 밖 dirty 정리

## Dirty Worktree Gate

관련 Core power/employee/world/RunPolicy/RON/canonical 파일은 기존 campaign 변경과 겹친다. Production
수정 전 current code와 scoped diff를 읽어 의미를 기록하고 사용자 변경을 보존한다. 안전하게 병합할 수
없으면 중단하며 reset/restore/stash/clean/repository-wide format을 하지 않는다.

## Implementation Plan

1. current certified formula, deployability predicate, all call sites, version/save validation과 scoped diff 조사.
2. eligibility matrix/current threat/run peak/live RON focused baseline 확보.
3. 한 combat-eligibility authority를 certified calculation에 연결하고 version을 갱신.
4. Critical/Breakdown/Dead 제외, eligible HP/trauma 불민감, zero denominator와 peak monotonic 회귀 추가.
5. canonical과 legacy/version rejection을 갱신하고 focused→broad 검증.
6. 독립 completion review `Complete / High` 후 P02 활성화.

## Test Requirements

- alive/Dead 및 physical damage 0/4/5, trauma 0/99/100 eligibility matrix
- eligible HP/Trauma 값과 Minor/Major damage의 coefficient 불민감
- Critical/Breakdown/Dead의 current certified total 제외
- zero current certified power의 `VeryHigh` + `NoDeployableEmployees`
- current threat/Floor 1/combat-entry 동일 formula와 historical peak non-decrease
- live power model version strict validation, old version/save rejection과 compatibility 부재
- Core focused/broad, live RON, server 영향 및 workspace all-targets
- scoped legacy search/diff/status 문서 검증

## Completion Conditions

- Master P01 exit gate와 모든 test requirement가 current evidence로 입증된다.
- formula/version authority가 하나이고 readiness/current power/run peak 의미가 승인 정책과 일치한다.
- old version/schema fallback이 없고 P00 profile lifetime 및 기존 사용자 변경이 보존된다.
- completion review `Complete / High`다.

## Stop Conditions

- 새 coefficient/balance/DTO/save compatibility 결정이 필요함
- 대상 dirty 변경과 안전하게 병합할 수 없음
- current deployability authority를 재사용할 수 없어 gameplay 의미 선택이 필요함
- 필수 live RON/server 검증에 접근할 수 없음
