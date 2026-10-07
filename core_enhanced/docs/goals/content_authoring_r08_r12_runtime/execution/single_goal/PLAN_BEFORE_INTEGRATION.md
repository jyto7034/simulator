# R08~R12 실행 — 현재 위치

2026-09-30. GS-CARX-001 / GOAL-CARX-01 / HA-CARX-001. GEX-00 인수 완료, GEX-10 WP11 상세 설계·구현 착수. 구현·검증 완료 아님.

현재 역할 원본은 이 디렉터리 PLAN.md, EXPERIMENTS.md, EXPERIMENT_NOTES.md다. 이전 preparation 기록은 역사적 기준선이며 현행 상태가 아니다. 선행 reviews/r06_r07_20260930_001의 감사 pass는 종료했고 이후 R08~R12 구현자 역할이다.

사용자 “감사 수행하고 r08~r12 구현 수행해”가 감사와 연속 구현을 승인했다. 정식 계획은 ../../P2G_30_FREEZE.json / P2G_40_PHASE_PLAN.json / P2G_50_GOAL_SET.ko.md. P2G40 receipt 실제 생성·재검증과 P2G50 quality2는 passed. 형식 오류 두 번과 교정은 ../../P2G_40_REJECTED_001.json 및 GENERATION_002/003.json 보존.

## 기준선·범위

baseline/SOURCE_BEFORE.tar.gz와 MANIFEST.json은 실제 dirty/untracked source367+roadmap 원본이다. git worktree/index patch와 untracked 상태 별도 보존. 기존 R06/R07 exact source 감사 Complete/High, currentness367/367. fresh focused56개와 CLI6회, 이전 exact source broad default/feature1581씩 인수. 기존 recovery preview 배열 순서 OBS001은 게임 값과 무관하며 원자료 보존.

WP08→WP09→WP10→WP11→WP12→INT 순차. 각 WP 내부 AT는 raw/compile/runtime/consumer 동반 수정. 중간 감사 요청 없이 계속한다. 차기 원정/GUI/R13/stress/metrics/20분 제외. peak9030188 단독 초과 경고, 나머지 typed fault/rollback/hard guard 보존.

## WP08 상세 경계

boss_runtime_data의 StartAction raw/compiled/result/capability와 cross-database skill reference를 함께 변경한다. ActiveBossTask가 action handle을 보관한다. 기존 run_boss_runtime_boundary의 preflight clone→staged plan/effects→commit 루프가 action admission 실패까지 감싸게 한다. plan.commit을 fallible로 전환하고 후보 staged에서만 skill을 시작한다. player 권한은 유지하고 boss 권한을 별도 검사한다. Released/EffectsSettled waiter는 활성 task만 query, 전체 action store scan 없음. state 이탈/timeout은 명시 policy, 미방출 취소/방출분 보존. deadline과 actual action milestone에서 boundary wake. retention은 task 종료에 해제한다.

cross-database compile caller, action interruption/fact-retention APIs, clone/retained owner 목록은 제품 변경 직전 추가 조사한다. 공개 RON stationary→실제 피해→result graph 시험부터 연결한 후 실패/취소/rollback을 확장한다.

## 검증과 종료

COVERAGE.json의55조건 전부 미실행. 각 변경은 focused nonzero tests와 receipt/source archive를 run.py로 보존. 최종 default/feature all-target tests, check/event_log_checks/release bins/clippy/fmt, CLI/PC36/R05, fresh-process save, currentness와 isolated forward/reverse restoration 필수. 실패/교정/재실행을 EXPERIMENTS에 기록한다.

종료점은 구현·자체 검증 완료 / 통합 target 동결 / 독립 감사 요청 대기. 필수 기능이 남으면 완료 아님. 구현자가 R08~R12 독립 PASS를 발행하지 않는다.

## 현재 검증 위치

R08 focused8개 중간 통과. R09 focused13개 중간 통과(r09_tests_010), 공개 RON/normal World명령/결과/저장/fresh-process 및 stage별 실패 복원을 포함한다. R09 lib regression002는1544통과/1환경 EPERM 명시 제외. WP10 상세 설계는 WP10_DETAIL.ko.md; WP10~12/최종55조건/동결은 남아 있다.

## WP09 실제 상세 설계

R08 중간 source8개 focused 통과와 typed late admission rollback을 보존했다. WP09 공통 start gate는 새 event로 manual 입력 접수 뒤 실제 execution에 진입하며 auto도 같은 침식 함수로 수렴한다. pending start_seq가 stale/중복 입력을 막는다. 강제 정지는 BattleCore의 host-elapsed 잔여 상태가 소유하고 직접 step/apply command와 World application route를 함께 막는다. server tick만 raw elapsed를 소비하며 초과분은 버린다. 현재 timestamp의 미처리 events는 identity를 바꾸지 않고 requeue한다.

기존 draft.employee_start_condition.trauma를 runtime에 전달하고 exact fragment source에 결속한다. 침식 대가는 runtime corroded marker→attempt roster trauma100→terminal result 순서로 먼저 반영한다. ordinary death 경로와 별개 dispatch policy를 사용해 mandatory bookkeeping을 보존한다. Breakdown durable counter와 node recovery/asset return/새 schema 명시 변환은 같은 WP에서 완료해야 한다.

WP10 focused11개 및 lib1552개(환경1개 제외), release CLI4회 중간 통과. WP11_DETAIL.ko.md에 따라 source-owned ledger부터 구현한다. WP11/WP12/최종통합은 미완료.
