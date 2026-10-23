# U09 F01 및 후속 대상 수명 교정의 현재성 검토

Root Astra의 실제 코드·시험 의미 검토다. 독립 완료 감사가 아니며, 기존 `PREDECESSOR_REVIEW.ko.md`와 target029의 유효한 비영향 검토에 아래 변경 판단을 추가한다. 기준은 target029 이후 제품032/035와 실제 post033/post036이다. 현재 실행 source는 `2fdaa433e7ba457531508fd2ab05547cd05373c77cfb2f6820108ccf651754f6`(492 files)다. 문서·최종화 결속 변경은 제품 실행 입력 변경이 아니다.

## 제품과 시험의 실제 변경

- `product_parts/bundle_f001/u09_f01_product032/PRODUCT.diff`: `execution.rs`의 original/InlineEffects/FiniteFlow 반환 사거리에 같은 private merge 검사를 적용한다. None은 기존 값을 유지하고 동일 사거리는 허용하며 서로 다른 값은 기존 InvalidReference 진단으로 거부한다. provider 순서·target 소비·수명·자원 한도 변경이 아니다. 원래 요구된 충돌 거부의 구현 누락을 교정했다.
- `product_parts/bundle_f001/u09_f01_runtime_product035/PRODUCT.diff`: `resonance_runtime.rs`의 대상 수명 검사와 step-count가 live accepted action source에 결속된 skill을 사용한다. 현재 장비를 재선택하지 않는다. pending cast 또는 matching Flow의 use-control 의무가 있을 때 보존 출처가 없으면 기존 취소 경로를 따른다. current-action deferral, 이미 취소된 use 제외, 원래 Self target 규칙, original use_lifetime 권한 검사, scheduling과 독립 child lifetime은 유지한다.
- 원래 channel에 결속된 생성 효과의 충돌 matrix는 그대로 유지했다. World의 유한 positive는 같은 skill/provider 안의 별도 Self/Instant Heal1 anchor를 사용한다. 이 anchor가 먼저 끝나야 1ms에 유한 child가 존재한다는 Flow 계약과 일치한다. channel-settlement 전에 child를 기대했던 pre031 원실패는 보존한다.
- 같은 World positive에 현재 소유 장비를 제거하는 세 번째 변형을 추가했다. 기존 helper로 current Self와 retained PlayerUnit을 확인하며, 시간·pool200·사거리2m·고정 연결 전체 geometry 기대값을 바꾸지 않았다. 진단 출력은 실패 관찰용이며 oracle이 아니다.

Root는 두 제품 diff 전체와 대상 의무 조회, accepted_source/validate_live_action/settle 경계, skill_for_source, 시험의 anchor와 owner-removal helper를 직접 읽었다. 공개 API·저장 형식·canonical 규칙 문서의 추가 변경은 없다.

## 실제 실행과 재사용 범위

post033 전체는 7PASS/2FAIL로 남는다. 그중 게시 matrix 및 기존 회귀 7개만 재사용한다. 60개 pre-overlay는 모두 유효하고 transformed 후보는 40개 충돌 typed refusal/20개 동일 사거리 허용을 만족했다. 게시 work544787·보수 retained3768386의 exact/one-short·취소·재시도도 유지됐다. 035는 publication/lowering/compiler 경로를 변경하지 않아 게시·recipe 검증 결과를 재사용한다. 나머지 fixed-support/fixed-channel-cost/plain-selected 런타임 대조는 실제 fixture도 다시 읽었다. 원래 한 action의 즉시 소비 뒤 검사할 추가 pending action이 없는 구성이라 이번에 드러난 첫 action 뒤의 미실행 continuation 취소와 구별된다. source 불변만으로 전체 시험을 재사용하는 것이 아니며, shared geometry는 같은 helper를 사용하는036의 세 변형, target/CC와 자원 경계는036의 직접 영향 검사로 보완했다. 원래 채널 비용·reinput·회복 조건의 제품 경로는 바뀌지 않았다.

post036은 실제 8PASS/0FAIL/0ignored다. Root가 `test_parts/runtime_validation001/performance001/evidence/f01_post036_tests.log` 전체를 읽었다.

- World의 Inline/Finite/Finite-current-owner-removed 세 변형 모두 personal200, Self channel, reach2m, regular40/speed20, out/return 동일 연결, recipient 철수 뒤 terminal/no-retarget을 만족했다.
- staged4 충돌은 정상 publication에서 typed refusal로 거부되고 held catalog/loadout/resource/save/owner가 보존됐다. 이후 합법 명령의 실제 소비도 통과했다. 게시 불가능한 자료를 World에 주입한 실행 증거로 표시하지 않는다.
- pending selected personal은 현재 소유 제거 뒤에도 보존 계획으로 선택한 ally에825를 전달했다. 기존 manual target invalidation·target withdrawal·unreleased delayed step hard-CC도 통과했다.
- 실제 lethal tick은 HP loss10 뒤 Self37 회복, empty/invulnerable은 회복 없음, nonlethal은 기존 결과를 만족했다. native whole-tick 최소 work326255, one-short326254의 application_deployed_context 실패에서 accepted roots 전체 rollback과 exact retry가 확인됐다. 과거 최소326456은 원기록대로 유지한다. 전체 tick의 −201 차이를 한 site의 비용 감소나 최적화 성과로 귀속하지 않는다.
- 동일 SkillId의 서로 다른 두 structural owner는 delay0/20에서 각각 Normal30+75/Corrupt40 및 actor105/40을 유지했다. query/copy/prototype 격리도 통과했다.

최종 collector의 선택은 서로 겹치지 않는 post033 성공7, post036 World2, post036 나머지6으로 결속한다. F01의9개 및 GEX10의8개 composite는 의미별 참조이며 서로 다른 17개 실행으로 더하지 않는다. 원150에15개 named selections가 추가되는 것은 선택 이력의 합계이며 고유 시험 수나 이번 fresh 실행 수가 아니다. 이번 fresh batch는8개다.

기존 전체 Goal 증거, original69·configuration/server 관찰·외부 CLI·public root/DEV preset·선행254행은 각 실제 원source와 제한을 유지한다. 이번 두 private 경계의 직접 영향은 위 publication/retained-owner/target cancellation/native resource 검증으로 보완한다. 불변인 lowering 외 compiler·DTO·save/clone·public adapter·configuration 경로를 절차 전환만으로 반복 실행하지 않는다. target029 감사의 원 F01 및 이번 추가 runtime 결함은 이 후속 증거로 교정 제시하며 독립 감사자가 닫아야 한다.

## 작업량 변화와 성능 증거의 한계

035의 작업량은 제품 전체에서 불변이 아니다. live effective skill의 생성 steps가 대상 검사의 기준이므로 해당 step 수를 계량한다. 반대로 일반 Employee의 취소/종료 use가 잠시 남아 있어도 action이 settled되어 active map에서 제거되면 accepted_source는 unavailable이다. 이 경우 실제 skill.steps 검사도 없으므로 기존 canonical steps 보수 debit이 제거될 수 있다. unit/cast scan의 기본 debit, authoring/native 두 charge, 제한·rollback 의미는 유지된다. 현재 소유 skill을 fallback하여 실행 권한을 부여하거나 work를 숨기지 않는다. 단지 과거 수치와 맞추기 위해 별도 canonical floor를 추가하지 않는다.

Root는 실제 생성 경로와 고정 성능 fixture를 읽어 PC/R05 재사용을 판단했다. `resonance_use = Some` 생성은 `admit_employee_resonance_use`의 Player/Employee gate 뒤 한 곳뿐이며 초기값은 None이다. PC의 battle 및 adapter는 `value_expr_runtime::tests::draft`의 TestFixture를 사용하고 kernel은 expression 검사다. R05 event-heavy와 concurrent-session은 `final_performance.rs::spawn`의 Abnormality만 사용한다. 따라서 이 고정 workload는 새 accepted-use helper와 step-count 변화에 도달하지 않는다. 제품032의 충돌 거부도 해당 고정 후보에 새로운 분기를 만들지 않는다. 새 할당·clone·durable field나 layout 변경은 없다.

이 근거로 기존 PC36/576행의 strict immediate-U07M allowed_delta0, 17 negative controls 및 R05 원측정을 재사용한다. 새 source에서 재측정했다고 표시하거나 직원 경로 전체 비용이 같다고 주장하지 않는다. 새 Employee 실제 소비의 한도 의미는 위 post036 native 시험의 별도 증거다. 빌드/CPU 시간의 보편적 무비용도 주장하지 않는다.

R05 원실행 exit101 및 concurrent peak9709348 > cap9030188 경고를 유지한다. p9510014462ns의 관측10ms 초과와 original allocation-observer의 latency_gate_applicable=false도 그대로다. event-heavy 직전 대비 +12 allocation operations/+256 retained/+512 peak의 정확한 원인 귀속은 입증하지 않았다. summary 성공을 원성능 all-caps PASS나 배포 보장으로 바꾸지 않는다.

## 다음 완료 경계

현재 source와 실제 receipts로 evidence selection·predecessor impact·adaptation·coverage를 결속한 후 Root가 generated 결과를 검토한다. 새 target의 격리 정·역복원과 exact 동결을 수행하고, 해당 Goal의 계획·구현·시험 작성에 참여하지 않은 기존 감사자에게 전체 Goal 재감사를 맡긴다. target029와 failed audit/실행 원본은 변경하지 않는다. Complete/High/open0의 독립 완료 전에는 U09 완료나 후속 단계 진입으로 표시하지 않는다.
