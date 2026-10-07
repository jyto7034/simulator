# R06 WP02 D04 실행 PLAN — 실제 resolved target 집계

- Goal `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`, WP02 부분 구현. 선행 action 결과·release Snapshot target SHA-256 `b16697bfe8c518944f0bddcf800b0f1e005ed639edd6fe915e4d96c726a6e1f4`, 최신 독립 검토 부분 PASS. `freeze.py verify` currentness와 양방향 복원 PASS.
- baseline: `baseline/entry_001/MANIFEST.json`의 실제 current 333개 bytes, branch/HEAD와 전체 dirty/untracked 상태. 기존 변경은 제거하지 않는다.

## owner, 호출, frame, 오류

1. `execute_ordered_skill_effects`가 `targets`를 소유한다. 이 배열은 effect별 command 전달 순서로 보존한다. 별도의 canonical resolved-member payload는 battle adapter가 입력 identity를 복사·정렬·dedup하여 만든다. evaluator의 `members`는 기존 정렬·group fold를 그대로 수행한다.
2. Instant는 `execute_ordered_skill_effects`의 실제 resolved set이 capture와 consumption에 도달한다. target-locked projectile은 release에서 고정된 한 대상만 Snapshot 가능하며 directional projectile과 area는 release에서 실제 set이 미확정이다. impact/tick의 `execute_ordered_skill_effects`는 그 occurrence의 확정 set을 Live frame에 넘긴다. source/Target scalar와 AggregateMember payload는 서로 다른 frame context이다.
3. `build_value_expr_delivery_capture`는 release 시점에 확정된 set을 Snapshot frame에만 고정한다. `evaluate_value_expr_capture_with_prefix`의 Live frame은 effect 시작마다 실제 consumption set으로 다시 만든다. 같은 effect의 대상별 평가 완료 전 command mutation은 시작하지 않는다. 공유 member payload는 Arc로 소유하고 target별 deep clone을 피한다.
4. adapter는 raw cardinality·identity 복사/정렬·member 값·tile key/Vec/Arc 구성 전에 native retained 및 authoring work를 승인받는다. evaluator의 node/member/group/nested 비용 검사는 별도로 유지한다. candidate 오류는 기존 `execute_ordered_skill_effects` → immediate/projectile/area caller → runtime transaction으로 전파되어 HP, fact, witness, queue, RNG, UUID, log, time이 복원된다.

## 이번 중단 경계

WholeSet·ByProjectedTile의 실제 battle 연결과 직접 의존한 capture/consume, 비용, rollback을 구현·검증한 뒤 중단한다. `Grouping::ByOwner`에는 resolved unit의 선언된 owner domain 인자가 없으므로 임의 domain 추론을 하지 않고 잔여 D04로 기록한다. 실제 unit의 `AggregateMember { unit }` scalar producer도 없으므로 typed refusal을 유지한다. D07 preview/explain, WP03/04, server/Unity, publication은 다음 구간이다.
