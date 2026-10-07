# R07 사본별 파편 — P2G-00 인수 및 작업 정책

2026-09-29. 신규 Topic; 기존 정식 R07 Goal/Handoff는 경로 검색에서 발견되지 않았다. R07 카드는 proposed_not_implemented 조사 근거다. R06 경계 검증 중 정책/코드 조사만 수행했으며 R07 제품 변경 전에는 경계 manifest를 인수한다.

사용자가 R06→R07 연속 구현과 미정 정책 결정을 명시적으로 위임했다. P2G는 계획 산출물만 소유하며 구현 권한은 이 사용자 요청에서 별도로 받는다. 단일 writer, 기존 dirty 작업 보존. R06 선행 독립 감사는 착수 blocker가 아니다.

입력: 연속 구현 지시서 §§2–6, R07 카드, AT07, PG05/06, game_rules_next/equipment-and-fragments.md, game_rulebook.md. 최신 active 슬롯 하나/일반 강화 없음이 과거 passive·공통 progress보다 우선한다. 직원 유물 3슬롯을 파편 슬롯으로 전용하지 않는다. intrinsic baseline modifier는 별도 직원 입력으로 보존한다.

P2G_POLICY_AND_REVIEW.json은 각 요소의 open→decided→code_reviewed 불변 revision, 위임 결정·대안·영향·되돌림, 실제 검토 source hash, binding와 review batch를 소유한다. P2G_30_FREEZE.json은 6개 reviewed 요소만 선택한다. P2G_40_PHASE_PLAN.json은 AT07 원자적 cutover와 최종 검증을 하나의 Phase에 배정한다. 구조 검사 통과를 구현 또는 독립 감사 PASS로 해석하지 않는다.

남은 구현 준비: R06 boundary checksum 인수, 명령 wire 변경과 legacy converter 인터페이스 상세화, 새 owner가 필요한 실제 resource binding 대조. 결정 미정은 없으며 구현 세부가 계약과 충돌하면 후속 revision으로 보존한다.
