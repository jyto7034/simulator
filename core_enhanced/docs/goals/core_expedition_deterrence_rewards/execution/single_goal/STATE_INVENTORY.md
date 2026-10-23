# U14 상태 수명

| 상태 | 권위/수명 | 복원 |
| --- | --- | --- |
| 비용·배율·성과·gold plan 정의 | validated catalog/content identity | 저장은 참조/구조 검증 |
| entry receipt·deterrence·gold assignment/collection | 현재 원정 run | checkpoint/Continue |
| trauma·remaining_cost·사망/자산 | 직원/run | pre-entry Continue/post-entry retreat 구분 |
| unrestrained/seal/revival scale·reward eligibility | 전투 진입 snapshot | 결과동안 불변, 후퇴후 새 전투에서 재확정 |
| 실제 피해/시간 성과 | 현재 battle attempt | 실제 mutation/시간; 결과 snapshot, 후퇴시 폐기 |
| 보상/가지/refill·노드소비 | staged result/confirmed result | 승패즉시확정·중복불가 |
| 원정 종료 영속기록/반출 | existing profile/paired store | U12 계약 유지 |

실제 구현 상세: `PerformanceBonusState`는 primary UUID와 최고 단일실효HP손실/당시 최대HP의 scalar 세 값만 보유한다. BattleCore clone shape에 포함하며 시도 종료/후퇴와 함께 해제한다. GoldenBoughState는 선택 plan ID/위치목록/회수집합/refill을 run owner로 보유한다. run 생성 때 한 번 선택하고 방복원은 같은 상태를 사용한다. preview는 추가 권위를 만들지 않고 employee UUID 순서로 투영한다. authored floor minimum만 manifestation의 적 난이도 기준이며 certified peak는 진단/저장 비교 용도로 남는다.

교정001: 공개 봉인 preview는 순회 중인 방ID+실제 node의 slot/부활 여부와 기존 GoldenBoughState를 조회한다. 새 저장 상태·DTO·수명은 없다. checkpoint 검증의 억제력 공급 상한은 각 snapshot의 초기 공급+회수집합 크기×보충량으로 계산한다. 현재값과 SavePoint의 회수집합을 섞지 않으며 실제 지급/overflow와 transaction owner는 기존 그대로다.
