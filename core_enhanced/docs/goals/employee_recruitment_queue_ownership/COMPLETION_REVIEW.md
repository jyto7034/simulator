# Employee RecruitmentQueue Ownership Completion Review

Date: 2026-07-18

Status: complete

## 완료 판단

Phase 2는 stable `employee_definition_id`만을 key로 하는 run-local finite queue, active recruitment session과 roster의 exactly-one ownership을 구현했다. 후보 순서는 run seed로 한 번 결정되고 HQ snapshot 반복, 비채용 분기, checkpoint load와 Floor 전환에서 보존된다.

## 최종 계약

- `RecruitmentState.queue`: 순서가 있는 미합류 definition IDs
- `RecruitmentState.active_session`: 현재 HQ가 제안 중인 definition IDs와 node ID
- `EmployeeRoster`: 합류한 definition ID와 새 run-local employee UUID
- queue/session은 profile 사본이나 recruited/offered bool을 저장하지 않는다.
- public HQ DTO는 active session ID를 unified `EmployeeDefinition` catalog에 투영한다.
- 채용은 선택 ID만 roster로 이동하고 나머지는 session order로 tail에 반환한다.
- 응급지원과 HQ shop은 active session 전체를 같은 순서로 tail에 반환한다.
- HQ mutation과 node completion은 rollback snapshot 안에서 commit된다.

## Persistence와 reconnect

- checkpoint payload는 `RecruitmentState` 전체와 roster를 복제하고 load 전에 exact ownership을 검증한다.
- Floor advance와 run completion은 같은 aggregate를 carry-forward한다.
- public SavePoint는 planning return에서만 생성되므로 정상 checkpoint의 active session은 비어 있다. 다음 후보 목록은 저장된 queue head에서 정확히 복원된다.
- 재접속은 동일 Core actor/world state의 active session과 HQ metadata를 사용하고 snapshot 재요청은 후보를 다시 뽑지 않는다.

## Producer/consumer 판단

Core의 public candidate/result/command shape는 Phase 1에서 이미 `employee_definition_id`와 unified profile로 전환돼 있었다. Phase 2는 내부 ownership source만 교체했으며 game_server와 Unity에 새 wire 필드나 compatibility path를 추가하지 않았다. 전체 server/Unity tests로 기존 consumer가 새 Core producer projection을 그대로 소비함을 확인했다.

## 검증 요약

- Core focused recruitment: 3/3
- Core focused HQ/session/checkpoint: 16/16
- Core Floor/run carry-forward: 2/2
- Core recovery integration: 1/1
- Core broad: 766/766
- game_server broad: 30/30
- Unity EditMode: 240/240
- Unity PlayMode: 57 pass, 0 fail, 4 explicit server-dependent skip
- legacy node-seeded recruitment search: 0
- `git diff --check`: clean

`cargo clippy --all-targets -- -D warnings`는 Phase 2와 무관한 기존 lint debt 117개 때문에 실패했다. 이 실패는 숨기지 않고 `EXPERIMENTS.md`에 기록했으며 compile과 모든 runtime suite는 통과했다.

## Legacy/compatibility 확인

- node-seeded recruitment shuffle/truncate 함수 제거
- HQ active content의 full candidate ownership 제거
- recruited/offered parallel flags 없음
- queue와 병행하는 per-definition ownership set 없음
- legacy candidate ID alias, dual read/write, fallback 없음

## 다음 Phase에 넘기는 안정 계약

Phase 3는 roster의 condition shape를 교체하더라도 다음을 유지해야 한다.

- roster record의 stable `employee_definition_id`
- `RecruitmentState::validate_exact_ownership`이 roster definition IDs를 검사할 수 있는 경계
- recruit transaction이 recovery/failure 재평가 전에 staged roster를 commit하는 순서
- checkpoint와 Floor carry-forward의 recruitment aggregate 보존
