# Compendium Transaction Rollback Integrity Correction Goal

Status: complete

Implementation complete: true

Goal name: `run_policy_correction_phase_00_compendium_transaction_rollback_integrity`

Master Goal: `run_persistence_facility_monolith_policy_correction_master`

Master Phase: P00

Created: 2026-07-28

Execution authority: 2026-07-28 사용자 요청 및 correction master 승인 정책

Direct predecessor: 없음

Completion review: `COMPLETION_REVIEW.md` — Complete / High (2026-07-28)

## Objective

공통 non-battle transaction과 active-run publish rollback이 이미 commit된 local encounter compendium
knowledge를 지우지 않게 하고, game_server가 AttachSession snapshot을 만들기 전에 durable singleton
profile을 Core에 합쳐 실패 직후 reconnect도 같은 monotonic union을 투영하게 한다.

## Execution Sources

- Master registry, P00 범위와 승인 정책:
  `../run_persistence_facility_monolith_policy_correction_master/PLAN.md` 1~12, 18~23절
- Master 실행 기록:
  `../run_persistence_facility_monolith_policy_correction_master/EXPERIMENTS.md`,
  `../run_persistence_facility_monolith_policy_correction_master/EXPERIMENT_NOTES.md`
- 재감사 finding과 승인 기록:
  `../../../tmp/run_persistence_facility_monolith_reaudit_2026-07-28.md`의 F-01, R-01
- 직접 관련 기존 구현 기록:
  `../run_policy_phase_02_local_compendium_persistence/PLAN.md`,
  `../run_policy_phase_02_local_compendium_persistence/EXPERIMENTS.md`,
  `../run_policy_phase_02_local_compendium_persistence/EXPERIMENT_NOTES.md`,
  `../run_policy_phase_02_local_compendium_persistence/COMPLETION_REVIEW.md`
- 공통 transaction predecessor 기록:
  `../run_policy_phase_00_savepoint_trauma_vertical_convergence/COMPLETION_REVIEW.md`,
  `../run_policy_phase_01_failure_checkpoint_active_run_persistence/COMPLETION_REVIEW.md`
- Governance/canonical:
  `../../README.md`, `../../engineering_workflow.md`, `../../goal_completion_review_guide.md`,
  `../../game_rulebook.md`, `../../core_runtime_contract.md`, `../../data_loading_contract.md`
- Unity-facing canonical/consumer는 snapshot shape 변경 여부가 확인될 때만 직접 검사한다:
  `/mnt/f/unity projects/ark/docs/unity_core_contract.md`,
  `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

## Approved Target And Invariants

- Local compendium은 active-run/checkpoint rollback 밖의 local-install monotonic set union이다.
- 실패한 transaction은 run-local mutation을 rollback하지만 transaction 전 profile과 transaction 중
  이미 commit된 discovery/defeat knowledge를 모두 보존한다.
- rollback 전후 profile 중 하나를 선택하지 않고 Core compendium domain의 idempotent union을 사용한다.
- active-run publish 실패도 run state만 되돌리고 profile 진행을 보존한다.
- AttachSession은 root snapshot을 만들기 전에 durable singleton profile을 Core에 merge한다.
- Profile file schema, current/backup/checksum/dirty/fatal/reset UX와 singleton identity는 바꾸지 않는다.
- 새 compatibility, alias, fallback, profile copy 또는 별도 profile authority를 추가하지 않는다.

## In Scope

- `GameCoreState` non-battle transaction snapshot/restore와 compendium lifetime
- 공통 `with_state_transaction()` 실패 rollback의 profile union
- active-run publish failure rollback과 공통 rollback의 의미 정렬
- game_server AttachSession-before-snapshot durable profile sync와 ordering
- non-empty profile을 사용한 failed transaction, active-run publish failure와 same-actor reconnect 회귀 test
- 관련 runtime/server/canonical 기록과 focused/broad validation

## Out Of Scope

- compendium discovery/defeat producer 규칙 재설계
- profile envelope/file schema, current/backup/lock/checksum/dirty/fatal/reset UX 변경
- account, cloud, multi-profile 또는 transport-player namespace
- certified power, Floor anchor, monolith stage/reward와 Unity presentation 변경
- hidden boss/P06 콘텐츠와 범위 밖 dirty worktree 정리

## Dirty Worktree Gate

- 2026-07-28 child 초기화 직전 workspace와 Unity repository 모두 광범위한 tracked/untracked 기존
  변경이 있었다.
- Core `world/state.rs`, `world/node_flow.rs`, `world/active_run.rs`, compendium domain과 game_server actor/
  persistence 경로는 기존 campaign 변경과 겹칠 가능성이 높다.
- Production 수정 전 P00 대상 경로의 scoped status와 diff를 읽고 현재 의미를 기록한다.
- 기존 변경은 사용자 변경으로 보존한다. 안전하게 병합할 수 없는 overlap이면 구현을 중단한다.
- reset, restore, stash, clean, repository-wide format과 관련 없는 생성물 정리를 하지 않는다.

## Implementation Plan

1. P00 대상 Core/server/canonical의 현재 runtime mutation, profile union API, AttachSession ordering과 scoped
   dirty diff를 조사한다.
2. Production 변경 전 focused Core/server baseline을 실행하고 실제 pass/fail/skip과 기존 failure를 기록한다.
3. 공통 rollback이 transaction 전후 compendium union을 보존하도록 가장 가까운 Core authority에서
   수정하고 active-run rollback과 중복 authority를 만들지 않는다.
4. AttachSession snapshot 전 durable singleton profile merge를 server actor lifecycle에 연결한다.
5. failed ordinary transaction, active-run publish failure, 실패 직후 snapshot/reconnect와 profile file
   status 회귀를 직접 검증한다.
6. 관련 canonical/Goal evidence를 갱신하고 focused에서 broad 순서로 검증한다.
7. 독립 completion review가 `Complete / High`일 때만 Master P01을 활성화한다.

## Test Requirements

- non-empty discovered/defeated profile에서 의도적으로 실패하는 일반 non-battle transaction의 union 보존
- transaction 중 추가된 knowledge와 transaction 전 knowledge의 동시 보존
- active-run publish failure가 run state만 rollback하고 profile union을 보존
- AttachSession root snapshot 전 durable profile merge와 same-actor reconnect projection
- profile publish dirty/fatal/reset, current/backup와 singleton scope의 기존 회귀 없음
- Core/server serialized snapshot shape가 바뀌지 않았는지 확인
- `cargo test -p game_core --lib`의 관련 focused filter와 broad
- `cargo test -p game_server`의 관련 actor/persistence focused filter와 broad
- 영향 범위가 요구하면 `cargo test --workspace --all-targets`
- scoped legacy search, `git diff --check`와 Goal/canonical link/status 확인

각 실행은 정확한 command/runner, 실제 test 수, pass/fail/skip, baseline failure와 새 회귀를 기록한다.

## Completion Conditions

- Master P00 사용자-visible exit gate를 모두 충족한다.
- 공통 transaction, active-run rollback과 AttachSession이 같은 monotonic profile lifetime을 사용한다.
- failed command와 직후 reconnect에서 discovered/defeated union이 감소하지 않는다.
- profile persistence의 기존 failure/dirty/reset UX와 singleton/file schema가 바뀌지 않는다.
- focused Core/server와 영향 범위 broad 검증이 현재 worktree에서 통과한다.
- 승인되지 않은 compatibility/fallback/competing profile copy가 없다.
- 독립 completion review verdict가 `Complete / High`다.

## Stop Conditions

- 승인 정책으로 답할 수 없는 profile identity/lifetime, DTO, save 또는 compatibility 의미가 필요함
- P00 대상 기존 dirty 변경과 안전하게 병합할 수 없음
- 필수 filesystem/server actor 검증을 실행할 수 없음
- P01~P03 source-of-truth를 먼저 바꿔야만 P00을 성립시킬 수 있는 새 dependency가 확인됨
- baseline 또는 변경 후 검증이 없어 회귀를 구분할 수 없음

정책 결정이 필요하면 `policy_decision_required`, 필수 외부 요소가 없으면 `externally_blocked`로 바꾸고
영향받는 production 수정을 중단한다.
