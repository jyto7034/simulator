# R06 WP02 D04-OWNER-F01 교정 PLAN

- 시작점은 owner relation target manifest SHA-256 `c2e703c2a08e1fc38141d3d070e052d8207cd7aaf9790f650c685429728a2a9f`, 확장 337개 digest `eb9bd8fb3927f8a9f3c5b35908012300e4fad9c4e8d7a03a9e612e44b0dc804a`다. `freeze.py verify`의 currentness와 격리 양방향 복원을 통과한 실제 현재 바이트를 새 baseline으로 보존한다.
- 독립 검토 finding D04-OWNER-F01만 교정한다. runtime slot entry에 기존 binding과 정확한 runtime item instance ID를 함께 보관하고, 장비 조건·관계의 전투 전체 items 순회를 `items.get(id)` 직접 조회로 바꾼다.
- 두 spawn 경로에서 기존 `equipped_items` 원 순번, RuntimeItem/provider ID, 생성·등록 순서를 보존한다. ByType 슬롯 칸이나 Any3 주소를 item ID salt로 사용하지 않는다.
- 직접 조회 뒤 map key/internal instance ID/owner/base/copy 일치를 모두 검증한다. 정상 빈 슬롯, unknown layout, copy 미제공, nil/중복/stale identity의 기존 typed 의미와 no-fallback 계약을 유지한다.
- `make_item_instance_id`의 기존 37바이트 입력을 stack buffer로 구성하되 namespace, side tag, big-endian salt, stable seed와 최종 UUID 체계를 바꾸지 않는다.
- C01–C09와 기존 O01–O15, D04-F01, Snapshot/Live, rollback, 실제 inventory projection을 검증한다. 실제 직접 조회 지점은 test 전용 thread-local counter로 관측하며 여러 node·두 pass의 조회 수가 `2 * N * (P + R)` 이하인지 확인한다.
- 새 slot entry의 실제 `size_of` 증가를 기존 projection retained admission과 runtime fact extent가 반영하는지 경계 시험으로 확인한다. Arc 공유 clone/scratch 계약을 유지한다.
- 최종 source에서 지정된 check, value_expr 집중 시험, 전체 target 시험, clippy, PC36, R05 exact workload를 별도 product target으로 실행하고 command/env/source identity/raw 실패를 보존한다. R05 원 cap 9,030,188 bytes의 concurrent peak 단일 초과만 경고다.
- 새 successor `target/d04_owner_f01_correction_001`에 before/after, 양방향 patch, 증거와 인계서를 동결한다. 독립 PASS, D04/WP02/R06 완료나 publication 해제를 선언하지 않는다.
