# R06 WP02 F01-R01 교정 기록

상태: 교정·자체 검증 완료 / 별도 단계검토 타깃 동결 / 재검토 요청 대기. 이전 `f01_admission_001` 타깃의 bytes와 archive는 수정하지 않았다. 구현 전 실제 328개 입력 파일은 `baseline/entry_001`에 보존했다.

## 저장 구조와 비용 소유권

`ContextRequirements.references`는 컴파일 과정의 중복 제거된 참조 집합에서 만들어진다. BattleCore는 member-scoped 참조를 제외한 개수를 먼저 계산하고, `(ValueRef, TypedInteger)`의 그 개수만큼 `Vec::with_capacity`를 호출한다. Rust 표준 `Vec` 계약상 이 호출은 요소 수와 요소 크기를 곱한 정확한 allocator 요청 크기를 사용한다. 정렬된 Vec는 중복 key를 제자리 교체하고 이진 탐색으로 조회한다. 프레임 생성 중 새 참조 삽입은 승인한 논리적 용량을 넘기지 않으며, 용량 초과라면 성장 할당 전에 오류를 반환한다. 이로써 첫 BTreeMap 노드와 뒤따르는 분할·성장 할당을 제거했다. 삽입 때 정렬 이동은 참조 수에 선형이고, 평가 중 반복 조회는 참조 수에 로그 비용이다.

Live frame의 선청구는 `count × (size_of::<(ValueRef, TypedInteger)>() + 256) + 4 × 동적 참조 문자열 bytes`다. 기존 참조당 청구 수치와 총량은 유지하되, **전체 금액을 Vec 할당 전에 한 번에 승인**한다. `+256`은 frame 생성 도중의 부수 고정 비용, 문자열의 네 배는 key·value unit·검사 및 resource lease에 필요한 복제 요청을 보수적으로 덮는다. 검토에서 관측된 1,072 bytes를 상수로 쓰지 않는다. 누적 청구·미환급 정책과 기본/최대 예산은 그대로다.

Snapshot은 capture가 frame을 소유한다. 기존 `reference_slots`가 새 Vec 버퍼 요청량을 넘고, capture의 동적 참조 문자열 청구에 frame 소유 문자열 몫을 추가했다. Snapshot frame 생성에는 Live의 별도 선청구를 하지 않는다. 평가기의 `admit_frame_values`는 기존대로 참조 검사 work만 청구하며, 이미 capture가 소유한 frame 저장 용량은 평가 때 다시 청구하지 않는다. evaluator가 새로 만드는 trace 등은 기존 별도 청구 대상이다.

## 검증과 결과

- 검토자의 실패 probe 2개를 `verification/probes.rs`에 복사해 부족 예산에서 **Vec allocator 요청 이전 거부**, 충분 예산에서 **실제 신규 요청 합계 ≤ 청구량**을 모두 검사하도록 강화했다. 원본 probe와 실패 증거는 이전 review 디렉터리에 그대로 있다. 격리 release 빌드에서 두 probe와 두 참조/반복 평가, 100자 동적 resource key probe가 4/4 통과했다. 단일 frame은 352 청구/96 요청, 전체 평가는 1,120 청구/736 요청, 문자열 key frame은 776 청구/520 요청이었다. 이 수치는 Linux/WSL2 release allocator 요청량이며 RSS가 아니다.
- 제품 테스트는 bounded frame의 정렬 순서, 중복 교체, 용량 초과 전 거부, 문자열 key 및 128회 반복 조회를 확인한다. 전체 `game_core` all-target 1,463개가 통과했다. 여기에는 Snapshot/Live 값 분리, native work·취소, resource lease, rounding, fingerprint 회귀가 포함된다. 포맷 검사도 통과했다.
- 원본 PC fixture 36조건을 동일한 sample 방식으로 격리 release 재측정했다. 이전 F01 측정과 같은 25완료/11거부이며, 추가 5건의 128 Read 누적 retained 거부와 요청 site·native work가 유지된다. Live adapter 12조건의 실제 allocation 요청은 감소했고 나머지 조건의 요청량, HP 결과 및 event-log hash는 동일하다. `verification/pc_comparison.json`에 조건별 대조가 있다.
- 이전 동결 타깃의 manifest·source/evidence archive와 새 baseline archive는 해시로 재검증했다. 이전 currentness 검사기가 현재 수정 source에서 실패하는 것은 새 타깃으로 넘어간 결과이며, 이전 동결 자료의 손상을 뜻하지 않는다.

미검증: 다른 OS/allocator의 실측 요청량, 제품 규모 공세에서의 적합성. 이 타깃은 F01-R01만 교정한다. F02/F03/CAP-01/CAP-02, 다음 WP, 게시 경로는 미착수다. 자체 검증은 재검토 PASS 또는 WP02 완료 판정이 아니다.
