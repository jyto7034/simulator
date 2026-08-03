# Run Persistence, Facility, And Monolith Policy Implementation Master

Status: complete

Implementation complete: true

Goal name: `run_persistence_facility_monolith_policy_master`

Created: 2026-07-27

Audit finding ID: 없음

Execution authority: 2026-07-27 사용자 요청

## 1. 목표

현재 코드베이스에서 중단된 `Support/Rest -> SavePoint` 전환을 임시 중간 계약으로 한 번 더
고정하지 않고 승인된 최종 SavePoint/Gate Trauma 계약으로 수렴시킨 뒤, 아래 다섯 승인된 decision
record를 실제 runtime, live RON, game_server, 외부 Unity client, 저장 파일과 canonical 문서에
의존성 순서대로 구현한다.

1. [SavePoint·Trauma 회복 및 출전 불가 실패 정책](../../decisions/save_point_trauma_recovery_policy_draft_2026-07-25.md)
2. [로컬 영구 조우 도감 저장·복구 정책](../../decisions/local_encounter_compendium_persistence_policy_draft_2026-07-25.md)
3. [시설 맵 템플릿·노드 배정·조건부 발현 정책](../../decisions/facility_map_and_node_manifestation_policy_2026-07-25_26.md)
4. [플레이어 전투력·Floor 난이도 분포 정책](../../decisions/combat_power_floor_difficulty_policy_draft_2026-07-26.md)
5. [Elite 모노리스·강화 재현 및 숨겨진 도감 보스 정책](../../decisions/elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md)의 P05 모노리스 범위

이 master는 서로 다른 source-of-truth 경계를 하나의 거대한 구현 patch로 합치지 않는다. Master는
phase registry, 의존성, 공유 seam, 진입/완료 gate와 최종 통합 검토를 소유한다. 실제 production
변경은 활성 phase의 child Goal에서 수행한다.

## 2. 해결할 사용자-visible 문제

- Core, live RON, game_server test와 Unity가 `Support`, `Rest`, `SavePoint`를 서로 다르게 해석해
  live data와 전체 계약 검증이 깨진 현재 중간 상태를 제거한다.
- SavePoint를 완료하고 종료한 플레이어가 마지막 완료 SavePoint에서 안전하게 이어할 수 있게 한다.
- 출전 가능한 직원이 없는 상태에서 가능한 회복 경로와 Run Failed 확정을 일관되게 제시한다.
- 조우 발견과 격파 기록이 run/checkpoint와 분리된 로컬 profile에 원자적으로 보존되게 한다.
- 절차적 depth/lane placeholder map을 authored facility template, semantic slot, 조건부 발현과
  결정론적 assignment가 소유하는 구조로 교체한다.
- 실제 편성에서 산출한 certified combat power와 Floor anchor로 node 난이도와 보상을 Floor 생성
  시 한 번 확정한다.
- Endless Elite의 자연 조우, 전용 모노리스, 강화 재현, 잔여 교환과 변질된 모노리스 획득을 앞선
  저장·도감·시설·난이도 계약 위에서 완결한다.
- 실제 숨겨진 보스 전투·연출·특수 완료는 현재 master 범위와 완료 조건에서 제외하고, 콘텐츠 개발
  시점의 별도 후반 Goal에서 구현한다.

## 3. Master 문서의 책임과 비책임

### 소유하는 것

- phase 순서와 dependency graph
- child Goal 이름, 상태와 활성화 조건
- phase 사이에 전달해야 하는 schema/state/transaction seam
- 각 phase의 사용자-visible exit gate
- phase별 completion review와 마지막 cross-component review 요구사항
- 정책 질문, 외부 차단과 dirty worktree 충돌 시 중단 기준

### 소유하지 않는 것

- gameplay/runtime/data/wire 계약의 durable canonical
- child Goal의 상세 구현 순서와 내부 추상화 모양
- 결정문에 없는 UX, 밸런스, save compatibility 정책
- 이 campaign과 독립적으로 구현·검증할 수 있는 현재 dirty worktree의 다른 변경

안정된 정책은 구현 phase 안에서 가장 가까운 canonical 문서로 옮긴다. 이 master나 child Goal을
최종 gameplay source of truth로 사용하지 않는다.

## 4. 필수 시작 및 재개 절차

Master를 시작하거나 재개할 때 production 코드 검색·수정·검증 전에 다음을 처음부터 끝까지 읽는다.

- 이 디렉터리의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
- [문서 인덱스](../../README.md)
- [Engineering Workflow](../../engineering_workflow.md)
- [Goal 완료 검토 가이드](../../goal_completion_review_guide.md)
- 다섯 decision record
- 활성 child Goal의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`

활성 phase가 Unity-facing 계약을 바꾸면 외부 Unity 프로젝트의 다음 canonical도 읽는다.

- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- battle event/result identity나 전송 순서에 닿으면
  `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- Unity 구현/UX 범위는 `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

Child 문서는 해당 phase를 실제로 활성화할 때 생성한다. 모든 child 디렉터리를 master 생성 시점에
미리 만들지 않는다. 불완전한 빈 Goal과 실제 active 작업 경계의 혼동을 막기 위해서다.

## 5. Source Of Truth 순서

각 phase는 다음 순서로 현재 사실을 확인한다.

1. 실제 runtime code와 state mutation 경로
2. production live RON/data와 loader/validation
3. 실제 Core DTO, game_server mapping, WebSocket payload와 외부 Unity 소비자
4. 최신 durable canonical 문서
5. 승인된 decision record
6. master/child Goal과 과거 goal 기록

Decision record는 승인된 목표 정책의 근거지만 현재 구현 사실이나 durable canonical을 대신하지
않는다. Runtime과 decision이 다르면 현재 동작과 목표 동작을 나누어 기록하고, 승인된 정책에 없는
의미를 임의로 채우지 않는다.

## 6. 현재 runtime 및 live data 근거

2026-07-27의 현재 dirty worktree에서 다음 부분 구현과 충돌을 확인했다.

### 중단된 SavePoint 전환

- `src/game/map/types.rs`는 `MapNodeCategory::Support`를 `SavePoint`로 바꾸고 `Rest`와 선택형
  support payload를 제거했다.
- `src/game/world/support.rs`는 `SavePointSessionState`와 flat trauma heal로 일부 전환했지만
  module/function/type 이름 일부는 support terminology를 유지하고 SavePoint psychology event를
  여전히 생산한다.
- `data/run/policy.ron`과 `RunPolicyData`는 구 percent+Rest 필드에서 flat SavePoint heal로 일부
  이동했지만 승인된 공통 tagged `TraumaRecovery` 계약은 아직 없다.
- `game_resources/data/map/generation_policy.ron`의 safe category에는 삭제된 `Support`가 남아 있다.
- `game_server/src/game/player_game_actor/messages.rs` test는 삭제된 `SupportNodeType::Rest`와
  `PlayerBehavior::ChooseSupport`를 참조한다.
- 외부 Unity runtime, mock, test와 `unity_core_contract.md`에는 `SupportChoice`, `ChooseSupport`,
  `Rest` 계약이 남아 있다.

### 저장과 도감

- `RunCheckpointState`는 process-local `Option<RunCheckpointPayload>`이고 durable active-run host
  storage가 없다.
- `EncounterCompendiumProfileState`는 발견 abnormality/corrosion set만 가지며 defeated set,
  authored placeholder projection과 dormant-ID 보존 계약이 없다.
- game_server는 profile을 load-balance actor의 process-local `HashMap`에만 보존한다.
- active-run과 local profile에 필요한 atomic current/backup/checksum/locking/dirty retry host
  persistence가 없다.

### 맵과 난이도

- `MapNode`에는 `map_template_id`와 `slot_id` DTO 기반이 있으나 generator는 여전히 procedural
  depth/lane topology와 `depth_XX_lane_YY` placeholder slot을 만든다.
- Unity `NodeMapTemplateCatalog`도 `act_01_floor_a` placeholder 한 개를 좌표 template로 사용한다.
- certified combat power, run peak, floor power anchor, difficulty band와 node target power는 없다.
- 현재 난이도는 `floor_combat_scaling`의 Floor별 고정 multiplier가 소유한다.

### Elite/모노리스

- 모노리스, 강화 재현과 변질된 모노리스 runtime/data는 아직 없다.
- 현재 Endless 연구 point/completion과 repeat weighting은 새 hard-exclusion/3단계 정책과 충돌한다.
- 일반 abnormality attempt 최대값은 3이고, live FinalBoss encounter는 현재 5종이다.
- Shop merchandise에는 전용 모노리스와 관련 run-local ledger가 없다.

## 7. 소유 계층

| 책임 | Authority | 소비자/전달 계층 |
| --- | --- | --- |
| gameplay state와 원자 transition | `game_core::GameCore` / `RunState` | game_server actor, Unity projection |
| authored policy/content | `game_resources/data/**/*.ron` + strict Core loader/validation | Core runtime |
| active-run/checkpoint payload 의미 | Core runtime contract | host persistence, game_server, Unity continue UX |
| 로컬 파일 경로·원자 publish·lock·backup | host/game_server persistence 계층 | Core bootstrap/profile injection, Unity status projection |
| 영구 도감 set 검증·union·projection | Core compendium domain | host file, root snapshot, Unity reader |
| facility topology·assignment·난이도 확정 | Core map/Floor generation transaction | snapshot, Unity authored visual template |
| transport mapping/order | game_server | Unity WebSocket client |
| 화면·입력·local visual resources | 외부 Unity client | 플레이어 |
| durable gameplay 의미 | `game_rulebook.md` | 모든 계층 |
| runtime/checkpoint/save 불변식 | `core_runtime_contract.md` | Core/server/test |
| live RON schema/loading | `data_loading_contract.md` | authoring/loader/test |
| exact Unity wire contract | 외부 `unity_core_contract.md` | Core/server/Unity |

Unity는 Core state, map eligibility, discovery, power와 난이도를 재계산하지 않는다. Host는 Core가
소유하는 gameplay set/state 의미를 다시 해석하지 않고 durable lifecycle만 소유한다.

## 8. 공통 불변식

- 고정 seed와 저장된 결정 상태에서 map, assignment, difficulty와 encounter identity가 결정론적이다.
- failed/rejected transaction은 회복, checkpoint, node/Floor completion, reward, inventory나 persistent
  file 중 일부만 commit하지 않는다.
- checkpoint load는 checkpoint-owned run-local state를 함께 복원하고 `loads_used` 같은 명시된
  monotonic metadata와 local compendium은 되돌리지 않는다.
- active-run file과 local compendium profile은 별도 canonical, 별도 transaction, 별도 failure
  surface다.
- Canonical/backup 손상과 unsupported version을 빈 상태로 조용히 대체하지 않는다.
- 새로 commit하는 ID는 현재 catalog에서 strict validation하며, 이미 저장된 dormant ID는 삭제하지
  않는다.
- 같은 Floor의 reconnect, snapshot/map 재요청과 Safezone 왕복은 template, assignment, anchor,
  difficulty와 encounter seed를 재추첨하지 않는다.
- Facility topology, 연결, 선택 가능성과 conditional manifestation은 Core authority다. Unity는 좌표나
  그림에서 gameplay 연결을 추론하지 않는다.
- difficulty/reward는 Floor/node 생성 commit에서 고정되며 현재 편성 변화에 맞춰 같은 node를
  재스케일하지 않는다.
- 발견/격파, reward, 모노리스 사용과 node 변환은 승인된 commit 시점에 정확히 한 번 적용된다.
- 제거 대상 legacy는 runtime, RON, DTO, server, Unity, mock, fixture, test와 canonical에서 함께
  제거한다. 승인 없는 alias, dual-read/write와 fallback을 두지 않는다.
- 현재 dirty worktree의 사용자 변경을 덮어쓰거나 관련 없는 변경을 campaign에 섞지 않는다.

## 9. Dependency Graph

```text
P00 최종 SavePoint / Gate Trauma 수직 전환
  -> P01 출전 불가 실패 / checkpoint / active-run
      -> P02 로컬 영구 도감 저장·실제 commit producer
      -> P03 시설 맵 / assignment / manifestation
          -> P04 certified power / Floor difficulty

P01 + P02 + P03 + P04
          -> P05 Elite 모노리스 / 강화 재현 / 잔여 교환 / 변질된 모노리스
              -> P07 P00-P05 전체 통합 완료 검토

현재 master 밖 후반 콘텐츠:
P06으로 계획했던 숨겨진 보스 / capstone / 특수 완료
  -> 콘텐츠 설계·제작 시 별도 Goal로 시작
```

P02와 P03은 서로의 hard prerequisite가 아니다. 둘 다 P01 뒤에 시작할 수 있다. 기본 실행 순서는
P01에서 검증한 host atomic storage 기술과 profile/run 분리를 먼저 재사용하기 위해 P02를 P03보다
앞에 둔다. 현재 dirty 변경이 큰 Core/server/Unity 표면을 동시에 수정하는 위험을 줄이고 하나의
phase review verdict를 다음 baseline으로 사용하기 위한 **선호 직렬 순서**다. P02가 외부 파일 API나
profile UI 문제로 차단돼도 P03의 source-of-truth가 준비됐다면 P03까지 hard-block하지 않는다.

## 10. Phase Registry

| Phase | Child Goal | Initial status | Hard prerequisite | 핵심 결과 |
| --- | --- | --- | --- | --- |
| P00 | `run_policy_phase_00_savepoint_trauma_vertical_convergence` | complete; Complete/High 2026-07-27 | 없음 | 최종 SavePoint/Gate Trauma 계약과 Support/Rest 제거 |
| P01 | `run_policy_phase_01_failure_checkpoint_active_run_persistence` | complete; Complete/High 2026-07-27 | P00 Complete/High | 실패·checkpoint·durable continue |
| P02 | `run_policy_phase_02_local_compendium_persistence` | complete; Complete/High 2026-07-27 | P01 Complete/High | run 밖 영구 discovered/defeated profile |
| P03 | `run_policy_phase_03_facility_map_manifestation` | complete; Complete/High 2026-07-27 | P01 Complete/High; P02 뒤 실행 선호 | authored facility, assignment, generic capstone |
| P04 | `run_policy_phase_04_combat_power_floor_difficulty` | complete; Complete/High 2026-07-27 | P03 Complete/High | certified power, anchor, node/role 난이도 |
| P05 | `run_policy_phase_05_elite_monolith_remanifestation` | complete; Complete/High 2026-07-27 | P01-P04 Complete/High | 자연 Elite, 상인/모노리스, 강화 재현, 잔재 교환, 변질된 모노리스 획득 |
| P06 | 없음 — 현재 master에서 제외 | future_content_goal | 현재 master dependency 아님 | 실제 hidden-boss 전투·VN·capstone·특수 완료는 콘텐츠 개발 시 별도 Goal |
| P07 | `run_policy_master_final_completion_review` | complete; Complete/High 2026-07-27 | P00-P05 Complete/High | cross-component 독립 감사와 master 종료 |

Master `PLAN.md`의 이 표가 phase 활성 순서와 상태의 유일한 master-level source다. Child가 complete를
주장해도 review verdict가 `Complete / High`가 아니면 다음 phase를 열지 않는다.

### 10.1 Phase 진입 readiness gate

Phase를 활성화하기 전에 구현 도중 처음 발견하면 큰 partial migration을 남길 입력을 먼저 확인한다.

- P00: 현재 Support/Rest dirty diff의 파일별 의도와 최종 tagged `TraumaRecovery` schema/기본값을
  decision과 대조한다. 최종 형태가 아닌 flat/percent 혼합 public contract를 intermediate canonical로
  고정하지 않는다.
- P01: campaign 중간 build가 실제 배포되는지, 기존 active-run file을 유지해야 하는지 먼저 확인한다.
  유지 요구가 있으면 version/migration 정책 승인 전 persistence schema 변경을 시작하지 않는다.
- P03: Standard Floor 1~3 최소 template, Endless 초기 pool, 대표 conditional-manifestation template,
  BossOmen/hidden-boss capstone template와 Unity-local slot/reveal resource의 authoring 가능성을 확인한다.
  최종 아트는 비범위지만 contract test가 읽을 수 있는 실제 template/anchor resource는 필요하다.
- P04: provisional power coefficient, model version, Floor minimum/rise, band quota/modifier와 role profile의
  초기 live authoring 입력을 확인한다. 정책으로 확정되지 않은 balance 값이 필요하면 production
  migration 전에 `policy_decision_required`로 전환한다.
- P05: monolith price/offer/refresh/stage reward, exchange VN resource와 corrupted-monolith key에 필요한
  실제 asset/data를 확인한다. 구조가 승인된 초기 balance 값은 decision 12절 범위에서 보수적으로
  authoring할 수 있지만 누락 리소스를 빈 placeholder로 완료 처리하지 않는다.
- 후반 hidden-boss Goal: actor/encounter/skill, 관리자 VN, 전용 capstone, 일반 FinalBoss 재분류와 특수
  완료에 필요한 콘텐츠 정책 및 실제 asset/data를 먼저 확인한다. 콘텐츠 개발의 명시적 재개 전에는
  Goal이나 placeholder 구현을 만들지 않는다.

## 11. Phase 00 — 최종 SavePoint·Gate Trauma 수직 전환

### 범위

- 현재 diff에서 이미 시작된 `Support/Rest -> SavePoint` 전환을 승인된 최종 SavePoint/Gate 정책에
  맞춰 수렴한다.
- Core runtime/data/DTO, live RON, game_server mapping/test, 외부 Unity reader/router/command/view,
  mock/fixture/test와 canonical의 제거 경계를 함께 정리한다.
- 공통 tagged `TraumaRecovery`와 SavePoint `CurrentPercent { 15 }`, Gate
  `CurrentPercent { 10 }`를 final live schema/runtime으로 구현한다.
- SavePoint psychology/trust producer, `RecoveredAtSavePoint` memory/event와 구 Support/Rest alias를
  제거한다.
- SavePoint/Gate preview와 Core completion transaction이 recovery, node/Floor completion, pending
  research와 process-local checkpoint state를 승인된 순서와 atomicity로 처리하게 한다.
- 이 phase는 durable active-run file, recovery restriction과 실패 사전 선택 modal을 구현하지 않는다.
  P00이 소유한 public schema와 recovery 의미는 P01이 다시 교체하는 임시 계약이 아니다.
- 현재 campaign과 무관한 dirty worktree의 다른 compile/test 실패는 수정하지 않고 baseline으로
  분류한다.

### Exit gate

- production live bundle이 삭제된 `Support`/`Rest` schema 없이 strict load된다.
- Core/server/Unity의 official non-combat flow에 `ChooseSupport`, `SupportChoice`, `Rest`가 남지 않는다.
- SavePoint/Gate가 동일한 tagged recovery type과 독립된 final live 값을 사용하고 Unity가 Core preview를
  재계산하지 않는다.
- SavePoint psychology/trust producer와 관련 memory/event/DTO가 남지 않는다.
- SavePoint node 진입·확인·완료, process-local checkpoint와 Gate transition이 final recovery 의미로
  계층 사이에서 일치한다.
- 관련 legacy symbol/schema 검색 결과가 설명 가능한 0건이거나 역사 문서/비소유 명칭만 남는다.
- `cargo check -p game_core`, `cargo check -p game_server`, live RON test, Core/server focused test와
  접근 가능한 Unity focused test가 통과한다.
- completion review verdict가 `Complete / High`다.

## 12. Phase 01 — 출전 불가 실패·Checkpoint·Active-Run

### 범위

- 출전 가능성 계산, recovery path evaluator, `AwaitingRunFailureDecision`, `ConfirmRunFailure`, 허용
  command와 UX를 구현한다.
- 기존 준비 화면의 employee-targeted consumable UX로 Physical Treatment와 단일 지속 modifier 교체
  규칙을 구현하고 유용성과 command eligibility를 분리한다.
- checkpoint payload/monotonic metadata 원자 복원과 load 제한을 구현한다.
- 마지막 완료 SavePoint의 versioned active-run current/backup/checksum/lock/continue lifecycle을
  구현한다.
- RunComplete, 확정 RunFailed, AbandonRun과 새 run 대체 시 active-run 무효화/확인 정책을 구현한다.

### 내부 milestone

1. P01-A: deployability와 현재 Floor recovery path evaluator, restriction command matrix
2. P01-B: 모든 gameplay failure 후보의 사전 checkpoint 선택, Treatment UX와 Unity blocking modal
3. P01-C: checkpoint monotonic envelope, atomic current/backup/lock, continue와 terminal invalidation

각 milestone은 focused 검증을 남기되 P01 전체 completion review는 세 milestone과 결합 경계를 모두
검증한 뒤 한 번만 `Complete / High`로 닫는다.

### Exit gate

- SavePoint와 Gate 회복이 정확히 한 번 적용되고 실패 시 partial commit이 없다.
- 모든 진입 불가/회복 가능/실패 확정 분기가 승인된 상태·command·Unity UX로 검증된다.
- 성공한 checkpoint load만 횟수를 소비하고 process restart로 monotonic metadata를 초기화할 수 없다.
- valid current, backup recovery, both-corrupt, unsupported-version, write/publish failure와 continue
  invalidation이 실제 파일 검증으로 보호된다.
- P01 시점의 모든 checkpoint-owned run state가 round-trip되고 local profile copy는 payload에 없다.
- active-run canonical과 monotonic metadata가 transport `player_id`별 임시 actor state가 아니라 승인된
  local-install scope를 사용한다.
- focused, live RON, Core/server, host persistence, WebSocket/Unity와 broad 검증이 통과한다.
- completion review verdict가 `Complete / High`다.

## 13. Phase 02 — 로컬 영구 조우 도감 저장·복구

### 범위

- Core profile을 authored order 기반 `discovered`/`defeated` set과 placeholder projection으로 확장한다.
- `defeated` 부분집합, 현재 catalog 신규 ID validation, dormant ID 보존과 idempotent union을 구현한다.
- active-run과 별도인 local profile current/backup/checksum/lock/bootstrap/recovery/dirty retry를
  host에 구현한다.
- Unity가 저장 정상/dirty/recovered/error 상태와 발견/격파 projection을 Core에서만 읽게 한다.
- 현재 존재하는 일반 combat 결과에서 실제 spawn/result identity를 사용해 발견과 격파를 승인된
  시점에 정확히 한 번 commit한다. Primary 승리와 non-primary `UnitDied` 규칙을 대표 live flow로
  고정한다.
- P05의 remanifestation과 미래 hidden-boss producer는 generic commit seam을 재사용하되 콘텐츠 전용
  trigger를 선행 구현하지 않는다.

### Exit gate

- 새 run, RunFailed, checkpoint load와 active-run continue가 profile set을 되돌리거나 stale copy로
  덮지 않는다.
- active-run write와 profile write 실패가 서로 rollback하지 않는다.
- current/backup 복구, both-corrupt blocking, unsupported version, single-writer lock과 dirty retry가
  실제 파일 검증으로 보호된다.
- authored 전체 항목의 placeholder/discovered/defeated projection과 Unity 표시가 일치한다.
- 기존 대표 encounter의 승리/패배/비primary 사망이 실제 Core-owned discovered/defeated producer와
  persistence를 통과하며, P05와 미래 콘텐츠가 같은 API에 신규 producer만 연결할 수 있다.
- local profile canonical은 transport/player actor UUID별로 분리되지 않고 process-local per-player
  `HashMap`이 durable authority로 남지 않는다.
- completion review verdict가 `Complete / High`다.

## 14. Phase 03 — 시설 맵·노드 배정·조건부 발현

### 범위

- procedural depth/lane generator를 authored facility template, semantic slot, directed connection,
  tag와 strict validation으로 교체한다.
- mode/Floor template pool, deterministic order/cycle, standard learning schedule와 constrained node
  assignment를 구현한다.
- Shop-tag guarantee와 조건부 bonus manifestation을 위한 generic requirement/manifestation seam을
  구현한다.
- Core-owned navigation과 generic atomic capstone map switch를 구현한다.
- Unity는 authored template visual/slot position을 사용하되 topology와 eligibility는 Core에서 읽는다.
- Floor generation transaction에는 P04 difficulty 계산과 P05 monolith Shop guarantee가 연결될 명시적
  typed extension seam을 제공한다. 미래 capstone도 별도 Goal에서 같은 generic seam을 소비할 수 있다.
- P03은 아직 존재하지 않는 P04 profile을 가리키는 dangling `difficulty_profile_id`를 live RON에
  저작하지 않는다. Difficulty reference field와 strict reference validation은 유효 profile data와
  함께 P04에서 추가한다.

### Exit gate

- template/schema/reference/topology/required tag와 assignment 불가능성이 startup/generation에서
  strict rejection된다.
- 고정 seed와 저장된 pool/cycle/assignment에서 같은 map이 생성되고 reload/reconnect가 재추첨하지
  않는다.
- Gate와 BossOmen generic capstone transition이 partial commit 없이 동작한다.
- P05 및 미래 콘텐츠 전용 eligibility를 시설 계층이 재계산하지 않고 typed requirement 입력만 소비한다.
- checkpoint/active-run round-trip이 template pool/order/cycle/assignment/current map을 보존한다.
- Core/server/Unity map DTO, rendering, navigation flow와 live RON 검증이 일치한다.
- completion review verdict가 `Complete / High`다.

## 15. Phase 04 — Certified Combat Power·Floor 난이도

### 범위

- 실제 전투 편성의 provisional certified combat power와 `power_model_version`을 구현한다.
- monotonic run peak와 Floor power anchor를 Floor creation transaction에 저장한다.
- ordinary Combat band quota/rounding, node target power, encounter/wave budget과 reward modifier를
  facility assignment profile에 결합한다.
- Elite/NormalBoss/FinalBoss/BossOmen/monolith remanifestation용 role difficulty profile을 분리한다.
- readiness, risk와 reward projection을 Core authority로 Unity에 제공한다.
- 기존 `floor_combat_scaling`과 경쟁하는 난이도 source를 제거하거나 승인된 단일 migration으로
  교체한다.

### Exit gate

- 동일 loadout/version은 동일 certified power를 만들고 run peak/anchor는 승인된 규칙대로 단조다.
- Floor 생성이 template, assignment, band, target power, seed와 reward contract를 한 transaction에서
  결정한다.
- reconnect/load/same-Floor 편성 변경이 기존 node difficulty를 재계산하지 않는다.
- ordinary quota와 role profile 대상이 섞이지 않고 rounding 합계가 대상 node 수와 일치한다.
- 모든 수치와 schedule은 live RON + strict validation이 소유하며 임의 runtime fallback이 없다.
- checkpoint/active-run과 Unity projection이 version/peak/anchor/node difficulty를 정확히 보존·표시한다.
- completion review verdict가 `Complete / High`다.
- P04가 Floor generation transaction과 assignment profile을 materially 확장하므로 P03의 determinism,
  save round-trip과 assignment review를 P04 completion review에서 함께 재실행한다.

## 16. Phase 05 — Elite 모노리스·강화 재현·잔여 교환

### 범위

- Endless Elite 최초 자연 조우 후 hard exclusion과 기존 research/repeat completion 충돌을 정리한다.
- 환상체별 typed monolith merchandise, Floor 조건, 구매 exclusion과 run-local ledger를 구현한다.
- 일반 Combat node의 원자 변환, 3단계 강화 재현, 최대 2회 attempt, reward/fragment progression을
  구현한다.
- 잔여 모노리스, 다음 Floor Shop-tag guarantee, 일회성 VN 교환과 변질된 모노리스를 구현한다.
- 변질된 모노리스는 run-local key로 checkpoint/active-run에 보존하되, 이 phase에서는 소비처를
  만들지 않는다. 기존 Endless 최종 보스와 일반 완료 흐름은 그대로 유지한다.
- 실제 강화 재현 spawn/result identity에서 discovered/defeated를 승인된 commit 시점에 local profile로
  보낸다. 기존 encounter 공통 producer를 다시 만들지 않고 P02 API에 remanifestation identity만 연결한다.

### 내부 milestone

1. P05-A: 자연 Elite hard exclusion, 떠돌이 상인 offer와 monolith purchase/run-local ledger
2. P05-B: 일반 Combat node 원자 변환, 강화 재현 attempt/reward/fragment progression
3. P05-C: 잔여 ledger, 다음 Floor Shop guarantee, 교환 VN과 변질된 모노리스 key

각 milestone은 checkpoint/active-run rollback과 duplicate commit을 focused 검증하고, P05 전체
completion review는 세 milestone을 실제 Endless flow로 결합해 다시 검증한다.

### Exit gate

- 자연 Elite exclusion, monolith purchase/use, node conversion, attempt, reward와 exchange가 고정 seed와
  checkpoint rollback에서 일관된다.
- 변환·소비·reward/profile mutation이 partial 또는 duplicate commit되지 않는다.
- 다음 Floor Shop guarantee와 generic facility manifestation seam이 실제 flow로 검증된다.
- remanifestation difficulty가 P04 `MonolithRemanifestation` role profile을 사용하고 ordinary quota를
  오염시키지 않는다.
- 변질된 모노리스가 독립 run-local key로 정확히 한 번 획득되고 checkpoint/active-run에 보존되며,
  미래 hidden-boss Goal 전까지 일반 최종 보스·기존 Endless completion을 가로채지 않는다.
- P01 active-run payload와 P03 map state가 모든 새 monolith state를 round-trip한다.
- 결정문이 요구하는 content/balance authoring이 live RON에 존재하고 strict load된다.
- completion review verdict가 `Complete / High`다.

## 17. 현재 master에서 제외된 후반 콘텐츠 — 숨겨진 보스·Capstone·특수 완료

과거 P06으로 계획했던 실제 `흉탄의 사수` 콘텐츠는 사용자 결정에 따라 현재 master의 phase,
dependency와 완료 조건에서 제외됐다. 콘텐츠 설계·제작을 시작할 때 README의 후반 작업 표식을
입력으로 별도 Goal을 만들며, 현재는 production 구현과 child Goal 생성을 하지 않는다.

### 범위

- 일반 FinalBoss pool을 승인된 세 종으로 재분류하고 변질된 모노리스 보유 시 일반 완료를 가로채는
  eligibility와 단일 관리자 VN 선택을 구현한다.
- 승인된 distinct hidden-boss actor/skill/encounter, Unity-local cue/resource와 전용 single-terminal
  capstone을 live content로 구현한다.
- hidden boss 발견/격파, one-attempt FinalBoss resolution과 typed special completion을 P02 compendium,
  Core/server/Unity/persistence에 수직 연결한다.

### 미래 Goal의 Exit gate 참고

- 승인된 실제 actor/encounter/skill/VN/capstone이 placeholder나 기존 Der Freischutz 재사용 없이 strict
  load되고 사용자-visible flow에서 도달 가능하다.
- 변질된 모노리스 소비, 일반 완료 interception, VN choice, capstone manifestation과 특수 완료가 partial
  또는 duplicate commit되지 않는다.
- hidden boss discovered/defeated와 completion reason이 Core/server/Unity/profile에서 일치한다.
- completion review verdict가 `Complete / High`다.

## 18. Phase 07 — 최종 통합 완료 검토

P07은 새 기능을 조용히 구현하는 phase가 아니라 독립적인 completion review Goal이다. P00-P05의
child 주장과 test 결과를 그대로 믿지 않고 [Goal 완료 검토 가이드](../../goal_completion_review_guide.md)에
따라 runtime/data/server/Unity/file evidence를 다시 수집한다.

### 필수 cross-component 질문

- P03-P05에서 추가한 모든 run-local state가 P01 checkpoint/active-run에 포함됐는가?
- local compendium은 어떤 load/continue/failure에서도 checkpoint와 분리돼 있는가?
- active-run과 profile file의 failure/backup/lock transaction이 서로 오염되지 않는가?
- facility assignment와 difficulty가 같은 Floor generation commit을 공유하는가?
- monolith Shop guarantee가 facility generic seam을 사용하고 topology를 별도 수정하지 않는가?
- discovered/defeated producer가 실제 combat result identity와 정확히 한 번 결합되는가?
- Core, live RON, game_server, Unity, mock/probe/canonical에 제거된 legacy가 남지 않았는가?

### Exit gate

- P00-P05 각각의 audit verdict가 `Complete / High`다.
- cross-component finding이 없거나 모두 같은 phase 범위의 correction으로 수정·재검토됐다.
- focused evidence와 최종 broad validation이 현재 dirty worktree에서 재현된다.
- 다섯 decision record의 모든 승인된 완료 조건이 runtime/data/contract evidence에 연결된다.
- 최신 gameplay/runtime/data/Unity canonical이 실제 구현과 일치한다.
- 범위 밖 dirty 변경이 보존됐고 campaign diff가 설명 가능하다.

## 19. Cross-Phase Seam Ledger

| Seam | 최초 owner | 후속 consumer | 완료 규칙 |
| --- | --- | --- | --- |
| atomic file publish/backup/checksum/lock primitive | P01 host persistence | P02 profile persistence | 기술 primitive만 재사용하고 canonical file과 transaction은 분리 |
| checkpoint payload + monotonic envelope | P01 | P03, P04, P05 | 각 phase가 새 run-local 필드와 round-trip test를 같은 phase에서 추가 |
| compendium discovered/defeated commit interface | P02 | P05, 미래 콘텐츠 | P02가 기존 encounter producer와 identity/union/exact timing을 소유하고 후속 콘텐츠는 신규 identity만 연결 |
| authored facility assignment profile | P03 | P04 | P03가 slot/tag/quota 입력, P04가 power/band/target 산정 소유 |
| conditional Shop/capstone manifestation | P03 | P05, 미래 콘텐츠 | P03가 generic mechanism, P05는 Shop guarantee, 미래 Goal은 hidden capstone trigger 소유 |
| role difficulty profile | P04 | P05, 미래 콘텐츠 | P04가 계산/validation, P05는 remanifestation, 미래 Goal은 hidden boss role 사용 |
| Unity root/non-combat projection | 각 생산 phase | 다음 phase | wire 변경을 마지막에 몰지 않고 producer와 같은 phase에 소비자까지 수직 완료 |

Seam consumer가 predecessor의 public contract를 materially 바꿔야 하면 조용히 덮어쓰지 않는다.
영향을 predecessor completion review에 다시 연결하고 필요하면 해당 phase를 correction_required로
되돌린 뒤 같은 review를 재실행한다.

## 20. DTO, Data, Save Migration Matrix

| Phase | DTO migration | live RON/data migration | save/persistence migration |
| --- | --- | --- | --- |
| P00 | Support/Rest command/result/selected-event 제거, SavePoint/Gate preview | Support 제거와 final tagged TraumaRecovery | process-local checkpoint 의미 정리; durable file 없음 |
| P01 | failure decision, checkpoint/continue/status projection | failure/recovery/Treatment policy | 신규 versioned active-run envelope |
| P02 | placeholder/discovered/defeated/storage status | authored compendium order/metadata 필요 시 | 신규 versioned local profile envelope |
| P03 | template/semantic slot/topology/manifestation projection | authored facility templates/assignment profiles | active-run/checkpoint map state 확장 |
| P04 | power/anchor/difficulty/readiness projection | power model와 difficulty profiles | active-run/checkpoint power state 확장 |
| P05 | merchandise/monolith/progression/exchange projection | monolith, remanifestation encounters, shops, exchange VN, rewards | active-run/checkpoint monolith·residual·corrupted-key state 확장; remanifestation profile producer 추가 |
| 미래 hidden-boss Goal (현재 master 밖) | hidden eligibility/VN/capstone/completion projection | hidden actor/skill/encounter, administrator VN, capstone, final-boss classification | active-run/checkpoint hidden-capstone state 확장; hidden-boss profile producer 추가 |

Intermediate phase build의 active-run/profile file을 다음 phase build와 호환해야 한다는 배포 정책은
현재 승인돼 있지 않다. Phase 사이에 실제 배포 또는 기존 file 유지가 요구되면 migration version,
지원 범위, 제거 조건과 tests를 사용자 승인받기 전까지 해당 schema 변경을 중단한다. 승인 없는
`serde(default)`, alias, dual-read 또는 silent reset으로 문제를 숨기지 않는다.

## 21. Child Goal 공통 요구사항

각 child Goal은 최소 다음을 포함한다.

- 사용자-visible 문제와 완료 조건
- 현재 runtime/live data/DTO/Unity/baseline 근거
- source of truth와 소유 계층
- phase가 유지할 master 불변식과 seam
- 명시적 범위/비범위
- 승인된 decision section과 아직 결정되지 않은 항목
- DTO/data/save migration 여부
- focused/broad validation과 실제 test count 기록법
- dirty worktree overlap audit
- compatibility 요구가 생길 때의 사용자 승인 gate
- 정책 결정, 외부 차단, 안전하지 않은 diff merge와 dependency 중단 조건
- canonical 갱신 목록
- completion review evidence mapping

Child Goal은 활성 phase 밖의 독립 문제를 구현하거나 새 master를 자동 생성하지 않는다. 범위 밖
문제는 child `EXPERIMENT_NOTES.md`에 후보로만 기록한다.

## 22. 검증 정책

### 공통 baseline

각 phase는 production 변경 전에 현재 dirty worktree에서 자신의 영향을 직접 측정하는 baseline을
다시 확보한다. 이전 phase 결과는 참고 가능하지만 관련 file/live data/Unity가 변했다면 재검증한다.

각 명령에는 다음을 기록한다.

- 정확한 명령 또는 runner job
- 실제 실행 test 수, pass/fail/skip 수
- runner가 수를 제공하지 않으면 그 사실
- 기존 실패와 phase 변경 뒤 새 실패의 구분
- 실패 원인, 수정/되돌림과 재실행 결과

### 최소 broad 후보

실제 phase blast radius에 맞춰 조정하되 campaign 종료 전 최소 다음 표면을 증명한다.

```text
cargo check -p game_core
cargo check -p game_server
cargo test -p game_core --test ron_loading -- --nocapture
cargo test -p game_core
cargo test -p game_server
```

Unity-facing phase는 접근 가능한 EditMode/PlayMode test, server deserialize/mapping test와 필요한
WebSocket probe를 추가한다. Live server/probe 경로는 `docs/probe_registry.md`를 따른다. Compile만
통과하거나 mock끼리만 맞는 검증은 사용자-visible 완료 증거로 충분하지 않다.

## 23. 중단 조건과 Status

다음 조건에서는 영향받는 production 변경을 중단한다.

- 승인된 decision으로 답할 수 없는 gameplay/UX/schema/balance/content/save 의미가 발견됨
- intermediate release file compatibility가 필요하지만 정책이 없음
- 필수 Unity 저장소, asset, runner, host filesystem API나 권한에 접근할 수 없음
- active phase 파일의 기존 사용자 변경과 안전하게 병합할 수 없음
- 선행 phase의 source of truth나 exit gate가 실제로 불완전함
- 두 phase가 같은 public schema/transaction을 충돌하는 방식으로 소유하려 함
- 필수 baseline이나 변경 후 검증이 없어 회귀를 구분할 수 없음

사용 상태:

- `active`: 안전하게 조사/구현/검증 가능
- `blocked_by_dependency`: predecessor Complete/High 전에는 시작하지 않음
- `scope_decision_required`: 독립 source-of-truth 경계를 분리/선택해야 함
- `policy_decision_required`: 사용자-visible 의미 결정이 필요함
- `deferred_by_user`: 범위와 prerequisite는 알려졌지만 사용자가 후속 구현을 명시적으로 보류함
- `externally_blocked`: 필수 외부 요소가 없어 완료 검증 불가
- `correction_required`: completion review가 material in-scope defect를 발견함
- `complete`: 모든 exit gate와 필수 검증·review를 통과함

정책 결정 보고서 작성 자체를 구현 완료로 표시하지 않는다. 해당 child와 master를
`policy_decision_required`로 두고 필요한 사용자 입력을 명시한다. 사용자가 결정하면 같은 child를
재개한다.

## 24. 명시적 비범위

- 다섯 decision record에 직접 필요하지 않은 전투 runtime/refactor
- 관련 없는 직원, 스킬, 장비, 이벤트 또는 UI redesign
- 승인된 정책을 넘어서는 추가 스토리 분기나 엔딩 서사
- provisional combat power 이후의 최종 장기 balance formula 연구
- 새 cloud/account sync, 온라인 profile merge와 cross-device persistence
- 여러 OS에서의 신규 launcher/installer 설계
- 현재 dirty worktree의 무관한 삭제/로그/debug artifact 정리
- 사용자 승인 없는 과거 배포 save/schema compatibility

## 25. Master 완료 조건

다음이 모두 충족될 때만 Status를 `complete`, Implementation complete를 `true`로 변경한다.

- P00-P05가 각자 사용자-visible exit gate와 필수 focused/broad 검증을 통과했다.
- 각 phase 직후 독립 completion review가 `Complete / High`를 판정했다.
- P07 final review가 모든 cross-phase seam과 decision 완료 조건을 실제 evidence로 재검증했다.
- Core, live RON, game_server, 외부 Unity producer/consumer와 canonical이 같은 계약을 사용한다.
- active-run/checkpoint/profile의 rollback, monotonic, atomicity와 corruption invariants가 보존된다.
- legacy Support/Rest, procedural placeholder authority, competing floor scaling과 old Elite completion
  path가 승인된 제거 범위 안에서 정리됐다.
- 승인되지 않은 compatibility/fallback/dual schema가 남지 않았다.
- 실행한 모든 검증과 실제 runner 결과가 child/master `EXPERIMENTS.md`에 기록됐다.
- 기존 baseline failure와 새 failure가 구분되고 필수 미검증 항목이 없다.
- 범위 밖 dirty worktree의 사용자 변경이 보존됐다.
- durable canonical이 실제 구현으로 갱신되고 decision record에는 implementation disposition/evidence
  pointer가 추가됐다.

## 26. 현재 다음 작업

P00~P05와 P07은 모두 Complete / High이며 이 master는 완료됐다. 후속 시스템 구현 작업은 없다.
실제 hidden boss 구현은 README에 기록된 현재 master 밖의 후반 콘텐츠 Goal이다. 콘텐츠 정책 확정과
사용자의 명시적 재개 전에는 actor/skill/encounter/VN/capstone/특수 완료를 구현하지 않는다.
