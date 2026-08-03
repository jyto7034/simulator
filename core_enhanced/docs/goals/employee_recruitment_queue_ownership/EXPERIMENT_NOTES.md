# Employee RecruitmentQueue Ownership Experiment Notes

## 초기화 기록

- goal_name: `employee_recruitment_queue_ownership`
- path check: `phase2_missing`
- 선행 Phase 1: `complete`
- 선행 review: `docs/goals/employee_definition_profile_contract/COMPLETION_REVIEW.md`
- Core 관련 paths는 기존 modified, Goal docs는 untracked 상태다.

## 구조적 판단

- Phase 2는 authored Profile이 아니라 run-local recruitment ownership state machine을 소유한다.
- `employee_definition_id`만 Queue/Session/Roster를 연결하는 definition key로 사용하고 roster UUID는 합류 instance identity로 유지한다.

## Source-of-truth 충돌

- `RecruitmentState`가 queue와 active session의 유일한 ownership authority다.
- HQ active content는 node/options/shop metadata만 보유하며 후보 profile 사본을 소유하지 않는다.
- snapshot/result는 active session ID를 Phase 1 unified catalog에 투영한다.
- roster 합류 전까지 UUID를 만들지 않고, staged roster/UUID/queue를 node completion과 함께 commit한다.
- checkpoint payload와 Floor/run-complete carry-forward가 같은 aggregate를 clone한다.

## 정책 질문

없음. 실제 비채용 종료 분기인 응급지원과 HQ 상점은 active session 순서를 queue tail에 반환한다. 별도 HQ cancel command는 존재하지 않는다.

## 외부 시스템과 직접 의존성

- game_server recruitment behavior transport
- Unity HQ recruitment reader/command/presentation
- Core in-memory SavePoint checkpoint와 Floor carry-forward
- 별도 durable save는 발견되지 않았고 현재 persistence는 Core in-memory checkpoint다.

## Dirty Worktree 주의사항

- `world.rs`, `state.rs`, `map_content.rs`, `headquarters.rs`, `node_flow.rs`, `resources/selection.rs`, `behavior.rs`, `snapshot.rs`는 기존 modified 상태다.
- Phase 1 employee/data files와 Unity recruitment files도 기존 큰 diff 위에 있다.
- 파일별 current diff를 읽고 사용자 변경을 보존할 수 있을 때만 patch한다.

## 접근하거나 검증하지 못한 소비자

- 배포 workspace 밖 durable save/DTO consumer
- 배포 legacy payload 보유 상태

## 적용하지 않은 대안과 이유

- roster bool flags나 per-definition recruited set을 queue와 병행 저장하지 않는다. ownership 자체가 source다.
- node seed reroll을 compatibility path로 남기지 않는다.
- Phase 1 Profile schema를 queue 편의를 위해 복제하지 않는다.

## 생략한 검증과 근거

- live RON migration: queue는 Phase 1 catalog를 직접 소비하고 새 authored schema가 없다.
- game_server/Unity production 변경: public wire shape가 이미 `employee_definition_id` 계약이므로 변경하지 않고 전체 회귀 검증으로 소비 일치를 확인했다.
- active HQ checkpoint 생성: public SavePoint는 planning return에서만 생성된다. payload에는 aggregate 전체가 들어가며 정상 checkpoint의 다음 후보 목록은 queue에서 복원된다.

## 남은 위험

- Phase 3가 roster/condition shape를 바꿀 때 `validate_exact_ownership`의 roster definition iteration과 recovery-node 판단을 보존해야 한다.
- broad clippy `-D warnings`는 Phase 2와 무관한 기존 lint 진단 117개 때문에 실패했다. compile과 모든 runtime test는 통과했고 Phase 2 파일에서 새 compile warning은 없었다.

## 범위 밖 후속 후보

- external durable save가 실제로 도입되면 별도 승인된 persistence migration이 필요하다.
