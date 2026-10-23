# U12 독립 읽기 전용 조사 결과 — 미검토 후속 입력

작성 귀속: `/root/u11_tests_resume`, Astra high. 파일 작성/시험 실행 없음. 질문 전달 전 이미 배정되어 실행 중이던 사실 조사만 완료했다. Root는 결과를 보존했으며 답변 대기 중 후속 설계/통합/완료 판정으로 승격하지 않는다.

- PS01: `employee_psychology.rs:14` uninhabited kind; nonempty memory/recent 불가, expiry decrement만 남고 merge/recent8 mutation 없음. representation test와 정상 gameplay production을 구분해야 한다. 고정 성격은 독립적으로 실질 보존 검증 가능.
- G01: `world.rs:862` 첫 Endless 시작 gate; `world/tests/mod.rs:453` 공통 helper는 tutorial 완료를 미리 설정하므로 fresh-profile 정상 명령 시험 필요.
- 채용: queue/offer/roster ownership 및 starter grant/rollback/10로스터 기존 경계 있음. `node_flow.rs:606`, `map_content.rs:308`, `headquarters.rs:33`까지 추적한 HQ 채용 경로에 price debit 없음. 최종 정책에서 paid opportunity를 인수하면 구현/oracle 필요.
- S09: `combat.rs:1031–1130`은 배치 가능성·소지 치료·즉시 SavePoint/Gate만 살핌. 이용 가능한 채용·장비 재배치·실제 귀환의 회복 가능성까지 입증하지 않음.
- G04/G05: `encounter_compendium.rs:23` vault/loan ledger 없음, `node_flow.rs:711` Gate는 층 진행. 총2copy·copy identity·resume중복·깊이2/3·실제 귀환 정산·반입 손실/미반입 보존 oracle 필요.
- G06: 정산 후 직원 처분·다음 직원 새 상태·예산1회·첫귀환 pending의 restart/play/skip ack와 중복 방지 필요.
- PS02: `active_run/continuation.rs:758` strict version 외에 `application.rs:1688,1700` V7/V8 converter도 존재. 구형 trust 거절의 원본 bytes/state 보호와 `world/state.rs:549,759` checkpoint/battle-entry roster 개인 상태 경계 확인 필요.

이 보고는 P2G-20 formal code review, 시험 설계 완료 또는 U12 완료 판정이 아니다.
