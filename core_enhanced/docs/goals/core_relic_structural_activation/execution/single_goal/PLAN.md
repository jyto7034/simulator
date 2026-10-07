# U09 유한 활성·typed 속성·구조 변형 실행

## 현재

GEX-00 인수를 완료하고 GEX-10 제품 연결 묶음 A에 진입한다. 현재 exact Goal은 `GS-CRSA-003 / GOAL-CRSA-01`, phase는 `PHASE-CRSA-01`이다. 기존 같은 실행 경로에서 아직 구현하지 않은 Goal의 successor를 인수하며 별도 중복 실행 디렉터리를 만들지 않는다.

[현재 P2G 인계](../../P2G_60_RESULT_002.json)가 `WR006 → RB002 → FPF002 → PP002/PVR002 → GS003 → HA002`와 CHG001 O10→60 수리 완료를 소유한다. 옛 shared2S basis와 실패 후보는 immutable history로 보존한다. [BASELINE001](baseline/BASELINE_001.json)의438 exact source/preimage·58 기존 receipt를 재사용한다. 제품 설계001의 비shield 부분, [개인 grant delta002](product_parts/DESIGN_DELTA_002.md), [독립 시험 delta002](test_parts/TEST_PLAN_DELTA_002.json)와 Root [코드 검토002](../../CODE_REVIEW_002.ko.md)를 읽고 인수했다. 소스 mutation/제품·시험 실행은 GEX00 인수까지0이다.

연속 요청 IN001이 계획·구현·검증·교정·비참여 전체 감사와 후속을 승인했다. 중간 사용자 단계검토 opt-in은 없으며 일반 내부 선택마다 승인을 요구하지 않는다. 전체 독립 Complete/High/open0와 출구 인수 전 U09 완료나 U10 진입을 주장하지 않는다. Core 우선·Unity 후속·개별 최종보스 제외를 유지한다.

## 문서·역할과 현재 소유

이 디렉터리는 새 exact Goal member에 대응하는 유일 실행 경로다. P2G-50과 Phase Plan의 `execution/single_goal` locator를 따른다. Root가 이번 session에서 identity를 생성했고 source 확인에서도 이전 CRSA 실행 기록은 없었다. 다른 Goal의 동등 문서를 새 Goal 기록으로 복사하지 않는다.

| 역할 | 현재 담당/locator | 경계 |
| --- | --- | --- |
| 계획·조정·인수 | Root gpt-6-astra/high; 이 PLAN | 제품·시험 작성 및 최종 독립 감사 제외 |
| 제품·통합 실행 | `/root/u08_product_runner_resume`, gpt-6.1-sol/xhigh; `product_parts/`, `baseline/` | B 제품 runtime 연결 및 작은 library check 단독 runner |
| 독립 시험 설계·작성·focused 실행 | `/root/u08_test_author_resume`, gpt-6-astra/high; `test_parts/` | TA001 인수 완료, B 실제 API까지 source/run HOLD |
| 시간순 작업/실행 | [EXPERIMENTS.md](EXPERIMENTS.md) | Root가 보고서/receipt를 연결; 원시 원본 보존 |
| 현재 판단/미확인 | [EXPERIMENT_NOTES.md](EXPERIMENT_NOTES.md) | 중복 current locator를 만들지 않음 |
| 최종 독립 감사 | GEX-20 시 U09 비참여 Astra/high 배정 | 기존 제품·시험·계획 담당자는 제외; 아직 verdict 없음 |

기존 두 agent의 지정 모델과 참여 이력을 재사용한다. P2G 코드 검토에 참여했으므로 최종 감사자 후보가 아니다. 구체 파일 소유는 product/test 준비 결과에서 확정하고, 같은 파일의 registry/inline test도 순차 이관한다. 제품 통합·시험 focused·성능을 겹치지 않으며 실행 동안 관련 입력을 수정하지 않는다.

## 기준선·선행과 복원

직접 선행 U08은 `core_relic_numeric_variants/post_audit/u08_current_013_pass002/AUDIT_CLOSURE.json`의 whole-goal Complete/High/open0다. `core_first/execution/u08_closure_20261007_001/PUBLICATION_RECORD.json`의 두 현황문서 delta까지 이미 Root가 인수했다. 실제 조건·source·receipt 연결은 그 원본과 TEST_SEAMS_REVIEW_001의 regression_handoff를 재사용한다. 과거 PLAN의 실행 중 문구를 current 상태로 읽지 않는다.

GEX-00 source/dirty·toolchain·lock·visible process·선행 archive와 현재성 비교는 Sol baseline 보고서가 소유한다. 계획 전 코드 검토의 hash를 mutation 직전 baseline으로 추정하지 않는다. dirty/untracked와 unrelated work는 보존하고 복원 가능한 preimage를 확보한 뒤 구현한다. 공유 호스트 전체 quiescence를 /proc만으로 주장하지 않는다.

Goal Set의 Shared Validation Baseline Reuse Contract를 그대로 인수한다. 초기에는 U08 유효 exact evidence를 한 번 인수하며, source/test/config/fixture/toolchain/command 변화나 비영향 입증 실패가 있는 seam만 재검증한다. 223 선행 condition과 별도 R06 31행, PC 원 workload/31성공5거절, R05 원 cap·실패·peak-only 경고와 U07M/native/관측 한계를 유지한다. 최종에는 current integrated 필수 검증을 제품 runner가 수행한다.

## 설계·검증·완료 인계

[Goal](../../P2G_50_GOAL_SET.ko.md), [실행 경계](../../P2G_40_DESIGN_001.ko.md), [실제 코드와 TD01–06](../../CODE_REVIEW_001.ko.md), 두 [runtime](../../review_parts/RUNTIME_REVIEW_001.json)/[test](../../review_parts/TEST_SEAMS_REVIEW_001.json) 보고서가 상세 계약을 소유한다. 이 PLAN에서 정책/34문장을 다시 정의하지 않는다.

최종 node는 `AG-CRSA-RUNTIME`, `WP-CRSA-PUBLIC`, `WP-CRSA-VERIFY`다. AG의 schema/plan/consumer/world-save-query-accounting을 함께 구현하고 정상 외부 RON과 실제 지원 소비를 닫는다. 17 integration seam과 10 required dimension의 owner는 모두 Goal에 지정돼 있다. 실제 API가 준비되기 전 proposed signature를 존재하는 것으로 다루지 않는다.

Sol이 enum/descriptor, finite identity/dependency proof, pure plan, DTO, effect-local 또는 동등 lowering, accepted/released owner와 mutable pool/recipient, admission/restore의 구체 계약을 제안한다. Root는 기존 정책·권한·수명·원자성을 확인하고 Astra 시험 담당자는 독립 입력/oracle·첫 연결을 맞춘다. 정상 설계 세부 선택은 새 사용자 승인 단계가 아니다.

새 테스트는 T01–12/E01/E02에서 독립 산출한다. private fixture는 정상 공개 경로의 대체 증거가 아니며 latest witness 하나로 동시 hit 전체를 증명하지 않는다. 실패 원인은 제품/시험·도구/설계 전제로 구분해 각 owner에 반환한다. 기대값·한도·workload 완화로 통과시키지 않는다.

최종화는 evidence selection→predecessor impact→adaptation 현재 결속→coverage→자체검증→PLAN/NOTES→exact target와 격리 정역복원→비참여 전체 독립 감사 순서다. 현재 U09 기능·성능·완료 판정은 미실행/미검증이다.


## GEX-00 출구와 GEX-10 묶음 A

Root는 baseline/두 설계/두 시험계획과 correction/delta를 검토했다. 모델은 기존 확인된 Sol/xhigh와 Astra/high이며 최종 감사 독립성은 별도 유지한다. 실제 source438과 회복 가능한 preimage, dirty 작업 보존, process 가시성 한계 및 runner0를 인수했다. 변경된 정책은 이미 HA002로 수리했고 필수 미결정 정책·source gap0이다. 구체 타입과 실제 연결은 구현 대상이며 문서 완료를 기능 완료로 취급하지 않는다.

묶음 A 목적은 엄격한 authoring/finite proof·closure/typed composition과 순수 effective plan·read-only DTO의 첫 실제 연결이다. DESIGN001 §12 product 파일 및 새6파일을 Sol이 독점 소유한다. 새 PersonalShieldGrant 선언/amount dispatch를 위해 `game/ability.rs`, `game/value_expr/mod.rs`도 제품 소유 범위에 포함한다. 같은 폴더 안의 새 private implementation module 분할은 승인한 동일 contract 안에서 가능하며 실제 생성 path를 보고한다. 기존 다른 파일로 확대하거나 공개 의미·수명·권한 변경은 Root에 근거를 반환한다. inline test·test registry·fixture·expected/comparator는 Sol이 수정하지 않는다.

A는 schema만 완료로 게시하는 독립 Phase가 아니다. 원자 runtime node 내부의 연결 묶음이며 전체 consumer/lifecycle/public execution이 이어져야 한다. actual serde/enum/diagnostic·compiler/query signature와 최소 실제 RON 예제를 먼저 닫아 Astra에 전달한다. 예시는 제품 syntax 설명이며 fixture/oracle를 대신 작성하지 않는다. 공개 설명은 미실행 부분을 지원 완료로 표시하지 않는다.

Sol은 소유 파일만 포맷하며 test module을 재귀 포맷하지 않는다. 실제 API/타입 연결의 작은 검사는 `cargo check -p game_core --lib --offline --locked -j 1`을 허용한다. 기존 baseline의 빌드 환경/의존캐시를 먼저 확인하고 jobs1·debug0·incremental0을 유지한다. unique U09 label의 전후 입력/명령/env/stdout/stderr/exit를 보존한다. 실행 중 모든 관련 writer0, 이 묶음 runner는 Sol 한 명이다. 테스트·성능·통합 suite는 아직 실행하지 않는다. 일반 타입/import/product compiler 오류는 같은 범위에서 자율 교정·새 label 재검사한다. test 변경이 필요한 실패는 Astra로 반환한다.

첫 작은 연결 확인 뒤 Root는 실제 API와 독립 시험 seams를 맞춰 테스트 파일 작성/registration 순차 이관 및 focused runner를 연결한다. Sol 전체 구현과 Astra 집중 실행을 겹치지 않는다. 나머지 product runtime/amount/catalog/Flow/recipient/source/query/save/admission 소비는 동일 승인 Goal 안의 다음 묶음으로 이어간다.

check001의 제품 enum 연결 오류에 따라 Root가 실제 exhaustive 분기를 확인하고 `src/authoring/amount.rs` 및 `src/game/battle/core/target_usefulness.rs`를 묶음 A 제품 소유에 추가했다. PersonalShieldGrant의 기존 Shield amount 설명 분류와 두 hostile 판별의 false 분류만 연결한다. 정책·공개 계약 변화는 없으며, 실패 원본을 보존하고 같은 목적의 고유 label로 교정 검사한다.

## A001 인수 / TA001 첫 독립 연결

실제 산출물은 [CLOSED](product_parts/bundle_a001/CLOSED.json)와 [API](product_parts/bundle_a001/API_HANDOFF_001.md)가 소유한다. 기존11/신규9 제품 파일, source447/bd332d3a…, check001 실패와 check002/003 library PASS를 인수했다. Root는 실제 주요 API와 임시 publication gate를 확인하고 기록5ref·check raw9ref·current delta20members를 대조했다. 기존 tests/fixtures/cfg(test) 보존 보고를 인수하며 컴파일만으로 기능 완료를 주장하지 않는다.

TA001 동안 Astra가 `src/game/relic/structural.rs` 전체파일을 독점 소유해 제품본문을 보존하며 cfg(test) 등록만 추가한다. 새 `structural/tests.rs`와 그 전용 하위 시험모듈, 필요한 경우 기존 `relic/tests.rs`가 시험 편집 범위다. 제품 담당자는 모든 source/run HOLD다. 다른 기존 파일이나 제품 결함은 Root에 근거와 함께 반환한다.

시험 준비 결과의 `game::relic::u09_structural_tests` 모듈 배치가 필요하면 `src/game/relic.rs` 전체파일의 등록 소유도 Astra에 넘긴다. 이 경우 실제 모듈에 맞춰 실행 필터를 함께 변경하고 selected count가 0인 실행을 PASS로 처리하지 않는다. 기본은 앞서 지정한 `structural::tests`다.

connect001의 lib-test 컴파일 오류에 따라 `src/game/battle/core/sim.rs` 전체파일도 TA001 동안 Astra에게 넘긴다. 편집은 `#[cfg(test)] build_skill_step_commands`의 신규 PersonalShieldGrant 명시 거부 분기에 한정한다. Root가 기존 CreateObject의 ordered occurrence executor 필요 오류 경계를 확인했으며 제품본문/기존 oracle는 보존한다. connect001은 exit101/selected0/입력불변으로 실패 원본을 유지한다.

T01–T03 등의 실제 구현된 serde/condition guard·finite identity/closure/same-witness·typed composition/recipe property·읽기 DTO에서 첫 소수 사례를 연결한다. 명령은 `cargo test -p game_core --lib game::relic::structural::tests --offline --locked -j 1 -- --nocapture`; 기존 cache/env/jobs1/debug0/incremental0을 확인해 사용한다. 고유 label·전후 입력·raw·exit·실제 selected count와 실패 원본을 보존하고 실행 중 writer0를 지킨다. 같은 목적의 시험 연결 오류는 자율 교정한다. fixture-generation compiler 시험과 정상 publication 거부 관찰을 분리하고 공개 지원 PASS로 주장하지 않는다. 이후 정상 publication/co-equip/runtime/World 연결은 같은 Goal의 제품 후속 묶음이다.

## TA001 인수 / B 런타임 연결

[TA001 CLOSED](test_parts/bundle_ta001/CLOSED.json)의 source449/38269ec3…와 최종12 selected/12 PASS를 인수했다. 첫 connect001은 시험 연결 컴파일8오류/0실행, 교정 후2→9→12 실제 검사는 각 변경된 입력에서 PASS였다. Root는 신규 시험2파일 전체와 기존3파일 최소 diff를 읽고 closed 참조·4회 receipt/raw·current5delta를 대조했다. 제품본문 변경0/관찰된 제품결함0이며 정상 publication/runtime 기능 증거는 아니다.

Sol에 DESIGN001 §12 B/C 제품 범위와 A의 실제 제품 파일을 반환하고 동일 계약의 새 private implementation module 분할을 허용한다. `structural.rs`, `sim.rs`도 제품 전체파일 소유로 되돌리되 Astra의 모든 cfg(test)·등록·oracle를 보존한다. Astra는 source/run HOLD다. 다른 기존 파일 확대나 계약 변경은 Root에 근거를 반환한다.

B는 DESIGN001 §4–10과 DESIGN_DELTA002/TD07–12의 실제 소비를 연결한다. plan-local source/catalog/finite Flow, accepted/released strong binding, effect-local footprint/Monk occurrence, fixed support/regular eligibility, 개인 grant의 source계산1회·대상별 finite pool·전체 old-set 원자재시전·정체성 종료, World staged 변경/Continue 재구성/query와 실제 retained/admission이 범위다. co-equip producer proof와 소비가 완성된 뒤 임시 publication gate/지원 표기를 교체한다. 현재 Employee 경계·기존 저장권위·과거 비용/회귀 한도를 유지한다.

제품 작은 `cargo check -p game_core --lib --offline --locked -j 1`과 소유 제품만의 scoped format을 단독으로 허용한다. A 환경·고유 label·입력 고정·실패 원본·일반 교정 자율성을 유지한다. 전체시험/성능은 아직 배정하지 않는다. 정상지원 전환에 따라 TA001의 임시 거부/미연결 상태 기대값은 다음 Astra 묶음이 수정한다. 제품 API·실행 계약과 작은 check 결과를 받아 후속 runtime/public 시험으로 연결한다.

B의 새 query 연결에 `src/game/world/action_capability.rs`를 추가한다. Root는 실제 COMMON_ACTION_PROJECTION_ORDER와 기존 numeric/state query의 run-is-some 평가를 확인했다. 동일 읽기 context/descriptor와 순서 표를 연결하며 변이 권한·cfg(test)를 변경하지 않는다.

같은 query의 실제 `execute_equipment_maintenance_behavior` dispatcher를 Root가 확인하고 `src/game/world.rs`도 B 제품소유에 추가했다. 새 handler 분기만 연결하며 cfg(test) registry는 보존한다.

FiniteFlow graft의 구체 의미는 원 anchor의 실제 occurrence가 EffectsSettled에 도달할 때 1회 시작하는 유한 side branch다. Root가 실제 Action의 Released/EffectsSettled 판별과 FlowState continuation/use-control을 확인했다. 단순 Sequence 삽입은 원 parent Released continuation을 지연시킬 수 있어 채택하지 않는다. 원 parent wait/delay/continuation/비용/use·channel·입력 시간을 보존하고, child 결과·occurrence는 분리하며 승인된 prior original result만 읽는다. generated anchor 재귀 spawn은 없고 Repeat의 실제 원 occurrence마다 독립1회다. 강한 plan 보유와 실행권한을 구분하여 기존 합법 capture/finite-child lease를 미리 결속하고, 취소 뒤 source 부활이나 권한 확장은 하지 않는다. 표현 불가능한 실제 경계는 Root에 반환한다. actual cleanup 소유자인 `src/game/battle/core/skill_runtime/cast.rs`를 같은 B 제품 소유에 추가했다. 구현·계량·부모 Released/긴 delivery/반복/취소 보존 시험은 아직 미검증이며 Astra 후속범위에 인계했다.

자원 경계 정정: 사전 예약 source는 branch의 chain hold/action lease다. 각 새 authored child Action은 기존 `finite`로 독립 Delivery scope를 만든다. `finite_completion`은 같은 projectile/area payload의 impact/tick 정산에만 해당하며, source에 delivery가 있다는 이유로 전역 전환하지 않는다. Root가 resource runtime/store의 살아있는 hold retain 계약을 확인했고 Sol도 이 구분을 인수했다. 검토 과정에서 요청한 `resource_runtime.rs` 변경은 불필요하여 수행하지 않는다. 후속 시험은 child Action 간 Delivery resource 격리 및 branch 완료/취소 lease 회수를 포함한다.

결과 범위 정정: 기존 Amount `CurrentActionPrefix`는 ActionHandle 전체의 누적 ActionResultWitness다. 새 finite child는 bounded occurrence witness를 별도로 소유하고 해당 source의 읽기/settle를 연결한다. child 시작 시각의 latest parent 누적값을 통째로 복사하면 Released 뒤 먼저 정산된 parent 후속·sibling이 섞일 수 있으므로, compiled dependency/scope로 허용된 정확한 prior original 결과만 capture한다. child-local 결과와 반복도 occurrence별로 분리하며 legacy witness의 기존 의미는 유지한다. 같은 시각 후속 효과·늦은 anchor 정산·반복 오염 및 계량/rollback 검증을 후속 Astra에 인계했다.

게시 proof의 actual 호환 컨텍스트: Root가 `UnitCombatProfile::employee_default/for_employee`→`apply_weapon_profile` 및 기존 data validation 선례를 확인했다. 이는 순수 data profile이며 실제 Employee/행동 권한을 생성하지 않는다. 현재 fragment predicate는 weapon profile 축과 context capability tags만 읽으며 `from_combat_profile`은 빈 tags를 사용한다. 임의 장비 tags 추가로 정책을 바꾸지 않는다. 실제 safezone profile helper는 무기 최대1개, 없으면 None을 유지하고 ActiveSkill 호환은 무기를 요구한다. ItemSlot Any3의 단순 저장 허용과 실제 전투/파편 호환을 구분한다. 한 active form과 합법 장비 묶음·3슬롯 유물 조합을 선계량하여 proof하고 불가능한 모든 provider union을 사용하지 않는다.

같은 장비 사본/SkillId의 여러 AbilityActivationBinding은 기존 validation이 허용한다. Root가 실제 validator를 확인했고 World/proof에서 같은 정의·사본·skill의 provider만 coalesce하도록 인수했다. 다른 사본/형태/정의나 실제 action occurrence는 합치지 않고 TriggerProc binding·발동 횟수·스케줄을 유지한다. compiler의 중복 입력 진단을 일반적으로 완화하지 않는다.

## B 최종화 중 CFG001 판정도구 교정

B check010 이후 cfg 수집기가 world/application.rs의 test-only enum variant/match arm 끝을 잘못 잡아 뒤의 제품 `behavior_owner`까지 포함했다. 실제 새 query 제품분기를 test edit로 오인한 초기 진단과 354 span 보존 판정은 무효다. 그 오인 복구로 제품분기가 제거되어 check011은 exit101/입력불변으로 실패했다. Root가 실제 코드와 diff를 확인했으며 해당 자료·잘못된 복구/proof·원시 check는 모두 보존한다. 이를 실제 test semantic edit로 기록하지 않는다.

Sol은 제품분기 및 새 explicit-selection guard 교정과 작은 library check를 계속 소유한다. cfg 판정기 추가 수정/실행은 중지하고 Astra가 `test_parts/bundle_cfg001/`에서 successor verifier·정적 검증 입력만 작성한다. 기존 제품/게임시험/실패collector/proof는 수정하지 않는다. 현재는 도구 작성만 가능하며 Sol의 최종 writer0/runner0 입력 고정 이후에만 Astra가 검증을 실행한다. accepted449 preimage를 재사용해 실제 test-conditioned 구간·등록의 불변, 진짜 시험 변경 검출 및 제품분기 변경의 오인 방지를 검증한다. 불명확 구문은 성공으로 추정하지 않는다. 이 결과 전 B 인수 완료를 선언하지 않는다.

check012의 제품 source456/da23e705… 고정 뒤 CFG001의 독립 tool build·정/부 정적 control·accepted449 대비 비교 실행을 Astra 단독 runner에 넘겼다. Sol은 writer0/runner0이며 B CLOSED는 대기한다. 제출된 [B 구현 인계](product_parts/bundle_b001/IMPLEMENTATION_HANDOFF_001.json)와 [API](product_parts/bundle_b001/API_B001.md)는 Root가 읽고 48 exact 참조, 기존33+신규7 delta, archive40과 현재 변경파일 바이트를 대조했다. 핵심 lowering/plan-local amount binding/Flow scope·publication·World·개인/고정지원 계약을 실제 코드로 검토했으며, 이는 런타임 기능 통과나 독립 최종 감사가 아니다. 최종 cfg 보존/포맷 인수 후 Astra runtime/public 연결 시험으로 진행한다.

CFG001 verify002는 controls26/26, 명시 difference0, macro manual2건을 반환했으나 Root 도구 전문 검토에서 인수 공백을 발견했다. `compare()`가 parser review를 만나면 fingerprint 대조 전 반환하므로 application.rs/types.rs의 일반 AST 보호구간은 macro-container 증거만으로 보장되지 않는다. Astra에 그 두 파일의 정상 보호구간/context/순서 exact 비교와 opaque macro+별도 test변경 검출 control을 successor로 보강하도록 반환했다. 원 verify002/manual001의 exit/status는 보존하며 제품 입력은 고정한다.

CFG001은 [verify002](test_parts/bundle_cfg001/runs/u09_cfg001_verify002/verification.json) + [manual001](test_parts/bundle_cfg001/runs/u09_cfg001_manual001/manual_evidence.json) + [manual002](test_parts/bundle_cfg001/runs/u09_cfg001_manual002/manual_ast_comparison.json)를 함께 인수한다. 일반 scanner exit2는 보존하고, 두 opaque macro는 실제 archive/current 전체 byte 동일로만 해결했다. successor는 일반 AST17/15구간의 종류·context·순서·원문을 별도 비교했고 동일/변경 대조4건 모두 통과했다. Root는 도구전문과26+4control 결과, 참조492건(현재 source456 포함), 실제 macro2개를 대조했다. 이로써 accepted449→B456의 시험/등록 byte 보존을 인정한다. 기능시험 증거는 아니며 Sol scoped format currentness 후 TB 연결로 진행한다.

## B 인수 / TB001 정상 경로 연결

[B CLOSED](product_parts/bundle_b001/CLOSED.json)의 최종456/64983f9661f4…를 인수했다. format_check002의38개 성공과 format003의2개 성공을 결속하고, Root가 실제 네 제품 format hunk(줄바꿈/두 선택적 trailing comma)를 읽어 check012 및 CFG001의 비영향 재사용을 인정했다. CLOSED57 exactrefs와 archive 전후2파일을 대조했다. 제품 구현·작은 컴파일·보존·포맷 완료이며 런타임/Goal 완료는 아니다.

TB001은 Astra 단독 작성/집중 runner, Sol source/run HOLD다. `relic/structural/tests.rs`와 하위, `world/tests/mod.rs`의 등록 및 신규 `world/tests/u09_structural.rs` 하위, 신규 `battle/core/relic_structural_tests.rs` 하위가 시험 소유다. `battle/core/mod.rs` 전체파일을 Astra에 넘기되 신규 cfg(test) 등록만 변경한다. 필요 시 `relic/structural.rs`도 시험등록만 허용하며 제품본문/기존시험을 보존한다. 증거는 `test_parts/bundle_tb001/`에 고유 label로 남긴다.

TA 임시 publication 거부/미연결 기대의 successor와 정상 RON/일반 builder publication→질의→첫 accepted cast/효과를 우선1–3개 연결한다. 이후 같은 범위에서 합법 co-equip conflict/legacy no-op/read-only/current-vs-retained 경계를 소규모 확장한다. 일반 준비 경로와 World owner, native 설치 seam, private fixture-generation 증거를 구분한다. [API B](product_parts/bundle_b001/API_B001.md)와 [최종 차이](product_parts/bundle_b001/API_B002_DELTA.md), 실제 world/relic_runtime·effective_skill·execution 계약 및 world/tests의 기존 준비 예제를 직접 확인한다. 기대값은 GS003/HA002/WR006/설계에서 독립 도출한다.

기존 locked/offline/jobs1/debug0/incremental0 환경에서 실제 시험모듈 필터의 `cargo test -p game_core --lib`와 소유 시험 scoped format을 단독 자율 반복한다. TA12도 변경 영향에 맞춰 successor 검사한다. 일반 연결 오류는 자율교정하고 제품결함/기존 oracle·한도·공개계약 변경은 Root에 반환한다. 전체 회귀/성능/독립감사는 아직 실행하지 않는다. Flow 수명/결과격리·Monk footprint·fixed/regular/personal·World 저장/Continue/admission 확대는 연결 결과 이후 후속 배정이다.

TB001 connect001은 exit101/selected0/입력불변으로 시험 연결 실패다. own fixture/import3건 외에 새 BehaviorResult variant로 기존 cfg(test) exhaustive match가 실패했다. Root가 실제 behavior.rs의 시험 enum·왕복변환·characterization 및 제품 query projection을 확인했고, 해당 전체파일을 Astra 시험소유에 추가했다. RelicEffectivePlan/RequestRelicEffectivePlan의 cfg(test) 계약 항목·fixture·종류54→55·ActionKind행만 완결하고 기존54항목과 권위 기대는 유지한다. 관련 두 characterization 집중 검사도 TB001 범위에 포함하며 제품본문은 보존한다.
