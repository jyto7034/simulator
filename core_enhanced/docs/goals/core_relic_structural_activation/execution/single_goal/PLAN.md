# U09 유한 활성·typed 속성·구조 변형 실행

## 현재

현재는 target029 전체 감사의 F01 및 후속 대상 수명 결함을 교정하고 새 최종화·동결·재감사를 준비 중이다. 제품032의 생성 사거리 충돌 거부는 post033 성공7개로, 제품035의 접수 계획 기반 대상 수명 검사는 post036 실제8PASS로 확인하여 Root가 코드와 원시 결과를 인수했다. target029의 독립 판정은 `Not Complete / High / open1`로 보존하며 U09는 미완료다. 기존 기능·PC 증거와 R05 concurrent peak 경고·지연 시간 관측 한계도 보존한다. exact Goal은 `GS-CRSA-003 / GOAL-CRSA-01`, phase는 `PHASE-CRSA-01`이다. 최신 인수·소유·검증 원칙은 이 문서 마지막 절을 따른다. 새 target·독립 재감사가 남았으며, 아래 초기 계획과 시간순 결과는 당시 이력으로 보존한다.

[현재 P2G 인계](../../P2G_60_RESULT_002.json)가 `WR006 → RB002 → FPF002 → PP002/PVR002 → GS003 → HA002`와 CHG001 O10→60 수리 완료를 소유한다. 옛 shared2S basis와 실패 후보는 immutable history로 보존한다. [BASELINE001](baseline/BASELINE_001.json)의438 exact source/preimage·58 기존 receipt를 재사용한다. 제품 설계001의 비shield 부분, [개인 grant delta002](product_parts/DESIGN_DELTA_002.md), [독립 시험 delta002](test_parts/TEST_PLAN_DELTA_002.json)와 Root [코드 검토002](../../CODE_REVIEW_002.ko.md)를 읽고 인수했다. 소스 mutation/제품·시험 실행은 GEX00 인수까지0이다.

연속 요청 IN001이 계획·구현·검증·교정·비참여 전체 감사와 후속을 승인했다. 중간 사용자 단계검토 opt-in은 없으며 일반 내부 선택마다 승인을 요구하지 않는다. 전체 독립 Complete/High/open0와 출구 인수 전 U09 완료나 U10 진입을 주장하지 않는다. Core 우선·Unity 후속·개별 최종보스 제외를 유지한다.

## 문서·역할과 현재 소유

이 디렉터리는 새 exact Goal member에 대응하는 유일 실행 경로다. P2G-50과 Phase Plan의 `execution/single_goal` locator를 따른다. Root가 이번 session에서 identity를 생성했고 source 확인에서도 이전 CRSA 실행 기록은 없었다. 다른 Goal의 동등 문서를 새 Goal 기록으로 복사하지 않는다.

| 역할 | 현재 담당/locator | 경계 |
| --- | --- | --- |
| 계획·조정·인수 | Root gpt-6-astra/high; 이 PLAN | 제품·시험 작성 및 최종 독립 감사 제외 |
| 제품·검사 실행 | `/root/u09_execution_continue`, gpt-6.1-sol/xhigh; 기존 작성·실행 귀속 보존 | F/G 제품 고정, 개발용 명시 설정·필요 구현 문서 및 준비된 검사 실행/수집; 시험/fixture/판정도구 편집 제외 |
| 독립 시험 설계·작성·해석 | `/root/u09_tests_continue`, gpt-6-astra/high; `test_parts/` | RV 기존 Rust 시험 모듈/하위·fixture·probe·판정 도구 작성/해석; 실행은 Sol |
| 시간순 작업/실행 | [EXPERIMENTS.md](EXPERIMENTS.md) | Root가 보고서/receipt를 연결; 원시 원본 보존 |
| 현재 판단/미확인 | [EXPERIMENT_NOTES.md](EXPERIMENT_NOTES.md) | 중복 current locator를 만들지 않음 |
| 최종 독립 감사 | `/root/u09_independent_audit029`, gpt-6-astra/high, U09 작성 비참여 | target029 Not Complete/High/open1 종료; 새 target의 전체 재감사 예정 |

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

## TB001-F01 / C001 retained bound 교정

[TB001 인계](test_parts/bundle_tb001/HANDOFF_002.json)의 source457/b5922a77…를 인수했다. TA successor12·behavior2는 통과했고, 정상 RON/builder publication→World 장착/질의→전투/배치/ActivateSkill 승인 뒤 첫 tick에서 `latest_settled_action` RetainedByteLimit(8,388,608/9,437,617)이 발생했다. [F01](test_parts/bundle_tb001/FINDING_001.json)은 OPEN이며 원105 피해 기대·fixture·한도를 유지한다. connect001은 컴파일/0시험, connect002는1시험 실패, behavior004는 새 행을 추가한 뒤 수75→76 누락의 시험교정이며 원본을 보존한다. Root는 신규World 시험207줄 전체·기존3파일 diff·raw실패·인계37참조·6회receipt와 시험수/입력불변을 확인했다. 시험4파일 외 제품본문은 그대로다.

제품 읽기 분석에서 compiler Work의 누적 예약값(종료 시 해제되는 fingerprint/검증복사 포함)을 EffectiveBattleOwner/EffectiveSkillBinding이 지속 retained payload로 사용하는 metric 혼합을 확인했다. C001은 compile usage/work/예약을 유지하고 완성된 immutable plan의 별도 cached conservative retained bound를 계산한다. 살아남는 Vec capacity/Map·Arc header/key/text/tag/property/grant/body/lowered SkillDef/Flow/private catalog를 빠짐없이 계량하고 기존 shared-pointee per-owner 청구·권한·수명·저장·한도는 유지한다. JSON 크기나 Arc handle만으로 보유 graph를 대체하지 않는다.

Astra source/tool/run0 반환 뒤 Sol 단독 제품 소유·runner로 전환했다. 범위는 `relic/structural.rs`, `structural/compiler.rs`, `battle/core/effective_skill.rs`와 신규 private `structural/storage.rs`이며 시험/등록은 보존한다. 증거 `product_parts/bundle_c001/`, scoped format·작은 library check 후 고정된 World 원시험1개를 --exact로 교정 확인하도록 승인했다. 한도·oracle·fixture·comparator 변경0, 전체회귀/성능은 미배정이다. 교정 결과 반환 뒤 Astra에 native retained/admission 경계 및 남은 TB 확대를 연결한다.

C001의 native text 계량을 위해 Root가 실제 private String wrappers를 확인했다. `ability.rs::SkillId`, `data/skill_fragment_data.rs::SkillFragmentId`, `resource/mod.rs::ResourceId`에 crate-private `retained_text_capacity` 접근자만 추가하도록 제품소유를 확대한다. 문자열 길이로 allocation capacity를 추정하지 않기 위한 관찰 경계이며 공개 ID/serde/생성검증·시험은 보존한다.

## C001 인수 / TB002 독립 경계 검증

[C001 CLOSED](product_parts/bundle_c001/CLOSED.json)의 source458/452e522b…를 인수했다. 기존6+신규storage1이며 Root가 visitor775줄 전체·기존제품 diff·47 exactrefs·7delta archive/current를 대조했다. library check001과 고정된 World 원시험 focus001(1PASS, 실제30+75=105)은 입력불변으로 통과했다. TB001-F01 재현은 해소됐지만 native bound/admission 수용 검증은 남아 있다. 실제 peak 값은 출력되지 않았으므로 추정하지 않는다.

TB002는 Astra source/집중runner, Sol source/run HOLD다. 기존 TB 시험 소유에 새 structural/tests/storage_cases.rs 등을 허용하고 증거는 `test_parts/bundle_tb002/`에 남긴다. 완성된 retained payload/동적capacity/serde-skipped Flow·catalog 소유의 포함, 실제 native owner·source/clone/admission의 상한·한단위부족·rollback·정리/재시도, 정상 legal co-equip conflict/동일provider coalesce·legacy None·readonly/current-vs-retained 경계를 독립 입력과 기대값으로 검증한다. getter 자기비교나 visitor 복제 oracle로 대체하지 않는다. 고정 World1의 Cfocus 통과와 TB14는 exact/명시 비영향 근거로 재사용하고 관련 변화에 필요한 선택만 실행한다. 작은 연결 뒤 승인 범위의 일반교정을 자율 진행하며 제품결함은 반환한다. 전체Flow/support·회귀/perf/최종감사는 후속이다.

## TB002 인수 / TC001 Flow·효과별 소비

[TB002 인계](test_parts/bundle_tb002/HANDOFF_001.json)의 source461/8f2e0d151afd…를 인수했다. 신규8사례는 expand005의 변경 없는7 + native007의 교정1을 구분해 결속했고, 영향받은 compiler9는 ta008 최종입력에서 통과했다. Root가 신규시험3파일 전체·등록3곳·7사례 재사용 비영향diff와 raw, 인계67참조, archive/current6, 최종461, 8receipt의 입력불변·raw 결속을 확인했다. 제품본문0이며 C의 정상World105, TA순수3, behavior2는 명시 비영향으로 재사용한다. Root 임시 등록제거 검토의 첫 명령은 삽입된 빈줄을 놓쳐 assertion 실패했고, 실제diff의 정확한 newline을 포함한 비교에서3개preimage 모두 일치했다. 제품/시험 결함이나 실행실패로 해석하지 않는다.

실제 native install26848/26847, cast62370/62369, clone37203/37202의 경계·거부무변이·재시도, accepted2/current1효과와 실제11+37heal, provider/generation/stale권한거부·Weak해제, capacity/serde-skippedFlow/catalog의 독립 하한이 검증됐다. prepared native와 정상publication/World 증거는 분리한다. 합법coequip conflict/상호배제·activeform·legacy None·동일provider coalesce 후 실제trigger binding dispatch 유지도 통과했다. 전체Flow/support나 회귀·성능 완료는 아니다.

TC001은 Astra 단독 source/집중runner, Sol source/run HOLD다. 기존 신규 `battle/core/relic_structural_tests.rs` 및 하위, `world/tests/u09_structural.rs` 및 하위, `relic/structural/tests.rs` 및 하위와 필요한 기존 cfg(test)등록만 시험소유로 유지한다. 제품본문·기존oracle는 보존하고 증거는 `test_parts/bundle_tc001/`에 남긴다. 먼저 정상builder→acceptedcast의 작은 finitechild/효과별HpLost·Selfheal 연결1–3개, 이후 같은 범위에서 Released parent timing·long delivery·original/childRepeat·취소/방출수명, bounded prior-original/sibling/동시각 결과격리, child Action별 Delivery resource와 lease해제, effect-local Instant/TileArea footprint·형제효과·hit history·정확EffectCompare 양성/음성을 독립적으로 확장한다. 실제 private seam의 증거 경계를 명시하며 필요 native admission만 변화에 맞춰 추가한다. 첫연결/중요finding을 공유하고 일반 fixture/compile교정은 자율 진행한다. 기존 offline/locked/jobs1/debug0/incremental0 focused 환경·고유label·입력고정·원실패 보존을 유지한다. fixed/regular/personal·Worldsave/Continue·전체회귀/perf/public CLI/최종감사는 후속묶음이다.

## TC001-F01 / D001 철수 시 finite branch 정리

[TC001 인계](test_parts/bundle_tc001/HANDOFF_001.json)의 source466/9837493f…와 [F01](test_parts/bundle_tc001/FINDING_001.json)을 인수했다. 첫 정상World Flow3과 지연/반복2는 통과했으나 resource005는 cold Wait branch가 있는 clone의 정상World 철수 후 Delivery scope 비어있음 assertion에서 실패했다. 미방출child 실행은 차단됐고 자원예약이 남았다. 후반 정상완료/child별scope assertions는 미도달, footprint2는 컴파일만 했으며 TC001은 미완료다. Root는 실패시험·관찰helper·실제withdraw/Flow/terminal/stagedtransaction을 읽고 인계참조·현재466·5receipt입력불변/raw·archive7·기존등록2만의변경을 확인했다. 원실패와 고정기대값은 보존한다.

Sol의 읽기 분석과 Root 실제코드 검토가 같은 원인을 확인했다. `build.rs` 철수 helper의 unconditional cast remove는 cold branch의 별도 finite Delivery/Chain hold를 닫지 않는다. 기존 action terminal은 outstanding finite hold를 의도적으로 보존한다. 기존 void `cancel_structured_flows`만 호출한 뒤 remove하면 이미Releasedchild의 cast context도 사라지므로 충분하지 않다.

D001은 Sol 제품·단독runner로 전환한다. source 범위는 `battle/core/build.rs`와 `battle/core/skill_runtime/flow.rs` 두파일이며 cfg/tests는 보존한다. 구조적 side-plan cast의 철수 전용 Result 경로에서 reservation을 닫고 미시작 control을 취소하되 started/releasedcarrier 문맥은 정산까지 남긴다. flow 임시소유를 오류에도 복원하고 resource/terminal 실패를 withdraw→World staged mutation에 전파한다. legacy/no-side 경로, 철수의 all-caster 취소, 이미Releasedchild 독립Delivery 수명·기존 비용/권한을 유지한다. 다른cancel/boss/reset의 무관한 개편은 범위가 아니다.

Astra source/run/artifact0 HOLD 후 제품 scopedformat·작은librarycheck와 원resource시험1 --exact만 승인했다. 기존환경/새label/raw/전후고정/원실패보존을 유지하며 증거는 `product_parts/bundle_d001/`에 남긴다. 제품교정 결과 뒤 Astra에 released child 취소/정산·transaction fault rollback·남은TC검증을 연결한다. 전체회귀/성능은 아직 미배정이다.

D001의 첫2파일 교정은 check001과 고정resource1 재현이 통과했다. 처음 미도달했던 anchor/reservation/child의 실제Delivery10/9/11/12, heal7+7, 정상·철수 정리와 강한plan의 무권한까지 관찰됐다. 그러나 제품 readback 중 `cancel_sides.clear()`가 이미Releasedchild의 scoped result owner도 제거하는 경계를 발견했다. 기존 released-child guard는 parent 결과 오염을 막지만 CurrentActionPrefix는 필요한 owner를 잃어 NotSettled가 될 수 있다. Root도 실제scope lookup/정산·소유 경계를 확인했다. 이는 첫cold시험의 검증 범위 밖이며 합법 방출 payload의 문맥 보존에 필요한 보완이다.

첫 실행을 닫고 `skill_runtime/flow/structural.rs`를 D001 제품소유에 추가했다. reservation 종료와 settlement-only child 결과owner 보존을 분리하고 새 control/추가발동을 금지하며 실제payload terminal 뒤 회수한다. 기존work/storage/clone계량과 legacy/no-side를 유지하고 reservation 중복종료를 피한다. 다른 기존파일 필요시는 반환한다. 원check/focus는 보존하고 보완 후 scopedformat·작은check·영향있는 동일resource1 successor만 실행한다. Releasedpayload/latefault의 독립시험은 후속 Astra 범위이며 이 교정만으로 통과를 주장하지 않는다.

## 2026-10-08 KST 연속 요청 갱신 — 기능 단위로 실행·보고 통합

사용자 재개·절차 간소화 지시를 인수했다. 한 번의 재개 대조에서 HANDOFF_LATEST는 이전 U08 진행 기록임을 확인했으나 이후 유효 U08 완료 인수와 현재 U09 TC001/D001 기록을 우선한다. 진행 중인 D001을 중단하거나 과거 source로 되돌리지 않는다. 같은 지정 모델의 기존 Sol 제품·통합 담당과 Astra 시험 담당을 재사용하고 Root는 계획·읽기·조정만 계속한다. 현재 Sol은 D001 세 제품파일의 편집/단독실행, Astra는 HOLD이며 가시적 PID 범위에서 중복 cargo/rustc 실행이 없었다. 공유 host 전체의 정숙성 주장은 아니다.

다음 안전한 소유 경계부터 남은 관련 시험 작성·준비 사례 실행·일반 교정을 기능 단위로 통합한다. 최초연결이나 소수 통과만으로 별도묶음/인계/승인을 만들지 않는다. 실제 제품결함으로 담당자 전환이 필요할 때 입력을 고정해 짧게 반환한다. 같은 입력의 명령은 공통 입력 식별과 명령별 결과를 공유하며 소규모 수정마다 전체 hash/archive를 새로 만들지 않는다. 기존 collector/기록/검사를 우선 사용하고 과거 증거를 재생성하지 않는다. 현재 제품검사가 닫힌 뒤 Astra에 남은 Flow·효과별 범위와 지원효과·Worldsave/Continue의 관련 시험 소유를 함께 넘겨 자율 진행하게 한다. 준비된 같은 환경의 사례는 묶어서 실행하며 전체 기능 준비 뒤 Sol 최종 통합으로 넘긴다. 전체회귀·설정별 검사·외부CLI/서버·독립 격리 필요 검사·성능 및 wholeGoal 독립감사의 완료 기준은 그대로 유지한다.

## D001 인수 / 남은 U09 기능 검증 통합 이관

[D001 CLOSED](product_parts/bundle_d001/CLOSED.json)의 최종466/df564966…를 인수했다. Root는 실제3제품diff/API·52참조·archive/current3·현재466·최종formatcheck/check/focus 입력불변과 원시1PASS를 대조했다. 철수의 cold reservation 누수와 정상child별Delivery10/9/11/12·heal7+7·최종정리가 고정시험으로 해소됐다. Released child의 settlement-only scoped owner 보존은 구현/컴파일됐으나 전용취소·오류복원검증은 남아 있다. 기존취소호출의 선택의미는 유지하고 새 finitechild의 정산owner만 보존한다. Sol은 writer/runner0 HOLD다.

Astra에 남은 U09 runtime acceptance를 하나의 기능검증 범위로 넘겼다. Flow/효과별footprint·Monk의 남은 사례와 D의 Released/Repeat/mixed취소/latefault rollback, fixed recipient·RegularOnly eligibility/contributor snapshot, 최신WR006/DELTA002 personal finite shield P01–P05, World staged/save/Continue 및 관련 정상외부pack/CLI/probe/comparator 준비를 함께 진행한다. 기존TB/TC/D증거는 실제변경 영향에 따라 재사용한다. 준비된같은환경시험은 묶어서 실행하고 일반연결/컴파일/fixture교정은 자율 수행한다. 제품결함·중요계약·소유충돌 또는 실제범위완료 때만 반환하며 소수통과마다 새인계하지 않는다.

시험소유는 기존 U09 core/relic/world 신규시험모듈과 모든하위, 필요한 기존cfg(test)등록, 신규 `test_parts/runtime_validation001/`의 fixture/probe/comparator/runnerconfig까지 함께 부여했다. 기존다른제품본문·behavior/외부기존시험변경은 근거를 반환한다. Astra 단독 focused runner로 기존 offline/locked/jobs1/debug0/incremental0 환경·고유실행명·입력고정을 유지하며 같은입력은 공통식별+명령별raw를 공유한다. 외부CLI/서버/재시작은 필요한환경별 실행을 같은검증묶음으로 관리한다. 최종전체회귀·설정행렬·공개통합·성능은 기능시험 준비완료 뒤 Sol 통합으로 연결하며 wholeGoal 독립감사를 유지한다.

## RV001-F01 — 선택 소비자 readiness 교정

통합 검증의 [compact finding](test_parts/runtime_validation001/FINDING_001.json)을 인수했다. 고정469/bad134b8…에서 personal012는3선택/2통과/1실패다. 범위개인450·독립잔량·전체old-set재시전/수명은 통과했고, 정상World targeting 뒤 eligible ally의 PersonalShieldSingle825 시전은 NoValidTarget으로 거부됐다. 앞선 flow004/runtime010은 effect footprint·Released/Repeat/scopedprefix·World late-cancellation rollback 등의 실제 통과를 보존하며 후속지원/저장/공개검증은 미완료다. Root가 finding참조·현재469·delta6·원시실패와 원선택시험, 실제lowering/usefulness/명시대상/소비계약을 확인했다. 시험담당자는 source/run/artifact0을 반환했다.

원SelfUnit carrier는 source-only1회계산/형제효과 보존을 위한 적법한 계약이지만 기존 AllySingle usefulness gate가 실제 AllySingle step을 강제해 새 선택 소비자를 거부했다. E001은 Sol 제품·단독runner이며 `battle/core/target_usefulness.rs` 한파일에 typed selected-consumer 자격 연결을 교정한다. retained binding/current 동일provider계획과 실제 Personal Selected/Fixed recipe만 인정하고 normalPlayerEmployee·원reach·fixedcaster제외/personalqualifiedcaster허용을 유지한다. typed자격실패를legacy sibling으로 우회하지 않고 원carrier와 placement/channel/sustained/hostile/명시선택권한을 유지한다. Plain authoredSelected도 기존허용형태만 같은자격판정에 포함한다. 다른공개정책이나authoring형태를 확대하지 않는다.

기존collector/공통입력을 재사용해 scopedformat·librarycheck와 고정selected시험1 또는 관련personal3을 중복없이 묶어 확인하도록 했다. 원fixture/oracle/cap은 동결, 증거는 `product_parts/bundle_e001/`의 한 compact 결과로 연결한다. 교정후 Astra가 기존runtime_validation001 통합범위를 그대로 재개하며 Fixed/Plainselected 부정·양성관찰도 그 범위에서 진행한다.

E001의 [compact CLOSED](product_parts/bundle_e001/CLOSED.json)를 인수했다. 최종469/f6e3b971…이며 target_usefulness 한파일의 typedhelper·gate만 변경했다. Root가 실제helper/두diff hunk·26참조·기준대비한파일변경·공통입력의format/check/personal3명령 전후불변과 원시3PASS를 확인했다. Normal825/Corrupt1200 각 eligible ally/qualifiedcaster4경우 및 미선택/적거부, 기존area450·wholeoldset재시전이 통과했다. Sol 소유0 후 Astra에 기존runtime_validation001의 소유·단독focused실행을 반환했다. 기존배정범위에 plainSelected/Fixed·mixedconsumer엄격자격과 readiness/retained독립대조를 결합해 계속하며, 역할전환만으로 통과personal3을 반복하지 않는다. 별도새묶음 없이 남은지원/개인수명/World저장/공개probe검증을 이어간다.

## 2026-10-08 KST 최신 연속 요청 — 영향 기반 검증과 담당자 복구

한 번의 재개 대조에서 HANDOFF_LATEST의 U08 진행 문구보다 이후 U08 Complete/High/open0와 위 U09 E001 인수/RV 진행을 우선했다. E의 현재 target_usefulness.rs는 닫힌 교정 바이트와 일치한다. 시험 담당자의 진행 중 입력은 E의 과거 source469와 같다고 추정하지 않으며 다음 실제 실행 식별을 따른다. 가시적 PID 조회에서는 cargo/rustc가 없었고 이는 해당 순간·PID namespace의 관측일 뿐 공유 host 전체 정숙성이나 담당자의 실행 완료를 뜻하지 않는다. 중단된 최초 git 조회 결과는 확보하지 못했으므로 상태 근거로 쓰지 않는다. 이어 완료된 tracked-only 조회는 변경15파일(문서2·소스/등록/시험13)을 확인했다. untracked 전체는 이 조회 대상이 아니며 기존 보존 기록을 유지한다.

기존 Sol 세션이 가용 목록에서 사라져 지정 gpt-6.1-sol/xhigh의 `/root/u09_product_resume`를 배정했고 모델 지정 성공과 인수 ACK를 확인했다. 기존 제품 작성자·실행 이력과 증거의 귀속은 바꾸지 않는다. 새 Sol은 읽기 전용 변경 영향 검토만 수행했으며 source/run/artifact0 HOLD다. 기존 Astra/high 시험 담당자는 runtime_validation001의 소유와 단독 focused runner를 유지한다. Root는 계획·문서·읽기 검토만 수행하고 최종 감사는 Goal 비참여 Astra/high에 별도 배정한다.

최신 사용자 §5를 이후 개발·최종 검증에 공통 적용한다. 위 시간순 이력 및 기존 P2G/GEX의 ‘최종 전체 회귀/설정행렬’ 문구를 새 무조건 실행 지시로 사용하지 않는다. 현재 Goal 수용 조건, 수정 코드의 직접·간접 호출, 공유 계약과 영향받는 선행 동작에 필요한 범위만 실행한다. 구체 회귀나 영향 경계 불확실성이 있으면 코드/의존성을 먼저 검토하고 필요한 범위부터 확대한다. 단계 완료·최종화·감사 준비나 막연한 정보 부족은 전체 실행 근거가 아니다. 요구사항·기대값·원한도·실패 판정·Goal 전체 독립 감사는 유지하며 과거 요청/실행/동결 원본은 수정하지 않는다.

현재 선택은 남은 U09 Flow·효과별 결과·fixed/RegularOnly/personal·World staged/save/Continue·공개 소비 수용 검증과 관련 선행 계약이다. 최종 통합 시 shared source/Flow/results/sustained/usefulness/clone 변경에 연결되는 relic/schema/amount, U08 owner/carrier/Continue/lifecycle, readiness·manual/hostile target, released Snapshot/Live·취소·admission, R10 Flow·R11 aura/channel·R12 기존 shield/recharge/HealthCost 소비를 선별한다. 정상 외부 RON/CLI·실제 query/시전·fresh Continue, 필요한 server command/result·paused/reconnect·fault 보존과 default/observation 동등성도 해당 영향 경로에 포함한다. 시험 담당자의 닫힌 결과가 경로를 이미 검증했다면 동일 입력/명시 비영향으로 재사용하며, 실제 선택 명령은 실행 기록에 남긴다. 전체 Core/host/auth/ws 또는 모든 feature 조합은 기본 선택이 아니다. 기존 exporter/registry/config parser 등은 비영향이 입증된 범위만 U08 근거를 재사용한다.

성능은 변경된 BattleCore/Flow/clone/retained 경로의 기존 PC와 R05 workload가 관련된다. PC는 battle/adapter24행이 영향받고 kernel12행은 비영향 후보지만 현행 runner에 행 선택 기능이 없어 기존36행 묶음을 한 번 실행하는 범위는 허용한다. 이를 모든36행의 변경 영향으로 설명하지 않는다. R05의 event-heavy/64session은 둘 다 해당 BattleCore 경로를 사용한다. 원 workload·cap·31성공5거절·기존 peak 경고는 유지하고 기능 실행 종료 뒤 단독 측정한다. 최종 입력의 변경 영향에 따라 이미 유효한 측정 재사용 가능성을 다시 판단한다.

E personal3, TB retained/admission, TA compiler/behavior 및 기존 RV 통과는 역할 전환 때문에 재실행하지 않는다. 같은 환경의 준비 사례를 묶고 실패 교정은 영향 범위만 추가한다. 실행·재사용·중요 제외/미검증 위험을 구분하되 모든 미실행 시험을 열거하지 않는다. 현재 문서 정리는 제품 실행 입력을 변경하지 않으므로 추가 빌드/시험·전체 hash/archive 생성 근거가 아니다. 정식 evidence selection→impact→adaptation→coverage→자체검증→문서→동결/복원→독립감사는 Goal 완료 경계에서 연속 처리한다.

Astra가 기존 변경/raw 보존과 sole focused runner를 ACK했고 personal026 실행 중임을 보고했다. 미회신된 외부 연결 소유 요청을 해소했다. Root가 trial.rs의 cfg(test) registry와 기존 u08_relics의 정상 ExternalFixture/reward/command/battle·CLI export/fresh child 경계를 확인하여 `authoring/trial/tests/u09_structural.rs` 및 하위 신규시험과 `authoring/trial.rs` 전체파일을 Astra에 넘긴다. 후자는 cfg(test) 등록 한 줄만 변경하고 제품본문/기존시험은 보존한다. 현재 실행 종료 후 입력을 바꾸며 기존 runtime_validation001 안에서 정상 외부 RON/CLI·fresh Continue·generation 원자거부를 계속한다. Sol은 read-only/HOLD이고 추가 승인 단계는 없다.

외부031은 debug0 정상 loader 단계만 약3분이 걸렸다는 담당자 관측에 따라, 이후 외부 전용 focused 검사는 기존 Cargo release profile로 묶어 실행하도록 허용했다. 현재031은 중단하지 않고 원환경에서 종료한다. 기존 locked/offline/jobs1·동일 입력/기대값/cap/workload와 sole runner를 유지하고 profile 차이를 기록한다. 최종 release CLI 경로에도 필요한 검증이며 custom 최적화/새 도구를 추가하지 않는다. native debug 통과를 이 전환만으로 반복하지 않고, release 결과를 debug PASS나 성능 측정으로 바꾸어 기록하지 않는다.

외부031의 마지막 accepted_work 동일 assertion은 trace와 별도의 amount observation을 함께 off→on 전환한 시험 입력 때문에 실패했다. 담당자는 실제 HP105/generated75·fresh Continue/changed-generation 거부와 checkpoint/mandatory facts 동일을 관측했고, 원실패/JSONL을 보존했다. Root가 trial의 두 독립 설정과 amount_observation의 retained admission/recorded_amount_commit charge 및 시험 mode!=0 입력을 직접 확인했다. 다음 입력은 amount=true를 고정하여 trace off/on/saturated의 결과·필수 facts·accepted_work 동등성을 그대로 요구하고, 별도 default amount=false는 gameplay/facts 동등성과 실제 비용 차이를 구분해 기록한다. 필수 capture 비용을 optional trace 비용과 혼동한 입력 교정이며 제품 한도/필수 계량/기대 동작의 완화가 아니다. 원031의 work511366/500927 차이 전체를 특정 한 charge의 비용으로 단정하지 않는다.

## 2026-10-08 후속 역할 지시 — Astra 작성·판단 / Sol 실행

최신 사용자 역할 변경을 기존 runtime_validation001에 적용한다. 완료된 재개 점검과 모델/참여 이력은 재사용하고 새 중복 실행이나 전체 재검증을 시작하지 않는다. 사용자 요청 시점에 Astra036은 정상 publication의 실제 legal co-equip 검증 중이며 기존 root1개와 신규6개를 합친 유물7개 카탈로그의 family 결과는 아직 미판정이다. Astra는 현재 실행을 끝까지 수집하고 새 실행을 시작하지 않는다고 ACK했다. Sol은 해당 실행 종료와 입력 고정 전 runner0를 유지한다.

다음 안전한 경계부터 `/root/u09_product_resume` Sol/xhigh가 준비된 집중·통합·외부·설정·성능 검사의 실행, 환경/명령 연결 복구, 원시 결과 수집과 기존 도구의 기계 집계를 맡는다. `/root/u08_test_author_resume` Astra/high는 기존 소유 시험/fixture/probe/oracle/comparator/판정도구의 작성·교정과 결과 의미·타당성·누락 검토를 맡는다. 제품 구현과 일반 제품 컴파일 교정은 Sol, 중요 계약·범위·재사용/품질 판단은 Root, 최종 독립 감사는 Goal 비참여 Astra/high다. 원 작성/실행 귀속은 유지한다.

Astra는 현재 실행 종료 후 기존 기록으로 준비된 명령/선택 목록·입력·환경/순서/격리·예상 실행범위·0시험/skip/실패 기준·collector/raw 위치·재사용/중요 제외/실패 반환 대상을 Sol에게 직접 전달한다. 모든 시험 통과 후에야 넘기는 방식으로 운영하지 않는다. 같은 작업 묶음에서 Sol runner0→Astra 시험 교정→Astra writer0/입력 고정→Sol 영향 범위 실행을 직접 협의하며, 일반 반복마다 새 계획·메인 승인·사용자 승인·전체 archive를 만들지 않는다. 관련 입력의 편집/실행은 겹치지 않는다. 중요한 계약 문제·소유 충돌·범위 변경·실제 묶음 완료만 Root에 반환한다.

현재 영향 기반 검사 선정, release 외부 실행 구성, 관측 축별 비교 계약과 기존 실패 보존은 그대로다. native029와 external032 등 유효 결과는 역할 변경으로 무효화하거나 재실행하지 않는다. 실행 완료와 수집 완료를 구분하고, 기계 집계를 Goal 충족이나 독립 품질 판정으로 확대하지 않는다. 이번 문서 변경은 제품/시험 실행 입력 변경이 아니다.

공개 CLI 확인 순서를 우선화했다. Astra가 외부 unit의 정상 content_pack::load가 authoring scope 밖이라는 경계를 발견했고 Root도 publication.rs의 `(variants+1)^3 × (equipment+1)^3 × (1+active forms)` 선청구가 실제 multiset/합법장비 필터 전에 수행됨을 확인했다. 이는 default CLI 예산에서 거부될 위험이며 아직 실제 CLI 실패로 판정하지 않는다. 현재036 종료 뒤 Sol이 고정된 root032 입력·원 default cap으로 CLI를 우선 실행하고 site/requested/limit/원시 결과를 반환한다. cap 상향·계량 삭제·fixture 축소로 이 경계를 우회하지 않는다. family별 준비 비용 분리는 원036 결과와 조합 검증의 증거 경계를 보존할 때만 별도 대표소비 준비로 인정한다.

## 2026-10-09 KST 개정 연속 지침 인수 / F001 publication 증명 재사용

사용자가 지정한 [개정 연속 실행 지침](../../../../roadmaps/core_first/execution/CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md)을 인수했다. 기존 역할·영향 기반 검증·증거를 유지하고 §4-B의 실제 제품 변경/검증 주장에 대한 Astra 검토, §5-A의 관련 실패 경로를 드러내는 독립 시험, §5-H의 장시간 진단/중단 기준을 적용한다. A–E의 기존 Root 코드/계약 검토를 재사용한다. 이후 RV 결과들은 담당자 실행 보고 상태이며 새 입력/시험 타당성과 주장 범위의 Astra 인수는 남아 있다. 구현/실행 완료와 Astra 검토, 최종 독립 Goal 완료를 구분한다.

036은 입력 고정으로 종료됐다. 총1835.13초(시험1458.73초), 정상7relic publication 성공 뒤 첫 closure의 BeginSkillTargeting/no_eligible_payload에서0PASS/1FAIL, 나머지 family는 미도달이다. Astra가 runner0/writer0를 반환하고 Sol이 실행을 인수했다. [CLI037](test_parts/runtime_validation001/u09_rv003cli037.json)은 release build231.77초 성공 후 동일 public030/default cap simulate에서1.18초 만에 exit1, stdout0/실제소비 미도달이다. Root가 원진단과 명령/입력불변 기록 및 publication/compiler/closure 코드를 확인했다. site publication_combination_bound, limit10,000,000, requested100,220,664를 보존한다.

정확한 고정 public030 계수는 equipment93·fragments40·ActiveSkill forms14·Normal relic1이다. 앞선94/44 표현은 다른 family 입력 또는 재료UUID를 포함한 근사였고 이 입력의 실제 계수로 사용하지 않는다. cube 선청구는2³×94³×15=99,670,080, 이전 누적550,584다. 원단일 유물은 중복 불허여서 실제 유효 structural multiset은 더 좁다. PE13의 정상 공개 소비와 PE15의 기존 typed fault/한도를 함께 유지한다. cap 거부 자체를 곧바로 안전 결함으로 판정하지 않지만, 공식 seed에 첫 구조 유물을 더한 기본 공개 사용성과 중복 proof 비용은 현재 기능에 필요한 품질 개선으로 정했다. 작은 소비 fixture 성공만으로 원037/036의 비용·경계를 해소했다고 하지 않는다.

F001은 Sol 제품·단독 runner이며 소유는 structural/publication.rs 및 필요한 private 하위 helper다. 누적 publication cache와 compiler 임시 graph의 동시 extent를 선할당 전에 보장하는 데 필요하면 structural.rs의 Work와 structural/compiler.rs의 private publication entry/basis 연결까지 허용한다. cfg(test)/등록/기존시험은 보존하고 공통 runtime_control·World·저장 계약 변경은 범위가 아니다. Astra는 source/fixture/tool writer0·runner0이며 관련 독립 반례를 읽기 설계한다.

개선은 invocation-local exact 전체 provider vector(종류·정의·skill·form·symbolic copy ordinal)와 동일 data/generation/employee/합법 relic multiset에 결속된 성공 proof만 재사용한다. 실제 ItemSlot Any3의 모든 합법성·중복/상호배제와 정확히1weapon의 native form compatibility 증인은 유지한다. 동일 weapon 정의/profile/form 입력의 호환 결과만 재사용한다. 서로 다른 provider의 Exists/Not/closure·recipe 충돌은 기존 compiler가 전체 context로 검증한다. 모든 provider의 union, 독립 provider별 대체, 근거 없는 provider 제거·identity 정규화는 하지 않는다. Discovery/compatibility/cache/context×relic의 단계별 checked 선청구와 실제 lookup/string/clone/work/cancel·누적 key/cache 및 compiler 동시 peak 계량을 유지한다. cache는 게시 호출 종료에 폐기하며 runtime plan·저장·권한은 불변이다.

93 장비의 실제 Any3 multiset은142,880개지만 ability provider 정의는2개이고, 기존 copy ordinal을 보존한 exact 장비 provider-vector는18종이라는 읽기 분석을 인수했다. 14 active forms를 모두 붙인 context270은 보수적 추정이며 실제 cap 내 PASS가 아니다. 단순 cube→multiset 수식 교체만으로 충분하다고 주장하지 않는다. Sol은 기존 helper/collector를 재사용하여 제품 scoped format/library check와 fresh release bin→원CLI037/public030/default cap 교정검사를 수행한다. cap/입력/oracle/workload는 고정하고 실패·미도달과 실제 소비 결과를 구분한다. 제품교정 결과/실제 diff와 중요한 호출·계량·실패 정리 경계는 Root Astra가 검토한 뒤 인수한다.

장시간 운영 기준: 앞선 release bin build232초·lib-test374초를 기준으로 build8분 미완료 또는 개선 CLI publication60초 미완료 때 기존 session의 단계·CPU/RSS를 한 번 진단한다. build15분/CLI5분 도달 시 진행 근거를 확인하여 자신이 소유한 실행을 안전하게 중단할 수 있다. 종료 임박의 명시적 근거가 있으면 기록하고 계속할 수 있다. 중단 시 입력·명령·시간·진행/자원·원출력·이유·정리 상태를 보존하고 원인 교정은 계속한다. 이는 제품의 자원 한도나 성능 합격 기준이 아니며 시간만으로 제품 FAIL/PASS를 정하지 않는다.

Astra 독립 보강은 잘못된 exact key 재사용, copy/form/variant/relic 집합과 합법 weapon 호환의 구별, cross-provider Exists/Not/grant/Replace conflict, duplicate/incompatible·no-structural 경계, 실패/취소 후 재시도·다른 invocation/generation 격리, cache+compiler 자원 exact/one-short·부분 게시 거부 중 실제 위험에 필요한 사례를 고른다. 기존 TA/TB·native029·외부032의 유효 범위는 변경 영향에 따라 재사용한다. F001 종료 뒤 시험 소유를 반환하여 남은 family fixture·public probe/comparator와 함께 작성하고 Sol이 같은 RV 묶음에서 실행한다. 작은 교정마다 새 계획/승인/전체 검사/전체 archive를 만들지 않는다.

F001 validation002에서 exact context 재사용만으로는 원 default cap을 충족하지 못했다. fresh release build222.53초 성공 후 원CLI는1.23초/exit1/stdout0, final_validation/relic_structural_fingerprint_copy의 requested10,002,394 >10,000,000이다. 조합 선청구 문제 해소를 실제 소비 PASS로 승격하지 않고 원실패를 보존한다.

Root가 compiler의 최종 조립과 execution::lower 전체를 직접 확인하여 private publication entry에서만 버릴 fingerprint JSON count/encode/copy/hash를 생략하도록 허용했다. Lowering에는 실제 selected-support reach·Flow·amount catalog/capability 거부가 있으므로 전부 유지하고 storage::measure와 poll도 유지한다. Runtime compile/fingerprint/usage는 불변이다. 게시 proof는 결과를 폐기하므로 fingerprint를 소비하지 않지만, 최종 runtime fingerprint의64byte extent를 놓치지 않도록 선계량된 고정길이 placeholder 또는 동등한 명시 basis를 사용한다. 해당 publication 결과/placeholder가 실행 가능한 runtime plan이나 공개 DTO로 탈출하지 않게 한다. 만들지 않은 임시 JSON의 비용 제거와 실제 작업 계량 누락을 구분하며, inventory BTree 상계·slot reset 오류 확인도 같은 제품 범위에서 교정한다. 원입력·기대값·한도는 유지하고 필요한 관련 검사만 재실행한다.

validation003도 원CLI 소비 전에 final_validation/relic_structural_owned_copy, requested10,002,903 >10,000,000으로 실패했다(fresh release build225.86초, simulate1.29초, stdout0). 원입력과 실패를 보존한다. Sol은 준비된 기존 publication 충돌 대조2개를 별도 기능 범위로 실행하며 그 통과를 CLI 성공으로 대체하지 않는다.

다음 비용 교정은 추가 proof 의미 변경보다 실제 소유와 선계량의 정합을 우선한다. Root와 Sol이 compiler 원본 effect 초기화를 읽어 `admit_serialized_copy(e)` 뒤에 원문 e.op 전체 복사가 없고 tags·typed properties·site/key/anchor/body 메타데이터만 소유함을 확인했다. 현재 검사 종료 후 private publication mode에 한해 실제 owned fields의 checked 보수 상계로 선계량하도록 허용한다. 효과 record/Vec capacity, 태그 문자열·컨테이너, property 및 초기 program/contribution 컨테이너, 각 독립 site/key/body 문자열과 별도 공유 delivery 계량을 포함하며 할당 전에 admission한다. 원 amount validation·closure·typed/recipe·lower/catalog·storage 검사는 유지하고 runtime 기존 계량 경로는 바꾸지 않는다. 실제 생성하지 않는 원문 전체 복사의 과도 상계 제거를 실제 할당/검사 계량 누락으로 대체하지 않는다. 기존3파일 범위의 일반 디버깅과 관련 재검증은 자율 계속하며 새로운 memo/검증 의미 변경은 이 단계에 포함하지 않는다.

## 2026-10-09 KST 자원 한도에 대한 최신 사용자 합의

사용자는 기존 자원 한도를 검증 기준선으로 보존하되 영구 불변값으로 취급하지 않는다고 직접 지시했다. 로드맵 반영은 예정이며 아직 이루어지지 않았다는 설명도 인수했다. 이 직접 합의를 이후 판단에 적용하고 과거 PLAN·P2G·실행·감사 원본은 소급 수정하지 않는다. 기존 ‘한도 고정’ 문구는 당시 검증의 기준을 나타내며, 이후 근거 있는 조정까지 일괄 금지하는 상위 규칙으로 사용하지 않는다.

초과 원인을 불필요한 계산, 과도한 비용 산정, 정당한 콘텐츠 증가, 비정상 입력으로 구분한다. 정상 콘텐츠와 근거 있는 예상 확장 규모의 실제 시간·메모리·작업량을 측정하여 기본 한도 변경이나 개발용 설정을 검토할 수 있다. 실패를 감추기 위한 상향과 측정에 근거한 조정을 구분하며 변경 전 실패·측정·설정 식별을 보존한다. 기대 동작·원자성·취소·자원 거부와 설정별 exact/one-short 검증은 유지한다.

현재 F001의 중복 proof·불필요 fingerprint·실제 owned-field 계량 교정은 이미 확인한 계산/산정 문제이므로 중단하지 않고 기존 baseline cap에서 계속하여 개선 효과를 분리한다. 이후 정당한 비용이 남으면 실제 측정으로 설정 조정을 판단할 수 있으며, 기존 cap 초과 자체를 영구 장애나 자동 완료 실패로 취급하지 않는다. 아직 새 기본값·개발 프로파일·확장 규모를 확정하거나 설정 파일 소유를 확대하지 않았다. 실제 한도/공개 계약/완료 기준을 바꿀 때는 해당 P2G/GEX owner에서 이 최신 합의와 측정 근거를 반영하고 필요한 영향 범위만 검증한다. 기존037/002/003 실패와 다음 실제 결과는 구분해 유지한다.


### F001 validation004와 정상·확장 측정 범위

validation004는 원 public030/default baseline에서 release build225.81초 성공 후 simulate1.27초/exit1/stdout0/trace없음이다. 원시 `product_parts/bundle_f001/u09_f001_validation004/04_simulate.stderr`는 final_validation/relic_structural_owned_copy requested10,014,418 >10,000,000이다. 원본 effect 전체를 복사한 것으로 산정하던 교정 후에도 완주 비용이 기존 한도를 넘으며, 이 값은 실패 지점의 누적량이지 전체 필요량이 아니다. 해당 실패를 보존한다.

새 사용자 합의에 따라 기존 공개 CLI `validate --runtime-safety`와 validation_usage를 사용해 완주 작업량, 별도 순차 `/usr/bin/time -v`의 실제 시간·RSS를 측정한다. 기존 `ExecutionLimits::validate`가 허용하는 authoring max_work 상한1<<30 내의 측정 전용 설정을 사용할 수 있다. 이 설정은 새 기본값 확정이나 원 baseline 기능 PASS가 아니다. 원 public030의 정상 structural relic1과 기존 family fixture의 root+6 확장 사례를 우선 검토하며, fixture의 유효성과 기능 기대값은 Astra가 독립 확인한다. 새 제품 계량 API는 현재 필요하지 않다.

Sol은 기존3제품 파일의 allocation overhead 보수 보강 후 원본대비 완결 diff를 제공하고 단독 실행한다. Astra 시험 담당은 기존 RV 경로 안에서 측정 설정/fixture/명령을 준비하여 Sol과 입력 고정·runner0를 직접 맞춘다. 기본 runtime_safety 설정은 아직 변경하지 않는다. Root는 exact full binding/copy UUID, 실제 Any3 admission, one-weapon/form 대안의 보존을 실제 diff와 관련 함수로 검토 중이다. publication held basis에는 해제된 임시 할당도 누적되므로 실제 동시 메모리 사용량으로 보고하지 않는다. native retained 거부가 발생하면 이 과대 산정 가능성을 먼저 분류한다.

Root 실제 코드 검토는 `product_parts/bundle_f001/u09_f001_product005/PRODUCT.diff`(원 F001 preimage 대비3파일)와 호출 경계를 기준으로 수행했다. Catalog token은 definition/skill/form을 보존하고 각 장비의 원래 copy UUID를 key에 포함한다. 실제 Any3 admission을 거친 같은 전체 장비 provider vector만 공유하며, fragment는 합법 one-weapon witness에서 수집한 각 form을 하나씩 별도 compile한다. compiler가 받는 나머지 data/generation/employee/relic 집합은 같으므로 이 범위에서 proof 재사용 근거가 성립한다. `apply_weapon_profile`와 `form_compatibility_report`를 읽어 profile/form predicate의 입력도 확인했다.

publication 전용 원본 effect 계량은 실제 site/key/anchor/body/tags/property 할당과 Vec 교체 순간을 포함하고, source_expr의 실제 amount proof와 그 source 메타데이터를 유지한다. private publication entry는 Result<()>로 discarded plan을 봉쇄하고, runtime fingerprint branch는 보존한다. full closure/typed/recipe 계산 뒤 실제 execution lower와 storage::measure, 최종 cancellation poll도 유지한다. allocation64 보강과 generated effect Vec growth도 확인했다. 이것은 코드 검토 결과이며, 아직 측정·독립 반례와 남은 U09 공개 경로 검증을 대신하지 않는다.


### measurement006 해석과 G001 금액 조회 교정

`product_parts/bundle_f001/u09_f001_measurement006/RECEIPT.json`은 같은 고정 source514/동일 fresh binary의7명령 수집을 완료했다. baseline root1 simulate는 authoring_work_limit/final_validation/relic_structural_deadline_owner requested10,000,008/10,000,000, stdout0이다. 측정 설정의 validate는 root1 work15,745,019·GNU wall0.41초/user0.21초/RSS20,492KiB, root7 work597,706,650·wall0.69초/user0.46초/RSS20,788KiB로 성공했다. 이 수치는 이번 환경의 관측값이며 보편적 처리시간 보장이나 native live peak가 아니다. 두 입력의 authoring metadata peak119,541/123,941, clone peak167은 해당 계량명으로만 해석한다. 원 baseline 검증 통과로 기록하지 않는다.

작업량은 실제 명령 수나 CPU 시간이 아니다. `runtime_control::admit_retained_growth`가 증가분 bytes/8 올림을 authoring 작업량으로 청구하고, compiler의 기존 closed-copy 선계량은 serialized bytes×EffectiveProperty 크기의 보수 상계다. cache held basis를 compile마다 재청구하지는 않지만 독립 proof들의 할당 상계는 전체 작업량에 누적된다. 또한 정상 relic1과7은 중복/침식 없음 조건에서 합법 nonempty relic 집합이1에서 최대63개로 늘어난다. 정당 조합·provider/rule 증가와 보수 산정 증폭을 구분하며, 기존 총계만으로 각 비율을 단정하지 않는다. 한도 결정을 위해 계량을 삭제하거나 원 실패를 숨기지 않는다.

같은 측정 profile의 root1 simulate는 실제 original30+generated75(HP488→458→383)를 소비했지만 마지막 금액 조회에서 `authoring_amount_record_invalid`로 전체 실패했다. 원시06 stdout/stderr와cli045 trace를 보존한다. 기존032 시험은 capture/facts75를 검사했으나 public preview_amount/explain_amount 호출이 없어 이 조회 경계를 검증하지 못했다. 기존 PASS의 증거 범위를 정정하며 폐기하거나 재표기하지 않는다.

Root와Sol/Astra가 실제 코드를 읽어 원인을 확인했다. runtime value_expr_runtime은 structural binding validate/compose 뒤 numeric binding을 적용하지만 amount_query replay는 numeric만 적용했다. G001은 authoring/amount_query.rs 단1제품 파일에서 같은 structural→numeric 순서로 재평가하고 narrowed/typed base·evaluated/trace와 엄격한 결과 비교를 유지한다. 기존 effective_relic_amount는 두 binding 중 하나라도 있을 때 최종 결과를 제공한다. 새 DTO/owner 탈출·현재 World 재구성·게임 실행 의미 변경은 없다. Root는 `u09_g001_product007/PRODUCT.diff`의 실제14행 추가/기존 투영 변경을 검토해 설계 적합성을 인수했다. 독립 query base40/final75 oracle와 기존 numeric 회귀 등 관련 실행은 아직 준비/검증 중이다.

Sol은 G001 제품/단독 runner, Astra는 기존 authoring/trial/tests/u09_structural.rs와 하위·structural/tests/publication_cases.rs·필요 structural.rs 등록 및 기존 RV 시험 작성 소유를 재개한다. 같은 파일 동시 편집은 없고, 입력 고정 후 F 반례·G query·남은 공개 소비를 관련 묶음으로 실행한다. 기본값/개발용 정식 설정은 아직 변경하지 않았다.

G001 기대값 정정: Root가 원 public030 relic/skill RON을 직접 확인했다. generated primitive40에Add10 후Scale increment50%가 적용되어75이며, original23+Add7은30이다. 처음 논의한 base50은 중간 Add 결과와 원본 기본값을 혼동한 것이다. 아직 query 성공 관측값은 없으며 독립 oracle를 typed/narrowed base40·effective75로 정한다. 제품은 값을 하드코딩하지 않아 변경 불필요하다.

최신 canonical 반영 확인: 자원 합의는 현재 개정 연속 지침 §5-I, Core-first ROADMAP §6.4와 core_runtime_contract의 Trusted Runtime Safety Profile에도 반영되어 있다. Root는 이 변경 절을 읽어 인수했으며 roadmap/canonical 원문은 수정하지 않았다. 이 지침에 따라 정상·확장 입력의 단회006 측정 외에 후속 기능 묶음 종료 후 같은 root1/root7/profile1b validate를 각각5회 순차 측정하여 표본·최솟값·최댓값을 확인한다. 기존 명령/collector를 사용하고 성능 실행은 기능 검사와 격리한다. p95나 보편적 시간 보장을 주장하지 않는다. 프로세스 종료/회수와 RSS를 기록하되 native/heap peak로 바꿔 부르지 않는다. 개발용 설정의 정식 채택 판단은 이 분포와 관련 거부·취소·원자성·재시도/profile 식별 검증을 함께 검토한 뒤 한다. 현재 단일 측정 profile은 기본값 변경이나 완료 판정이 아니다.

Root는 최종 고정된 F/G 독립 시험과 판정 도구를 실제로 읽었다. F의5시험은 네 장비를 요구하는 불가능 union의 허용/세 장비로 묶인 실제 cross-provider 충돌 거부, Not Exists의 별도 합법 witness, 같은 skill의 다른 장비 definition 및 Normal/Corrupt·invocation 격리, 같은 skill의 form/weapon 호환 구별, lower 단계 selected reach 충돌, 실제 작업량/누적 보수 retained의 exact/one-short와 취소 재시도를 검증한다. form 반례는 정상 base가 충돌 zz_alternate보다 먼저 검증되도록 보강했다. 자원 시험의 수치는 live heap/RSS가 아니라 기존 admission 계량으로 명시한다.

G의 새 시험은 실제 TrialSession의 Active/Exact preview/explain4회 호출로 typed/narrowed base40·effective75와 동일 record identity/trace, nonzero fingerprint, query 후 receipt·accepted work·인증 state 불변을 확인한다. CLI comparator는 Active2문서의40/75와 첫 전투 LastTerminal2문서의 unavailable/not_yet_evaluated를 분리한다. 첫 전투는 active fragment/장비 능력 없이 기본 공격만 수행한다는 정적 fixture/경로 근거로 기대값을 정했으며 실행 관측값을 복제하지 않았다. 별도 기존 legacy query 시험은 legacy 회귀이며 구조적 LastTerminal 성공 증거로 대신 표시하지 않는다.

Sol이 source/입력 고정 뒤 F5, public U09 root+families+G3, exact legacy1, fresh CLI와 원006실패/새query comparator를 관련 묶음으로 실행한다. 기존 publication040의2시험은 본문·제품 의미의 비영향으로 재사용하며 재실행하지 않는다. 자동 타겟팅 family의 잘못된 수동 BeginSkillTargeting 단계만 정상 RequestBattleState로 바꾸고 원 실패와 기대 소비값은 유지한다. 아직 신규 시험 실행 결과는 미수집이다.

기존 RV 증거 추가 품질 검토: Root는 personal_cases의 RV002 본문, selected_cases/support_cases/world_state_cases 및 관련 native_probe를 실제로 읽고 personal029 raw15/15와 world025 raw18PASS/2FAIL을 확인했다. personal029는 source Snapshot/Live·recipient 수와 무관한1회 평가, release 시점 집합, 별도 source/effect 그룹, nonchannel 실제 Silence/Stun buff consumer, 소진/재충전 제외, lethal/withdraw/redeploy/retreat, late cancellation와 native admitted boundary의 원자 retry를 검증한다. buff 소비자 직접 입력 및 준비용 위치/스탯 설정은 정상 외부 RON 소비와 구분한다. world025의 selected/fixed/regular/World/Continue PASS는 별도 해당 범위만 재사용하며 전체 묶음 PASS로 바꾸지 않는다. 두 실패는 personal029의 후속 교정과 연결한다.

이 검토에서 지원 효과 시험의 coverage 누락을 발견했다. `fixed_corrupt`라는 기존 시험 branch는 실제 Corrupt 전환이 아니라 Normal fixture의 damage edit만80%로 바꾸고 speed20을 유지한다. 기존80/20 PASS는 원 기록대로 보존하며 PE09의 침식80/30 검증으로 확대하지 않는다. Astra에80/30 공격·속도 실제 소비 보강과 실제 variant 전환 증거의 별도 연결 여부를 검토하도록 반환했다. 현재 실행 입력을 바꾸지 않고 다음 안전한 시험 교정 경계에서 같은 RV 묶음으로 처리한다. 제품 결함 판정은 아니다.

F/G 결과 인수: batch008/046의 import 컴파일 실패는0시험으로 보존했다. batch009/focused050은3PASS/2FAIL이며 실패2건은 시험 작성의 필수 corruption penalty/exact fragment selector 누락으로 목표 경로 이전에 거부된 것이다. 해당2조건만 교정한 batch010의 variant054/form055가 각각1PASS이며, 비영향050의3PASS와 합쳐 F5가 유효하다. Root는 실제 시험 교정과 원시050/054/055를 확인했다. F exactwork544,787·누적 보수 admission3,768,386의 one-short 거부/exact/retry와 두 취소 지점 성공은 해당 의미로만 인수한다.

public056은3시험 중2PASS/1FAIL이다. 새 G 실제 Active/Exact preview/explain4문서의 typed/narrowed40·effective75와 같은 identity/nonzero fingerprint를 Root가 raw public056_amount/queries.json에서도 확인했다. 기존 root의 실제30+75 소비·fresh Continue·관측 모드 동등성도 통과했다. family는 RequestBattleState로 교체한 단계의 남은 resolved context가 맞지 않아 authoring_context_invalid로 실패했으며 목표 소비 미도달이다. Astra가 이 시험 입력과 승인된 실제Corrupt80/30 보강을 다음 같은 쓰기 경계에 묶고 Sol이 영향 family/support와 아직 미실행인 legacy/CLI/반복측정을 계속한다. F5와 G/root056은 제품 비영향 근거로 재사용한다. 실제 CLI 전체 성공·정식 개발 profile·U09 완료는 아직 미판정이다.

## 2026-10-09 재개 인수와 명시적 authoring 개발 설정

사용자의 계속 진행 지시 뒤 한 번의 확인에서 이전 담당자는 사용 불가였고, 보이는 프로세스에는 cargo/시험 runner가 없었다. 전체 호스트 정숙을 주장하지 않는다. 현 소스·F EXECUTION·RV 원본은 기존 기록보다 진행되어 있어 되돌리지 않고 인수했다. 새 Sol/xhigh와 Astra/high를 위 표에 배정하고 실제 모델 확인을 받는다. 과거 작성·실행 귀속은 보존하며 모델 교체만으로 재실행하지 않는다. family068의 focused 결과는 완료됐고 outer wrapper 수집만 Sol이 복구한다.

Root는 support063 원시8분기의 실제 Corrupt80/30·regular180/triggered100과 legacy064의1시험 통과를 확인했다. 앞서 검토한 실제 Corrupt 전환·770ms interval·HP·공격자 귀속/자원 시험과 연결해 인수한다. CLI066은 validation014의 종료 성공과016의 판정 결과를 함께 사용한다. Root가 comparator의 top-level/payload 파싱 교정과 리터럴30/75·HP105·Active40/75·LastTerminal unavailable/not_yet_evaluated·native4문서 검사를 읽었다. 원006의 gameplay PASS/inspection FAIL은 구별해 보존한다. family059의 closure 소비만 유효하며,067/068의 area 보호량 소모 실패와 나머지 미도달은 시험 담당자가 실제 원인을 분류한다.

measurement015는 같은 고정 binary·원 root1/root7·측정 profile에서 각각5회 종료0/회수 완료다. Root가 기존 요약과 대표 GNU time 원시를 확인했다. root1은 work15,745,019·wall0.42–0.43초·CPU0.24–0.25초·RSS20,192–20,548KiB, root7은 work597,706,650·wall0.82–0.83초·CPU0.64–0.66초·RSS20,584–20,948KiB다. 표본 범위이며 p95/하드웨어 일반 보장 또는 native heap peak가 아니다.

§5-I의 기존 권한으로 명시적 trusted offline 개발 preset `config/runtime_safety_authoring_dev.ron`을 채택하기로 결정했다. Sol은 기존 설정과 동일하되 authoring_execution.max_work만1,073,741,824로 지정하고 기존 CLI 선택법을 core_runtime_contract에 짧게 연결한다. 측정 profile과 같은 effective identity를 사용하며 배포 기본10m/native/engine ceiling은 변경하지 않는다. 정상 확장 root7의 측정 work 대비 남은476,035,174는 이 입력의 여유이지 임의 콘텐츠 규모 보장이 아니다. 조합 증가와 보수적 admission 비용을 수용하는 유한 offline 선택이며 최적화 성과로 보고하지 않는다. 기존 기본 설정 실패, 실제 초과/취소·원자성/retry·profile 식별 검증과 비영향 근거를 유지한다. 새 파일의 파싱/동일 effective 값과 영향 있는 기존 profile 회귀만 검증하며, 주석 차이만으로 전체 CLI/성능을 반복하지 않는다.

후속 기능 인수: family068은 적의 initial resonance100/max100와 pending_cast=false 때문에 실제 자동 시전이 발생하지 않은 fixture 오류였다. Root가 add_resonance의 below-max→max 조건을 읽었고, start90에서 정상 기본 공격+10으로 도달하도록 교정했다. validation018의069–072는 같은 고정 입력에서 area/single/support/monk 각각1PASS다. Root는 원시 출력과 실제 consumer assertion을 확인했다. area450/단일825 보호량 및 실제 enemy pressure1 흡수, 지원40/20의 실제 아군 기본 공격, Monk 각 적 Physical40·자기 cost10 뒤 회복7 1회를 인수한다. closure059의 이미 유효한 소비는 별도 재사용하며 과거 실패 묶음을 전체 PASS로 바꾸지 않는다.

Root는 `u09_dev_profile001/PRODUCT.diff` 전체와 `u09_canonical_docs001/PRODUCT.diff` 및 관련 schema/capability source를 실제 검토했다. 개발 설정의 숫자 범위와 기존 작성 문서의 구조 유물 참조를 인수한다. 실제 명령은 `RequestRelicEffectivePlan`/`request_relic_effective_plan`이며 prospective DTO는 실행 권한이 아니다. 금액 조회는 기록된 structural→numeric 평가 단계이며 후행 개인 source_creation·수혜자 pool 생성량까지 같다고 보장하지 않는 점을 문서에 교정했다. 문서 변경을 근거로 제품 검사를 반복하지 않는다.

현재 통합 범위는 `test_parts/runtime_validation001/IMPACT_ROSTER.json`의69개 기존 검사다. U08 carrier/World owner·relic admission, R10 Flow, R11 support 수명, R12 보호·실제 HP 기반 lifesteal/방출 provider, World 저장/rollback, target usefulness, profile identity, 관측 owner, 서버 paused/reconnect/tick·manual-retreat fault를 변경된 공통 경로에 따라 선정했다. Root가3개 lifesteal/수동철수 실제 본문을 검토해 추가했고 이미 선정된 World parry 검증을 이유로 전체 parry군까지 확대하지 않았다. 같은 구성의 libtest는 한 번 빌드하고 선택 목록을 묶어 실행한다. 아직 실행 결과를 PASS로 기록하지 않는다.

PC/R05의 `performance001/ADAPTATION.json`·RUNPLAN과 실제 adapter/helper/comparator를 Root가 읽었다. 이전 probe·workload·31성공5거절·cap·strict cost oracle를 보존하고 current source/output/import 및 필요한 compile-time fixture 결속만 U09로 옮겼다. 이전 원본과 frozen 출력은 변경하지 않는다. 기능 검사 종료 뒤 단독 실행하며 실제 source/layout/cost 차이는 원 실패와 수치를 보존해 §5-I로 분류한다. 원 strict comparator를 새 비용에 맞추어 자동 완화하지 않는다.

통합 결과019/020: 기존69개는 Core 기본64·runtime-observation1·서버4 모두 통과했다. Root는 Core 원시 종료 요약과 서버4개 전체 raw를 확인했다. 최초 서버 release 빌드는 의존 Core의 test-fixtures 미선택으로31개 컴파일 오류/0시험이었고, 같은 제품 소스에 명시적 feature를 추가한020에서4개가 통과했다. 최초 실패는 보존한다. 각 실행은 입력 불변·종료 회수 완료이며 새 테스트 보조 코드 변경은 기존69의 제품 의미를 바꾸지 않는다.

실제 수용 문장 PE10.2/11.2의 남은 경계를 두 rv005 시험으로 보완했다. Root는 두 함수와 준비 helper를 읽었다. Fixed support는 원래 Self HP cost10, 선택 수혜자 불변, Pause, 재입력/실제 Stun 중단, HP10에서 소비 틱 HealthCostUnavailable 및 수락 후 snapshot 복구/재시도를 확인한다. footprint는 빈 수혜자 집합·준비된 native 무적·lethal/nonlethal 실제 HpLost와1회 Self 회복을 확인한다. 무적 준비를 공개 parry cast end-to-end 증거로 확대하지 않는다.020에서는 helper의 private field 접근으로0시험 컴파일 실패했고 제품 visibility 변경 없이 기존 native parry 처리로 시험 측 교정 후 이2개만 재실행한다. 아직 신규2개와 PC/R05는 미완료다.

최종화 준비: Root의 `finalization001/PREDECESSOR_REVIEW.ko.md`는 원본191조건+32문장과 별도31개 R06 조건의 실제 의미 및 baseline438→현재 제품 경로를 검토한 비영향/영향 분리 근거다. Astra의 collector와 bindings를 Root가 실제 읽었으며254행을 원본 pointer+canonical digest로 연결해 재귀 복사를 피한다. 이 도구는 현재성 검토를 위한 증거 목록 생성이며 완료 판정이 아니다. 실패한 부모 묶음·현재 미결 증거·미래 독립 감사는 분리 유지한다. 실제 수집·고정 target/복원·비참여자 전체 Goal 감사는 남아 있다.

기능 보강 종료024:021의rv005 두 실패는 장애물 좌표/RON 치환의 준비 오류였고022에서 교정 후 본 동작에 도달했다.022의 추가rv006은PE06.1의동일SkillId 두 직원 구조계획을 한전투에서 분리해 검증했다. 정상유물30+75와 실제침식40, delay0/20·Pause접수→Resume, 직원별실제피해105/40와query·sourcecopy·공유원형불변을 Root가 본문과raw에서 확인해 이한행만 인수했다. 전체022는rv005 두실패로FAIL을보존한다.

rv005채널중단은전용정상명령`ReinputSustainedSkill`의activation을사용하도록시험만교정했다. lethal회복실패는제품결함이었다. Root가실제SkillStep/Flow/source/cancel코드를검토한뒤Sol이023에서`sim.rs`의원래타겟생존검사에실제구조side occurrence 예외만추가했다. 일반source의`use_control`bool로권한을추정하지않고기존livechild조회·prelease·단계target·취소규칙을보존한다. Root는실제diff전체를인수했다.

024의exact9는rv0052와변경영향originaltarget/refund/CC·side반복/철수/latecancel/resource7이며모두PASS다. Root가fullraw에서lethalHpLost10→Self537회복1·nonlethal/empty/invul,채널cost10/Pause/reinput/Stun/부족rollback을확인했다. 새로운실행tick의nativework최소326456/one-short326455에서acceptedstate전체복구·exactretry도통과했다. 원래69와rv006의비영향결과를재사용하며fullsuite를추가하지않는다. 현재기능결과를인수하고승인된PC/R05단독성능실행을이어간다.

Root는`finalization001/freeze_target.py`와실행계획전체를검토했다. 기존baseline438및3canonical문서preimage와현재scanner union을격리before→after→before로정·역복원하고정확한경로/byte를검증한다. U09원실패기록과참조는보존하며기존tar를재귀삽입하지않는다. writers/functional/performance runner0·기능/성능미결없음이실행조건이고,복원/audit만미래gate로남긴다. 아직이도구의실제실행이나독립완료는이루어지지않았다.

## 최종 기능·성능 증거 인수와 동결 준비

현재 실행 입력은 `finalization001/CURRENT_SOURCE.json`의492파일/`1ca0fbe68a2ba80603470dde38a6c50e67c9d0ca3938391dbc4b211e875c492e`다. Root는 새 source와 기존 증거를 명시적인 영향 경로에 따라 검토했다. 기존69개는019/020, 동일 스킬 두 직원의 구조 계획 분리는022의rv006 통과행, 마지막 제품023의 직접 영향9개는024에 연결한다. 이전 묶음 전체 실패를 일부 통과행으로 대체하지 않는다. 정상 외부 RON·보상/장착·실제 소비·query·fresh Continue, F publication 반례·자원/취소와 G query 결과의 재사용 근거는 앞선 실제 코드 검토와 최종화 입력에 유지한다.

PC는 격리 복사본에서 한 번 실행해36조건/576행,31성공·5typed refusal를 확인했다. Root가 실제 probe 종료와 gameplay/cost comparator 원시를 읽었다. 직전 U07M에 대한 시간 외 관측은 allowed_delta0으로 일치했고17개 음성 대조군도 거절됐다. 이전 U06에 대한 기존200바이트 owner accounting 차이와 historical requested-detail80개는 별도 유지된다. compiler layout은 source136/object152/placement48이며 새 비용 예외를 만들지 않았다. 격리 준비 digest5941e61e와 실행 digest1ca0fbe는 동일492파일의 tuple/dict 직렬화 차이로, 서로 다른 소스 범위가 아니다. 이 결과를 시간 보장이나 새 구조 콘텐츠의 무비용 주장으로 확대하지 않는다.

R05 최초001은 고정 dependency 캐시 누락으로 exit101/측정0이었다. 정확한 locked 패키지만 복구한 뒤002를 offline/locked로 실행했고 소스·lock·workload·cap은 바뀌지 않았다. Root는 현재 summary 전체, 원시 panic, 기존 summarizer와 실제 latency gate 코드를 읽었다.002의 원 exit101은 concurrent peak 한도9030188에 대한 관측9709348바이트 실패로 보존하며, 기존 `warning_concurrent_peak_only` 분류를 적용한다. 다른 hard guard 실패는 없고 repeatability/parity가 유지된다. event-heavy ops44206/retained266102/peak271927는 원 cap 안이다. concurrent ops370928와 retained4196928은 직전 U08과 같고, peak는 직전9683689보다25659바이트 높다. 이는 기존 경고의 현재 관측이며 신규 한도 상향 근거로 쓰지 않는다.

allocation-observer 모드의 R05는 원래 `latency_gate_applicable=false`다. 이번 concurrent p95는10014462ns로 관측10ms cap을14462ns 초과했고 `absolute_cap_pass=false`다. 기존 통계 판정에서는 회귀가 검출되지 않았으나, 이를10ms 충족이나 배포 성능 보장으로 바꾸지 않는다. 직전 U08 역시 이 관측 cap을 넘었다. event-heavy의 직전 대비+12 allocation operations/+256 retained/+512 peak도 숨기지 않으며, 정확한 원인 귀속은 별도 source 분석에서 입증된 범위만 사용한다. 이 작은 증가를 정당화하기 위한 재측정·oracle 변경·cap 변경은 하지 않았다.

선정한 기능 검증의 필수 미결은 해소됐고 성능은 위 기존 경고/관측 한계를 포함해 인수한다. 이제 준비된 collector로34수용 문장·17seam·10dimension·254선행 행을 실제 증거에 결속하고, generated 결과 검토 및 문서 handoff 뒤 격리 정·역복원과 exact target 동결을 수행한다. 이 인수는 GEX-20 독립 완료 판정이 아니며, 전체 Goal을 작성에 참여하지 않은 Astra/high에게 감사받아야 한다. 후속 감사 finding은 해당 담당자가 변경 영향에 맞게 교정한다.

## target029 전체 감사 인수와 F01 교정

`target/u09_current_029/MANIFEST.json`의 source492 및 canonical 문서3개를 격리 정·역복원하여 before441/after495의 membership·bytes 일치를 확인했다. 원 작업 트리에는 역패치를 적용하지 않았다. `finalization001/generated003`과 `HANDOFF_029.json`은 그 시점 자체검증 결속이며, 원본과 target029는 보존한다.

비참여 `/root/u09_independent_audit029`의 실제 배정은 gpt-6-astra/high로 최초 확인했다. [전체 감사](../../post_audit/u09_current_029_pass001/COMPLETION_AUDIT.md)와 [판정](../../post_audit/u09_current_029_pass001/AUDIT_CLOSURE.json)은34문장·17seam·10차원·3node·254선행 행을 모두 검토하고 `Not Complete / High / open1(F01)`로 닫았다. 입구·출구 각각2641 참조의 불일치는0이었다. 감사는 동적 재현을 수행했다고 주장하지 않았으며, 유효한 기존 증거와 R05 경고를 보존했다. Root는 감사 보고서·finding·closure와 실제 관련 코드를 읽고 교정 범위를 인수했다.

F01은 같은 provider의 원래 효과에서는 거절하던 서로 다른 selected recipient reach를 생성 `InlineEffects`와 `FiniteFlow`가 공용 값에 덮어쓰는 결함이다. 정상 SelfUnit/Instant 지원에 생성 area personal grant를 추가하고 single recipe를 적용하면 pre-overlay carrier와 최종 generated-site 검사를 통과할 수 있다. 원 fixed support의2m와 생성 single의4m가 결합될 때 fixed admission과 저장된 연결까지4m를 소비한다. 기존 원래 효과끼리의 충돌 시험은 이 경로를 검증하지 않았다. 같은 스킬의 충돌 거절은 이미 공개 문서와 제품의 원래 경로가 약속하므로 새 정책 승인이 필요하지 않다.

교정은 모든 원래/생성 reach 기여에 기존 typed conflict 거절과 동일 reach 허용 규칙을 적용한다. 독립 시험 Astra가 publication 및 관련 World 시험/fixture를 소유하고, Sol은 `structural/execution.rs` 제품과 단독 실행을 소유한다. 먼저 준비된 반례를 수정 전 소스에서 실행해 실패 원본을 남긴 뒤, Sol이 입력이 닫힌 경계에서 최소 제품 교정을 수행하고 같은 관련 묶음을 검증한다. 원래/생성 및 생성/생성, Inline/Finite, 큰·작은 충돌과 효과·grant·사본 순서, 동일 reach 정상 시전·실제 연결 사거리, 적용 가능한 공개 게시/전환 거절의 원자성이 범위다. 불가능한 invalid catalog를 우회 주입하여 정상 공개 검증으로 주장하지 않는다.

기존 original-only 충돌과 영향받는 compiler/publication/selected-consumer 회귀를 포함한다. 영향 없는 기능·PC/R05를 최종화라는 이유로 반복하지 않으며 실제 제품 delta를 읽고 재사용 가능성을 판단한다. 시험 기대값·한도·workload를 완화하지 않는다. 감사 중에는 모든 입력 쓰기를 멈췄고 출구 확인 뒤에만 이 교정을 시작했다. 성공 후 새 현재성 결속·동결·독립 재감사가 필요하며, F01이 닫히기 전 U09 완료나 U10 진입으로 게시하지 않는다.

## F01 동적 재현·교정과 후속 수명 결함034

수정 전030은 기존6통과/새3실패였으며 일부 잘못된 게시를 관찰한 뒤 시험의 수명 참조 순서·RON 직렬화 문제로 나머지에 도달하지 못했다. 시험 준비를 교정한031에서는60개 변환 전 입력과 동일 사거리20개가 모두 게시됐고, 충돌40개도 모두 잘못 게시되어 F01의 Inline/Finite·원래/생성·생성/생성·순서·큰/작은 reach 경로를 실제 재현했다. 별도 live World를 보유한4개 잘못된 후보도 게시됐다. 두 실패 원본과 당시 입력은 보존한다.

Root는 `u09_f01_product032/PRODUCT.diff`의 한 파일 교정을 실제 검토했다. private `merge_selected_recipient_reach`가 원래/Inline/Finite의 세 반환값에 기존 동일 검사·진단을 적용하며 provider별 누적 범위, 대상 소비, 실행 수명과 계량 호출은 유지된다.033은7통과/2실패다. 게시60개는 이제 기대한40 typed refusal/20허용을 만족하고 기존6회귀도 통과했다. 기존 work544787/보수적 retained3768386의 exact·one-short·취소·재시도 결과도 같다. 성공7개는 그 범위로 재사용하며033 전체를 PASS로 바꾸지 않는다.

World 유한 생성 효과의 첫 관찰 오류는 원 ChannelAura의 정산 전에 child를 기대한 시험 전제였다. 기존 `EffectsSettled` 규칙을 바꾸지 않고 같은 Skill/provider의 별도 Self/Instant Heal1 anchor를 가진 정상 positive로 수정했으며, 정확한 channel-anchor 충돌은60개 게시 시험에 남겼다. 그러나 이 정상 positive도1ms에 pool0이었다. 시간을 임의로 늘리지 않고034의두 시험에 읽기 전용 진단을 추가했다. 실제 두 결과는 ally 대상으로 접수·시전한 뒤 첫 Self Heal1(500→501)만 실행되고, resonance use가 consumed100/control_cancelled=true, cast/관련queue는 비며 sustained started/ended/active가 모두0이었다. 생성 후 회수나 단순 늦은 wake가 아니라 나머지 원래 행동과 생성 child 전의 조기 취소다.

Root와 시험 Astra가 실제 `sim::flow_step_dispatched` 뒤의 `cancel_uses_with_unavailable_targets`, `resonance_runtime.rs`의 canonical Skill 조회, `admitted_skill_target_is_valid`의 Self 분기, Flow의 남은 use-control 검사를 읽어 원인을 확인했다. 선택 대상형으로 접수된 effective skill을 canonical Self 스킬로 다시 판정해 살아 있는 ally를 잘못 거절한다. 이는034에서 확인한 추가 GEX-10 제품 결함이며, 원 독립 감사029의 역사적 finding 수나 판정을 수정하지 않는다.

Sol의 제품 소유를 `resonance_runtime.rs`의 대상 수명 검사와 그 work step-count로 확장한다. 실제 accepted action source의 보존된 skill을 같은 경계의 검사·계량에 사용하며, 현재 loadout 재선택이나 canonical Self 규칙 완화로 우회하지 않는다. 원래 use_lifetime 권한 검사는 별도이므로 일괄 교체하지 않는다. 시험 Astra는 기존 accepted-owner 제거 helper로 같은 유한 positive를 보강한다. 다음 범위는 World2, 기존 pending-plan 보존1, original target 무효화/철수/지연 CC3, 실제 lethal tick의 native exact·one-short·전체 rollback/retry1, 동일 SkillId 두 직원의 서로 다른 binding1로8개다. 제품·시험의 서로 다른 파일만 병행 준비하며 실행 전 두 writer를 닫는다. 이전 성공7개는 비영향 범위를 유지하고, 이번 runtime delta의 성능 증거 재사용은 실제 교정 코드를 읽고 별도로 판단한다.

## 교정036 인수와 후속 전체 재감사 준비

제품035 전체 diff·실제 source 및 post036 전체 raw를 Root가 읽었다. 실제8PASS/0FAIL/0ignored이며 Inline/Finite/Finite-current-owner-removed의 personal200·Self channel2m·전체 연결 geometry, staged4 typed refusal와 live roots 보존, retained pending825, 원래 invalidation/withdrawal/CC, 동일 SkillId 두 owner delay0/20이 통과했다. native whole1ms 최소326255/one-short326254에서 accepted-state 전체 rollback·exact retry가 확인됐다. 이전326456과의 −201 전체tick 차이를 특정 site 비용 개선으로 귀속하지 않는다. 제품·시험 source는036 실행 전후492파일 동일이다.

Root의 [현재성·범위 검토](finalization001/F01_CURRENTNESS_REVIEW.ko.md)가 post033 성공7 재사용, post036 실제8, 기존 전체 Goal/선행 증거의 비영향 근거 및 한계를 소유한다. 일반 Employee의 종료/취소 source가 없을 때 canonical step 보수 debit이 사라질 수 있고 live effective generated steps는 실제 계량되므로 제품 전체 work 불변을 주장하지 않는다. PC는 TestFixture, R05는 Abnormality만 생성하여 유일 Employee resonance_use gate에 도달하지 않음을 실제 코드로 확인했다. 기존 PC/R05 측정과 strict oracle 및 원경고를 재사용하며 추가 성능 실행·canonical meter floor·cap 변경은 하지 않는다.

최종화 Astra는033 성공7/036 World2/036 나머지6의 겹치지 않는 receipts와 두 의미 composite를 결속한다. 원실패030/031/033/034, target029 및 실패 전체 감사는 그대로 보존한다. Root 문서·최종화 입력 writer가 닫힌 후 Sol이 successor collector를 실행하고, Root generated 의미 검토 뒤 새 target 격리 정·역복원·동결과 기존 비참여 감사자의 전체 Goal 재감사를 이어간다. 이 인수는 독립 완료 판정이 아니다.
