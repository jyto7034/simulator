# S08 전투 내부 수명·이벤트 순서 조사와 개선 계획

작성·갱신일: 2026-10-18

현재 단계: 대표 조사 후 R1 생성 조립·R2 buff 정리 통합·D1 주석 교정·각 영향 검증·분리된 완료 검토 완료.
최초 조사와 검증은 §1~5, R1/D1은 §7, 변경 전 전수 결과는 §8, R2 완료는 §9에 구분한다. 다음은 §6의 S09다.
최신 진행 순서는 [출발점](README.md)을 따른다.

## 1 범위·기준과 기존 근거 재사용

README의 다음 행동에 따라 **시작 → 한 시각의 이벤트 처리 → 생성·철수·사망·소환 정리 → 종료**를 조사한다.
종료점은 대표 조사·유지/개선 판정·작업 및 검증 계획·현황 갱신이다. 제품 코드·테스트·계약은 수정하지 않는다.
S09 이동/저지의 상세 알고리즘, S10의 모든 스킬·효과·보스 분기는 해당 시스템 조사에 남긴다.

- [판단 기준](../refactoring_criteria.ko.md) §3~7과 [실행 지침](../system_refactoring_execution_guide.ko.md)
  §3~5·9를 적용했다. 동작 오류뿐 아니라 원본 소유권·중복·변경 및 검증 비용을 평가했다.
- [Runtime 계약](../core_runtime_contract.md)의 Determinism And RNG, Terminal And Same-Time Event Authority,
  Atomic Battle Advance Transaction, Withdrawal/Detached Delivery/Redeployment, Boss Runtime Foundation,
  Native runtime facts와 Typed resources 절을 적용했다. 로그와 권위 상태, bare Core와 world transaction을 구분한다.
- [로드맵 §1.9](../roadmaps/core_first/ROADMAP.ko.md#19-검증-범위-제한--외부-임의-변조-방어-제외)의 지원 경로 범위를 유지한다.
  내부 fixture로 정상 수명 전이를 준비하는 것과 불가능한 상태의 임의 변조 방어를 구분했다.
- [S07](S07_COMBAT_BOUNDARY_REVIEW.ko.md)의 진입·terminal 정산,
  [S01](S01_APPLICATION_BOUNDARY_REVIEW.ko.md)의 candidate/commit 경계를 재사용했다.
  `world/combat.rs`는 S07 R1 완료 대상과 동일하다. `world/application.rs`도 지도 기준선 이후의 S01 D1 주석 교정 대상과 동일하다.
- [선택 입력 기준선](evidence/s08_001/baseline.json)은 소스·테스트·계약·build 등 34개를 식별한다.
  지도 입력과 겹치는 28개 중 26개가 동일하며 차이는 위 두 world 파일이다. 전체 의존성 freeze는 아니다.
  S07의 [15개 원시 통과 기록](evidence/s07_r1_001/regression001.log)은 기존 경계 검증 이력으로 보존하며
  이번 S08 실행 수에 더하지 않는다. 현재 S08 코드 조사와 선택 실행으로 필요한 내부 근거만 보완한다.

작업 트리에는 기존 대규모 변경이 있다. HEAD diff를 이번 변경으로 취급하거나 되돌리지 않는다.

## 2 대표 실행과 상태 소유권

| 흐름 | 실제 생산자 → 소유자 → 소비자 | 순서·실패·수명 판단 |
| --- | --- | --- |
| 시작 | S07의 scenario/data → `sim::start_battle_execution*` → `BattleCore` + `BattleExecutionState` | scenario 검증, runtime reset/자원 준비, 전장 구성, BattleStart, scenario event 예약. t0 생성 후 hook은 첫 bucket 처리 중 실행 |
| 한 시각 진행 | `EventQueue` → `step_battle_execution_in_candidate_with_setup` → event dispatcher | 같은 시각 queue를 bucket으로 옮겨 정렬·처리. 새로 생성된 같은 시각 작업을 다시 수집하고 wave/boss/object settlement 뒤 terminal 평가 |
| 순서 | `BattleEvent::Ord` → `QueuedEvent::Ord` → heap 및 bucket | time·semantic band·variant·stable key·payload 순서. queue의 source/fact metadata는 비교에서 제외. 관측 seq는 gameplay 순서/RNG 원본이 아님 |
| command | S07 authority 확인 → `apply_live_command*` → 배치·철수 등 도메인 처리 | 일반 진입은 `runtime_transaction`; world candidate 내부 경로는 상위 소유자의 원자성 이용. low-level command의 전달 시각을 별도 playback authority로 취급하지 않음 |
| 일반 생성 | `SpawnGroup` 또는 live 배치 → `build::spawn_scenario_group` → units/items/unit_refs | 작성 순서대로 생성, 모두 생성한 뒤 각 deployment 자원 scope를 열고 group spawned 표시 |
| Wave/소환 생성 | wave/summon producer → `execute_spawn_request` → `spawn_scenario_group_member` | kernel은 request identity·정규화·캐시·전체 batch 복구를 소유. member helper는 유닛·장비·provider·unit_ref 생성과 해당 deployment 시작을 수행 |
| 철수 | active lifecycle → `withdraw_unit` → Withdrawn | cast/target/이동/buff·deployment 정리. 이미 방출된 projectile/GroundFixed는 별도 snapshot으로 지속. 같은 instance 재철수는 거절 |
| 사망 | lethal/direct kill → prepared death → `finalize_prepared_death_with_dispatch` | 죽기 전 자료 확보 후 flow/이동/자원·공명 정리, wave death edge·death fact, buff 정리·graveyard·UnitDied·선택적 반응. death와 optional proc를 구분 |
| 소환 제거 | summon batch 수명·signal·summoner removal → `remove_summoned_unit` | Kill은 사망 경로, Despawn은 철수 정리 후 Despawned로 표시. 후자는 death fact/시체/kill proc를 만들지 않음. batch clear는 멤버별 alive 해제 후 한 번 |
| 종료 | named outcome evaluator → execution terminal → `finalize_battle_execution*` → S07 result | Defeat 우선, priority 내림차순, ID 오름차순. 미생성 required wave는 승리 보류. sustained/object 정리와 result 생성 성공 후 finalized. 재finalize 거절 |

주요 근거: [sim.rs](../../src/game/battle/core/sim.rs), [queue.rs](../../src/game/battle/core/queue.rs),
[event 순서](../../src/game/battle/enums.rs), [생성·철수](../../src/game/battle/core/build.rs),
[death](../../src/game/battle/core/death_runtime.rs), [spawn kernel](../../src/game/battle/core/spawn_kernel.rs),
[wave](../../src/game/battle/core/wave_runtime.rs), [summon](../../src/game/battle/core/summon_runtime.rs),
[transaction](../../src/game/battle/core/runtime_facts.rs).

### 경계 해석에서 보존할 차이

- 한 bucket에 이미 들어온 event를 처리하는 동안 새로 생긴 같은 시각 event는 다음 수집·정렬에서 처리한다.
  “같은 시각 전체를 미래 생성 event까지 포함해 처음부터 한 번 정렬한다”는 모델이 아니다.
  침식 forced pause는 남은 bucket을 원본 metadata와 함께 재queue하고 종료한다. 이 예외를 일반 terminal drain과 혼동하지 않는다.
- bare Core의 여러 timestamp advance는 성공한 앞 timestamp를 보존하고 실패한 candidate만 폐기한다.
  world의 한 요청은 전투·실행·투영을 포함한 전체 candidate를 폐기할 수 있다. 두 transaction의 통합은 개선안이 아니다.
- `finalize`는 모든 queue·유닛을 즉시 비우는 API가 아니다. 결과 capsule을 만들고 execution을 닫으며,
  남은 컨테이너는 Core/상위 session owner 수명으로 회수된다. S07의 active session 제거를 재사용한다.
- 사망과 철수, 소환 Despawn, HP형 독립 object 파괴는 같은 사건이 아니다. 독립 object는
  `finalize_unit_death`에서 전용 retire 경로로 분기한다. 이 조사로 S10의 모든 effect cleanup까지 검증한 것은 아니다.

## 3 설계 품질 평가와 후보 판정

| 항목 | 현재 근거와 판정 |
| --- | --- |
| 책임·도메인 경계 | sim의 orchestration, queue 저장, spawn kernel request, wave/summon 정책 구분은 유지. 긴 sim 파일 자체를 분할 근거로 사용하지 않음 |
| 상태 소유권 | terminal/finalized는 execution, 실제 unit/queue/fact는 Core, 원정 정산은 S07. 읽기 snapshot·log를 원본으로 재해석하지 않음 |
| 상태 전이·수명 | Active/Withdrawn/Dead/Despawned, cast control과 released delivery 구분 유지. 공통 buff 정리는 R2 |
| 실패·commit | native candidate·world candidate·spawn request의 복구 범위가 다름. batch 실패/재시도와 timestamp 실패 prefix를 집중 검증 |
| 규칙 중복 | 일반 group/member의 생성 조립은 R1, death/withdraw buff 정리는 R2로 확인. 서로 다른 원인이므로 별도 작업 |
| 의존·공개 범위 | 내부 helper로 해결 가능. 새 public lifecycle API·trait·registry·crate가 필요하다는 근거 없음 |
| 변경 용이성 | RuntimeUnit의 기존 초기 상태/장비·provider 연결 규칙 변경이 두 생성 본문에 전파됨. R1은 이 수동 동기화를 줄임 |
| 데이터·외부 계약 | captured scenario/generation을 실행에 사용. R1/R2는 schema·DTO·저장 형식·outcome 정책을 바꾸지 않도록 설계 |
| 결정론·재개 | queue 비교와 명시 sort 유지. 생성 순서·ID·buff event 순서·같은 시각 terminal이 보존 조건 |
| 오류·관측 | typed fault는 enclosing candidate에서 처리. 로그 누락은 gameplay 결론이 아님. buff 정리 로그는 기존 event 순서까지 보존 |
| 검증 가능성 | 기존 테스트로 대표 lifecycle을 준비할 수 있음. kernel fixture와 공개 application 검증을 구분. R2의 self-buff/정확한 순서는 추가 보강 필요 |
| 성능·자원 | queue의 packed/Arc owner·할당 전 admission·timestamp clone 재사용 조건 확인. 이번 성능 측정 없음. payload fallback의 Debug 비용이나 clone을 추측만으로 최적화하지 않음 |

**동작 오류:** 조사한 대표 흐름에서 확정 계약 위반을 확인하지 못했다. 전체 전투 시스템 무결점 판정은 아니다.
**설계 개선:** 아래 R1/R2는 현재 호출·상태 흐름으로 확인한 중복 소유 책임이다. 오류 재현을 전제로 하지 않는다.
**설명 불일치 D1:** `wave_runtime.rs::settle_wave_triggers` 주석의 “timestamp bucket has committed”는
실제 호출 위치와 다르다. 이 함수는 아직 게시하지 않은 timestamp candidate 안에서, 현재 bucket 처리 뒤 호출된다.
실제 commit owner는 `runtime_timestamp_reusing` 또는 상위 world다. 후보 구현 시 해당 주석을 국소 교정할 수 있다.

## 4 개선 작업 계획

### R1 — 유닛 생성 조립의 단일 책임

- **문제:** `build.rs::spawn_scenario_group`와 `spawn_scenario_group_member`가 combat profile·relic stats·
  identity·finite body·RuntimeUnit 필드 초기화·item/provider·unit_refs 연결을 각각 구현한다.
  첫 경로는 scenario event/live 배치, 둘째는 Wave/소환에서 실제 사용한다. dead code가 아니다.
- **영향:** 기존 resonance 초기값, fragment source, 장비 provenance 같은 초기화 규칙을 고칠 때 두 본문을
  함께 수정해야 한다. struct 필드 추가는 compiler가 위치를 알려 주지만, 같은 필드의 계산/등록 순서 변경은 누락을 잡지 못한다.
- **목표:** 같은 파일의 private member 생성 연산 하나가 **유닛 생성부터 item/provider/unit_ref 연결까지** 소유한다.
  일반 batch와 kernel member wrapper는 현재의 자원 scope 시작·그룹 확정 시점과 복구 정책을 계속 소유한다.
- **중요한 차이:** 일반 batch는 모든 member를 만든 뒤 deployment scope를 열고, kernel member는 각 member 뒤에 연다.
  `spawn_scenario_group`를 기존 member helper 호출로 단순 치환하면 이 순서가 달라진다. 공통 부분을 scope 시작 전까지 추출한다.
  기존 `prepare_runtime_resources` 호출 위치·횟수도 wrapper에 유지한다.
- **범위:** `build.rs`의 두 생성 본문과 직접 관련 검증. Wave/소환 정책, placement/occupancy 알고리즘,
  request cache, stats 계산식, 새 factory 계층·DTO는 제외한다. D1 주석은 별도 설명 교정으로 표시한다.
- **보존:** 작성 member 순서, `next_spawn_order`, instance/item UUID, 삽입/중복 오류, provider 및 resource admission,
  unit_refs/group ledger의 확정 위치, 오류 순서·rollback·retry, 외부 로그와 RNG.
- **검증:** 기존 일반 scenario/delayed required group/live deploy, Wave priority/required gating,
  summon scaled stats/provenance, kernel 중간 실패·중복 identity·cached retry를 연결한다.
  typed resource deployment 시점은 관련 owner 테스트와 코드 순서 대조를 함께 사용한다. 기존 테스트로
  두 경로의 핵심 초기값을 비교할 수 없다면 정상 scenario 입력을 사용하는 작은 관측 사례만 보강한다.
- **구조 수용:** 두 경로에서 RuntimeUnit 및 item/provider/unit_ref 조립이 한 연산을 사용하며, wrapper의
  서로 다른 resource/group commit 책임이 유지된다. 공유 helper를 호출한다는 사실만 검사하는 테스트는 만들지 않는다.

### R2 — 수명 종료에 따른 buff 정리 절차 통합

- **문제:** `death_runtime.rs::clear_active_buffs_for_dead_unit`와
  `build.rs::clear_active_buffs_for_withdrawn_unit`가 같은 대상 선택→정렬→제거→BuffExpired 기록을 복제한다.
  차이는 TargetDied/CasterDied와 TargetWithdrawn/CasterWithdrawn reason이다.
- **영향:** 현재 지원하는 source/target 정리와 event ordering 규칙을 바꾸려면 두 위치를 수동으로 맞춰야 한다.
  사망 쪽만 고치면 철수와 소환 Despawn 경로는 남는다. 전체 death/withdraw 처리까지 같은 규칙이라는 뜻은 아니다.
- **목표·범위:** 내부 buff cleanup 연산 하나가 selection/sort/remove/log를 소유하고 호출자가 수명 종료 원인을 전달한다.
  위치는 기존 Core 내부 모듈을 우선 사용한다. 별도 범용 lifecycle framework를 만들지 않는다.
  작은 내부 원인 enum 또는 두 명시 wrapper로 reason 쌍을 제한할 수 있으며 상세 표현은 구현 시 정한다.
- **보존:** self-buff에서는 target reason 우선, target ID→caster ID→buff ID→reason의 기존 정렬,
  시간·event cause·제거 횟수. 사망에서는 predeath capture와 death fact 뒤/UnitDied·proc 앞,
  철수에서는 UnitWithdrawn 뒤/잔여 cast 정리 앞이라는 호출 위치를 바꾸지 않는다.
  자연 만료, detached projectile/area, death proc와 Despawn 구분은 통합 대상 밖이다.
- **검증:** 기존 death/withdraw reason 테스트, 소환 Despawn의 death 부재, released/unreleased 보존을 사용한다.
  현재 두 reason 테스트는 `.any()`로 존재를 확인하므로 정확한 순서·self-buff의 단일 target reason·무관한 buff 유지까지
  증명하지 않는다. 구현 전에 기존 fixture를 작은 사례로 확장해 이 공백만 보강하고 정렬 후 비교로 순서 차이를 숨기지 않는다.
- **구조 수용:** selection/sort/remove/log 조립 지점 하나, 두 호출 경로의 reason·호출 위치 보존.

### 순서와 완료 조건

추천 순서는 **R1 → R2**, 각각 구현·영향 검증 후 분리된 완료 검토다. R1은 일반 배치/Wave/소환의 공통 생성 규칙에
영향을 주므로 먼저 처리하며, 두 후보 사이에 실행상 선행 의존은 없다. 둘을 결합해야 정상 실행할 수 있는 구조도 아니다.
R1만 끝난 중간 상태에서도 R2는 기존 방식으로 정확히 실행된다. 실패하면 해당 작업만 원인 분석·교정하고 사용자 변경은 보존한다.
작업별 변경 전 근거는 현재 입력과 대조해 재사용하며, 구현 후 영향받은 테스트만 재실행한다.
구현 완료와 조사 완료를 구분하고 R1/R2가 미구현인 동안 S08 개선 완료로 표시하지 않는다.

## 5 검증 기록과 한계

[실행 명령·환경·선택 목록](evidence/s08_001/commands.json)은 기존 offline Cargo cache와 target을 재사용한다.
default feature의 Core lib 테스트 25개를 한 명령으로 선택했다. 제품·fixture를 고치지 않고 입력을 고정했다.
원시 출력은 [core001.log](evidence/s08_001/core001.log), 종료 코드는 [execution.json](evidence/s08_001/execution.json)에 둔다.

결과는 **25개 통과, 실패·무시 0, 종료 코드 0**이다. 전체 명령 52.33초, 테스트 2.48초였다.
제품·테스트 추가나 재실행은 없고 기존 Core 경고는 남아 있다. 기존 S07 통과 수는 이번 실행 수에 포함하지 않는다.

검증 범위는 event comparator/queue 보존, terminal 우선순위·동일 시각 drain·단발 finalize, timestamp 실패 prefix,
배치/철수/사망 buff·미방출 취소/방출 area 보존, Wave 및 summon 생성·제거, spawn 실패 원자성이다.
내부 fixture 기반 테스트이므로 production pack·정상 승인 host·Unity를 끝까지 실행했다는 근거로 사용하지 않는다.

검증의 한계:

- `battle_event_cmp_equal_iff_eq_for_every_live_variant`는 이름과 달리 현재 fixture의 24쌍만 검사한다.
  현재 variant discriminator는 31개다. 이번 통과를 모든 variant의 payload/order 전수 증거로 해석하지 않는다.
  새 variant의 ordering을 바꾸는 작업에서는 fixture 범위를 먼저 보완해야 한다. 현재 comparator 오류를 확인한 것은 아니다.
- heap permutation 테스트의 720개는 선택한 6개 event의 삽입 순열이다. 모든 payload 조합이 아니다.
- buff reason 테스트는 제거·reason 존재 근거이며, R2에서 보존할 모든 event 순서의 검증은 아직 아니다.
- standalone kernel의 callback 실패 복구 테스트는 실제 Wave/소환 경로 테스트와 구분한다.
  S07 외부 정산 이력과 이번 내부 테스트를 합쳐 새 end-to-end 실행으로 집계하지 않는다.
- 전체 Core suite/release/feature 조합·host crash recovery·저장 후 실제 전투 재개·Unity·최대 규모 성능은 미실행이다.
  active battle의 durable 재개는 현재 지원 계약이 아니며 별도 신규 요구로 추가하지 않는다.

검증 종료 후 소스의 호출 위치·자원 scope 순서·원시 결과·문서 주장을 별도 시점에 대조했다.
판정은 **선택한 대표 조사·계획 완료**다. 제품 개선 완료나 비참여 독립 감사가 아니다.
작업 중 제품 입력을 수정하지 않았고 외부 변경 징후가 없어 같은 입력을 다시 전수 해시하거나 테스트하지 않았다.
문서·증거 확인은 [verification.json](evidence/s08_001/verification.json)에 기록한다.

## 6 다음 재개 위치

후속 [S09 대표 조사·계획](S09_MOVEMENT_BLOCKING_REVIEW.ko.md)을 완료했다. 최신 재개 위치는 그 기록의 §6을 따른다.
아래는 S08 종료 시 S09에 넘긴 조사 범위다.

다음은 **S09 전장·이동·저지 대표 흐름 조사와 유지/개선 판정·계획**이다.
`core/movement/engine.rs`의 실제 이동 tick에서 `planner.rs`·`lifecycle.rs`·`blocking.rs`·`rapier_backend.rs`,
`spatial.rs`와 전술 위치 요청 경계로 연결한다. 계획/이동 상태의 원본, 점유·저지 갱신, 실패·중단·재진입 시점과
S08의 생성·철수·사망 뒤 reconcile을 대표 흐름으로 대조한다. 상세 조사는 다음 작업에서 시작한다.

S08 선택 작업 R1/R2/D1은 완료했으므로 다시 구현하지 않는다. §9의 입력·변경과 관련 소스의 차이를 확인하고
유효한 수명·순서 검증을 재사용한다. 변경 전 전수 결과 §8을 R2 이후 전수 통과로 해석하지 않는다.
S10의 모든 효과·스킬 분기는 후속 조사이며 comparator fixture 한계는 §5대로 남는다.

## 7 R1·D1 구현 기록

2026-10-18 재개 요청으로 R1 구현·영향 검증·분리된 완료 검토를 완료했다. R2는 별도 후속 작업이다.
§1~5의 무변경·미구현 설명은 최초 조사 시점 기록이며, §6은 최신 재개 위치다.

- 인수: S08 선택 입력 34개가 현재와 동일하다. 기존 25개 통과 중 관련 사례를 변경 전 근거로 재사용한다.
  [이번 기준선](evidence/s08_r1_001/baseline.json)과 수정 전 build/wave 사본을 보존했다.
- 구현 선택: `build.rs`의 private `spawn_scenario_unit`에 기존 member 생성 본문을 옮긴다.
  두 wrapper의 resource 준비·deployment 시작·group 표시·오류 전파는 기존 순서로 남긴다.
  일반 batch의 반환용 ID 벡터에 append하는 시점만 item/provider 등록 뒤로 이동한다.
  이 지역 벡터는 helper 실패 시 반환되지 않고 다른 함수에서 관찰하지 않으며, gameplay owner 변경 순서는 같다.
- 별도 D1: wave 주석은 timestamp가 이미 commit됐다는 표현을 제거하고 candidate 내부 settlement임을 설명한다.
- 검증 선택: 일반 scenario/live deploy, Wave 우선순위·required gating·batch 실패,
  소환 stats/provenance·실패/UUID·수명, kernel cached retry, 실제 외부 RON 기반 재배치의 typed resource 수명을 확인한다.
  기존 테스트와 이동 본문/호출 순서 대조로 보존을 판정하며 helper 호출 여부만 검사하는 새 테스트는 추가하지 않는다.
- 실행: 같은 offline cache/target의 단일 Cargo 명령으로 묶는다. 직전 build 49.65초·test 2.48초를 참고하되
  추가 world 수명 사례의 시간은 미확정이다. 120초를 넘으면 기존 세션의 로그·실행 상태를 확인하고 중복 실행하지 않는다.

### 실제 변경과 검증

[이번 diff](evidence/s08_r1_001/source.diff)와 [구현 대상 식별](evidence/s08_r1_001/implementation.json)을 보존했다.
`spawn_scenario_unit`의 본문은 기존 kernel member의 생성 본문과 동일하다. 기존 일반 batch와도 공백·지역 index 이름,
`tactical_anchor` 인라인화·반환용 ID append 위치를 제외하면 동일함을 대조했다. 두 RuntimeUnit 생성·등록 조립이
하나가 됐으며 `prepare_runtime_resources`, deployment 시작, spawned group 확정은 기존 wrapper가 맡는다.
장비·provider·fragment·resonance 초기화 규칙을 바꿀 때 두 본문을 수동으로 맞추던 부담을 줄였다.
새 public API·DTO·저장 형식·테스트·별도 추상 계층은 추가하지 않았다.

[선택 명령·환경](evidence/s08_r1_001/commands.json)의 기존 테스트 **16개 통과, 실패·무시 0, 종료 코드 0**이다.
[원시 로그](evidence/s08_r1_001/regression001.log), [실행 결과](evidence/s08_r1_001/execution.json):
빌드 42.44초·테스트 25.20초·명령 전체 67.82초, 기존 warning 33개. 변경 후 Cargo는 한 번 실행했다.

- 일반 scenario 지연 생성·live 배치/철수, Wave priority/required gating·batch 실패, summon stats/provenance·
  실패 시 UUID/total 보존·소환자 제거·승리 비간섭을 확인했다.
- standalone Spawn Kernel callback 실패/중복 identity/작성 순서와 timestamp clone의 insertion 순서는 내부 fixture 검증이다.
  지원 데이터 publication 전체 검증이나 임의 변조 방어 완료로 해석하지 않는다.
- 실제 외부 RON 기반 world 테스트 2개가 Run/Attempt/Deployment/Activation/Delivery/Chain 여섯 수명의
  직원·장비 사본 source와, Run/Attempt/Deployment 세 수명의 실제 철수·대기·재배치 후 값을 확인했다.
  이 입력은 제품 loader·전투 진입·명령·advance를 사용한다. 모든 공식 콘텐츠와 모든 초기 필드를 전수 확인한 것은 아니다.
- S07의 live terminal/ACK/Continue 테스트 1개로 이번 생성 변경 이후의 상위 정산 연결도 확인했다.
  기존 25개 조사 통과를 이번 16개에 더하지 않았고 comparator·buff 정리 등 비영향 검사는 반복하지 않았다.
- 전체 suite/release/feature 조합·host crash·Unity·성능은 미실행이며 성능 개선 수치를 주장하지 않는다.

### 분리된 완료 검토

구현과 테스트가 끝난 후 고정한 두 source·diff·원시 결과를 다시 읽고 아래 조건을 대조했다.
같은 실행자의 별도 pass이며 비참여 독립 감사는 아니다. 검토 중 제품 코드를 수정하지 않았다.

| 수용 조건 | 판정·근거 |
| --- | --- |
| 생성 책임 단일화 | 두 실제 wrapper가 private helper 사용. RuntimeUnit/item/provider/unit_ref 조립은 한 곳 |
| 자원·group 확정 시점 | batch의 준비·deployment loop·group commit, member의 준비/lookup·deploy 순서 보존 |
| 초기값·ID·admission·실패 | 이동 본문 동일성과 16개 집중 검증. kernel 정책과 native/world transaction은 변경 없음 |
| D1 정확성 | `sim.rs`의 세 settlement 호출은 candidate 내부이며 terminal 평가/commit 전. 실행 줄은 불변 |
| 범위·사용자 변경 보존 | `build.rs`의 나머지 본문은 동일. `wave_runtime.rs`는 주석만 변경. 수정 전 사본과 이번 diff로 구분 |

**판정: R1·D1 완료.** [완료 검토](evidence/s08_r1_001/review.json)에 대상과 조건을 고정했다.
R1/D1 종료 당시 필수 잔여·실행 중 명령은 없었다. 당시 R2 미구현과 comparator fixture 범위 한계는 그대로였으며
전체 S08 개선 완료로 표시하지 않는다. 문서·최종 확인은 [verification.json](evidence/s08_r1_001/verification.json)에 둔다.

## 8 사용자 요청에 따른 Core 전수 테스트

2026-10-18 R1·D1 완료 후 사용자가 전수 테스트 1회를 요청했다. §5·7의 전체 suite 미실행 설명은 각 당시 범위이며,
이번에는 `game_core` 기본 feature의 `--all-targets --no-fail-fast`와 별도 `--doc`를 각각 한 번 실행했다.
기존 offline cache/target을 사용하고 G11 입력 override 환경변수 둘은 제거했다. 제품·테스트·fixture는 변경하지 않았다.

| 실행 | 결과 | 소요 |
| --- | --- | --- |
| 전체 타깃 | 단위 2,164 + 통합 72 = **2,236 통과, 실패 0, 기존 ignore 6**, exit 0 | 311.65초 |
| 문서 테스트 | 등록 0개, 정상 종료, exit 0 | 34.76초 |

전체 타깃에서 추가 filter/skip 없이 Unix 소켓 검사도 실제 통과했다. 소켓 생성을 허용하는 환경에서 실행했으며,
기존 ignore 6개는 실제 검토된 issuer cache·calibration 관리 환경이 필요한 테스트다. 정확한 이름과 사유는
[전체 결과](evidence/s08_full_001/full-result.json)에 보존했다. ignore와 시험 0개인 target은 기능 통과 수에 더하지 않는다.
모든 선택 feature 조합·release·서버·Unity·성능 검증까지 수행했다는 뜻은 아니다.

[명령·환경](evidence/s08_full_001/commands.json), [종합 결과](evidence/s08_full_001/RESULT.json),
[전체 원시 로그](evidence/s08_full_001/full.log), [문서 로그](evidence/s08_full_001/doc.log)를 연결한다.
총 명령 시간은 346.41초다. 실패 교정이나 테스트 재실행은 없으며 실행 중 명령도 없다. 이 전수 실행 종료 당시 다음 작업은 R2였으며, 후속 결과는 §9에 기록한다.

## 9 R2 구현·검증과 완료 검토

2026-10-18 재개 요청으로 §4 R2의 구현·영향 검증·분리된 완료 검토를 마쳤다.

### 인수와 변경

- [기준선](evidence/s08_r2_001/baseline.json): 선택 입력 34개가 R1/D1 변경을 반영한 기존 근거와 일치한다.
  §8의 기본 Core 전수 2,236개 통과·실패 0·기존 ignore 6을 **변경 전 근거**로 재사용했다.
  기존 수정이 있는 작업 트리이므로 세 소스와 진행 문서의 수정 전 사본·이번 차이를 별도 보존했다.
- 기존 death/withdraw 테스트 두 개의 `.any()` 검증을 확장했다. 실제 ApplyBuff로 다섯 버프를 만들고
  self-buff 단일 target reason, 무관한 버프 유지, target→caster→buff ID의 정확한 출력 순서,
  시간·cause와 UnitWithdrawn 뒤/UnitDied 앞의 로그 위치를 확인한다. 실제 결과를 정렬해서 비교하지 않는다.
  제품 변경 전에 보강한 두 테스트가 통과했다. [테스트 차이](evidence/s08_r2_001/tests.diff).
- `core/mod.rs`의 기존 buff 제어 접근 옆에 private `clear_active_buffs_for_removed_unit`을 두었다.
  private `UnitBuffRemovalCause::{Died, Withdrawn}`가 유효한 reason 쌍을 제한한다.
  새 모듈·범용 lifecycle 체계 없이 기존 상태 owner가 공통 정리를 담당한다.
- `death_runtime.rs`·`build.rs`의 중복 본문을 제거하고 같은 위치에서 공통 함수를 호출한다.
  사망은 predeath/death fact 뒤·graveyard/UnitDied/proc 앞, 철수는 UnitWithdrawn 뒤·잔여 cast 정리 앞이다.
  자연 만료·released delivery·소환 수명 정책과 공개 API·상태 필드·저장 형식은 변경하지 않았다.

[제품·테스트 전체 차이](evidence/s08_r2_001/source.diff), [구현 대상](evidence/s08_r2_001/implementation.json),
[구조 대조](evidence/s08_r2_001/structure-verification.json)를 보존했다. 두 원래 본문은 원인별 reason/매개변수를
대입하고 공백·주석을 제외하면 공통 함수와 동일하다. 두 호출 파일의 나머지 내용과 R1 생성 조립도 그대로다.
초기 대조 도구가 import뿐 아니라 호출문의 enum 문자열까지 지워 오탐 assertion을 냈으며,
import 첫 항목만 정규화하도록 교정했다. 제품·테스트 입력 변경이나 테스트 실패는 없었다.

### 구조 효과와 실행 결과

| 판단 | 근거 |
| --- | --- |
| 공통 정리 규칙 변경 책임 | selection/sort/remove/log 조립 지점 **2→1**. 두 caller는 종료 원인과 호출 시점만 결정 |
| 유효한 입력·상태 | 원인 enum으로 사망/철수 reason의 임의 혼합 방지. buff 저장소·수명 owner 추가 없음 |
| 동작 보존 | self-buff target 우선·한 번 제거, 결정론적 순서·시간·cause, 두 caller 전후 코드 보존 |
| 비용 | 동일 map 순회·임시 Vec·정렬·제거/로그 절차 유지. 원인 match만 추가. 별도 성능 측정·성능 개선 주장은 없음 |

- [변경 전 보강 테스트](evidence/s08_r2_001/baseline-command-result.json): **2개 통과·실패 0**, 43.79초.
- [변경 후 영향 검증](evidence/s08_r2_001/commands-result.json): **18개 통과·실패 0·ignore 0**, 57.61초
  (빌드 42.69초, 테스트 14.85초). [명령·환경·선택 목록](evidence/s08_r2_001/commands.json),
  [원시 로그](evidence/s08_r2_001/regression.log). 기존 경고 33개가 출력됐다.
- 보강 두 사례 외에 철수 거절·pending/active/queued/unreleased 취소·released projectile/fixed area 유지,
  소환 Persist/Despawn/Kill, predeath 실패 복구·fatal retaliation·lethal modifier의 정리/proc,
  timestamp 성공 prefix·자연 만료와 실제 외부 RON world의 철수/재배치 자원 수명을 검증했다.
  내부 Core fixture와 외부 RON world 경로를 구분하며 새 전체 end-to-end 실행으로 집계하지 않는다.

### 분리된 완료 검토와 한계

구현·테스트 종료 후 제품 입력을 고정하고 기준·현재 소스·수정 전 사본·원시 로그를 별도 읽기 전용 pass에서 대조했다.
동일 실행자의 분리된 완료 검토이며 비참여 독립 감사는 아니다.
[완료 검토](evidence/s08_r2_001/review.json)의 모든 수용 조건을 충족하여 **R2 완료**로 판정했다.
테스트 이름 18개가 선택 목록과 각각 대응하고, `mod.rs`의 helper/enum/import·두 테스트 외 내용과 기존 변경도 보존됐다.

S08에서 선택한 R1/R2/D1은 모두 완료했으며 R2 필수 잔여·실행 중 명령은 없다.
S09/S10 상세 조사, comparator 24쌍/31 variant 한계, 선택 feature·release·서버·Unity·최대 규모 성능은 남는다.
R2 이후 전수 테스트는 반복하지 않았다. 이번 18개 검증과 변경 전 전수 근거의 범위를 구분해 인계한다.
