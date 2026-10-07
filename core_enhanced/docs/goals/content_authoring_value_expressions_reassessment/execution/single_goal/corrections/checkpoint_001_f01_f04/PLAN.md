# R06 checkpoint 001 F01–F04 교정 PLAN

Current state: `교정본 재검토 준비 완료 / 재검토 요청 대기`

## Identity와 권한

- Handoff / Goal: `HA-CAVER-P2G60-001 / GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`
- Correction input: `reviews/checkpoint_001_review_001/REVIEW.md`, verdict `수정 필요`, findings F01–F04.
- Frozen parent target: `R06_STAGE_REVIEW_TARGET_001`, manifest SHA-256 `e62085bbcbdb8cfe9108f640c7d61859a0e12c29aae8171fa62d64b98474bfe1`.
- Applicable GEX: `core_enhanced/docs/goal_execution_pipeline(1).md`, revision `2026-09-13-gex-r7`; checkpoint 001 실행 때와 동일하여 새 policy 의미 변화는 없다.
- 이번 권한은 F01–F04와 같은 원인의 값 표현식 커널 경로 교정, 자체 검증, 새 교정 target 및 재검토 인계까지다. authored RON/catalog, 전투 연결, 저장/stat/weapon/public CLI, 다음 R06 기능, GEX-20/30은 범위 밖이다.

기존 checkpoint 001의 baseline/target/PLAN/evidence와 `checkpoint_001_review_001`의 report/probes/log/currentness는 수정하지 않는다. 이 디렉터리가 교정 기록과 새 evidence를 별도로 소유한다.

## 설계 판단

확정 핵심 원칙은 바꾸지 않는다. typed AST → compiled immutable plan → evaluation 구조, 단위·반올림·checked overflow, literal lowering, 정상 입력의 stable order/fingerprint 의미를 유지한다. 네 finding은 다음의 경계 교정으로 해결 가능하다.

1. F01: raw 입력과 동적 payload를 복제·정렬 전에 checked scan하고, preprocessing work 및 커널이 추가 보유하는 계획/실행 buffer를 보수적으로 admission한다. raw cardinality work와 canonical dynamic-member count를 분리한다.
2. F02: Add child 최소 node 수를 선검사하고, compiler task가 sibling 전부를 push하지 않도록 index를 가진 순차 frame으로 바꾼다. source operand order는 유지한다.
3. F03: parent/child segment가 동시에 보유하는 값 수를 포함한 plan stack requirement를 계산한다. evaluate 입구와 각 push에서 실제 meter `max_stack`을 적용하고 task/group 메모리는 retained/structural limits로 별도 보호한다.
4. F04: target-context 시점 검사를 공통 helper로 만들고 TargetEmployee resource에는 `resources`와 Snapshot/Live target capability를 모두 요구한다. source resource에는 target 조건을 붙이지 않는다.

예산 숫자를 늘리거나 기존 테스트를 약화하지 않는다. 정확한 RSS를 계약으로 삼지 않고, 커널이 새로 할당·보유하는 capacity와 동적 payload를 실제보다 작지 않게 세는 보수적 모델을 사용한다. 외부 `EvalFrame` 자체의 사전 보유 메모리는 execution retained와 구분하되, 이를 읽고 정규화하는 raw work는 반드시 부과한다.

## 구현 순서와 검증

1. parent target/current source/review evidence currentness를 확인하고 `baseline/pre_fix_001`에 actual source bytes, per-file hashes, dirty 상태와 참조 hashes를 고정한다.
2. review probe 7건과 기존 value_expr unit 11건을 수정 전 evidence로 재확인한다.
3. F04 capability 누적 검사, F02 순차 compiler frame, F03 nested stack requirement/runtime enforcement, F01 compile/evaluate admission 순으로 교정하되 공통 cost helper와 error path는 한 번만 만든다.
4. 기존 재현을 durable tests로 포함하고 각 node/stack/work/retained/raw/canonical 경계의 limit−1/limit/limit+1, duplicate/long identifier/wide+nested AST/aggregate, normal result/order/rounding/overflow/fingerprint를 검증한다. reviewer allocator probe도 직렬 재실행한다.
5. 영향 회귀와 all-target build, focused fmt, baseline lint를 분리한 Clippy를 실행한다.
6. completed/remaining/known limits, before/after actual bytes와 delta, command receipts를 새 target과 `CORRECTION_REVIEW_HANDOFF.md`에 고정한다.

## 단계검토 결과

1. **수정 전 기준 고정** — parent target manifest와 detached hash를 확인하고 source 311개가 전부 일치하는 상태를 `baseline/pre_fix_001`에 보존했다. 원 review 증거 7개도 각각 해시했다.
2. **수정 전 재현** — 기존 value_expr unit 11건과 build는 통과했고, 원 review probe는 예상대로 7건 중 5건이 실패하여 F01–F04 재현 기준을 고정했다.
3. **구현 단계검토** — F04 capability 누적, F02 sequential compiler traversal, F03 nested stack admission, F01 raw/payload work·memory admission 순서로 같은 kernel cost 경계를 함께 교정했다. 설계 핵심 원칙의 변경은 없었다.
4. **경계·회귀 검증** — value_expr unit 18건, correction allocator probe 7건, 전체 all-target test, check, focused fmt, baseline allow를 적용한 Clippy, 수정 전/후 fingerprint byte 비교를 통과했다.
5. **target 고정** — source delta를 수정 전 archive와 대조하여 네 파일만 변경된 새 correction target을 만들고 read-only verifier로 검사한다. 이 target 이후 source·record·evidence mutation을 중단한다.

## 단계검토 중단 조건

단계검토는 활성화되어 있다. 이번 요청은 F01–F04가 서로 같은 admission/cost 구조를 공유하므로 네 finding과 동일 원인 경로를 모두 교정하고 재현·경계·회귀가 통과한 한 지점을 첫 의미 있는 중단점으로 선택한다. 그 지점에서 새 correction target을 고정하고 mutation을 멈춘다. 종료 상태는 `교정본 재검토 준비 완료 / 재검토 요청 대기`이며 WP/Phase/Goal 완료나 다음 구현 승인을 뜻하지 않는다.
