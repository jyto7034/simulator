# Employee Domain Implementation Master Experiment Notes

## 구조적 판단

- `employee_definition_profile_contract`, `employee_recruitment_queue_ownership`, `employee_condition_lifecycle_migration`, `employee_psychology_state_contract`를 각각 Phase 1~4로 취급한다.
- Phase 경계는 저장소 수가 아니라 authored Profile, run recruitment ownership, condition lifecycle, Psychology라는 독립 source-of-truth 책임을 따른다.
- Phase 3은 난이도가 매우 높지만 Run HP 제거, condition, eligibility, death/treatment와 battle rollback/commit을 별도 Goal로 나누지 않는다. 분할 시 legacy와 신규 authority가 공존하기 때문이다.
- Master는 orchestration과 최종 통합 증거를 소유하며 production 구현과 durable 계약을 직접 소유하지 않는다.
- Child Goal 문서는 실제 Phase 착수 시 생성한다. 미리 생성해 discovery가 오래된 상태로 굳는 것을 피한다.

## 지침 준수 감사

- 사용자 요청에 따라 기존 네 subgoal을 Phase로 조정하는 Master를 명시적으로 생성했으므로, 지침의 "Master Goal 자동 생성 금지"에는 해당하지 않는다.
- Master는 여러 source-of-truth 경계를 직접 구현하지 않고 네 독립 Child Goal의 순서와 게이트만 조정한다. 단일 Goal 경계 규칙은 각 Child에 적용한다.
- `employee_domain_implementation_master`라는 이름은 사용자가 기존 subgoal을 Phase로 삼는 Master 작성을 요청한 문맥과 저장소의 `_master` 관례에서 선택했다. 사용자가 exact goal_name을 직접 쓰지는 않았다. 엄격한 실행 입력 해석에서는 생성 전에 이름 확인을 요청했어야 한다.
- 생성 전에 경로 충돌, 저장소 루트와 `git status --short`를 확인한 것은 지침 2.1과 일치한다.
- 그러나 필수 문서 3개를 생성하기 전에 repository-wide Goal 파일 목록, 기존 Master 예시와 완료된 난이도 감사 문서를 읽었다. 이는 read-only였지만 지침 2.1의 허용 범위를 넘은 절차 순서 위반이다.
- 실제 수행 명령은 `EXPERIMENTS.md`에 숨기지 않고 기록돼 있다. 과거 실행 순서는 소급 수정할 수 없으므로, 이후 Phase에서는 문서 초기화를 먼저 수행해 같은 위반을 반복하지 않는다.
- PLAN의 필수 항목 중 여러 절에 분산돼 있던 구현 범위, focused 검증, broad 검증과 생략 검증을 독립 제목으로 보강했다.

## Phase 의존성 판단

- Phase 2는 Phase 1의 stable `employee_definition_id` 없이는 queue dedupe와 exactly-one ownership을 안전하게 구현할 수 없다.
- Phase 3은 Phase 1의 최종 EmployeeRunState identity/Profile과 Phase 2의 checkpoint ownership shape가 안정된 뒤 수행해야 DTO/checkpoint 재작업을 줄일 수 있다.
- Phase 4는 Phase 1 personality와 Phase 3 death/checkpoint lifecycle을 소비하므로 마지막에 수행한다.
- Dice System과 employee-check Event는 Master 범위가 아니다. 최소 Phase 1과 Phase 3 이후 별도 Goal로 판단할 수 있다.

## Source-of-truth 충돌

- 현재 runtime과 live data는 승인된 목표 정책으로 아직 migration되지 않았다.
- `docs/decisions/employee_domain_policy_2026-07-17.md`의 초기 문단과 2026-07-18 승인 결정 사이에는 문구 차이가 있다.
- 각 Phase는 현재 동작, 기존 canonical 문서와 승인 목표를 구분하고 구현이 안정될 때 책임에 가까운 canonical 문서를 갱신한다.
- Goal 문서를 runtime 또는 wire source of truth로 사용하지 않는다.

## 정책 질문

현재 Phase 시작에 필요한 정책 질문은 없다.

다음은 의도적으로 보류된 후속 authoring이다.

- 직원별 최종 stats/personality/aptitude 배정
- physical Treatment와 특별 trauma override 수치
- trust 기반 high-risk/refusal/modifier/dialogue/reaction 콘텐츠
- account memorial/legacy

이 항목이 Phase 완료에 필수라는 새로운 근거가 나오면 해당 Child를 `policy_decision_required`로 전환한다.

## 외부 시스템과 직접 의존성

- production live data 저장소: `/mnt/f/work/simulator/game_resources`
- game_server 저장소: `/mnt/f/work/simulator/game_server`
- Unity 저장소: `/mnt/f/unity projects/ark`
- Unity test runner와 외부 durable save/database의 배포 상태는 Master 생성 시 검증하지 않았다.

## Dirty Worktree 주의사항

- Master 생성 전 Core 및 인접 저장소는 광범위한 modified/deleted/untracked 상태였다.
- 특히 employee, data loader, battle/world, snapshot, tests, game rulebook, runtime/data-loading canonical 문서와 game_server에 기존 변경이 있다.
- `docs/tmp/`와 여러 기존 Goal 디렉터리도 untracked 상태다.
- 이 Master에서 기존 dirty 파일은 수정하지 않았다.
- 각 Phase는 관련 저장소의 현재 `git status --short`와 수정 대상 파일의 diff를 다시 확인하고, 사용자 변경과 안전하게 병합할 수 없으면 해당 파일을 수정하지 않는다.
- Phase 1 재개 시 경로 확인 command가 첫 `test ! -e`에서 실패해 뒤의 `git status`가 실행되지 않았지만, 빈 출력을 clean 성공으로 잘못 해석했다. 실제 worktree는 Master 초기 기록대로 광범위한 dirty 상태였고 Phase 1 implementation도 이미 완료돼 있었다.
- 잘못 덮어쓴 Phase 1 문서는 이전 session patch log로 복구했으며 production 파일은 변경하지 않았다. 이후 Phase에서는 존재 확인의 exit code와 명시적 출력, 문서 내용 읽기를 분리해 확인한다.
- Phase 2는 existing dirty employee/world 변경 위에 추가됐으며 관련 파일의 현재 내용을 기준으로 patch했다. Core 전체 766 tests가 통과해 기존 변경과의 runtime 통합을 확인했다.

## 접근하거나 검증하지 못한 소비자

- 배포된 외부 durable save/database consumer 존재 여부
- 현재 workspace 밖의 employee DTO consumer
- 실제 배포 버전과 legacy payload 보유 상태
- Unity runner는 Phase 1·2에서 접근 가능함을 확인했다. Phase 2 결과는 EditMode 240 pass와 PlayMode 57 pass/4 explicit skip이다.

각 항목이 Child 완료에 필수라면 검증 없이 complete로 처리하지 않는다.

## 적용하지 않은 대안과 이유

- 네 Phase를 하나의 구현 Goal로 합치지 않았다. blast radius와 검증 비용이 과도하고 source-of-truth 전환 실패 원인을 분리하기 어렵다.
- Profile과 recruitment를 합치지 않았다. recruitment는 Profile catalog를 소비하지만 별도의 run ownership state machine이다.
- Phase 3을 Run HP 제거, death, treatment, eligibility Goal로 더 나누지 않았다. 중간 dual authority가 생긴다.
- trust 콘텐츠를 Phase 4에 넣지 않았다. 콘텐츠가 없는 상태에서 refusal/modifier/dialogue 의미를 임의로 정할 수 없다.
- Child Goal 디렉터리를 미리 만들지 않았다. 각 Phase 시작 시 최신 dirty/runtime baseline을 기록해야 한다.
- compatibility layer를 계획하지 않았다. 승인 정책이 complete cutover와 구형 계약의 명시적 거부를 요구한다.

## 생략한 검증과 근거

- Master 초기화 시 Rust/game_server/Unity tests는 문서 생성만 수행해 생략했다. Phase 1·2 completion review에서는 각 Child가 전체 계층을 실행했다.
- formatter: Markdown 문서만 생성했으며 관련 없는 dirty 파일을 포맷하지 않는다.
- runtime/live RON 재감사: 2026-07-18 완료된 난이도 감사 결과를 Master 초기 계획의 근거로 사용했다. 각 Phase 시작 시 관련 경로를 최신 상태에서 다시 조사한다.

## 남은 위험

- 네 Phase와 최종 통합 검증은 complete다.
- 배포된 외부 durable save가 후속에 발견되면 현재 no-compatibility 계약과 별도 승인 migration이 필요하다.
- 보류한 trust high-risk/refusal/modifier/dialogue와 account memorial은 현재 public/runtime 계약이 아니다.
- 외부 durable save/database가 발견되면 no-compatibility 정책과 실제 배포 migration 요구가 충돌할 수 있다.
- Unity의 legacy/unknown fallback tests는 승인된 strict cutover와 충돌하므로 각 DTO 소유 Phase에서 production reader와 함께 정리해야 한다.
- Phase 3은 매우 큰 lifecycle cutover다. Child Goal 내부 milestone과 focused validation 단위를 충분히 작게 유지하되 Phase 완료 경계를 분할하지 않아야 한다.
- Phase가 오래 걸리는 동안 선행 Phase 계약이 다른 작업으로 변경되면 다음 Phase 시작 전 완료 증거를 재검증해야 한다.

## 범위 밖 후속 후보

- Phase 1과 Phase 3 이후 Dice System과 employee-check Event 구현 가능성 재평가
- 실제 콘텐츠가 준비된 뒤 trust high-risk/refusal/modifier/dialogue 별도 Goal
- 외부 persistence가 확인된 경우 명시적으로 승인된 save migration Goal
- account memorial/legacy 정책 논의 및 별도 Goal

## 최종 통합 판단

- authored EmployeeDefinition/Profile, RecruitmentQueue ownership, condition/eligibility lifecycle과 Psychology aggregate가 서로 다른 단일 source를 가진다.
- run-local 연결은 stable definition identity와 roster UUID로 명확하며 checkpoint, retreat, result/death transition이 최종 aggregate를 보존한다.
- Core typed DTO가 public authority이고 game_server는 transport, Unity는 strict consumer로 검증됐다.
- 각 Child의 legacy 제거와 canonical 갱신이 final production 검색 및 별도 broad validation과 일치한다.
- Master completion review를 작성하고 Status/Implementation complete를 완료로 전환했다.
