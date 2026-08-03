# SavePoint And Gate Trauma Vertical Convergence Goal

Status: complete

Implementation complete: true

Goal name: `run_policy_phase_00_savepoint_trauma_vertical_convergence`

Master Goal: `run_persistence_facility_monolith_policy_master`

Master Phase: P00

Created: 2026-07-27

Execution authority: 2026-07-27 사용자 요청

Direct predecessor: 없음

Completion review: `COMPLETION_REVIEW.md` — Complete / High (2026-07-27)

## Objective

중단된 `Support/Rest -> SavePoint` 전환을 임시 flat/percent 혼합 계약으로 고정하지 않고, 승인된
최종 tagged `TraumaRecovery`와 SavePoint/Gate 의미로 Core, live RON, game_server, 외부 Unity와
canonical을 수직 수렴시킨다.

## Execution Sources

- Master registry와 P00 범위:
  `../run_persistence_facility_monolith_policy_master/PLAN.md` 10, 10.1, 11, 18, 19, 21~25절
- Master 현재 evidence와 위험:
  `../run_persistence_facility_monolith_policy_master/EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
- 승인 target policy:
  `../../decisions/save_point_trauma_recovery_policy_draft_2026-07-25.md` 1~6, 10~12, 14~15절
- 문서 governance와 작업 절차:
  `../../README.md`, `../../engineering_workflow.md`, `../../goal_completion_review_guide.md`
- Durable canonical:
  `../../game_rulebook.md`, `../../core_runtime_contract.md`, `../../data_loading_contract.md`
- Unity-facing canonical:
  `/mnt/f/unity projects/ark/docs/unity_core_contract.md`,
  `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

Goal 문서는 현재 runtime이나 durable gameplay/wire source of truth가 아니다. 현재 사실은 runtime,
production live data와 실제 producer/consumer에서 확인하고 migration 목표 의미는 위 승인 decision을
따른다.

## Approved Target

- 정식 node category는 `SavePoint`이며 `Support`, `Rest`, support choice mode를 제거한다.
- SavePoint와 Gate는 공통 tagged recovery type을 사용한다.
- 초기 live 값은 SavePoint `CurrentPercent { percent: 15 }`, Gate
  `CurrentPercent { percent: 10 }`다.
- `CurrentPercent`는 현재 Trauma 기준 올림이며, Flat과 CurrentPercent를 동시에 authoring하지 않는다.
- recovery는 category별 RunPolicy가 소유하고 node/template override를 허용하지 않는다.
- 생존 직원은 Critical/Breakdown을 포함해 대상이며 Dead는 제외한다. Physical Damage는 바꾸지 않는다.
- SavePoint recovery는 psychology/trust/memory/event를 생산하지 않는다.
- SavePoint 완료의 recovery, pending research planning-return 지급, node completion과 process-local
  checkpoint capture는 승인된 최종 상태를 기준으로 partial commit 없이 처리한다.
- Gate recovery, pending research와 다음 Floor transition도 partial commit 없이 처리한다.
- Unity는 Core가 투영한 preview를 표시하며 percent/eligibility를 재계산하지 않는다.
- P00은 durable active-run file, recovery restriction, failure decision modal과 Treatment 콘텐츠를
  구현하지 않는다. 이 범위는 P01이 소유한다.

## In Scope

- Core `TraumaRecovery` schema, validation과 deterministic application
- live `RunPolicy` SavePoint/Gate recovery authoring
- SavePoint/Gate preview DTO, result/snapshot/capability와 server mapping
- SavePoint/Gate completion ordering과 process-local checkpoint capture
- `Support`, `Rest`, `ChooseSupport`, `SupportChoice`, support selected-event/route 제거
- SavePoint psychology/trust producer와 `RecoveredAtSavePoint` producer/projection/test 제거
- 외부 Unity reader/store/router/command/presentation/mock/test 전환
- gameplay/runtime/data/Unity canonical 동기화
- focused, live-loading, server mapping, Unity와 broad validation

## Out Of Scope

- durable active-run current/backup/checksum/lock/continue
- `AwaitingRunFailureDecision`, `ConfirmRunFailure`와 recovery restriction
- Physical Treatment 신규 live item과 최종 콘텐츠 수치
- facility template, node assignment와 SavePoint 배치 주기
- local compendium, combat power와 monolith
- 관련 없는 battle/employee/skill/artifact/debug cleanup
- 승인되지 않은 save compatibility, alias, dual-read/write와 fallback

## Dirty Worktree Gate

P00 대상 Core/RON/server/Unity 파일은 이미 modified 상태다. 수정 전에 대상별 diff를 읽고 현재
사용자 변경을 보존한다. 안전하게 병합할 수 없는 overlap이 있으면 production 변경을 중단하고
충돌 경로와 필요한 사용자 입력을 보고한다.

## Implementation Plan

1. 필수 policy/canonical과 외부 Unity 문서를 끝까지 읽고 target/current/legacy를 구분한다.
2. P00 대상 Core/RON/server/Unity 파일의 scoped dirty diff와 legacy symbol surface를 감사한다.
3. 현재 baseline을 다시 실행하고 master 기록 이후 변경 여부와 실패 수를 기록한다.
4. final tagged `TraumaRecovery`, validation, live data와 runtime application을 구현한다.
5. SavePoint/Gate preview와 completion/checkpoint transaction을 final policy로 수렴시킨다.
6. Support/Rest와 SavePoint psychology legacy를 Core/RON/server/Unity/canonical에서 제거한다.
7. focused -> live loading -> Core/server -> Unity -> broad 순서로 검증한다.
8. 독립 completion review를 수행하고 `Complete / High`일 때만 master P01을 활성화한다.

## Migration Surface

- DTO/wire: Support command/result/selected-event 제거, SavePoint/Gate recovery preview 추가/수렴
- live RON: support flat/percent 혼합 field 제거, tagged category recovery로 교체
- save: durable file migration 없음; process-local checkpoint payload 의미만 final recovery와 정렬
- Unity: SupportChoice/Rest route와 command 제거, Node Exploration preview/presentation 전환

## Test Requirements

- tagged recovery Flat/CurrentPercent parsing, validation, ceil/saturating behavior
- SavePoint 15%, Gate 10%, zero/max Trauma, Critical/Breakdown/Dead 대상 규칙
- recovery/node/Floor/pending/checkpoint atomicity와 정확히 한 번 적용
- SavePoint psychology/trust producer 부재
- strict live RON loading과 legacy schema rejection
- Core DTO/server deserialize/result mapping/snapshot shape
- Unity reader/router/presentation/mock EditMode/PlayMode focused tests
- legacy symbol/schema 검색
- `cargo check -p game_core`
- `cargo check -p game_server`
- `cargo test -p game_core --test ron_loading -- --nocapture`
- 관련 Core/server focused tests
- `cargo test -p game_core`
- `cargo test -p game_server`

각 실행은 실제 test 수와 pass/fail/skip, 기존 baseline failure와 새 회귀를 기록한다.

## Completion Conditions

- Master P00 exit gate를 모두 충족한다.
- production live bundle이 Support/Rest schema 없이 strict load된다.
- Core/server/Unity official flow에 제거 대상 legacy가 남지 않는다.
- SavePoint/Gate final tagged recovery, preview와 transaction 의미가 모든 계층에서 일치한다.
- SavePoint psychology/trust producer와 관련 memory/event/projection이 남지 않는다.
- canonical과 실제 구현이 일치하고 승인 없는 compatibility/fallback이 없다.
- focused와 broad evidence가 현재 worktree에서 재현된다.
- 독립 completion review verdict가 `Complete / High`다.

## Stop Conditions

- 승인 decision으로 답할 수 없는 gameplay, UX, DTO, RON, save 또는 compatibility 의미가 발견됨
- P00 대상 dirty 변경과 안전하게 병합할 수 없음
- 필수 외부 Unity repository/runner/canonical에 접근할 수 없어 계약을 검증할 수 없음
- P01 범위를 구현해야만 P00 의미를 성립시킬 수 있는 새로운 dependency가 확인됨
- 현재 baseline이나 변경 후 검증이 없어 회귀를 구분할 수 없음

사용자 정책 결정이 필요하면 Status를 `policy_decision_required`로 변경하고 영향받는 production
migration을 중단한다. 문서 작성이나 부분 구현만으로 complete 처리하지 않는다.
