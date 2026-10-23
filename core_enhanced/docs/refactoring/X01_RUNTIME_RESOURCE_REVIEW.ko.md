# X01 실행 제어·typed resource 수명 대표 조사

작성·갱신일: 2026-10-18

**선택 대표 조사·유지 판단·연결 X02 확인 완료. 제품 변경과 테스트 재실행 없음.**
실행 예산의 소유권, 자원 capability와 종료, 실패 전파·Run 결과 반영을 조사했다.
이 범위에서 확정한 동작 오류나 필수 구조 개선 후보는 없다. X01 전체와 Core 전체의 완료 선언은 아니다.

## 1 범위·기준과 근거

[README](README.md), [기준](../refactoring_criteria.ko.md)의 후보 선정·구조 평가·유지 조건,
[실행 지침](../system_refactoring_execution_guide.ko.md)의 대표 흐름·근거 재사용·분리 검토를 적용했다.
[S10 인계](S10_ATTACK_SKILL_EFFECT_REVIEW.ko.md#6-다음-재개-위치)에서 선택한 범위를 따른다.
의미 기준은 [runtime 계약](../core_runtime_contract.md)의 Native runtime facts와 Typed resources 절이다.

- 실행 제어: `battle/runtime_control.rs`, `authoring/control.rs`, `core/runtime_facts.rs`, `world/application.rs`.
- 자원: `game/resource/{mod,store,runtime,durable}.rs`, `core/resource_runtime.rs`, `world/resource_runtime.rs`.
- 연결 소비: `core/value_expr_runtime.rs`의 host admission·자원 read, S10의 action·carrier 경계와 world 결과 확정.
- 검증 지원: 실제 resource world fixture, store/제어/식 소비 테스트, manifest·feature·설정·기존 원시 결과.

[선택 입력](evidence/x01_001/baseline.json) 71개는 지도 이후의 해당 최신 근거와 일치한다.
`world/application.rs`는 최초 지도 이후 변경된 파일이므로 뒤의 S05 기준선과 대조했다.
S08 R2·S09 R1·S10 R1의 변경과 검토는 각 기록을 재사용했으며 다시 조사하지 않았다.
이 목록은 전체 전이 의존성이나 embedded RON 전부의 동일성을 증명하는 manifest가 아니다.

## 2 실제 흐름과 상태 소유권

### 실행 시작·중첩·실패·종료

`world/application.rs::prepare_application_internal`은 active battle 또는 pending 결과의 limits를 선택한다.
World operation이 새 전투를 만들 수 있으면 `run_deferred`가 scope 수명을 소유하고 첫 battle 진입에서
예산을 활성화한다. 활성화 전 무관한 world 연산에 native battle 연료를 임의로 적용하지 않는다.
중첩 `run`은 현재 예산을 공유하고 안쪽 limits로 새 allowance를 발급하지 않는다.

`runtime_control`의 thread-local `Budget`은 gameplay candidate 밖에 있다. work·clone·retained admission은
누적되고 candidate 폐기나 객체 해제가 같은 operation의 연료를 되돌리지 않는다.
`reject`는 첫 typed fault를 latch하고 `poll`은 안쪽 호출자가 오류를 무시해도 밖의 commit을 막는다.
Scope/DeferredScope 종료에서 예산을 제거하므로 다음 공개 operation은 새 예산으로 재시도할 수 있다.
authoring `charge`/`poll`은 authoring 제한·취소 검사 뒤 native 제어에도 연결된다.
native fuel과 authoring trial의 cancellation/phase/provenance는 별도 책임이다.

`BattleCore::runtime_transaction`과 `runtime_timestamp_reusing`은 후보를 복사·실행한 뒤 보유량과 poll을
검사하고 게시한다. timestamp에서는 execution cursor도 후보와 함께 바뀐다.
World는 domain 작업 뒤 owner reconciliation·보유량·application 문맥·poll을 확인해 prepared result를 만든다.
현재 후보의 거부가 앞서 성공한 timestamp나 철수를 소급 취소하지 않는다.

### 정의·owner·scope·lease와 atomic 연산

`ResourceCatalog`는 definition/owner kind/lifetime/initial/cap을 소유한다. `ResourceStore`는 실제 owner token,
scope token, sparse value, reservation을 소유한다. UUID나 scope ordinal만으로 권한을 재생성하지 않는다.
scope와 lease는 private token identity를 확인하므로 폐기한 후보의 숫자 재사용이 권한 재사용이 되지 않는다.
유효한 미기록 key를 읽으면 initial을 반환하며 entry를 만들지 않는다.

`core/resource_runtime.rs`는 source/target·persistent employee/equipment/fragment copy를 실제 occurrence와
연결하고, lifetime별 scope를 선택해 store에 bind한다. `apply_typed_resource_operations`는 authored 순서대로
source 연산은 한 번, TargetEmployee 연산은 각 target에 확장한다. store batch 안에서 lease·revision을 다시
검사하고 Consume은 동기 reserve/commit으로 처리한다. 뒤의 연산이 실패하면 batch 후보를 버린다.
receipt를 runtime fact로 기록하다 실패하는 경우도 바깥 battle 후보가 게시되지 않는다.

`value_expr_runtime.rs::build_value_expr_frame_with_members`의 Resource read도 같은 resolver와 store 값을 쓴다.
다만 live 식은 먼저 source/target의 active lifecycle을 요구한다. 이 조건은 lease의 수명과 별개다.
따라서 released payload의 보존된 capability와 live 식의 현재 유닛 가용성을 하나의 판정으로 합치면 안 된다.
식 평가의 host admission은 poll/work/retained 제한을 연결하고 완료 후 다시 poll한다.

### 제어 종료·유한 전달 완료·Run 반영

| 수명/상태 | owner와 실제 종료 |
| --- | --- |
| Attempt | RuntimeResources 생성 시 scope. 전투 시도 상태가 전체를 소유 |
| Deployment | deploy/end_deployment에서 실제 배치별 scope 생성·종료 |
| Activation | accepted action마다 scope. terminal에서 닫고 완료 표식을 소모 |
| Chain | root 또는 child hold. root terminal 이후 이미 승인된 finite hold가 남으면 유지 |
| Delivery | finite payload scope. queued completion은 retain_delivery로 같은 scope를 유지 |
| Run | World durable root와 battle overlay의 basis/after projection. 결과 확정에서 일치 검사 후 반영 |

`RuntimeResources::terminal`은 Activation을 닫고 root/child의 Chain 기여를 끝낸다.
`settled`는 action record를 제거한다. 두 경계 사이에도 released delivery가 정산될 수 있다.
`finite_with_scope`와 `close_finite`는 Chain hold와 Delivery root/continuation hold를 함께 처리한다.
`finish_scope`는 finite hold를 기다리며 `close_scope`는 강제 종료다. 마지막 hold 해제와 강제 종료를
같은 연산으로 바꾸면 released carrier 의미가 달라진다.
즉시 effect도 `with_instant_resource_delivery`에서 실제 delivery를 만들고 source를 복원한 뒤 종료한다.

Store scope 종료는 관련 reservation 전체를 abort하고 해당 scope 값을 제거한다. 환불 receipt를 만들지 않는다.
`RunResourceResult::apply`는 revision 숫자만 아니라 full before basis를 비교한다.
`world/combat.rs::settle_combat_result`는 결과를 application 후보 안에서 적용하므로 후속 reward 실패도 함께
복원된다. 최종 owner reconciliation은 살아 있는 UUID의 값을 유지하고 사라진 owner만 제거한다.
durable 타입은 값과 persistent identity만 저장하며 runtime scope·hold·예약 token을 저장하지 않는다.

## 3 설계 품질과 판정

| 평가 | 판정·근거 |
| --- | --- |
| 책임·소유권 | 유지. domain adapter가 occurrence/owner를 결정하고 store가 capability·값·예약을 관리. 예산은 candidate 밖의 operation owner가 관리 |
| 규칙 중복 | 유지. 자원 read/write의 lease resolver가 이미 공통이다. live unit 가용성 검사와 token 검사는 서로 다른 계약 |
| 수명·상태 전이 | 유지. terminal/settled, root/hold, live/durable가 구분되어 있다. 수명이 다른 상태를 단순 삭제나 공통 종료로 합칠 근거 없음 |
| 실패·commit | 유지. store batch와 battle/world 후보의 원자성 범위가 다르고 첫 fault가 외부 poll까지 전파됨 |
| 의존·공개 범위 | 유지. 관측 projection은 borrowed read-only이고 mutation 권한 token은 비직렬화. 공통 resource와 battle/authoring 제어 사이 결합은 실제 두 host 소비를 지원하며 새 host 분리 요구는 없음 |
| 결정론·저장 | 선택 범위 유지. ordered batch와 BTree key 순서, full basis 비교, persistent owner를 사용. 전체 RNG/ID·저장 경로 재검증은 하지 않음 |
| 검증 가능성 | 유지. 실제 external RON→공개 world 실행→부족/철수/정산/재시도 시험과 store의 좁은 capability 시험이 서로 다른 경계를 보호 |
| 성능·자원 | 중첩 transaction 복사·정의 선형 탐색을 관찰했으나 비용 문제로 확정하지 않음. 현재 측정 없이 성능 개선 효과를 주장하지 않음 |

`RuntimeResources::accept/terminal/finite_with_scope`의 store transaction 안에서 scope 연산이 다시 transaction을
여는 것은 실제 중첩 복사다. 이를 단순 제거하면 clone/work/retained charge와 거부 지점도 달라진다.
현재는 국소 atomic 연산과 묶음 atomic 연산을 함께 보장하므로 텍스트 중첩만으로 결함이라 하지 않는다.
병목이나 지원 규모 문제가 확인되면 대표 accept/finite/terminal의 복사·스캔·연료를 측정하고, 보존할 실패
의미를 정한 뒤 private inner 연산을 검토한다. 이번 필수 구현 작업이나 완료를 막는 항목으로 지정하지 않는다.

이번 선택 범위에는 확정한 오류 교정·구조 리팩토링·설명 교정 작업이 없다.
기존 R05 등 역사적 절을 현재 전체 지원 범위로 확장하거나 재작성하지 않는다.

## 4 연결 X02와 기존 검증 재사용

[원시 결과 연결](evidence/x01_001/reused-tests.json)은 기존 성공 **28개**를 파일·줄·정확한 이름으로 기록한다.
S08 전수 결과의 25개, S08 R2 영향 결과의 1개, S10 R1 영향 결과의 2개다.
이번에 실행한 28개로 보고하지 않는다. 핵심 근거는 다음과 같다.

- native nested/deferred 예산, 첫 fault, clone 누적, candidate 폐기 후 retained 비환급.
- store의 stale token·각 수명 종료·finite hold·중복 release·다중 cost 부족·실패 후 ID/revision 보존.
- 실제 RON으로 여섯 lifetime 실행, 두 번째 cost 부족의 실제 도달과 timestamp snapshot 보존.
- 철수 후 fixed area/projectile의 추가 자원 소비와 최종 scope 정리, 닫힌 Activation의 stale 거절.
- nonempty Run 결과의 후속 실패·재시도·중복 거절·checkpoint, terminal native/authoring budget 거절.
- 식 소비의 host 제한·취소와 read-only lease, S10 변경 후 timestamp 실패 보존·실제 HP 소비.

`ExternalFixture::official`은 embedded source inventory를 임시 pack으로 쓰고 테스트가 필요한 resource·skill·
equipment 정의를 변경한 뒤 실제 loader로 게시한다. 내부 store만 조립한 시험을 공개 흐름 검증으로 세지 않았다.
부족 시험은 실제 오류 도달을 assert하고, carrier 시험은 철수 이후 cost와 정리된 scope를 확인한다.

기존 실행은 기본 feature의 `game_core`다. `runtime-observation`, `test-fixtures`, `event_log_checks`는 별도 선택이며
이번에 그 조합·release·Unity·서버 검증을 수행하지 않았다. 현재 별도 실패나 제품 변경이 없어 Cargo를 재실행하지
않았다. 전체 embedded 입력의 신규 동결·전체 재현을 수행한 것은 아니며, 과거 전수 통과를 최신 전수 결과로 바꾸지 않는다.

## 5 조사 완료 검토와 한계

조사 작성 후 동일 작성자의 별도 읽기 전용 검토로 source/call chain·기준·원시 통과 기록·판정·다음 범위를 대조했다.
[검토 기록](evidence/x01_001/review.json), [문서 검증](evidence/x01_001/verification.json)에 결과를 남긴다.
독립 감사나 새 실행 시험으로 부르지 않는다. 제품·테스트·fixture·설정은 변경하지 않았다.

수치식 compiler/evaluator 전체, 모든 capability 소비자와 예외 조합, 공통 컨테이너 전체의 clone 계보,
관측 feature의 동작 동등성·최대 규모 성능은 이 대표 조사로 완료하지 않았다.
S08 comparator 24쌍/31 variant의 기존 한계도 그대로다.

## 6 다음 행동과 계획

후속 수치식 대표 조사는 [X01 수치식 기록](X01_VALUE_EXPRESSION_REVIEW.ko.md)에서 완료했다. 아래는 당시 인계다.

**X01 수치식 compile→capture/frame→evaluate→실제 소비의 대표 조사와 유지/개선 판정·계획**을 진행한다.
`game/value_expr/{compile,eval,catalog}.rs`와 `core/value_expr_runtime.rs`에서 실제 plan 선택·snapshot/live·
단위/범위·오류의 소비 경계를 선정한다. 이번에 확인한 lease/host admission과 S10 ordered effect 실행은 재사용한다.
X02는 그 흐름의 실제 fixture·테스트·feature 선택에 필요한 범위로 연결한다.

먼저 기존 식 관련 조사·테스트와 선택 소스의 유효성을 확인한다. 공백이나 변화가 있으면 그 경로만 검증하고,
오류·설계 후보가 확정될 때만 보존 조건·소유자·범위·수용 기준을 가진 구현 작업을 만든다.
수치식 조사 뒤 공통 컨테이너·결정론·관측 등 남은 X01 항목의 선행 근거와 공백을 정리해 다음 범위를 정한다.
이번 단계에서 전체 Core 통합 완료나 모든 X01 항목의 조사 완료로 넘어가지 않는다.
