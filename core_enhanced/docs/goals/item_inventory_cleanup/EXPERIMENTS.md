# Item Inventory Cleanup Experiments

## 2026-06-13

### Inspect current flows

Command:

```bash
rg -n "Item::Abnormality|ItemRef::Abnormality|as_abnormality|Abnormality\\(" src/game tests --glob '!tests/skill_test/**'
```

Result:

- `Item::Abnormality` exists only in generic item/registry, inventory rejection, shop rejection, snapshot display, and tests.
- Combat setup uses abnormalities directly as combat/content data, not player inventory items.

Command:

```bash
rg -n "abnormality|Artifact|GrantArtifact|visible_items|hidden_items" ../game_resources/data/events ../game_resources/data/map ../game_resources/data
```

Result:

- Live shop data uses equipment/consumable/artifact item references, not abnormality item references.
- Reward data has explicit `ForbiddenAbnormalityGrant`, which already represents the forbidden legacy reward path outside `Item`.

### Decision

- Remove abnormalities from generic item registry and item enum.
- Keep artifacts as permanent inventory entries. Do not add artifact removal.

### Compile after removing abnormality item variants

Command:

```bash
cargo check -p game_core
```

Result:

- Passed.

### Focused inventory and data tests

Command:

```bash
cargo test -p game_core resources::inventory::tests -- --nocapture
```

Result:

- Passed: 6 tests.

Command:

```bash
cargo test -p game_core data::tests -- --nocapture
```

Initial result:

- Failed: `game_data_base_panics_on_duplicate_item_uuid_across_categories` still expected abnormality UUIDs to participate in item registry duplicate detection.

Fix:

- Replaced the legacy expectation with `game_data_base_panics_on_duplicate_item_uuid_across_inventory_item_categories`, which verifies duplicate UUID detection across actual inventory item categories.

Recheck result:

- Passed: 50 tests.

Command:

```bash
cargo test -p game_core world::tests::snapshots_and_start -- --nocapture
```

Result:

- Passed: 7 tests.

### Live RON loading

Command:

```bash
cargo test -p game_core --test ron_loading
```

Result:

- Failed: 15 tests fail during `GameDataBase` construction because `../game_resources/data/events/shops/base.ron` has `yesod_shop` referencing missing item UUID `19015400-0000-0000-0000-000000000117`.
- Inspection shows that UUID is Meat Lantern in `../game_resources/data/abnormalities/base.ron`.
- This proves live shop data still treats an abnormality as a purchasable/display item, which is a policy/content conflict after removing abnormalities from the generic item registry.

Stop condition reached:

- `docs/goals/item_inventory_cleanup/PLAN.md` says to stop if live shop/reward RON still references abnormality UUIDs as purchasable/display items.

Resolution after policy confirmation:

- User confirmed document policy should win and S/A/B content should be implemented first.
- Removed the C-grade/excluded Meat Lantern abnormality shop entry.
- Re-ran live RON loading and found `yesod_shop` had more abnormality UUIDs.
- Updated `../game_resources/data/events/shops/base.ron` so:
  - C-grade/excluded Scarecrow and Porccubus are not sold as shop items.
  - S/A/B candidates in `yesod_shop` are represented by placeholder equipment UUIDs, not abnormality UUIDs.
  - S-grade candidates in `binah_shop` are represented by placeholder equipment UUIDs, not abnormality UUIDs.

Command:

```bash
rg -n "<known abnormality uuids from yesod/binah shops>" ../game_resources/data/events/shops/base.ron
```

Result:

- No matches. The live shop data no longer references those abnormality UUIDs as purchasable/display items.

Command:

```bash
cargo test -p game_core --test ron_loading
```

Recheck result:

- Passed: 16 tests.

### Final focused verification

Commands:

```bash
cargo fmt
cargo check -p game_core
cargo test -p game_core resources::inventory::tests -- --nocapture
cargo test -p game_core data::tests -- --nocapture
cargo test -p game_core world::tests::snapshots_and_start -- --nocapture
```

Results:

- `cargo fmt`: passed.
- `cargo check -p game_core`: passed.
- Inventory tests: passed, 6 tests.
- Data tests: passed, 50 tests.
- Snapshot/start tests: passed, 7 tests.
