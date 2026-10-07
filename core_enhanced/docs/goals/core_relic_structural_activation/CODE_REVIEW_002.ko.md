# U09 변경 후 코드 검토 002 — 포큐버스 개인 방어막

입력은 `WR-CRSA-005`, 범위는 `RSP-CRSA-003`, 수리 의무는 `(COD-CRSA-001,O20)`이다. `CHG-CRSA-001`의 정책 영향은 PE04/08/09이며 나머지14정책의 의미·수용문장은 그대로다. 이 문서는 검토 완료한 본문이며 정식 `P2G_20_RESULT_002.json`이 complete result를 소유한다. 제품·시험 작성이나 실행 결과가 아니다.

## 현재 사실과 재사용

[코드 검토001](CODE_REVIEW_001.ko.md)의 실제 소스 관찰과 TD01–05의 유한 closure, 개인 불변 plan, consumer, source/저장/예산 경계는 재사용한다. TD06 중 포큐버스 shared-stage/2S·거리재가입 부분은 새 정책으로 대체하며 태아 고정 recipient/원 채널 및 legacy와 regular 구분은 유지한다. [제품 설계001](execution/single_goal/product_parts/DESIGN_001.md)의 API01–07은 현재 구현된 API가 아니라 제안이다. 해당 문서가 superseded-pending으로 표시한 shared/2S 제안을 현재 요구로 쓰지 않는다.

[기준선001](execution/single_goal/baseline/BASELINE_001.json)은 실제438/4c7bbd 제품 source와 기존438-member preimage의 전부 일치, dirty workspace 보존, 기존58 receipt/raw와 U08 Complete/High/open0를 결속한다. 새 제품·시험·빌드 실행은 없다. Root가 report71refs를 비교했을 때 변경은 Root가 successor head를 갱신한 mutable WORKING_POLICY 한 파일이며 나머지70refs는 일치했다. 옛 publication 두 현황문서의 after hash를 현재 정책 문서 hash로 주장하지 않는다. 최신 정책8파일은 POLICY_CHANGE_SOURCE_001.tar.gz가 별도 소유한다. 과거 archive는 과거 내용을 재현하는 근거이며 최신 정책 권위가 아니다.

Root는 현재 `ability/sustained.rs`의 Shield 정의와 `battle/core/sustained.rs`의 Record/retained, begin_sustained, reconcile_sustained, employee_members, refresh_aura_members, absorb_sustained_shields를 직접 대조했다. 다음은 실행하지 않고 코드에서 확인한 사실이다.

- Shield는 `duration_ms/amount/priority/shared_radius_units`를 가진다. `shared_radius_units=None`이면 Record.target 하나가 개인 membership이며 반경을 다시 검사하지 않는다. 각 Record의 capacity/remaining은 독립이다.
- begin_sustained는 caster/target/source와 유한 expiry를 소유하며 개인 shield에서 caster==target을 요구하지 않는다. shared shield에는 그 제약이 있다. 새 multi-recipient grant에 shared radius를 재해석할 필요가 없다.
- reconcile_sustained는 caster 또는 recipient가 비활성일 때 그 Record를 종료한다. target UnitInstanceId가 달라지는 재배치에 이전하지 않는다. expiry/소진 및 source owner 회수는 기존 finite state를 이용할 수 있다.
- employee_members는 active same-side Employee combatant/trauma 자격과 거리로 집합을 만든다. 현재 shared/aura는 이를 반복 평가한다. 새 정책의 cast-time 집합은 반복 membership으로 구현하면 안 된다.
- absorb_sustained_shields는 개인을 공유보다 먼저, 같은 단계에서 priority 내림차순·activation 순서로 선택해 잔량을 차감한다. 이 순서를 유지하면 새 개인 pool과 기존 보호를 함께 사용할 수 있다.
- 기존 begin_sustained에는 동일 source 재시전의 이전 recipient 집합 전체를 교체하는 보장이 없다. 단순 N번 호출만으로 원자 grant/교체를 완료했다고 할 수 없다.

## 목표와 구현 차이

정책·수용조건은 P2G_10_POLICY_003.json의 DR/PER04·08·09-002가 소유하며 이 검토에서 바꾸지 않는다. 시전 release 순간 자격 집합을 고정하고 대상마다 독립 pool 하나를 부여한다. source 생성량을 한 번 계산해 동일한 개인량을 배분하되 그 값에 N을 곱하지 않는다. 단일 변형은 한 명의 authored 강화량이며 old2S/4S 기본값이 아니다. late-entry·재진입·시전자 이동은 부여나 충전을 만들지 않는다.

원래부터 개인 보호·유한 source/target lifetime·보호 priority는 존재한다. 추가해야 할 경계는 엄격한 authored cast-time group producer, 초기량/대상 집합의 선검증과 사전 admission, effect/copy별 group identity, atomic publish/recast replacement, query의 target-policy와 개인 pool 설명이다. 기존 R12 shared input과 기존 ChannelAura behavior를 바꾸어 새 구조처럼 보이게 하지 않는다.

## 권장 구현과 기술 판단

TD07 — 개인 부여 집합. 작성 정의는 group의 eligibility/area 또는 fixed target, 초기량/기간/priority를 명시하고 compiled plan은 해당 실제 property와 write/preserve set을 노출한다. 현재 source SkillDef를 임의 전체 복제·전역 수정하지 않고 기존 effective plan의 typed effect 실행 overlay에 연결한다. 구체 enum/필드 이름은 제품 담당자가 결정하되 public descriptor/검증/실행/조회에서 같은 의미를 사용한다. 범위의 최종 공식 수치는 결정하지 않는다.

TD08 — 원자 교체와 소유. current provider/copy/effect/generation을 검증한 뒤 release 시 target 집합, source 계산, 모든 새 pool/source/expiry/교체 계획의 한도를 확인한다. 같은 provider battle identity와 원 copy/effect의 이전 부여분 전체를 새 집합과 함께 candidate에서 교체한다. source/action마다 새 identity를 만든다는 이유로 옛 집합을 찾지 못해서는 안 된다. action/occurrence는 관측과 생성 구분에 보존하고 replacement lineage는 policy의 안정된 source 범위로 별도 결속한다. 다른 provider/copy/effect는 유지한다. 실패하면 이전 pool/잔량·공명·RNG/ID·save-visible state를 보존한다.

TD09 — 개인 pool과 group 관계. 각 recipient의 잔량/expiry/source를 독립 value state로 두며 in-memory clone/rollback이 같은 mutable pool을 공유하지 않는다. 대상 종료·소진은 그 pool만 종료하고 provider 종료·전투 종료는 group의 남은 pool을 모두 종료한다. group은 잔량을 저장하는 공유 보호 pool이 아니다. recipient 목록/index를 둔다면 마지막 종료/재시전 후 정리하고 admission/retained/clone 방문에 포함한다. 기존 finite source와 receipt provenance를 재사용하되 private runtime authority가 관측 DTO로 새어나가지 않는다.

TD10 — 효과별 single-target 의미. 포큐버스는 cast-time 선택 거리만 검사하고 생성 후 위치 변화가 개인 pool을 제거하지 않는다. 태아는 기존 고정 recipient 연결의 현재 범위 membership과 원 channel/cost를 따른다. 두 효과에 하나의 공통 range-refresh boolean을 적용해 정책을 섞지 않는다. 시전 전 Pause accepted→Resume의 재검증과 accepted plan/source owner 의미는 기존 인수 계약을 따른다.

TD11 — source 초기량과 합성. plan의 typed 초기량 연산은 PE05의 동일 단위 Add/Scale/Replace 충돌 규칙을 따른다. 실제 초기량은 original caster source context에서 한 번 평가하고 source 생성 modifier도 한 번 적용한다. recipient accessor나 아군 수를 재평가 입력으로 넣지 않는다. 일반/침식의 대표 강화값과 duration/area는 작성 입력이며 공식 밸런스가 아니다. 기존 shared-shield source-expression 경로는 회귀 대상으로 유지한다.

TD12 — 기존 recharge 권한과 구별. Root가 `sustained.rs:1668–1715` 및 `commands.rs:868–918`을 추가 확인했다. 현재 RechargeShield는 caster와 source SkillId가 같은 최신 nonsegmented shield를 선택한다. 새 개인 grant를 일반 record로만 넣으면 이 느슨한 선택에 우연히 포함된다. 새 group/family의 충전 capability를 명시해 이를 차단하고 기존 generic RechargeShield의 정상 선택·충전은 유지한다. 새 grant가 보유하지 않은 권한을 관측 source나 SkillId 일치로 얻지 않도록 PE04/12를 실현하는 technical decision이다.

현재 코드에는 별도 Guest 타입/필드가 없다는 Sol의 확인을 인수한다. 정상 owned Employee 자격으로 현재 지원 대상을 제한하고 존재하지 않는 Guest actor fixture를 구현됐다고 주장하지 않는다. 이후 Guest가 Employee 표현에 합류하면 이 자격 경계를 재검토한다. 취소/늦은 실패의 before snapshot은 해당 operation/transaction 직전으로 잡고 이전에 이미 commit된 비용을 소급 환불하는 새 규칙을 추가하지 않는다.

## 전체17정책의 영향 연결

| 정책 | 검토 경로와 실제 영향 |
| --- | --- |
| 01·02·03 | 같은 effect 조건·유한 후보·seed worklist와 부정/비단조 proof 의미는 같음. 새 개인 grant primitive도 원래 유한 published 후보 및 normal executor 경계를 지켜야 함 |
| 04 | 새 개인 초기량·cast-time area/fixed support capability를 typed producer/consumer와 일치시키는 material review |
| 05 | 기존 산술·Replace 충돌 정책 재검증. old terminal2S 합성 제안 제거, 새 authored 값에 동일 typed 연산 적용 |
| 06·07 | 강한 개인plan owner와 effect-local preserve 의미 그대로. 새 group과 accepted/released·effect carrier 연결을 동일 compiler/소비 경계에 포함 |
| 08·09 | 입력/거리의 원본별 차이, 개인 pool과 원자 group 수명에 관한 material review |
| 10·11 | 태아 수치/regular-only·원 channel, Monk effect-occurrence HpLost/1회 heal은 불변. mixed carrier와 legacy 회귀를 유지 |
| 12 | 개인 pool receipt도 원 provider/source와 relic 기여를 보존; 새 event/resource authority를 만들지 않음 |
| 13 | 외부 RON→정상 grant/equip→실제 개인 shield와 query를 새 normal example에서 검증; private injection으로 대체 불가 |
| 14·15 | runtime pool/group은 battle-local. 기존 사본/variant durable 재구성과 generation pin 유지. N개 pool의 임시/보유/clone/교체 peak를 실제 계량 |
| 16·17 | 기존223+별도31조건·PC/R05·전체 독립 감사 조건 불변. 새정책 증거와 기존shared 회귀를 구분해 selection/impact/coverage에 연결 |

## 검증과 남은 상세 선택

독립 시험 담당자의 successor addendum이 기존 T05/T06 및 수명/공개/예산 영향의 입력·oracle를 소유한다. 기존 shared2S 산술 교정은 원 basis로만 보존한다. 신규 target 집합과 모든 pool 결과를 관찰하며 latest receipt 하나로 동시 대상 전체를 증명하지 않는다. source-generation-once와 receiver modifier 미적용, 같은 출처 전체교체와 다른 출처 보존, new identity 미이전, source/target 종료의 차이, 비용 우회·CC·priority를 포함한다.

제품 담당자의 작은 schema/plan/public 연결 확인 뒤 실제 API를 시험 담당자에게 인계한다. closure proof의 자료구조, Flow graft, plan-local catalog, immutable owner의 cold layout, 개인 group index의 구체 구현과 비용은 남은 구현 선택이다. 정책·권한·수명·수용 조건을 약화하는 변경은 owner에게 반환한다. 전체 runtime node는 schema만 게시하거나 새 구조를 미지원으로 남긴 채 완료하지 않는다.

현재 finding은 implementation_finding/technical_decision이며 추가 material-policy 충돌이나 접근 불가한 필수 source gap은 없다. 두 담당자 addendum을 전부 읽고 인수했다. 정식 review success는 binding 완전성을 검증한 complete result가 소유한다.

[제품 delta002](execution/single_goal/product_parts/DESIGN_DELTA_002.md)는 새 PersonalShieldGrant/PersonalShieldInitialAmount/PersonalShieldSingle과 source_creation typed program, release batch, finite pool source 및 legacy recharge 제외를 제안한다. source_creation은 기존 hidden bonus 필드가 아니라 새 authored capability다. group key는 Original EffectSite 또는 Generated EffectiveEffectKey를 손실 없이 구분해야 하며 illustrative EffectSite-only struct를 그대로 복제하지 않는다. 각 pool의 producer use lease를 종료와 분리하되 선택된 강한 plan과 관측 source는 보존한다.

[독립 시험 delta002](execution/single_goal/test_parts/TEST_PLAN_DELTA_002.json)의 P01–P05는 개인 부여·movement/잔량, authored 강화·source 연산 순서, priority/비용, 전체 재시전·late failure, 개별/제공자 수명을 실제 기존 seam에 연결한다. 450/825/1200은 구별 가능한 독립 시험 입력 계산이며 old2S/4S를 재사용하지 않는다. Guest 별도 actor 생성 가능성은 앞의 현재 타입 한계를 적용하고, 비용 취소 oracle는 기존 transaction 직전 기준을 따른다. T08 sibling/occurrence contamination 및 T07 applicable-family 선택 순서의 비변경 보강도 인수한다.
