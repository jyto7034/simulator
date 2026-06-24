# Item Inventory Cleanup Notes

## Runtime Findings

- `Inventory` comment already states abnormalities are no longer player-owned and are used as employee/combat-opponent data.
- `Inventory::remove_item` only removes equipment and consumables. Artifact removal returned `None` because `ArtifactSlots` lacked `remove_by_uuid`.
- `has_artifact` explicitly says artifacts are not removable/sellable and duplicate checks use metadata UUID.
- `world/shop.rs` still had an `Item::Abnormality` rejection branch because `GameDataBase::item` registered abnormalities.
- `world/snapshot.rs` could display an abnormality item snapshot only because `ItemRef::Abnormality` existed.

## Policy Captured In This Goal

- Artifacts are permanent owned items. Generic owned-item removal is for equipment and consumables only.
- Abnormalities are not inventory/shop/reward items. They remain in `AbnormalityDatabase` for combat, PvE, skill fragment origin, and content references.

## Follow-Up Candidates

- `Item`/`ItemRef` definitions still live in `data/mod.rs`; if data modules continue to grow, move item registry types to a dedicated `data/item_registry.rs`.

## Policy Questions

- Resolved: user confirmed document policy should be prioritized and S/A/B content should be implemented first.
- Applied policy:
  - C-grade/excluded abnormalities such as Meat Lantern, Scarecrow, and Porccubus are not valid shop items.
  - S/A/B abnormalities may be represented in shop content only through their itemized derivatives, currently placeholder E.G.O equipment.
  - Shops must not reference abnormality UUIDs as purchasable/display item UUIDs.
