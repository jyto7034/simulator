# Failure, Checkpoint, And Active-Run Persistence Goal

Status: complete

Implementation complete: true

Goal name: `run_policy_phase_01_failure_checkpoint_active_run_persistence`

Master Goal: `run_persistence_facility_monolith_policy_master`

Master Phase: P01

Created: 2026-07-27

Execution authority: 2026-07-27 사용자 요청

Direct predecessor:
- `../run_policy_phase_00_savepoint_trauma_vertical_convergence/COMPLETION_REVIEW.md`
  — Complete / High

## Objective

P00의 final SavePoint/Gate recovery 계약을 재교체하지 않고 출전 불가 recovery-path 판단,
실패 전 사용자 결정, Physical Treatment, checkpoint monotonic metadata와 마지막 완료 SavePoint의
durable active-run continue lifecycle을 Core, host/server, Unity와 canonical에 수직 구현한다.

## Execution Sources

- Master P01 registry/scope/exit gate:
  `../run_persistence_facility_monolith_policy_master/PLAN.md` 9~12, 18~25절
- P00 completion baseline:
  `../run_policy_phase_00_savepoint_trauma_vertical_convergence/COMPLETION_REVIEW.md`
- 승인 target policy:
  `../../decisions/save_point_trauma_recovery_policy_draft_2026-07-25.md` 3, 5.1, 8~11,
  14~15절
- governance:
  `../../README.md`, `../../engineering_workflow.md`,
  `../../goal_completion_review_guide.md`
- durable canonical:
  `../../game_rulebook.md`, `../../core_runtime_contract.md`,
  `../../data_loading_contract.md`
- external Unity canonical:
  `/mnt/f/unity projects/ark/docs/unity_core_contract.md`,
  `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

## Approved Boundary

- P00의 tagged `TraumaRecovery`, SavePoint/Gate 값·대상·preview·process-local transaction은
  선행 계약으로 유지한다.
- P01-A는 deployability와 현재 Floor의 실제 recovery path evaluator, restriction과 command
  matrix를 소유한다.
- P01-B는 모든 gameplay failure 후보의 checkpoint 사전 선택, 명시적 failure confirm,
  Physical Treatment와 Unity blocking UX를 소유한다.
- P01-C는 checkpoint monotonic envelope와 versioned local active-run
  current/backup/checksum/lock/continue/terminal invalidation을 소유한다.
- active-run과 local persistent profile은 다른 저장 aggregate다. P02 profile을 active-run payload에
  복제하지 않는다.
- 승인되지 않은 기존 save compatibility, alias, dual-read/write와 fallback을 추가하지 않는다.

## Out Of Scope

- P02 local encounter compendium persistence
- P03 facility template/assignment/manifestation
- P04 combat power/Floor difficulty
- P05 monolith/remanifestation/hidden boss
- P00 recovery schema나 값의 재설계
- 승인 decision에 없는 cloud/account/server-global save

## Dirty Worktree Gate

Core/server/Unity는 P00과 선행 goal들의 대규모 dirty 변경을 포함한다. P01 대상별 diff를 읽고
기존 변경을 보존한다. 안전하게 병합할 수 없는 overlap은 production 변경 전에 보고한다.

## Implementation Plan

1. 필수 Goal/선행 review/승인 policy/canonical을 읽고 current/target/legacy를 구분한다.
2. Core checkpoint/failure/deployability/consumable, host persistence, server actor/session, Unity
   recovery UX의 실제 producer/consumer와 dirty diff를 감사한다.
3. 현재 focused 및 broad baseline과 host filesystem test 가능성을 기록한다.
4. P01-A recovery path evaluator와 restriction command matrix를 구현·검증한다.
5. P01-B failure-decision state/command, Physical Treatment와 Unity modal을 구현·검증한다.
6. P01-C monotonic checkpoint envelope와 durable active-run lifecycle을 구현·실제 파일 검증한다.
7. DTO/server/Unity/canonical을 수렴시키고 legacy/fallback을 감사한다.
8. focused, live RON, Core/server, host persistence, WebSocket/Unity와 broad validation을 실행한다.
9. completion review가 `Complete / High`일 때만 master P02를 활성화한다.

## Test Requirements

- deployability 및 current-Floor recovery path의 positive/negative/edge case
- failure 후보별 checkpoint 선택/명시적 confirm/command restriction과 exact-once transition
- Treatment eligibility, usefulness 분리, 단일 지속 modifier 교체와 직원 target UX
- successful checkpoint load만 횟수 소비, 실패/재시작 후 monotonic metadata 보존
- current valid, backup recovery, both corrupt, unsupported version, checksum mismatch
- atomic temp/write/fsync/publish failure, single-writer lock과 continue lifecycle
- checkpoint-owned state round-trip 및 local profile 부재
- RunComplete/confirmed RunFailed/AbandonRun/new-run replacement invalidation
- Core DTO/server mapping/WebSocket ordering과 Unity reader/store/modal/presentation
- strict live loading, legacy search, focused/broad Core/server/Unity validation

## Completion Conditions

- Master P01 exit gate를 모두 충족한다.
- P00 recovery 의미가 보존된다.
- 모든 실패/회복/continue 분기가 실제 runtime과 파일 검증으로 보호된다.
- local-install active-run scope가 transport player actor 임시 state와 분리된다.
- canonical과 구현이 일치하고 material compatibility/fallback이 없다.
- completion review verdict가 `Complete / High`다.

## Stop Conditions

- 기존 배포 save 호환이 필요하나 version/migration 정책이 승인되지 않음
- active-run local identity/scope가 승인 policy만으로 결정되지 않음
- persistence root/lock/atomic publish를 실제 파일로 검증할 수 없음
- P02 profile 의미를 P01 payload에 넣어야만 진행 가능한 새 dependency
- 승인되지 않은 gameplay/UX/save/wire 의미 결정이 필요함

정책 결정이 필요하면 Status를 `policy_decision_required`로 바꾸고 영향받는 mutation/migration을
중단한다.
