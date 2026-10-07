# U04 단일 전환 단계

Bound Freeze: `FPF-CRRP-001`. Exact selectors/basis는 [Phase Plan](P2G_40_PHASE_PLAN_001.json)이 소유한다.
Repair Basis: obligations/resolutions/supersessions explicit empty. 동결 전 identifier 교정은 완료됐다.
Parallelization: none approved. 구현은 주 작업자가 순차 수행하고 독립 감사만 비참여 Astra High에 맡긴다.

| Work Package | 구현 소유 | 완료 경계 |
| --- | --- | --- |
| RESULT | 공통 즉시 정산·receipt·선택·ack | 전체 자산/직원/노드와 확정 권리가 같은 결과 |
| RESTORE | pre-entry/전투전/현재/SavePoint와 codec | 현재 모든 노드·복원 경계와 새 reader/writer |
| DURABLE | prepared publication·file journal/revision | 접수 전 rollback, 접수 후 같은 결과 복구, stale write 보호 |
| HOST | 실제 명령/tick/관리/재접속·DTO | Core 결과와 영속 상태를 실제 socket 소비 |
| VERIFY | V01–15·선행 영향·실제 crash와 감사 target | exact 자체 검증/복원과 독립 감사 |

앞 네 package는 `AG-CRRP-RESULT-PERSISTENCE` 하나의 최소 전환 단위다. 구현 내부 순서는
RESULT→RESTORE→DURABLE→HOST이고 새 public DTO/reader/writer를 같은 current target에 연결한다.
Core 정산만 완료하면 프로세스 종료에 결과가 사라지고 저장만 바꾸면 기존 ack 정산/노드 복구와 불일치한다.
부분 구현은 중간 작업 상태이며 독립 완료·release라고 게시하지 않는다. 공유 계약은 CODE_REVIEW TD01–08이다.

`PHASE-CRRP-01`이15요소의 유일 primary owner이며 atomic group과 VERIFY 두 final node를 포함한다.
단계 직후에는 즉시 결과·고정 선택·후퇴2·미확정 노드 복원·stable Continue·load3·실제 파일 원자 복구가
함께 유효해야 한다. 선행 U03 fault/관측·native 한도와 원본 보호도 유지한다. 새 schema는 현재 상태에 한정한다.
15개 integration acceptance의 필요한 durable test/probe는 VERIFY가 구현하며 GEX-30이나 감사자에게 미루지 않는다.
단일 단계 구조이므로 P2G-50은 Single Goal을 작성한다. 표준 receipt는 구조만 검증하며 의미적 중간 상태 감사는
이 문서·Phase Plan audits와 코드 검토가 소유한다. 독립 감사 통과 전 U05 종속 구현을 시작하지 않는다.
