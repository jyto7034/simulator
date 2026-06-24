# Item Inventory Cleanup Plan

## Objective

Make impossible item flows explicit in code by documenting permanent artifact ownership and removing abnormalities from the generic item/inventory/shop registry path.

## Scope

In scope:

- Confirm artifact ownership/removal policy from runtime code and live data.
- Remove `Item::Abnormality` and `ItemRef::Abnormality` from generic item flows if code confirms abnormalities are not player inventory items.
- Remove the stale artifact removal TODO and replace it with explicit permanent artifact behavior.
- Update focused tests that currently preserve legacy abnormality-as-item behavior.

Out of scope:

- New artifact functionality.
- Reward balance changes.
- Live RON reward/shop rewrites unless validation proves they are required.
- Unity-facing DTO shape changes.
- Save migration.

## Source Of Truth

1. Runtime code:
   - `src/game/data/mod.rs`
   - `src/game/resources/inventory.rs`
   - `src/game/world/shop.rs`
   - `src/game/reward.rs`
   - `src/game/world/snapshot.rs`
2. Live data:
   - `../game_resources/data/events/shops/base.ron`
   - `../game_resources/data/events/rewards/base.ron`
   - `../game_resources/data/artifacts/base.ron`
   - `../game_resources/data/abnormalities/base.ron`
3. Policy/design docs:
   - `docs/component_design_review.md`
   - `docs/game_rulebook.md`
   - `docs/item_inventory_cleanup_goal.md`

## Policy

- Artifacts are permanent owned run items. They can be granted and listed in inventory, but they are not removable/sellable through the generic owned-item removal path.
- Abnormalities are not player-owned inventory items. They belong to combat/content data and must not enter shop purchase, reward grant, inventory diff, or item snapshot paths.

## Plan

1. Record current artifact and abnormality item flows.
2. Remove `Item::Abnormality`, `ItemRef::Abnormality`, and `ItemIndex::Abnormality` from `GameDataBase::item`/`ItemRegistry`.
3. Remove abnormality-specific rejection branches made unreachable by the type change.
4. Update tests and fixture helpers that encoded abnormality-as-item legacy behavior.
5. Run focused inventory/data/shop/reward checks and live RON loading tests.

## Completion Conditions

- Artifact permanent ownership policy is explicit in code/tests.
- `ArtifactSlots` removal TODO is gone.
- `Item::Abnormality` no longer exists in the generic item enum.
- Shop/reward/inventory paths cannot receive abnormalities through the item registry.
- Live RON loading and focused inventory/data/shop/reward tests pass.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- Live shop/reward RON still references abnormality UUIDs as purchasable/display items.
- Removing `Item::Abnormality` requires a save migration or Unity-facing DTO change.
- A product decision is needed about artifact selling/removal.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Verification Commands

- `rg -n "Item::Abnormality|ItemRef::Abnormality|ItemIndex::Abnormality|as_abnormality|ArtifactSlots에 remove_by_uuid" src/game tests`
- `cargo test -p game_core resources::inventory::tests -- --nocapture`
- `cargo test -p game_core data::tests -- --nocapture`
- `cargo test -p game_core world::tests::snapshots_and_start -- --nocapture`
- `cargo test -p game_core --test ron_loading`
- `cargo check -p game_core`
