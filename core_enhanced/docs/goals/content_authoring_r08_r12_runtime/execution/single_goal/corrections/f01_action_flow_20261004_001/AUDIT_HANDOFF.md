# U00 인수 확인 / U01 F01 fresh GEX-20 인계

2026-10-04 · 구현자 자체 검증 완료, 독립 verdict 미발행.

## 정확 대상과 권한

- Goal `GS-CARX-001 / GOAL-CARX-01`, Handoff `HA-CARX-001`.
- 원 Goal: `../../../../P2G_50_GOAL_SET.ko.md`; 원 계획 인계 `../../../../PLAN_HANDOFF.ko.md`.
- 이전 Not Complete/F01: `../../../../post_audit/20261003_001/COMPLETION_AUDIT.md`.
- 새 target: `target/f01_action_flow_001/MANIFEST.json`. 전후 archive, scoped FORWARD.patch,
  fuzz0 정·역 복원, membership/bytes 검증, records/references 원자료를 포함한다.
- GEX-10 결정과 실험: PLAN.md / EXPERIMENTS.md / EXPERIMENT_NOTES.md / COVERAGE_CURRENT.json.
- 현재 source digest: 5b05f2454b83b9b1dc4a4db50b7d4fc066efeca84d319c6095212c5c505ac876 (388개).
- 사용자 연속 실행 위임: `../../../../../../roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md`.
  최신 요청은 독립 감사·실패 교정·재감사·통과 후 다음 작업을 포함한다. 옛 인계의 감사 제외 문구는
  당시 요청 범위이며 이번 명시적 요청을 취소하지 않는다.
- U00: `../../../../../../roadmaps/core_first/execution/u00_20261004_001/INTAKE.ko.md` 및
  `../DECISIONS_20261004.ko.md`(U00 디렉터리 기준). 현재성, 정책 권한, 범위 연결만이며 제품 완료가 아니다.

## 변경과 검토할 경계

기존 387개 중 제품 2파일 변경(boss_runtime.rs, queue.rs), 회귀 파일 1개 추가다.
정확 actor+opaque ActionHandle의 pending/active cast와 미실행 SkillStep/FlowWake만 회수한다.
준비 중 취소의 소유 lock만 풀고 이동/basic/다른 action을 actor-wide로 취소하지 않는다.
이미 방출된 projectile/area의 source/resource lease/완료 event는 보존한다. terminal과 queue 변경의
오류는 Result/poll로 전달하며 후보 transaction이 복원한다. Detach/정상 Released는 취소 경로에 들어가지 않는다.

필수 직접 영향은 A08-03/A08-04/A10-04/A10-06 및 X10이다. 나머지 55조건과 R06/R07 current 영향은
기존 condition reasoning과 새 양쪽 suite/공개 CLI/PC36/R05 증거를 연결해 판정한다.
기존 snapshot/live/source-copy/resource provenance와 이후 state marker를 포함한다.
추가 7개 시험에는 legacy delayed step, stale wake/추출 step, same-timestamp, exact capability 격리,
released persistent area, 취소 중 fault 6지점과 retry가 있다. 격리 시험의 synthetic cast는 내부 경계용이며
공개 제작 지원 증거로 대신하지 않는다. 5개 공개 팩은 기존 감사 입력 bytes 그대로 실행했다.

## 독립 감사 요청

구현 비참여 GPT-6 Astra / High reviewer가 `goal_execution_pipeline(1).md`의 GEX-20을 수행한다.
현재 target과 실제 경로를 읽고 모든 조건의 적용성·증거·currentness를 확인한다. source-level 의미 검토를
테스트 수로 대체하지 않는다. 유효한 명령 증거는 검증 후 재사용하고 부족한 안전 probe만 새 출력 경로에서 수행한다.
U00 인수 문서도 최신 범위/정책 위임/과거 증거 적용성을 점검한다.

출력은 새 `../../../../post_audit/20261004_f01_001/` 아래에 작성한다. target·구현·과거 감사는 수정하지 않는다.
COMPLETION_AUDIT.md에 정확 target, condition matrix/currentness, Complete / High 또는 Not Complete / Blocked,
finding의 교정 owner와 영향 범위를 명시한다. 제품 수정 필요시 finding을 반환하고 이번 read-only 감사를 닫는다.
후속 U02 구현은 해당 fresh 감사 통과 뒤 시작한다.
