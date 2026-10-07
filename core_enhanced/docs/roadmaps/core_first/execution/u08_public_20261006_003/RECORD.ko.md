# U08 공개13 회수와 query/Event owner 검사 — 2026-10-06/003

root는 계획/설계/조정만 수행했다. Sol 제품/포맷/실행, Astra 시험/fixture/oracle의 분리를 유지했다. mixed cfg(test) 3파일은 Sol source-ready/HOLD→Astra wholefile 독점→root actual preimage/diff/제품 prefix 확인→전체 HOLD→Sol syntax format 순서로 처리했다. 실제 시험 작성 증거는 Goal evidence/test_bc_cfg_checkpoint001에 있다.

공개 `relic_public_bc001` 13/13 PASS, source6005757c…에서 불변이다. query_owner001은 libtest E0308/test0 실패이다. 두 담당자의 별도 파일 1행 교정과 root actual diff/source delta 확인을 거쳐 현재31b4b015…에서 query_owner002를 실행 중이다. 성공 시 event_owner001 4개를 실행하도록 승인했다. EVIDENCE.json은 종료된 receipt만 포함하며 실행 중 결과를 PASS로 주장하지 않는다.

공개13 재사용의 비영향 근거는 두 exact delta: selection.rs의 자기 CloneShape impl 안 redundant trait import 제거, u08_event_owner.rs의 시험용 ApplicationCheckpoint 입력 Arc.clone→to_vec이다. 전자는 visitor/계량/직렬화 코드 불변, 후자는 integration 공개13에 컴파일되지 않는 libtest 입력 소유형 교정이다. product_bc_cleanup001 및 test_event_owner_compile_fix001 원본/after/diff를 보존한다.

본 publication의 DOCUMENTS_BEFORE는 쓰기 전 PLAN/NOTES 실제 bytes이다. 이전 문서 상단과 끝의 오래된 진행 문구를 최신 국소 증거·미완료 경계로 바꾸고 NOTES에 기록했다. P2G frozen 원본·과거 실행/감사/target·HANDOFF 원문은 변경하지 않았다. source는 모든 실행 동안 고정하고 기능과 성능을 겹치지 않는다. 프로세스 조회는 sandbox namespace에 한정된다.

전체 U08 완료, 최종 regression/PC/R05, current evidence/impact/adaptation/coverage, 동결/복원검증 또는 독립 감사 완료를 주장하지 않는다. 정상 lifecycle/trial/CLI/actual carriers는 집중19 회수 뒤 작은 단위로 계속한다.
