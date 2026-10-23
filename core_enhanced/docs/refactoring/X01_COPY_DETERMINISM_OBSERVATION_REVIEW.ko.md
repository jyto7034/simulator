# X01 상태 복사·결정론·선택적 관측 대표 조사

작성·갱신일: 2026-10-18

**선택 대표 조사·유지 판단·연결 X02 검증·분리된 검토 완료. 제품 변경 없음.**
기존 성공 18개를 재사용하고 기본 feature에서 실행되지 않는 관측 경로 13개를 추가 검증했다.
선택 범위에서 확정 오류나 필수 구조 개선 후보는 없다. 전체 Core 완료 선언은 아니다.

## 1 범위·기준과 근거

[README](README.md), [기준](../refactoring_criteria.ko.md)의 구조 평가·유지 조건,
[실행 지침](../system_refactoring_execution_guide.ko.md)의 대표 흐름·근거 재사용·분리 검토를 적용했다.
[수치식 조사 인계](X01_VALUE_EXPRESSION_REVIEW.ko.md#6-다음-행동과-범위)를 따르고
[실행 제어·자원 조사](X01_RUNTIME_RESOURCE_REVIEW.ko.md)의 예산 소유권을 재사용한다.
의미 기준은 [runtime 계약](../core_runtime_contract.md)의 Optional Runtime Observation·Determinism And RNG와
이미 확인한 application commit·checkpoint 복원 경계다.

주 대상은 `owned_collections.rs`, `authoring/clone_shape.rs`, `game/determinism.rs`, `managers/uuid_manager.rs`,
`runtime_observation.rs`와 `identity/retained`, 실제 application/world/battle 후보 소비다.
[선택 입력](evidence/x01_copy_001/baseline.json) **31개**가 기존 최신 근거와 일치한다.
전체 전이 의존성·공식 데이터·feature 조합을 새로 동결한 manifest는 아니다.

## 2 실제 상태 복사와 실패·재시도

`OwnedHashMap`은 native HashMap과 private `MapCloneHistory`를 함께 소유한다.
insert는 보수적으로 credit을 늘리고 vacant entry는 실제 삽입 없이도 예약할 수 있어 이력을 남긴다.
remove/retain/clear는 과거 할당 기여를 지우지 않는다. DerefMut나 raw mutable escape를 제공하지 않아
외부의 grow/shrink가 이력을 우회하지 않는다. serde·동등성·Debug는 gameplay map만 표현한다.
새 deserialize 결과는 tombstone 없는 예약·삽입 경로이므로 capacity로 초기 이력을 인수한다.
이 규칙을 임의의 오래된 HashMap에 적용하지 않는다.

`CloneShape`는 실제 소유 복사량을 계산한다. Arc는 handle, Box는 pointee, OwnedHashMap은 보수적 bucket 이력과
key/value를 방문한다. `controlled_clone`은 적용되는 native/authoring 예산을 먼저 인수하고 native Clone을 수행한다.
예산은 값 밖에 있으므로 후보 폐기에도 소모가 되돌아가지 않는다. retained extent와 실제 copy 기여는 다른 값이다.

BattleCore timestamp buffer 재사용은 같은 call의 이전 native clone 계보에서만 허용된다.
history가 포화되지 않고 길이·key 순서가 일치할 때 RuntimeUnit 값만 재사용하며, 삽입으로 계보가 바뀌면
일반 clone으로 돌아간다. candidate와 execution cursor를 성공 시 함께 게시하고 실패 시 buffer도 폐기한다.
관련 기존 시험은 실제 range row 메모리 재사용, 실패 전 상태, 새 unit 삽입 후 순서를 함께 확인한다.
public legacy map의 임의 변조에 대한 완전한 물리 메모리 상한을 이 조사로 주장하지 않는다.

World 후보는 UUID manager와 owner 상태를 복사하고 immutable 데이터·entry/checkpoint payload는 공유한다.
실패 후보의 변경이 live root에 게시되지 않으며 후퇴/복원은 각 계약의 checkpoint root를 사용한다.
일반 후보 폐기와 명시적 checkpoint 복원은 같은 연산이 아니다. 후자는 VN session처럼 지정된 전역 stream을
단조 유지하는 예외가 있고 `restore_checkpoint_preserving_counters`가 이를 명시적으로 처리한다.

## 3 결정론과 관측의 책임

`UuidManager`는 run seed·instance namespace·stream별 counter를 소유한다. peek는 진행하지 않고 next가 진행한다.
독립 namespace는 다른 종류의 생성 호출 추가가 모든 ID를 밀어내는 결합을 줄인다.
`seed_with_uuid`는 UUID 양쪽 절반을 섞으며 실제 node/상점 등의 도메인이 seed 소비·결과 고정을 소유한다.
피해 roll은 source/hit/actor/target identity, proc roll은 trigger/binding/occurrence identity를 사용한다.
선택한 두 판정은 event_log_seq와 proc 성공 횟수를 seed 원본으로 쓰지 않는다.
모든 RNG callsite를 전수 감사하거나 확률 분포·극단 counter wrap 정책을 새로 바꾸지 않았다.

`CoreApplication`의 observer는 clone되는 GameCore 밖에 있고 authoring fork는 observer를 상속하지 않는다.
feature가 꺼진 operation wrapper는 원래 body를 남긴다. 켜진 경우 OperationGuard가 최상위 TLS 관측을 소유한다.
prepare 반환 전에 TLS를 분리한 PendingObservation을 보관하므로 다른 application 작업과 섞이지 않는다.
폐기하면 attempt만 남고, 실제 prepared commit에서만 committed snapshot/count를 게시한다.
직접 실행 경로는 live root 주소가 맞는 publication만 인정하므로 nested draft commit이 gauge를 올리지 않는다.

실패 시 native 작업량·짧은 peak·최초 오류는 시도 단위로 남는다. `RetainedAccounting`은 attempted/accepted,
warning/hard limit, overflow와 복수 policy를 구분하는 관측값이다. safety allowance의 원본이 아니다.
기존 owner scan의 결과를 전달하며 관측을 위해 새로운 gameplay owner 순회나 RNG draw를 추가하지 않는다.

summary는 고정 크기이고 유한 queue의 try_send 실패는 dropped 관측으로 처리한다.
SessionReader는 weak ownership과 제한된 snapshot 재시도를 사용하며 collector 생존이 game commit의 조건이 아니다.
wall duration은 simulation time과 다르고, 메모리 기여 추정은 RSS가 아니다.
서버 exporter·metric 저장·외부 수집기의 운영 동작은 이번 Core 조사 범위 밖이다.

## 4 설계 판정과 X02 검증

| 평가 | 판정·근거 |
| --- | --- |
| 소유권·복사 | 유지. gameplay 값, 할당 이력, 외부 operation 연료를 분리. immutable 공유는 두 번째 mutable 원본이 아님 |
| 실패·수명 | 유지. candidate 폐기, checkpoint rewind, prepared observation 폐기와 실제 publication의 조건이 각각 명시됨 |
| 의존·규칙 | 유지. ID 파생은 공통 함수, occurrence 결정은 도메인, 관측은 소비자. 서로 다른 namespace/identity 조립을 단일 범용 RNG 정책으로 합칠 근거 없음 |
| 변경·검증 비용 | 유지. closed clone shape와 명시적 field 분해, mutation escape 제한이 누락 위험을 줄임. 예산·계보·실제 후속 실행 시험을 연결 가능 |
| 관측·성능 | 유지. 고정 summary·유한 queue·nonblocking send와 weak session. 최대 규모·고경합 성능 측정이나 메모리 계수의 새 증명은 수행하지 않음 |

[기존 성공 18개](evidence/x01_copy_001/reused-tests.json)는 S08 전수의 17개와 S10 영향 검증의 1개다.
컨테이너 churn/clear/deserialize/overflow, UUID peek/복원, 로그와 독립적인 crit/proc,
timestamp buffer 실패/삽입, 전체 battle/world clone 후 continuation·후퇴·checkpoint를 연결했다.
기존 테스트 이름의 V12 표기만으로 현재 저장 버전을 단정하지 않고 S04의 저장 계약을 재사용한다.

기본 실행에 없는 `runtime-observation` 경로는 다음 좁은 명령으로 새로 확인했다.

```text
cargo test -p game_core --lib --features runtime-observation --offline --locked -- application::runtime_observation_tests:: runtime_observation::identity::
```

**13개 통과·실패 0·ignore 0**, 명령 전체 **63.53초**. compiler warning은 33개이며 이번 제품 수정은 없다.
[정확한 환경·명령](evidence/x01_copy_001/commands.json), [원시 로그](evidence/x01_copy_001/observation.log),
[실행 결과](evidence/x01_copy_001/commands-result.json)를 남겼다.
11개 application 시험은 queue 포화/collector 종료에도 동일 결과·query, prepare 폐기/TLS 분리,
nested draft와 live commit 구분, first fault·accepted accounting, generation/rollback/fork 수명을 확인한다.
2개 identity 시험은 고정 표현의 missing 값·owner extent roundtrip을 확인한다.
이 결과는 전체 feature on/off 바이너리의 모든 gameplay 동등성이나 서버 수집기 검증을 뜻하지 않는다.
기본 전수·다른 feature·release는 반복하지 않았다.

## 5 분리된 완료 검토와 다음 행동

시험 종료 후 별도 읽기 전용 pass에서 계약·현재 source·재사용 근거·실행 결과와 유지 판단을 대조했다.
[검토 기록](evidence/x01_copy_001/review.json), [문서 검증](evidence/x01_copy_001/verification.json)을 연결한다.
동일 작성자 검토이며 독립 감사로 표기하지 않는다. 제품·테스트·fixture·설정 변경은 없다.

후속 [통합 준비·범위 판정](INTEGRATION_READINESS_REVIEW.ko.md)을 완료했다. 아래는 당시 인계이며 최신 재개 위치는 README를 따른다.

**다음은 시스템 간 통합 검토 준비와 남은 범위 판정**이다.
README·SYSTEM_MAP와 각 시스템 결과를 대조해 선정 개선의 종결, 후속 변경이 선행 근거에 미치는 영향,
생산자→소비자·실패/복원 연결과 조사 공백을 하나의 목록으로 정리한다.
특히 X01의 `lib.rs`/`game/mod.rs` 공개 경계·나머지 manager·runtime safety, X02의 fixture/CLI/trace는
이미 조사한 도메인 근거로 충분한지 판정하고 필요한 좁은 후속만 배정한다.
단순 파일 배정이나 이번 대표 흐름만으로 그 전체를 조사 완료로 처리하지 않는다.

유효한 기존 실행은 재사용하고 바뀐 공유 경계나 필수 공백에만 검증을 배정한다.
최종 통합 검토 준비 여부·필수 잔여·범위 밖을 구분한 다음 행동을 정하며, 지금 전체 Core 완료를 선언하지 않는다.
S08 comparator 24쌍/31 variant, 중첩 store 복사 비용, 전체 조합·최대 규모·Unity 등의 기존 한계는 유지한다.
