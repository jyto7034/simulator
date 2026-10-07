# R06 WP02 F01–F03 / CAP-01–02 교정 실행 계획

상태: 구현 진입. 단계검토 옵션 유지. WP02 감사 `CORRECTION_REQUIRED`가 선행한다.

## 진입 근거

- 기준: `handoffs/WP02_CORRECTION_PLAN_20260926.ko.md`, `reviews/wp02_runtime_frame_capture_001_review_001/REVIEW.md`, 원본 PC 부하 측정.
- 동결 WP02 target manifest SHA-256: `d3dbf3ba7c3c3be5a584db3092e7059c116d92b1ae812740f1fb68ccb05c2fc3`.
- 현재 HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`, branch `cleanup/simulator-dirty-worktree-20260803`. 작업 트리는 범위 밖 변경을 다수 포함한다. 이를 초기화하지 않는다.
- mutation 직전 actual-byte baseline: `baseline/entry_001/MANIFEST.json`, SHA-256 `d0c56f7c2e453db69dc986d31b0fe83389b261cc6a41d6070245c13445994f94`. 322개 현재 파일과 교정 입력의 bytes, archive, global/scoped status를 보존했다. 이전 target과 이 baseline은 별도 identity다.

## 범위와 순서

F01의 기존 evaluator 비용 경계에 호스트의 현재 operation을 연결한다. 독립 kernel meter는 유지하며 native/authoring의 work·retained·취소와 frame 준비를 먼저 청구한다. 이어 F02 action/actor/copy 결속, F03 Live unit 가용성, CAP-01 ActorSummaries 및 CAP-02 EventQueue의 bounded 성장과 owner extent를 각각 독립 교정한다. 실제 빌드·관련 회귀가 통과하고 책임 하나가 닫힌 첫 지점에서 before/after를 동결해 중단한다. 남은 항목은 다음 교정 구간 후보로 명시한다.

예상 변경은 `game/value_expr/{mod,eval}.rs`, `battle/core/{value_expr_runtime,runtime_facts,queue}.rs`, `battle/runtime_control.rs` 및 관련 테스트다. 필요할 때만 `authoring/{control,clone_shape}.rs`와 resource read 경계를 수정한다. 예산 수치·계산 값·fingerprint·R04/R05의 identity/lifetime·누적 admission 비환급 계약은 유지한다. 순차 effects/prefix, WP03, publication 해제, GEX-20/30은 범위 밖이다.

## 비용·소유권 회계

| 비용 | 생성 및 소유 | admission 방식 |
| --- | --- | --- |
| frame 조회·참조 탐색 | capture 또는 Live consumption stack | 조회 전 work, Live map node·key clone 전 cumulative allocation. Snapshot frame 저장은 capture extent에서 한 번만 계산 |
| evaluator task/stack/member/sort | 해당 평가의 local meter | 기존 per-expression 1 MiB와 각 동적 작업 직전 host work·allocation admission. 반환 후 해제돼도 operation allowance는 환급하지 않음 |
| evaluator trace/result | 반환된 `EvaluatedAmount` | local retained와 host allocation을 평가 전 청구. 반환 후 수명은 호출자가 소유하며 폐기를 조기 가정하지 않음 |
| capture source/plan/frame | immutable `EvalCapture` | 생성 전 기존 capture extent 및 current owner 기준 admission. plan `Arc`는 재복제하지 않음 |
| ActorSummaries·queue spare slots | BattleCore candidate | 새 buffer 전체 요청을 재할당 전 청구. live owner extent는 실제 capacity, clone은 실제 복제량을 별도 기록 |

정적 plan 상한을 매 평가 선소모하지 않는다. 한도 초과 실패 후 consumed work/allocation은 같은 operation 안에서 환급하지 않는다. 원본 R05 cap `9,030,188` 및 PC 부하 fixture를 바꾸지 않는다.

## 검증·동결

수정 전 실제 selector를 실행하고 결과를 보존한다. 변경 뒤 F01의 잔여 work/retained, 취소, Snapshot/Live, native/authoring과 실패 후 정상 retry를 우선 검증한다. CAP 변경 시 원본 300명 피해·500명 생성 fixture와 actor/queue limit·COW/rollback/order를 확인한다. focused rustfmt, all-target check/test 및 기존 Clippy allow 조건을 기록한다. 원본 부하 결과와 새 측정을 별도 series로 비교한다.

첫 의미 있는 경계에서 새 correction target에 source와 입력·출력·실패 기록·before/after archive·복원 delta·manifest와 detached hash를 고정한다. 재검토 PASS나 다음 WP 진입을 자체 선언하지 않는다.
