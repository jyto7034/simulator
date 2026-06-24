# Stat Pipeline And Inventory Contract Repair Plan

## Objective

최근 component refactor 이후 발견된 두 기술부채를 장기 방향으로 보완한다.

1. Battle-entry stat pipeline의 두 실행 단계 관계를 명시하고, 교차 단계 순서를 실제로 검증하는 테스트로 교체한다.
2. Artifact는 유지하되 generic inventory remove path에서 `None`으로 조용히 빠지는 대신 제거 불가를 명시적으로 표현한다.

## Source Of Truth

1. Runtime code:
   - `src/game/battle/stat_pipeline.rs`
   - `src/game/combat_setup/player_spawns.rs`
   - `src/game/battle/core/build.rs`
   - `src/game/resources/inventory.rs`
   - `src/game/world/shop.rs`
   - `src/game/world/maintenance.rs`
2. Unity-facing error/snapshot contract:
   - `src/game/world/snapshot.rs`
3. Tests:
   - `src/game/battle/stat_pipeline.rs` tests
   - `src/game/resources/inventory.rs` tests
   - world shop/maintenance/equipment tests
4. Policy:
   - Artifact is retained for future use as a global/passive owned item.
   - Artifact is not removed through the generic owned equipment/consumable removal path.

## Plan

1. Record the already-started `stat_pipeline.rs` edit as a process correction.
2. Ensure module-level stat pipeline docs describe the two runtime stages as one battle-entry contract.
3. Replace the weak consumable/equipment order test with an order-sensitive test.
4. Add an explicit inventory removal error for non-removable artifact UUIDs.
5. Update generic remove callers to propagate the explicit error where they use generic removal.
6. Update Unity-facing error code mapping.
7. Run focused tests and a broader `cargo check`/test pass appropriate to the touched surface.

## Completion Conditions

- `stat_pipeline.rs` documents the full two-stage battle-entry stat order.
- A test fails if equipment percent effects are effectively applied before the consumable employee-profile stage.
- Removing an artifact UUID through generic `Inventory::remove_item` returns an explicit not-removable error instead of `None`.
- Existing equipment/consumable removal behavior remains unchanged.
- Unity-facing error code has a stable explicit value for not-removable inventory items.
- Focused tests and `cargo check -p game_core` pass.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- Fixing the issue requires changing live RON schema or saved data shape.
- Fixing the issue requires changing artifact policy beyond "retain, global/passive, not generic-removable".
- Unity-facing error semantics require a product decision beyond adding a clearer code for an existing invalid request.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

