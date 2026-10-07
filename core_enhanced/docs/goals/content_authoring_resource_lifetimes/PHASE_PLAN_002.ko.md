# R05 Phase 계획

2026-09-12 · current `PP-CARL-P2G40-002` · exact Freeze `FPF-CARL-P2G30-001`. 관계의 authority는 [Plan JSON](P2G_40_PHASE_PLAN_002.json)이다. [표준 생성·검증 결과](P2G_40_RESULT_001.json)와 [별도 의미 검토](P2G_40_SEMANTIC_REVIEW_001.json)가 통과했다. 구현 완료를 뜻하지 않는다.

Repair basis의 obligation/resolution/supersession은 모두 explicit empty다. Parallelization: none approved.

| 작업 | 소유 경계 | 실행 배정 |
| --- | --- | --- |
| WP01 | typed owner/store/수명/예약·원자성/실제 battle hooks | AG01 |
| WP02 | 실제 owner·Run overlay/result/entry/floor/checkpoint/AT05 | AG01 |
| WP03 | RON parser/validator/compiler/executor/identity/descriptor/production 소비자 | AG01 |
| WP04 | durable focused·외부 RON 인수·실패/예산/결정론/회귀와 감사 입력 근거 | standalone |

WP01–03은 store·실제 Run authority·작성 정의/실행/저장 계약의 상호 의존으로 한 SCC다. AG01로 응축하면 AG01 → WP04의 비순환 graph다. 새 store만 공개하거나 저장 없는 Run 지원을 완료하는 중간 상태는 허용하지 않는다. WP04의 시험 코드는 구현 단계에서 만들며 read-only 감사자에게 생성을 미루지 않는다.

유일한 `PHASE-CARL-01`은 AG01과 WP04, 모든9요소의 primary ownership을 가진다. 종료 시 여섯 수명/실제 사본/원자적 비용/Run 저장·결과·rollback/외부 RON과 public 포트/유한 비용·결정론이 모두 유효해야 한다. 이후 R 없이 현재 기능을 사용할 수 있어야 한다. 최종 구조 사실은 `single_phase`이며 실제 Single Goal 작성은 다음 P2G-50의 책임이다.

IVA01–12가33 acceptance 전체를 연결한다. source/copy, Run/Attempt/Deployment, Activation/Delivery, Chain, 수치, 원자성, owner 생성/삭제, 결과·후퇴·층 이동, 저장, public 작성, 비용·결정론, 전체 회귀·fresh audit의 실제 검증을 WP04에 배정했다. source hash/명령/입력/실행 건수/expected-actual/raw output을 남겨야 하며0 tests·skip은 통과가 아니다.

SC01–03은 [코드 검토 C01–09](CODE_REVIEW_001.ko.md)의 의미와 생산자·소비자·구현 node·검증 node를 고정한다. 내부 자료형·문법·version·engineering limit은 GEX 상세 설계 재량이지만 owner/수명/원자성/AT05를 바꿀 수 없다. 아직 구현된 새 API는 없다. 별도 fresh 구현 비참여 reviewer의 감사는 실제 구현·시험 안정화 후 수행한다.

초기 candidate001은 검증기 method identifier 표현 오류로 receipt가 생성되지 않았다. [수정 기록](P2G_40_CANDIDATE_CORRECTION_001.json)에 실패를 보존하고 동일 의미의 successor002를 검증했다. Freeze·정책·검증기는 수정하지 않았다. Goal/Handoff/GEX는 아직 없고 R05 전체는 미완료다.
