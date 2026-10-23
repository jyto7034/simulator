# S09 전장·이동·저지 대표 조사와 개선 계획

작성·갱신일: 2026-10-18

현재 단계: **대표 조사·R1/D1 구현·영향 검증·분리된 완료 검토 완료**. 구현 결과는 §7에 기록한다.
§1–5는 변경 전 조사·계획·재사용 근거이며, 당시의 미구현·새 테스트 0회 표기는 그 시점의 기록이다.
다음은 §6의 S10 대표 조사다. 전체 Core 완료를 뜻하지 않는다.

## 1 범위·기준과 기존 근거

[출발점](README.md)의 다음 행동에 따라 전장/route 입력 → 연속 이동 tick → 접촉 저지와 상태 갱신,
전술 이동/교대 → 점유·작업·효과 재계산 → world 공개를 대표 흐름으로 조사했다.
[판단 기준](../refactoring_criteria.ko.md)의 책임·상태 소유권·수명·실패·중복·변경 비용을 적용하고,
[실행 지침](../system_refactoring_execution_guide.ko.md)의 조사 종료·근거 재사용·분리된 검토 규칙을 따른다.

- S08의 timestamp/candidate·유닛 수명·queue ordering과 [R2 완료 결과](S08_BATTLE_LIFECYCLE_REVIEW.ko.md#9-r2-구현검증과-완료-검토)를 재사용했다.
  R1/R2/D1을 다시 조사하거나 구현하지 않았다. 이동과 접하는 호출 위치·상태 갱신만 연결했다.
- [이번 선택 입력](evidence/s09_001/baseline.json) 59개가 이전 지도/S08 입력과 일치한다.
  S08 R1/R2의 알려진 변경은 최신 구현 해시를 적용했다. 전체 의존성/콘텐츠를 새로 전수 동결한 것은 아니다.
- [기존 전수 결과](evidence/s08_full_001/RESULT.json)의 관련 28개와 R2 이후 관련 4개의 실제 통과 로그를
  [재사용 목록](evidence/s09_001/reused-tests.json)에 연결했다. 새 테스트 실행 0회이며 32개를 새로 통과시켰다고 집계하지 않는다.
- 주 계약은 [runtime contract](../core_runtime_contract.md)의 Tactical Relocation/Atomic Swap,
  Control Status, Battle Setup And Movement Presentation, Route And Battlefield Projection 및 지상 저지 절이다.
  [전투 조작](../game_rules/battle-controls.md)은 world 명령·연출 경계를 확인하는 데 사용했다.

S10의 모든 스킬/효과·보스 task 알고리즘, Unity 실제 렌더링, 서버 transport, release/선택 feature 전체 조합,
지원 최대 규모의 성능 측정은 이번 대표 조사 밖이다. active battle의 durable 저장 재개도 현재 지원 계약이 아니다.

## 2 실제 흐름과 상태 소유권

### 입력·이동·접촉

| 단계 | 실제 코드와 상태 | 판단 |
| --- | --- | --- |
| 전장/경로 입력 | `combat_preview/validation.rs`가 route 비어 있음·start/end·valid tile·obstacle·상하좌우 인접을 검사. `combat_setup/mission_policy.rs`·`enemy_spawns.rs`가 기본/웨이브 route cells를 scenario/unit plan에 전달 | production preview 검증과 내부 scenario fixture의 검증 수준을 구분. 내부 두 점 fixture를 production route 허용 증거로 사용하지 않음 |
| 전장 투영 | `battlefield/field.rs`가 valid tile·static obstacle 원본과 정렬된 조회 제공. `movement/engine.rs::continuous_static_obstacles`가 obstacle/void를 물리 입력으로 변환 | 경로·배치 유효 타일과 공격 범위 overlay의 의미는 별도 계약 |
| 시각 처리 | `sim.rs`의 ContinuousMovementTick이 최소 tick 간격을 확인하고 이동·공격 재평가·다음 tick 예약 | S08 timestamp candidate/실패 보존 경계 사용. 같은 시각 예약 중복은 queue 검사로 방지 |
| 목표 생성 | `planner.rs`가 active/can_move/root/blocked를 검사하고 independent actor route, boss directive, player FixedDefense/enemy route를 선택 | boss directive의 Goal/Suppress/Fallback과 일반 계획 책임 구분. FixedDefense는 공격 대상이 있어도 추격 이동 권한을 만들지 않음 |
| 이동 입력 | `RuntimeUnit.body`의 위치·반경·속도, route goal과 channel/root/action lock/active/Hard CC/저지 상태를 읽음 | 이동 가능 여부는 현재 권위 상태에서 파생. 별도 이동 중 bool을 권위로 두지 않음 |
| 물리 계산 | `run_continuous_movement_tick`가 UUID 순으로 정렬하고 Rapier backend에 body/static obstacle을 동기화. 지상 static 충돌과 보드 경계 보정, 공중 static 우회 | unit collider를 static 장애물로 사용하지 않음. 물리 backend는 매 tick 동기화하는 파생 실행 상태이며 독립 gameplay 위치 원본이 아님 |
| 접촉 저지 | `blocking.rs::constrain_block_contacts`가 backend의 실제 선분에서 최초 접촉을 구하고 용량을 예약. route progress→spawn order→ID 우선순위 후 출력은 unit ID 순으로 복구 | 고속 이동 관통 방지와 결정론을 같은 경계에서 처리. 원형 blocker와 independent wall의 사각 sweep은 다른 정책이므로 무리하게 통합하지 않음 |
| 반영·소비 | `apply_continuous_movement_outputs`가 body/facing을 반영하고 immutable segment log와 `active_movement_segments`를 갱신. 이후 block refresh | body는 tick 계산 위치, segment는 시간 구간의 보간 상태. `commands.rs::sample_unit_body_at`와 projectile consumer가 중간 시각을 샘플링하므로 segment를 단순 UI 캐시로 제거할 수 없음 |

`refresh_block_state`는 유효한 기존 관계를 먼저 보존하고 남은 용량에 새 상대를 배정한다.
`enemy_to_blocker`/`blocker_to_enemies`는 같은 관계의 양방향 표현이며 현재 구현이 함께 갱신한다.
교대/이동은 기존 관계를 해제한 뒤 최종 위치에서 재구성한다. Hard CC, 사망·철수·반경 이탈은 유효성 검사에서 해제된다.
진행 중인 관계를 매번 거리 최솟값만으로 교체하지 않는 것은 의도된 incumbent 보존이다.

### 전술 이동·교대와 수명/실패

- `tactical_relocation.rs`의 공통 destination 검사가 valid/static tile·논리 점유를 확인하고,
  relocation은 actor 상태·Hard CC·unreleased action·cooldown·weapon affinity/profile·field-work cell을 검증한다.
  swap은 같은 gate에 두 참여자와 제3 점유자 제외 규칙, exclusive segment 검사를 더한다.
- `apply_tactical_position_update`는 같은 runtime identity의 body/anchor/facing/epoch와 cooldown을 갱신한다.
  swap은 두 유닛을 모두 검증하고 둘을 반영한 뒤 최종 occupancy로 field work를 reconcile한다.
  이전/최종 tactical effect를 한 번씩 대조하고 forced defense 또는 FieldWorking으로 이어진다.
  같은 타일 facing-only 변경은 기존 Rest cadence를 유지한다.
- 공개 Core mutation은 `runtime_transaction`의 candidate에서 수행한다. world는
  `with_staged_tactical_world_mutation`에서 active session을 staging하고 명령·자원 지불·revision·DTO를 준비한 뒤
  공개한다. future cursor/DTO 실패가 이미 지불한 원본 상태를 남기지 않는 경계는 기존 S01/S07와 일치한다.
- Rapier owner는 Core clone에서 Arc를 공유하다 실제 tick 쓰기 전에 controlled clone한다.
  clone 거절은 외부 control poll/transaction에서 실패로 확인된다. backend 내부 빈 result만 보고 성공으로 단정하지 않는다.
  backend의 retained history admission과 clone 후 동일 tick 결과는 기존 테스트로 확인했다.
- 사망은 S08 death 처리와 field-work/block 재계산, 철수는 segment/tactical 정리와 field-work/block 재계산을 사용한다.
  reset은 segment/last tick/backend를 초기화한다. Root·displacement·boss cancellation의 실제 segment 정리와
  `record_movement_stopped` 호출은 아래 R1의 무효 helper와 구분한다.

## 3 설계 평가와 분류

| 평가 항목 | 결과·근거 |
| --- | --- |
| 책임·의존 경계 | planner→공통 tick→Rapier resolver→block 보정→Core 반영 구조 유지. transport/I/O를 하위 이동 알고리즘에 추가할 근거 없음 |
| 상태 소유권 | body/시간 구간 segment/backend/양방향 block relation은 역할과 갱신 순서가 다름. 중복 필드라는 이유만으로 합치지 않음 |
| 수명·상태 전이 | 현재 동작하지 않는 `interrupt_movement`가 여섯 caller의 중단 책임처럼 남아 있음. **R1** |
| 실패·commit | Core candidate와 world staging 유지. final occupancy 후 effect 재계산이 swap의 중간 상태 노출을 막음 |
| 규칙 중복·변경 비용 | relocation/swap이 destination/position/effect helper를 이미 공유. 서로 다른 actor 수·로그·비용 경계를 범용 batch framework로 바꿀 근거 없음. R1의 죽은 분기는 제거 가치 있음 |
| 데이터·공개 범위 | validated preview→scenario/unit route→setup projection 연결 유지. 기존 공개 `ActionState`/`RuntimeUnit` API는 이번 후보에서 바꾸지 않음 |
| 결정론 | actor ID 정렬, route/spawn/ID priority, incumbent 보존, 출력 재정렬 확인. HashMap 저장 순서를 그대로 로그 순서로 사용하지 않음 |
| 관측·설명 | immutable segment와 lifecycle 위치 이벤트/DTO 구분 유지. 현재 없는 relocation 필드와 무효 중단 경로 설명은 **D1** |
| 검증 가능성 | 내부 geometry fixture와 실제 world command·projection 실패 검증을 구분해 재사용 가능. R1은 실제 caller의 로그/상태를 보존하는 검증으로 확인해야 함 |
| 성능·자원 | route center Vec 생성·per-comparison progress 계산·여러 block refresh·backend broad-phase/ignored set 구축 비용 신호가 있음. 현 성능 회귀나 예산 위반은 측정하지 않았으므로 최적화 후보로 확정하지 않음 |
| 저장/외부 소비자 | active battle durable resume 비적용. Unity 실물 소비·최대 규모·feature 조합은 미확인 범위로 유지 |

선택한 대표 흐름에서 **확정한 gameplay 동작 오류는 없다**. 다음은 구조 정리와 설명 불일치다.
테스트 통과만으로 전체 S09 무결점이나 tick 중간의 모든 제어/연출 조합까지 확인했다고 주장하지 않는다.

### R1 — 도달 불가능한 이동 중단 경로 제거

- **문제:** `movement/mod.rs::ActionState`는 `Idle | Dead` 두 variant뿐이다.
  `lifecycle.rs::interrupt_movement`의 `!matches!(..., Idle | Dead)`는 존재하는 모든 unit에서 false이고,
  없는 unit도 `is_some_and`가 false다. 따라서 함수는 항상 false를 반환하며 상태·로그를 바꾸지 않는다.
  [타입·조건·여섯 호출 위치 대조](evidence/s09_001/lifecycle-analysis.json)는 실행 테스트가 아닌 정적 증명이다.
- **영향:** attack/manual cast/auto cast/Hard CC/channel/death가 이 helper에 실제 중단을 맡기는 것처럼 보인다.
  cast의 `if interrupted` 예약과 death의 이중 branch는 도달 불가능한 선택지를 유지한다.
  현재 중단 규칙을 이해하거나 변경할 때 실제 input gate·segment 정리와 이 가짜 경로를 함께 추적해야 한다.
- **목표:** private helper와 무효 호출/후속 branch를 제거한다. death는 현재 항상 실행되는 fallback의
  lifecycle/HP/epoch/ActionState 변경과 단일 `MovementStopped(Died)` 기록을 그대로 남긴다.
  실제 중단 기록 함수·Root/displacement/boss 처리·input gate가 각자의 기존 책임을 계속 가진다.
- **대안 판단:** active segment 존재로 조건을 바꾸어 helper를 동작시키는 것은 event·epoch·재예약·projectile
  sampling 의미를 바꾸는 별도 동작 변경이다. 이번 리팩토링 대안으로 채택하지 않는다.

### D1 — 현재 이동 상태와 맞지 않는 설명 교정

`core_runtime_contract.md`의 Tactical Relocation 절은 body의 previous position/velocity/movement mode와
지속 goal을 함께 갱신한다고 설명한다. 실제 `UnitBody`는 position/radius/move_speed이며 relocation은
position/anchor/facing/ActionState/epoch·segment와 tactical cooldown을 갱신한다.
`sim.rs`의 자동 시전 주석도 무효 helper를 continuous movement 즉시 정지라고 설명한다.
계약의 identity·원자성·정지 정책을 새로 바꾸지 않고 실제 필드·lock/input gate를 설명하도록 교정한다.
자동 시전 주석은 R1에서 해당 블록을 제거할 때 함께 정리한다.

## 4 구현 계획과 수용 조건

R1을 먼저 동작 보존 정리하고 D1을 같은 변경 문맥에서 교정한 뒤, 별도 읽기 전용 완료 검토를 수행한다.

- **제품 경계:** `movement/lifecycle.rs`, `basic_attack.rs`, `sim.rs`, `sustained.rs`, `death_runtime.rs`의
  여섯 caller와 관련 import. 공개 enum/필드, movement tick·lock·control 판정·segment·queue 비교는 유지한다.
- **보존:** 현재 false 결과, death fallback의 위치·epoch 증가 횟수·로그 시간/cause/순서,
  manual/auto cast의 실제 lock deadline과 기존 scheduling, Hard CC retry, channel의 owned cancellation,
  Root/displacement/boss의 segment 처리, S08 R2 buff cleanup 순서와 candidate 실패 보존.
- **구조 수용:** private `interrupt_movement`와 여섯 호출·불가능 branch가 사라지고,
  실제 동작을 하지 않는 대체 wrapper를 남기지 않는다. 새로운 상태/공개 API/범용 lifecycle 계층은 만들지 않는다.
- **검증:** 이 기록의 변경 전 근거를 재사용한다. 수정 후 실제 basic attack/manual·auto cast/Hard CC/channel,
  death cleanup와 timestamp 실패 복구 사례를 한 명령으로 묶어 영향 검증한다.
  기존 사망 fixture가 stop event의 단일 기록/원인·epoch를 보호하지 않으면 그 공백만 먼저 보강한다.
  no-op 함수의 본문을 복제하는 새 테스트는 만들지 않는다.
- **검토:** 고정한 diff·원시 로그·caller 전후 대조로 제거가 새로운 stop event·segment 삭제·예약을 만들지 않았는지
  확인한다. D1은 실제 필드/쓰기 위치와 대조하고 문서 교정만으로 테스트를 다시 실행하지 않는다.
- **완료 구분:** 조사 종료 당시에는 구현 전이었다. R1/D1 구현·검토 완료 결과는 §7에 기록한다.
  성능 최적화나 중간 시각 정지 정책 변경을 R1에 묶지 않는다.

## 5 검증 근거·한계와 조사 완료 검토

[재사용 목록](evidence/s09_001/reused-tests.json)과 [원시 로그 위치 추출](evidence/s09_001/reused-tests.txt)에
32개 사례를 고정했다. 전수 로그의 28개는 route/입력·이동/충돌·저지·전술 위치·world 공개·finite/보간이고,
R2 이후 4개는 사망·철수 buff, timestamp 성공 prefix/실패 복구, 소환자 제거다.

관련 이동·전술 소스/테스트는 지도 기준선과 동일하고, 전수 실행 이후의 R2 변경은 이미 대조된 buff cleanup 추출이다.
수정된 caller의 주변 동작 보존은 S08 R2 구조 검증·완료 검토를 재사용한다.
전수 실행의 명령·환경은 `evidence/s08_full_001/commands.json`, R2 선택 명령은 `evidence/s08_r2_001/commands.json`이 소유한다.
이번에는 제품·테스트·fixture·설정을 변경하지 않았으므로 Cargo/build/full suite를 반복하지 않았다.

- 이 근거는 일반 route·지상/공중·선택한 저지/전술 실패/수명 경계의 대표 사례다. 모든 route 교차/보스 task/
  독립 object 진화 조합·중간 tick의 모든 control/연출 조합을 포괄하지 않는다.
- geometry fixture 일부는 직접 unit/goal을 조립한다. production 경로는 preview 검증/route projection과 world
  명령 사례로 별도로 연결하며 이를 Unity 실행이나 전체 production pack 실행으로 부풀리지 않는다.
- 성능 신호는 코드 비용 관찰이다. 시간/할당량/최대 규모 측정 없이 성능 문제·개선 수치를 확정하지 않는다.
- S08 comparator의 24쌍/31 variant 한계는 그대로이며 이번 이동 조사 때문에 다시 검사하지 않는다.

작성 후 현재 source·계약·로그와 후보/계획을 별도 시점에 읽기 전용으로 대조한 결과는
[조사 완료 검토](evidence/s09_001/review.json)에 기록한다. 같은 실행자의 검토이며 비참여 독립 감사는 아니다.
조사 당시 판정은 **선택한 대표 조사·계획 완료**였으며, 이후 제품 개선 완료 근거는 §7과 구분한다.

## 6 다음 재개 위치

후속 [S10 대표 조사·개선 계획](S10_ATTACK_SKILL_EFFECT_REVIEW.ko.md)이 완료됐다. 아래는 S09 종료 당시 인계다.
현재 다음 행동은 README와 S10 §6을 따른다.

**S10 공격·스킬·효과 대표 조사 → 동작·설계 품질의 유지/개선 판정 → 필요한 개선 계획**이다.
`core/basic_attack.rs`, `skill_runtime/**`, `targeting.rs`, `sim.rs`에서 실제 입력과 대표 스킬을 골라
시전→전달→해결→취소→후속 효과의 owner·상태·실패 경계를 연결한다.
projectile·sustained·boss·relic·resonance는 해당 대표 흐름에서 필요한 경계만 추적하고 상세 조사 범위를 정한다.

S08 timestamp/candidate·유닛 수명과 S09 이동/저지·전술 위치의 유효한 근거를 재사용한다.
기존 전수 로그와 §7의 선택 결과를 구분하며, 소스·입력·환경 변화 또는 검증 공백이 있는 부분만 확인한다.
S09 R1/D1을 재구현하거나 중간 시각 이동 중단 정책·성능 최적화를 자동으로 추가하지 않는다.

## 7 R1/D1 구현·검증과 완료 검토

### 변경과 구조 효과

- `movement/lifecycle.rs`의 private `interrupt_movement`와 basic attack·manual/auto cast·Hard CC·channel·death의
  여섯 호출을 제거했다. 기존 `ActionState`가 Idle/Dead뿐이어서 helper는 항상 false이고 부작용이 없었다.
  cast의 불가능한 tick 예약 분기와 사용하지 않는 import·local도 제거했다.
- death는 기존에 항상 실행되던 fallback을 같은 위치에 남겼다. lifecycle/HP/ActionState를 갱신하고
  `move_epoch`를 한 번 증가시킨 뒤 `MovementStopped(Died)`를 기록한다. R2 buff 정리와 사망 기록 순서를 유지했다.
- 실제 cast lock·Hard CC retry·channel 소유 취소와 Root/displacement/boss의 segment 처리를 보존했다.
  `record_movement_stopped` 본문과 공개 enum/필드도 유지한다. 새 wrapper·상태·공개 API는 추가하지 않았다.
- [runtime contract](../core_runtime_contract.md)의 Tactical Relocation 설명을 실제 body position·anchor·facing·
  action state·epoch·current target·segment 갱신으로 교정했다. 무효 자동 시전 블록의 즉시 중단 주석도 함께 제거했다.

효과는 실제 동작하지 않는 helper 하나·호출 여섯 곳과 불가능 분기를 추적할 필요가 없어진 것이다.
이동 소유권이나 gameplay 중단 정책을 바꾸거나 성능 개선 수치를 주장하는 작업은 아니다.
[변경 diff](evidence/s09_r1_001/source.diff)와 [구조 대조](evidence/s09_r1_001/structure-verification.json)는
사망 파일이 기존 항상 실행 경로와 공백을 제외하고 일치하며, 나머지 caller는 지정 제거 외에 동일함을 확인한다.

### 영향 검증

[변경 전 입력](evidence/s09_r1_001/baseline.json) 59개가 S09 조사 입력과 일치했다.
기존 32개 통과는 변경 전 근거로 재사용했다. 기존 실제 사망 테스트 한 곳에 단일 stop event의 시간·위치·원인,
`stop → buff expired → UnitDied` 순서, HP/lifecycle/ActionState와 단일 epoch 증가를 보강했다.
이 보강은 제품 변경 전에 먼저 통과시켜 기존 의미임을 확인했다.

| 실행 | 결과 | 시간·근거 |
| --- | --- | --- |
| 보강한 기존 사망 테스트, 제품 변경 전 | 1개 통과·실패 0 | 42.19초. [명령](evidence/s09_r1_001/baseline-command.json), [결과](evidence/s09_r1_001/baseline-command-result.json), [원시 로그](evidence/s09_r1_001/baseline-test.log) |
| 변경 후 호출부·인접 수명/control 집중 회귀 | 17개 통과·실패 0·ignore 0 | 43.69초(빌드 40.84초, 테스트 2.79초). [명령](evidence/s09_r1_001/commands.json), [결과](evidence/s09_r1_001/commands-result.json), [원시 로그](evidence/s09_r1_001/regression.log) |

17개는 사망·철수 buff, manual/auto cast, basic windup·manual focus·auto cast Hard CC 취소/재시도,
channel 공격 제한·aura·재입력 종료, slow/root 이동, pull·실패 복원, predeath 거절/회복,
timestamp 성공 prefix/실패 복구, 중첩 사망·치명타 반격 순서, 기본 이동/lock, segment 보간을 확인한다.
기본 feature의 offline/locked 환경을 사용했고 기존 경고 33개가 남았다.
전수 재실행은 없으며 과거 2,236개 통과를 수정 후 전수 결과로 사용하지 않는다.

### 분리된 완료 검토와 한계

구현과 테스트 종료 뒤 같은 실행자가 소스·고정 diff·원시 로그·§4 수용 조건을 별도 읽기 전용 단계에서 대조했다.
[완료 검토](evidence/s09_r1_001/review.json)의 판정은 **선택한 R1/D1 완료**, 발견 사항은 없다.
비참여 독립 감사는 아니다. 타입의 완전한 variant 집합, 제거된 인자의 부작용 없음, 실제 lock/segment/retry와
사망 경로 보존, 계약 설명과 실제 쓰기 위치, 정확한 테스트 17개 이름을 확인했다.

§5의 중간 tick 전체 조합·Unity·release/feature·최대 규모 성능·S08 comparator 한계는 유지한다.
실행 중인 명령이나 미해결 테스트 실패는 없다. S09 선택 개선은 완료했으며 S10 대표 조사는 아직 착수하지 않았다.
