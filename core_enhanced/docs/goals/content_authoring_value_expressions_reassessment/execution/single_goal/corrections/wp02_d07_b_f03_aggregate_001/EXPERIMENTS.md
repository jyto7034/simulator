# C06 시도 이력

## 시작

C03 사본 PASS 및 RON MIN probe 후속 의무를 확인했다. 직접 선행 currentness/격리 복원 통과 후 before343개를 보존했다.
기존 owner_fixture는 직원A(공격3, 무기X/방어구Y), 직원B(공격5, 무기Z), 소환물11, 적7을 제공한다. 기존 관계 시험은 frame 단계 중심이며 employee_filter_changes_only_aggregate_input은 실제 소비지만 관측 record/query 증거가 없다.
새 시험은 네 aggregate의 독립 결과18+8+8+3=37을 actual 소비로 대조한다. release 후 공격4/9/13/6, B의 첫 슬롯 비움이면 Live19+13+4+4=40이다. Snapshot은37을 유지한다. 실제 command tap과 frame view/owner/정렬/trace를 독립 기대값으로 검사할 계획이다.

## 최초 실행과 trace 기대값 교정

c06_aggregate_001: 2 pass/1 fail. 빈 집계와 부적격 관계 거부/기존 record 보존은 통과했다. 복수 view의 actual command/HP는 기대값과 일치한 뒤 frame view 조회에서 None으로 실패했다.
compiler::normalize_source는 canonical predicate마다 node ID를 소비한다. 최초 시험의 aggregate 1/3/5/7 가정이 틀렸고 실제 식 구조의 aggregate/read 쌍은 (1,2)/(3,5)/(6,9)/(10,13)이다. 독립 수치·owner·member·trace 순서 기대는 유지하고 번호만 실제 계약에 맞춘다. 실패 source와 로그를 보존했다.
사용자 중단 후 재개 시 Cargo/rustc/시험 프로세스가 없음을 확인했다. 최초 focused 실행은 중단 전에 exit101로 완료되어 영수증이 존재하며 중단을 성공으로 처리하지 않는다.

## 집중 재검증

c06_aggregate_002: 3 pass/0 fail. ByType/Any3 × Snapshot/Live의 네 복수 view 시나리오, 두 빈 집계, 네 relation typed refusal을 통과했다. 독립 수치/owner/member/trace 기대, actual command 및 HP, 이후 상태 변화 후 두 query 동일성을 확인했다. 제품 변경은 없고 cfg(test) 하위 module만 추가했다. 최종 source 전체 검증을 진행한다.

## 원자료 대조

집중 로그의18쌍 preview/explain과4개 typed refusal을 파싱해 노드별 독립 기대값 및 변경/거부 전후 문서 동일성을 확인했다. 첫 요약 스크립트가 empty_slot(거부)도 empty(정상0)로 분류해 assertion으로 중단했다. 정확한 두 empty 시나리오 이름으로 분류를 고쳐 AGGREGATE_MATRIX.json을 생성했다. 제품/시험 source와 실행 결과를 수정하지 않았으며 재실행을 성공 근거로 바꾸지 않았다.

## 실행 중 workspace 입력 추가 발견

첫 all_targets_final_001은1556개가 통과했다. 기존1552+신규3=1555와 차이를 조사해 작업 시작 뒤 core_enhanced/tests/battle_showcase.rs가 추가되어 별도 통합시험1개가 실행된 것을 확인했다. 시작 currentness의 완전 membership343과 현재 추가 경로를 대조했다. 이 작업이 생성하거나 수정한 파일이 아니므로 내용을 변경하지 않는다.
최초 실행 영수증은 계획된344개만 hash에 포함했으므로 전체 입력 결속의 최종 근거로 사용하지 않는다. WORKSPACE_ADDITION.json과 원본 archive에 출처/발견 시점/byte를 기록하고 after 입력에 추가해345개 source를 결속한다. 최초 before343개 기록은 수정하지 않고 신규 부재는 시작 membership 검증으로 입증한다. 이 파일은 C06 구현 범위 밖의 동시 입력으로 명시하며 source 전후 patch에 보존만 한다.
run/freeze의 source set에 이 선언된 추가 경로를 반영하고 focused/check/authoring/value_expr/전체/Clippy/format을 다시 실행한다. 실패·기존 실행 원자료는 보존한다.

## 최종345개 입력 검증·인계

c06_aggregate_final_001 신규3개, check_final_002, authoring112, value_expr104, all_targets_final_002 전체1556(기본 병렬 약107초), Clippy/변경 format을 모두 통과했다. 전후 source345개 digest와 로그 hash, 실제 전체 src/tests membership을 대조했다. 전체 건수는 선행1552+C06신규3+별도통합시험1이다.
AGGREGATE_MATRIX의18쌍/4거부도 최종 집중 로그로 다시 대조했고 최초 요약은 INITIAL 파일에 보존했다. PC36/R05는 시험 전용 추가와 외부 integration test 입력이 product library에 링크되지 않음을 확인해 재사용했다.
최종 지시서/검토/실행/실패/외부입력 원자료와 before343→after345 patch/archive를 동결하고 격리 양방향 byte 복원을 수행한다. 실제 결과는 target/RESTORATION_VERIFICATION.json 및 루트 freeze/verify 영수증이 소유한다. 후속 C07–C15는 시작하지 않는다.
