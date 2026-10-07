# R06 WP02 F01 첫 교정 경계 기술 메모

## 회계와 소유권

- `CompiledAmountPlan`과 `EvalCapture`는 `Arc`로 공유한다. capture의 Snapshot frame은 capture extent에서 선청구한다. Live frame은 매 consumption 때 별도 stack-local map을 만들므로 각 참조 조회/할당을 선청구한다.
- evaluator의 기존 `ExprMeter`는 계산 의미와 per-expression 1 MiB retained 한도의 owner다. host callback은 동일 작업 직전 현재 native/authoring scope의 work 및 누적 새 allocation admission에 연결한다. 임시 task/stack이 해제돼도 현재 operation의 누적 allowance를 환급하지 않는다.
- evaluator가 반환한 `EvaluatedAmount.trace`는 scratch 해제로 가정하지 않고 평가 중 local/host retained admission에 포함한다. 이후 production consumer가 이를 보존하는 시점의 live owner extent는 그 consumer 책임으로 별도 검토한다.
- frame 구성 작업과 evaluator의 frame admission scan은 별도의 실제 작업이다. plan의 정적 최대 work를 한 번에 소모하지 않는다. 같은 Snapshot frame allocation을 capture와 evaluator 양쪽에서 다시 청구하지 않는다.
- 추가된 Live frame allocation의 보수적 node allowance는 실제 reference 수에 따른다. `reference_node`의 정상 경로 plan 순회는 방문 operation마다 선청구한다. fault 진단용 scan은 유한 plan 한도 내에서만 실행되며 감사에서 bounded diagnostic allocation 확인이 필요하다.

## 검토자가 볼 지점

원본 감사 F01 두 재현은 동일 probe source의 임시 사본에서 통과했다. work=1의 fault는 `value_expr_frame_plan_lookup`의 요청 2에서 발생했고 요청 allocation은 342 bytes였다. retained 잔여 1 byte fault는 `value_expr_live_frame`에서 발생했다. 독립 kernel의 local meter 결과와 expression 값은 ValueExpr 36건 회귀 및 PC fixture의 실제 결과로 확인했다.

새 F01 host 연결은 현재 private adapter 사용 경계에만 있다. 공개 publication은 여전히 차단돼 있으며, 정상 production consumer의 scope 소유권과 반환 trace의 장기 보존은 후속 연결 시 별도 검토가 필요하다. F02/F03/CAP 문제는 현재 구현 범위 밖으로 남겨두었다. 예산 상향이나 historical manifest 수정은 하지 않았다.
