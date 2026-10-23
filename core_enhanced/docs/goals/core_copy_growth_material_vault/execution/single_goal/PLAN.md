# U15 — GEX-10 구현 pass001

현재 GS-CGMV-001 / GOAL-CGMV-01 / PHASE-CGMV-01. HA-CGMV-001, FPF-CGMV-001, PP-CGMV-002와 P2G40/50 검증 통과를 인수한다. 사용자 2026-10-14 U15 착수 요청으로 구현·자체검증·최종target고정까지 진행하며 감사 요청 인계에서 종료한다. 중간단계 검토 opt-in 없음. 단독 수행/서브에이전트 없음, 자동 감사/U16 없음. P2G/GEX Reader Contract와 지정 상시지시서를 적용한다.

U14 independent002 Complete/High/open0와 이후 테스트 교정·G11 기본경로 정합화, 전체Core2169통과/기존ignore6을 인수한다. 기존 dirty를 보존하고 BASELINE_SOURCE_001에서 이번 전환 전 source/test/data/config/계약을 보관한다. 새형식 전환의 영향이 있어 기존 전체통과를 최종 U15 검증으로 사용하지 않는다.

21수용/10차원/5통합 조건은 상위 Goal Set과 CODE_REVIEW를 따른다. PLAN은 구현·검증계획, EXPERIMENTS는 실제명령/원시결과, EXPERIMENT_NOTES는 판단/미해결, STATE_INVENTORY는 소유/수명이다.

구현 순서: (1) authored material purpose·무기 recipe·사본투자·공통출처결제, (2) 정비 성장/환급/장착분쇄 확인과 query, (3) 원정별창고 소비·선택귀환·대여·SavePoint/Continue·버전, (4) 정상 public/host/새프로세스/IO/제작·회귀·비용·최종고정. 하나의 atomic Phase이며 부분구현을 U15 완료로 표시하지 않는다.

상세설계: material purpose는 명시 분류로 dust/part/revival catalyst를 구별한다. copy paid ledger는 재료별 bag/vault 실제 소비를 보관한다. 현행 warehouse balance/debits는 lifecycle, checkpoint의 목표 debits는 RunProgression에 둔다. restore는 현행debit를 되돌리고 목표debit를 적용해 차이만 반영한다. 별도 전체profile rollback/합집합 수량merge 없음. 모든 checked 실패는 application candidate가 폐기하며 정비 내부도 staged economy를 사용한다. 무료grant의 빈원장과 구저장투자누락은 구별하며 새 필수필드는 serde default로 옛저장을 묵시 이행하지 않는다.

공개입력은 기존 소지전용 명령을 유지하고 명시출처/장착분쇄확인/선택귀환 variant를 같은 ActionKind에 연결하는 방식을 우선한다. 기존 명령으로 창고를 자동 소비하거나 미확인 장착분쇄하지 않는다. lookup/preview/실행은 같은 비용·환급 계산 authority를 쓴다. 입력명/정확한 field는 코드소비자 점검에 따라 기술적으로 조정한다.

정상검증 준비는 CODE_REVIEW의 구체 경로를 따른다. 실제 SavePoint 생성/정비입장/완료/깊이3귀환/새후보선발 전제는 각 주요 행동 뒤 확인한다. 별도 U15 지원 테스트 pack은 공식data와 목적을 구분하고 과거동결fixture는 보존한다. authored 초기 무기수치는 D02 범위에서 소지dust2/최대5와 공격력·주문력 보정으로 검증하며 구 armor HP/Defense강화를 유지하지 않는다. 실제균형 완성을 주장하지 않는다.

검증은 집중사례→외부공개consumer/host→공유schema/loader영향 전체Core·서버관련회귀→비용으로 진행한다. 최초빌드 예상1~3분(선행U14근거), 첫진단60초/5분이상 CPU·log검토. 입력은 실행동안 고정한다. CARGO_HOME=/tmp/wp02-cargo-home, CARGO_TARGET_DIR=.cache/u12-vault-cost-target, offline locked 캐시를 재사용한다. 원시실패와 실제 선택/실행/통과/실패/skip·exit를 기록한다. 소켓 허용환경 분리와 U11 별도승인 검사는 그대로 구분한다. 0시험/빌드성공을 기능PASS로 세지 않는다.

최종 상태: 구현·자체검증21수용/10차원/5통합을 COVERAGE에 결속했다. 전체 Core 실행의 영향 교정과 재사용은 BASELINE_REUSE, 외부/host/IO·구버전·비용은 TEST_RESULTS_INDEX와 EXPERIMENTS를 따른다. disk 부족으로 이번 소유 캐시 일부를 /tmp로 옮겼고 측정 전후는 같은 dev/debug0 profile이다. source/data/config는 focused005 이후 고정했다. 최종 SOURCE 한 번과 격리 정역복원 후 Audit Target Record를 인계하고 종료한다. GEX20 감사 판정과 그에 의존하는 U16은 수행하지 않는다.
