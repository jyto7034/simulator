# R06 WP02 D04 유닛 관계 구현 PLAN

- 기준선: D04-F01 successor manifest SHA-256 `e4f0d77b1bd3066493d605271605d7a4b6e16317a6d540d303909b076b480104`, source digest `34ceedbebc84b9396d42f92fa31033fe0b831a90e5325c8c4463e0c88f067145`. 구현 직전 currentness와 격리 양방향 복원을 재검증하고, 현재 branch/HEAD/dirty/untracked 및 333개 source bytes를 `baseline/entry_001`에 보존한다.
- 범위: 실제 resolved target을 Side, 실제 Employee UUID, 지정 장비 슬롯의 실제 copy UUID별로 집계한다. `IsActualEmployee`와 `HasEquippedSlot(address)` 조건을 bounded conjunction으로 추가한다. 장착 layout/slot을 ItemSlot→draft/profile→두 runtime spawn 경로로 전달하고 aggregate node별 immutable member view를 parser/compiler/evaluator/runtime/capture에 연결한다.
- 순서: (1) 슬롯 주소와 projection, (2) runtime 장착 snapshot/clone/extent, (3) source·grouping·bounded parser/compiler/fingerprint, (4) node별 view/evaluator, (5) 실제 runtime adapter와 Snapshot/Live/ordered effect 소비, (6) O01–O15·회귀·PC36·R05 비용 원자료·전체 검증.
- 보존 계약: 기존 result prefix 누적, release Snapshot/Live, WholeSet/ByProjectedTile, D04-F01 무대상 projectile 완료, target별 선평가 후 mutation, publication 차단을 유지한다. 자동 제외·0 fallback·첫 장비 선택을 넣지 않는다.
- 제외: D07 전체, WP03 stat 공식, 효과·상태 조회, 실험실·편집기, 안전 초과 자동 취소, 20분 시험, production publication 해제.
- 비용 판정: 기존 hard guard와 parity를 유지한다. 64-session concurrent peak `9,030,188` bytes 초과만 사용자 정책에 따라 warning으로 판정하고 raw exit/sample을 그대로 보존한다.
- 종료: 실제 명령·실패 이력·raw 비용·O01–O15 대응·변경 bytes/patch를 단계검토 target에 동결하고 currentness 및 격리 양방향 복원을 검증한다. 독립 검토와 WP02/R06 전체 완료 판정은 수행하지 않는다.
