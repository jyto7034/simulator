# DefenseRoute 전술 지휘 구현 Master Goal

Status: complete

Implementation Complete: true

Goal Type: master

Execution Mode: orchestration_only

Production Mutation Authorized: no — production 변경은 활성 Phase Goal에서만 수행

Current Phase: none — all phases complete

Goal Name: `defense_route_tactical_command_master`

Target Repository: `/mnt/f/work/simulator`

Audit Finding ID: 없음

## 문서 등급과 책임

이 문서는 DefenseRoute 전술 지휘 정책 `DR-01`~`DR-17`을 안전한 순서로 구현하는
명시적으로 승인된 Master Goal의 실행 색인이다. 2026-08-06 사용자는 서로 독립적인 구현
경계를 하나의 거대 단일 Goal로 취급하는 대신, 각 경계를 순차 Phase로 실행하고 Master가
전체 순서와 완료를 조정하는 구조를 승인했다.

Master Goal은 다음 책임만 소유한다.

- 전체 사용자-visible 결과와 공통 불변식
- 정책 ID와 Phase coverage
- Phase 선행 관계, 활성 상태와 완료 감사 게이트
- 교차 Phase migration 전략과 최종 통합 검증
- 전체 완료 판정

실제 코드·live data·server·canonical 변경은 활성 Phase의 별도 단일 Goal에서만 수행한다.
각 Phase Goal은 자신의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 가지며 하나의
source-of-truth 또는 밀접하게 결합된 하나의 완결된 계약 경계만 소유한다. 이 Master 문서는
gameplay, runtime, data, save 또는 wire 계약의 source of truth가 아니다.

## 승인된 Goal 설명

확정된 DefenseRoute 전술 지휘 정책 `DR-01`~`DR-17`을 Core runtime, production live data와
game_server까지 완결된 vertical slice로 구현한다. 관련 정책을 안전한 Phase로 묶고 선행
Phase를 완료 감사한 뒤 다음 Phase를 활성화한다.

Unity runtime·contract·tests는 사용자의 명시적 지시에 따라 이 Master의 구현 및 완료 범위에서
제외한다. 따라서 이 Master의 완료는 Core/live data/game_server 계약 완료를 뜻하며 Unity UI와
외부 Unity consumer 구현 완료를 뜻하지 않는다.

## 해결할 사용자-visible 문제

확정 정책은 현재 하나의 일관된 전술 지휘 체계로 구현되어 있지 않다. 전술 자원, 태세,
Ground/Platform 효과, 전술 이동, 자리 교대, 수동 지정 스킬과 현장 작업을 서로 다른 임시 의미나
fallback 없이 하나의 결정론적 Core authority 아래 제공해야 한다.

## 전체 사용자-visible 완료 조건

다음 조건을 모두 충족해야 Master를 완료할 수 있다.

- `DR-01`~`DR-17`의 확정 의미와 tuning이 누락 또는 축소 없이 구현된다.
- 회수·재배치, Hard CC·Silence, 전술 자원과 무기 전술 속성이 공통 기반 계약으로 동작한다.
- Attack/Defense, Ground DefenseFocus와 Platform Rest가 확정된 상태 전이와 효과를 가진다.
- 전술 이동과 자리 교대가 서로 다른 명령 의미를 유지하면서 공통 목적지 검증을 사용한다.
- 수동 지정 스킬이 명시적 대상과 실패 규칙을 따르며 암묵적 fallback이나 retarget을 만들지 않는다.
- 현장 작업이 배치·회수·사망·Hard CC·이동·교대와 일관되게 통합된다.
- Core state, checkpoint/event, live data와 game_server 생산자·소비자가 같은 승인 계약을 사용한다.
- 각 Phase의 focused/broad 검증과 완료 감사가 통과하고 마지막 교차 Phase 검증이 통과한다.
- 관련 canonical 문서가 실제 구현과 일치하며 승인되지 않은 compatibility가 남지 않는다.

## 승인 근거와 정책 source

확정된 목표 의미와 tuning의 source는 다음 decision record다.

- [DefenseRoute 전술 지휘 목표 정책 결정 — 2026-08-06](../../decisions/defense_route_tactical_command_policy_2026-08-06.md)

정책 검토 과정, 시점 code/live data/server 대조와 난이도 evidence는 다음 동결 문서에 있다.

- [DefenseRoute 전술 지휘 확장 임시 조사·설계 노트](../../tmp/defense_route_tactical_command_design_notes_2026-08-04.ko.md)

임시 문서는 Phase 0에서 historical evidence로 동결됐다. 과거 초안, 초기 권고안과 조사 당시 runtime
설명은 decision record의 확정 정책을 대체하지 않는다.

## 정책 coverage

정책 ID는 원래 검토 번호를 안정적으로 보존한다. Phase 재배치는 정책 의미를 변경하지 않는다.

| 정책 | 확정 주제 | 구현 Phase |
| --- | --- | --- |
| `DR-01` | 전투 목표 | Phase 0 |
| `DR-02` | 전술 포인트 | Phase 3 |
| `DR-03` | 기존 회수·재배치 | Phase 1 |
| `DR-04` | 전술 순간이동 | Phase 6 |
| `DR-05` | 아군 타일 점유와 배치 검사 | Phase 6 |
| `DR-06` | 공격/방어 태세 | Phase 5 |
| `DR-07` | 태세 전환 중 행동 처리 | Phase 5 |
| `DR-08` | Ground 방어 집중 | Phase 5 |
| `DR-09` | Platform 휴식 | Phase 5 |
| `DR-10` | 무기 데이터 소유권 | Phase 4 |
| `DR-11` | 무기 없는 직원 | Phase 4 |
| `DR-12` | Ground/Platform 순간이동 | Phase 6 |
| `DR-13` | Hard CC와 Silence | Phase 2 |
| `DR-14` | 자리 교대 | Phase 7 |
| `DR-15` | Pause 중 명령 | Phase 0 및 이후 모든 명령 Phase의 감사 게이트 |
| `DR-16` | 수동 지정 스킬 | Phase 8 |
| `DR-17` | 현장 작업 | Phase 9 |

확정된 대표 tuning은 다음과 같다. 정확한 의미와 예외는 정책 decision record가 소유한다.

| 정책 | 확정 tuning |
| --- | --- |
| `DR-02` | 초기 20, 최대 99, 전투 시간 1초당 1 회복, 배치 비용 10 |
| `DR-04` | 비용 5, 직원별 cooldown 10,000ms |
| `DR-06` | 태세 전환 cooldown 1,500ms |
| `DR-09` | 1,000ms tick, HP 3 및 정신력 5 회복 |
| `DR-14` | 교대 1회 비용 5, 양쪽 직원 cooldown 10,000ms |
| `DR-17` | 첫 live 작업: `(2, 6)`, 10초, 수리 100 |

## 현재 runtime 및 live data 근거

번호별 실제 기반, 일치·불일치, 필요한 구현 범위, 난이도 reference와 focused test 결과는 임시
조사 문서에 기록되어 있다. Master 초기화 시점의 종합 판정은 다음과 같다.

- `DR-01`~`DR-17`의 의미 정책, 난이도와 필요한 tuning/첫 live content는 모두 사용자 승인됨.
- 기존 runtime에는 배치, 회수, 전투 command/event/checkpoint, 무기·스킬 live data와 server 전달
  seam이 있으나 확정된 전술 지휘 전체 vertical slice가 구현된 것은 아님.
- 기반 존재는 확정 정책 구현 완료의 증거로 취급하지 않음.
- 각 Phase 시작 시 관련 파일의 현재 diff와 runtime/live data 경로를 다시 확인하고 baseline을
  확보해야 함.
- Phase 시작 전 파일 변경으로 기존 조사 근거가 달라졌다면 재조사 이유와 결과를 해당 Phase에
  기록해야 함.

## Source of truth와 소유 계층

현재 동작 판정의 우선순위는 저장소 문서 governance를 따른다.

1. 실제 Core/game_server runtime code
2. production live RON/data
3. 접근 가능한 wire producer와 consumer
4. 최신 durable canonical 문서
5. 이 Master, Phase Goal과 과거 조사 기록

구현 후 계약 소유 위치는 다음처럼 분리한다.

| 책임 | 소유 계층 또는 canonical |
| --- | --- |
| 플레이어-visible 규칙과 확정 tuning | `core_enhanced/docs/game_rulebook.md` |
| 결정론, state, event/checkpoint와 rollback 불변식 | `core_enhanced/docs/core_runtime_contract.md` |
| 스킬 대상 지정과 delivery 의미 | `core_enhanced/docs/skill_target_contract.md` |
| live RON schema, authoring과 validation | `core_enhanced/docs/data_loading_contract.md` |
| command/result 및 server message ordering | 관련 `game_server/docs` canonical과 실제 server mapping |
| 승인 배경과 목표 정책 snapshot | Phase 0에서 생성할 decision record |
| 실행 순서와 완료 증거 | 이 Master와 각 Phase Goal |

Unity canonical과 runtime은 이번 Master에서 접근·갱신·검증하지 않는다.

## 유지할 공통 불변식

- Core가 전투 상태와 명령 수락 여부의 authority다.
- Pause는 전투 simulation clock을 진행시키지 않으며 승인된 즉시 명령 의미만 허용한다.
- 명령 검증 실패는 비용, cooldown 또는 부분 state mutation을 남기지 않는다.
- 이동·교대처럼 여러 상태가 바뀌는 명령은 사전 검증 뒤 원자적으로 commit한다.
- 동일 입력, live data, seed와 명령 순서는 동일한 state/event 결과를 만든다.
- released delivery와 unreleased action 취소 의미를 구분한다.
- checkpoint와 reconnect는 저장된 authoritative state를 복원하며 누락 시간을 catch-up하지 않는다.
- 기존 직원 identity, HP, buff, resonance와 정책상 유지되는 cooldown을 임의로 재생성하지 않는다.
- Core 계약을 server나 다른 소비자가 재추론하지 않는다.
- 승인 없는 fallback, retarget, silent recovery, dual-read/write 또는 schema alias를 추가하지 않는다.
- 미래 Phase를 위한 빈 enum, placeholder state 또는 production 추상화를 미리 만들지 않는다.
- 관련 없는 dirty 변경을 보존하고 범위 밖 파일을 정리하거나 포맷하지 않는다.

## Master 범위

- 확정 정책 `DR-01`~`DR-17`의 Core implementation
- 필요한 production live RON/schema/validation
- 필요한 game_server command/result/state/event transport와 persistence mapping
- Core/server focused test, 영향 범위 broad test와 최종 교차 Phase 검증
- 각 Phase와 동시에 수행하는 관련 canonical 문서 갱신
- strict version cutover와 승인된 state/checkpoint migration
- 각 Phase 완료 감사와 Master 최종 감사

## 명시적 비범위

- 외부 Unity 문서, runtime, command/UI, store, presentation과 Unity tests
- 확정 목록에 없는 별도 gameplay, 콘텐츠 또는 밸런스 변경
- 임의 player pathfinding, reservation, pending movement 또는 자동 swap fallback
- 전술 이동과 독립적인 보행/Platform traversal 시스템
- 범용 필드 오브젝트 framework나 확정되지 않은 추가 작업 콘텐츠
- `PlayerTile`을 사용하는 첫 live skill 추가
- 별도 스킬 cooldown 체계
- 승인되지 않은 save/wire/data compatibility
- 관련 없는 리팩토링, 전면 재작성 또는 미래 기능 선행 추상화

## 미결정 정책

현재 승인 범위의 의미 정책과 tuning에는 알려진 미결정 항목이 없다. 구현 조사에서 새로운
사용자-visible 선택, DTO shape 선택, save migration, compatibility 또는 밸런스 결정이 발견되면
해당 Phase를 `policy_decision_required`로 전환하고 결정에 영향받는 변경을 중단한다.

## Phase 실행 순서와 상태

한 번에 하나의 Phase만 활성화한다. 이전 Phase가 완료 감사에서 `complete`가 되기 전에는 다음
Phase의 production 조사·수정·테스트를 시작하지 않는다. 다음 child goal 이름은 Phase 활성화 시
사용자 승인 또는 명시적 명명 위임을 받은 뒤 확정한다.

| Phase | 계약 경계 | 정책 | 선행 Phase | 계획 난이도 | 상태 |
| --- | --- | --- | --- | --- | --- |
| 0 | 정책/source-of-truth 분리와 baseline | `DR-01`, `DR-15`, 전체 traceability | 없음 | 문서·baseline | [complete](../defense_route_tactical_command_phase_00_policy_baseline/COMPLETION_AUDIT.md) |
| 1 | 회수·재배치 action lifecycle | `DR-03` | 0 | 2/5 | [complete](../defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle/COMPLETION_AUDIT.md) |
| 2 | 중앙 Hard CC/Silence capability | `DR-13` | 1 | 3/5 | [complete](../defense_route_tactical_command_phase_02_hard_cc_silence_capability/COMPLETION_AUDIT.md) |
| 3 | 전술 자원 authority | `DR-02` | 2 | 3/5 | [complete](../defense_route_tactical_command_phase_03_tactical_resource_authority/COMPLETION_AUDIT.md) |
| 4 | 무기 전술 계약과 weapon-required gate | `DR-10`, `DR-11` | 3 | 3/5 | [complete](../defense_route_tactical_command_phase_04_weapon_tactical_contract/COMPLETION_AUDIT.md) |
| 5 | Attack/Defense와 Ground/Platform 효과 | `DR-06`~`DR-09` | 4 | 4/5 | [complete](../defense_route_tactical_command_phase_05_stance_tile_effects/COMPLETION_AUDIT.md) |
| 6 | 전술 이동, 점유와 tile effect 원자 전환 | `DR-04`, `DR-05`, `DR-12` | 5 | 3/5 | [complete](../defense_route_tactical_command_phase_06_relocation_occupancy/COMPLETION_AUDIT.md) |
| 7 | 자리 교대 | `DR-14` | 6 | 3/5 | [complete](../defense_route_tactical_command_phase_07_atomic_swap/COMPLETION_AUDIT.md) |
| 8 | 수동 지정 스킬 | `DR-16` | 7 | 3/5 | [complete](../defense_route_tactical_command_phase_08_manual_skill_targeting/COMPLETION_AUDIT.md) |
| 9 | 현장 작업과 최종 교차 통합 | `DR-17` | 8 | 3/5 | [complete](../defense_route_tactical_command_phase_09_field_work_integration/COMPLETION_AUDIT.md) |

난이도는 Phase 시작 시 현재 tree와 선행 Phase 결과를 기준으로 다시 확인하되, 사용자-visible
정책을 축소하여 난이도를 낮추지 않는다.

## Phase별 구현 경계와 완료 조건

### Phase 0 — 정책/source-of-truth 분리와 baseline

- 확정 정책만 담은 간결한 decision record를 생성한다.
- 현재 거대 임시 문서를 시점 조사·난이도 evidence로 동결하고 superseding 문서를 링크한다.
- `DR-01`~`DR-17`과 Phase coverage가 한 번씩 모두 연결되는지 확인한다.
- 현재 runtime을 미래 정책이 이미 구현된 것처럼 canonical에 기록하지 않는다.
- 관련 baseline, version ownership과 Phase별 canonical 갱신 위치를 확정한다.
- `DR-01`, `DR-15`의 현재 구현 일치를 focused evidence로 확인하고 필요한 범위만 처리한다.

### Phase 1 — 회수·재배치 action lifecycle

- 회수 시 unreleased attack과 pending cast를 취소한다.
- 이미 released된 projectile와 GroundFixed delivery는 정책 수명주기를 유지한다.
- source-owned follow/channel/aura 계열 효과는 정책대로 종료한다.
- HP, 재배치 잠금과 최신 조건을 유지하고 공통 올림 비용으로 재배치한다.

### Phase 2 — 중앙 Hard CC/Silence capability

- Hard CC와 Silence를 중앙 capability로 판정한다.
- 중첩 Hard CC, 최대 해제 시각, unreleased 취소와 released delivery 유지 규칙을 구현한다.
- Silence는 새 skill 시작만 차단한다.
- control state와 reason이 checkpoint/recovery에서 보존된다.

### Phase 3 — 전술 자원 authority

- 전술 자원을 배치 상태에서 독립된 authoritative state로 분리한다.
- 확정 자원 경제와 run policy ownership을 적용한다.
- legacy 이름을 strict cutover하고 Core/checkpoint/server가 하나의 shape만 사용한다.

### Phase 4 — 무기 전술 계약과 weapon-required gate

- starter training sword를 포함한 확정 30개 무기가 전술 affinity/block/Ground/Platform profile을
  명시적으로 소유한다.
- 비무장 직원은 safe-zone 상태는 허용하되 battle gameplay 진입은 `weapon_required`로 거부한다.
- active-run/data version은 승인된 strict cutover를 사용한다.

### Phase 5 — Attack/Defense와 Ground/Platform 효과

- Attack/Defense state와 1,500ms 전환 cooldown을 구현한다.
- committed action 완료와 이후 새 시작 차단을 구분한다.
- 현재 tile과 weapon profile로 Ground DefenseFocus와 Platform Rest를 결정한다.
- Platform Rest는 1,000ms마다 HP 3·정신력 5를 회복하고 Hard CC/Pause 시간은 catch-up하지 않는다.
- Ground/Platform 효과 어느 쪽도 전술 포인트를 생산하지 않는다.

### Phase 6 — 전술 이동, 점유와 tile effect 원자 전환

- 목적지의 유효성, 장애물, zone, affinity, player tactical slot과 아군 점유를 공통 검증한다.
- 기존 단일 배치는 다른 아군이 점유한 타일을 거부한다.
- 전술 이동은 Defense 상태에서 비용 5와 직원별 10,000ms cooldown으로 동작한다.
- 성공 시 위치·방향·engagement·공간 및 tile 효과를 원자적으로 갱신한다.
- 점유 목적지는 자동 밀어내기나 swap fallback 없이 거부한다.

### Phase 7 — 자리 교대

- 자리 교대를 전술 이동 fallback이 아닌 명시적 command로 구현한다.
- 양쪽 조건을 사전 검증한 뒤 위치와 방향을 원자적으로 교환한다.
- 비용 5를 한 번 소비하고 양쪽 직원에 10,000ms cooldown을 적용한다.
- 양쪽 공간 및 tile 효과를 batch reconcile한다.

### Phase 8 — 수동 지정 스킬

- `Self`, `AutomaticUnit`, `PlayerUnit`, `PlayerTile` 대상을 명시적으로 구분한다.
- 모든 live skill의 targeting mode를 strict validation한다.
- 실패 시 fallback, retarget과 자원 환급을 만들지 않는다.
- Black Round progression의 첫 live 수동 대상을 `PlayerUnit`으로 적용한다.
- `PlayerTile` engine과 test는 제공하되 첫 live content는 추가하지 않는다.

### Phase 9 — 현장 작업과 최종 교차 통합

- stable object ID를 가진 좁은 FieldWork data/state와 실제 `Ready`/`FieldWorking` activity를 구현한다.
- 최종 점유 위치에서 작업을 자동 배정하고 event-time progress와 stale event no-op을 보장한다.
- Hard CC/Pause 정지는 catch-up하지 않으며 작업 중 공격·스킬·저지·태세·DefenseFocus·Rest를 막는다.
- 작업 중 회수·전술 이동·자리 교대는 허용하고 최종 위치에서 작업 상태를 재판정한다.
- 배치·재배치·회수·사망·이동·교대·checkpoint/server를 함께 통합한다.
- 첫 live 작업은 `(2, 6)`, 10초, 보호 대상 수리 100으로 제공한다.

## DTO, data와 save migration

### DTO migration

필요함. 전술 자원, 태세, 전술 이동, 자리 교대, 수동 지정과 현장 작업 command/state/event가
영향받는다. 정확한 producer/consumer와 serialized shape는 각 Phase 조사 후 기록한다. Unity DTO는
범위 밖이다.

### Data/live RON migration

필요함. run policy, 무기 전술 profile, skill targeting과 첫 FieldWork live content가 영향받는다.
승인되지 않은 alias나 dual schema는 만들지 않는다.

### Save/checkpoint migration

필요함. active-run과 battle checkpoint/setup shape가 영향받을 수 있다. 동일 production release에
포함되는 변경은 불필요한 연속 version bump를 합칠 수 있지만, 서로 다른 배포 단위라면 각 strict
cutover를 명시한다. compatibility reader는 별도 사용자 승인 없이는 추가하지 않는다.

## Contract/version 전략

- 미래 Phase 필드를 이전 Phase schema에 미리 추가하지 않는다.
- Phase 3과 Phase 4가 같은 미배포 release라면 active-run/data strict cutover를 하나의 최종 shape로
  합칠 수 있다.
- Phase 6과 Phase 9가 별도 release라면 setup/checkpoint version을 각각 올리고, 하나의 미배포
  branch/release라면 최종 shape에서 한 번 올릴 수 있다.
- event-log variant나 validator가 바뀌면 event-log version과 fixtures를 같은 Phase에서 갱신한다.
- 구 payload 지원이 필요해지면 구현 전에 지원 범위, 기간, 제거 조건과 테스트를 사용자에게
  승인받는다.

## Phase 완료 감사 게이트

각 Phase는 구현과 테스트가 끝났다는 이유만으로 완료하지 않는다. 다음 감사를 모두 통과한 뒤에만
`complete`로 변경하고 Master에서 다음 Phase를 활성화한다.

감사는 [Goal 완료 검토 가이드](../../goal_completion_review_guide.md)를 필수 기준으로 적용한다.
별도 audit Goal 생성은 선택 사항이지만 항목별 정책 재서술, runtime/data/외부 계약 evidence,
legacy/fallback audit, long-term fit, 명시적 verdict와 필요한 cross-component finding은 생략하지
않는다. 아래 항목은 이 Master가 추가로 요구하는 Phase gate다.

- 배정된 `DR-xx` 정책이 누락·축소·암묵적 의미 변경 없이 구현됐는지 대조
- command 진입점부터 authoritative state mutation까지 실제 runtime 경로 확인
- production live data가 새 경로를 사용하고 invalid data를 startup에서 거부하는지 확인
- Core DTO 생산자와 접근 가능한 server 소비자가 같은 serialized 계약을 사용하는지 확인
- 상태 전이 원자성, 결정론, Pause, cancellation, checkpoint/reconnect 불변식 확인
- 제거 대상 legacy symbol/schema/fallback이 runtime, data, fixture, mock, test와 문서에 남지 않았는지 검색
- focused 및 영향 범위 broad 검증의 정확한 명령과 실제 pass/fail/skip 수 기록
- 변경 전 baseline 실패와 새 실패 구분
- 관련 canonical 문서와 실제 구현 비교
- scoped diff, 범위 밖 변경 부재와 기존 사용자 dirty 변경 보존 확인
- 다음 Phase를 위한 빈 placeholder나 승인되지 않은 compatibility가 없는지 확인
- 접근하지 못한 소비자, 생략한 검증과 남은 위험 기록

감사 실패 시 실제 원인에 따라 Phase를 `active`, `scope_decision_required`,
`policy_decision_required` 또는 `externally_blocked`로 유지한다. 완료되지 않은 Phase를 건너뛰지
않는다.

## 필수 focused 검증

각 Phase Goal은 해당 command/state/data/transport 계약을 직접 증명하는 focused test를 조사 후
명시한다. 최소 검증 범주는 다음과 같다.

- 정상 command와 모든 확정 rejection reason
- 실패 시 비용/cooldown/state 무변경
- committed/released와 pending/unreleased 수명주기
- 중첩 상태, Pause, checkpoint/reconnect와 stale event
- exact serialization/deserialization shape와 server mapping
- production live RON loading과 invalid data rejection
- Phase에서 제거한 legacy contract 부재

정확한 명령과 예상 test 수는 runner를 확인하기 전까지 discovery pending이다.

## 필수 broad 검증

- 변경된 Core crate의 영향 범위 test suite
- 변경된 game_server의 영향 범위 test suite
- production live-data 전체 validation
- state/checkpoint/event-log 또는 active-run version이 바뀐 Phase의 persistence regression
- Phase 9 종료 후 `DR-01`~`DR-17` 교차 상호작용 통합 suite

정확한 명령, test binary와 test 수는 Phase별 조사 후 기록한다. compile 성공만으로 runtime 완료를
판정하지 않는다.

## 생략할 검증과 근거

- Unity runtime/contract/tests: 사용자가 구현되어 있지 않으며 앞으로 읽을 필요가 없다고 명시하여
  Master 범위에서 제외함.
- 현재 Phase와 무관한 콘텐츠 및 전체 repository test: Phase별 영향 근거 없이 기계적으로
  실행하지 않음. 최종 broad 범위는 실제 dependency 조사로 결정함.

생략 근거가 달라지거나 Core/server 완료에 필수인 외부 소비자가 발견되면 해당 Phase에 기록하고
필요하면 `externally_blocked`로 전환한다.

## 중단 조건

다음 중 하나가 발생하면 영향받는 변경을 중단한다.

- Phase가 두 개 이상의 독립 source-of-truth 경계로 다시 분리되어야 함
- 승인되지 않은 사용자-visible 정책 또는 tuning 선택이 필요함
- 승인되지 않은 compatibility, save migration 또는 wire migration이 필요함
- 필수 producer/consumer, live data, test runner 또는 저장소에 접근할 수 없음
- 기존 사용자 변경과 안전하게 병합할 수 없음
- baseline 없이 변경 전후 회귀를 판단할 수 없고 대체 증거도 불충분함
- 선행 Phase의 완료 감사가 끝나지 않음

범위 밖 문제는 해당 Phase `EXPERIMENT_NOTES.md`에 후속 후보로만 기록하고 자동으로 새 Goal이나
Master Goal을 만들거나 함께 구현하지 않는다.

## Compatibility

승인된 compatibility는 없다. strict cutover가 기본이다. fallback, dual-read, dual-write,
dual-schema 또는 alias가 필요하다고 판단되면 구현 전에 이유, 지원 배포 범위, 기간, 제거 조건과
compatibility 전용 테스트를 제시하고 사용자 승인을 받는다.

## Master 완료 감사

Phase 0~9가 모두 자체 완료 감사를 통과한 뒤 다음을 추가로 확인한다.

- `DR-01`~`DR-17` 각각에 정책 source, 구현 Phase, production 경로, focused test와 canonical 근거가 연결됨
- Phase 사이에 임시 schema, placeholder, 호환 shim 또는 중복 authority가 남지 않음
- 마지막 Core/live data/game_server shape가 하나의 strict contract로 일치함
- 최종 교차 Phase broad 검증이 실제로 실행되고 결과 수가 기록됨
- 모든 legacy 제거와 version cutover가 실제 repository 검색 및 fixture 검증으로 확인됨
- 초기 dirty 변경과 범위 밖 변경이 보존됨
- Unity 제외 범위와 후속 consumer 위험이 최종 보고에 명시됨

[Goal 완료 검토 가이드](../../goal_completion_review_guide.md)의 cross-component 절차와 verdict
기준을 Master 최종 감사에도 적용한다. 증거가 없는 완료 조건이 하나라도 남으면 Master Status를
`complete`로 변경하지 않는다.

## 현재 진행 상태

- Master Goal 이름과 Phase 구조: 사용자 승인 완료
- Master 필수 문서 초기화: 완료
- Phase 0: `Complete / High` 완료 감사 통과
- Phase 1: [Complete / High](../defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle/COMPLETION_AUDIT.md)
- Phase 1 production: release 전 action 취소, detached delivery 유지, source-bound cleanup과 공통 ceiling 재배치 비용 완료
- Phase 1 검증: focused 19, Core 855, server 54, RON 23 통과
- Phase 2 child Goal 이름과 문서: `defense_route_tactical_command_phase_02_hard_cc_silence_capability`으로 초기화하고 active 전환
- Phase 2: [Complete / High](../defense_route_tactical_command_phase_02_hard_cc_silence_capability/COMPLETION_AUDIT.md)
- Phase 3: [Complete / High](../defense_route_tactical_command_phase_03_tactical_resource_authority/COMPLETION_AUDIT.md)
- Phase 3 production: 분리된 tactical resource state/policy/effect/required DTO와 deploy/redeploy payment 완료
- Phase 3 검증: focused, Core 872, server 55, RON 23, fmt/diff 통과
- Phase 4 child Goal: `defense_route_tactical_command_phase_04_weapon_tactical_contract`으로 초기화하고 active 전환
- Phase 4: [Complete / High](../defense_route_tactical_command_phase_04_weapon_tactical_contract/COMPLETION_AUDIT.md)
- Phase 4 production: exact 30 weapon tactical profile, starter sword/reward exclusion, condition-priority
  weapon-required gate와 active-run Core V3 strict cutover 완료
- Phase 4 검증: Core 880, server 55, RON 24, all-target/fmt/diff/static 통과
- Phase 5 child Goal: `defense_route_tactical_command_phase_05_stance_tile_effects`로 초기화하고 active 전환
- Phase 5: [Complete / High](../defense_route_tactical_command_phase_05_stance_tile_effects/COMPLETION_AUDIT.md)
- Phase 6 child Goal: `defense_route_tactical_command_phase_06_relocation_occupancy`로 초기화하고 active 전환
- Phase 6: [Complete / High](../defense_route_tactical_command_phase_06_relocation_occupancy/COMPLETION_AUDIT.md)
- Phase 6 production: same-instance relocation, common allied logical occupancy, tracking 보존과 Any tile effect
  atomic transition 완료
- Phase 6 검증: Core 895, server 55, RON 24, all-target/fmt/diff/static 통과
- Phase 7 child Goal: `defense_route_tactical_command_phase_07_atomic_swap`로 초기화하고 active 전환
- Phase 7: [Complete / High](../defense_route_tactical_command_phase_07_atomic_swap/COMPLETION_AUDIT.md)
- Phase 7 production: explicit two-employee batch swap, pair-aware occupancy, cost-once/cooldown-both, one event/revision 완료
- Phase 7 검증: Core 898, server 55, RON 24, all-target/fmt/diff/static 통과
- Phase 8 child Goal: `defense_route_tactical_command_phase_08_manual_skill_targeting`으로 초기화하고 active 전환
- Phase 8: [Complete / High](../defense_route_tactical_command_phase_08_manual_skill_targeting/COMPLETION_AUDIT.md)
- Phase 8 production: required cast selection, explicit target acceptance/capture, tactical focus/playback와
  Black Round PlayerUnit strict live cutover 완료
- Phase 8 검증: Core 903, server 56, RON 24, all-target/fmt/diff/static 통과
- Phase 9 child Goal: `defense_route_tactical_command_phase_09_field_work_integration`으로 초기화하고 active 전환
- Phase 9: [Complete / High](../defense_route_tactical_command_phase_09_field_work_integration/COMPLETION_AUDIT.md)
- Phase 9 production: strict standalone FieldWork setup, object-owned Preserve progress, final-occupancy assignment,
  FieldWorking activity/gates, generation-safe typed repair와 exact Freischutz live slice 완료
- Phase 9 검증: Core 910, server 56, RON 24, all-target/fmt/diff/static 통과
- Master final audit: [Complete / High](COMPLETION_AUDIT.md)
- `DR-01`~`DR-17` policy/Phase/runtime/live-data/server/canonical traceability와 strict final versions 감사 완료
