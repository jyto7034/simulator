# U08 역할·검증 계약 기록 019

## 2026-10-06/019 — G3 실행 및 최종 증거 도구의 분리 작성

G3의 원형은 기존 AuthoredSkillEffect 항목을 뜻한다. public018의 “원형·시제품” 중 시제품은 original/prototype 번역 중복 오기이며 별도 item tier, OnBattleStart 또는 U09 generated effect가 아니다. 원018은 보존하고 docs_checkpoint021/G3_TERMINOLOGY.ko.md의 같은 정정을 인수한다.

Astra/High u08_test_author가 기존 test-only world/tests/u08_lifecycle.rs와 u08_carriers.rs를 단독 소유하고 집중 실행한다. 먼저 동일 세 사본의6슬롯순열에서 실제 피해·-15/-10 패널티·base/certified 불변을 비교한다. 이후 원형 효과 add/remove 및 EquipItem, Maintenance의 AwakenSkillFragment/SelectSkillFragmentForm 이후 실제 fresh battle을 사용한다. 무기교체 자체가 합법적으로 바꾸는 base까지 불변을 요구하지 않는다. 기존 시전·배치·scheduler를 사용하며 runtime actor나 유물 owner를 주입하지 않는다. 물리/마법은 기존 authored ModifyDamage와 저항 후 정수 절삭을 독립 손계산하고 실제 HP차이·incoming penalty를 대조한다. source438/c5283500d3884794fe82b89a830312f9ff1350fabb072cb2a17ecf93a51d8414의 permutation001은 이 기록 시 실행 중이며 아직 PASS가 아니다.

기존 사용자 역할 지시에 따라 보조 테스트/판정도구 작성자 u08_finalizer_author를 gpt-6-astra/high로 새 배정했다. 실제 배정 context의 model과 effort를 확인했다. 이 담당자는 U08 계획·도구 작성 참여자이며 최종 독립 감사자가 될 수 없다. 새 H/tools/{evidence_common,select_evidence,build_predecessor_impact,finalize_adaptation,coverage,coverage_contracts,finalize_gex,finalization_paths,freeze_current}.py, finalization_contracts.json과 새 evidence/finalizer_prep001만 편집한다. 기존45검증도구·public_relics·제품·시험·fixture·완료산출물은 편집하지 않는다. in-memory Python syntax/준비 자체검사는 가능하나 제품/통합/성능/실제 finalization 실행은 하지 않는다. run.identity는 이 신규 도구 영역을 포함하지 않으므로 현재 G3 실행 입력과 분리되며, 도구 자체는 별도 manifest로 결속한 뒤 인수한다.

모든 finalizer는 explicit immutable config를 받고 현재 source·receipt/raw·실제 nonzero count와 historical focused의 구체적 delta/비영향 근거를 검증해야 한다. latest 추측·준비완료를 기능PASS로 변환·U07 과거 label/README/server main 예외 상속을 금지한다. 즉시전임 U07M181+10=191, U08 16군 각각2수용문장,10차원/3node/16seam 및 별도31R06을 연결한다. 16.2의 정역복원·독립감사는 GEX10에서 pending이고 후속 closure가 완료를 결정한다. 현재GEX10 자기검증과GoalComplete를 구분한다. 11.2의 현typed침식자산/no-substitute 증거와 U17 실제부활촉매 교차시험 미실행 한계도 별도 표시한다.

Sol/xHigh는 제품·통합·성능 runner0/HOLD이며 docs_checkpoint021 완료 원본을 보존한다. root는 계획·문서·읽기 조정만 담당한다. G3→G5/G6→G7 이후 현재 기능통합, 모든 functional writer/runner0 이후 PC/R05, evidence→impact→adaptation→coverage→자체검증→PLAN/NOTES→exact freeze/정역복원→비참여 Astra 전체감사 순서를 유지한다.
