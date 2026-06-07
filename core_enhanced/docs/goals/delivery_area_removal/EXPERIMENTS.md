# Delivery Area Removal Experiments

## Usage Audit

Command:

```text
rg DeliveryDef::Area / SkillAreaDeliveryDef / SkillAreaShapeDef / AreaQueryShape
```

Result:

- Official live `base.ron` does not use geometric Area.
- `legacy_base.generated.ron` used geometric Area heavily and was a legacy generated artifact.
- Remaining Rust uses were schema/runtime/test/catalog residue.

Decision:

- No official live dependency was found.
- Proceed with deletion rather than validation-only compatibility.

## Deletion Pass

- Removed the `DeliveryDef::Area` schema path and related area shape structs.
- Removed geometric runtime helpers and collapsed timeline area shape to `tile_pattern`.
- Deleted geometric Area-focused tests instead of preserving them with `#[ignore]`.
- Removed the unused spatial query backend left behind by geometric Area deletion; projectile collision keeps the existing direct sweep helper.

## Verification

- `cargo test -p game_core ron_loading -- --nocapture`: pass.
- `cargo test -p game_core skill_refactor_validation -- --nocapture`: pass.
- `cargo test -p game_core skill_test_suite -- --nocapture`: pass.
- `cargo test -p game_core -- --nocapture`: pass.
- `cargo check -p game_core`: pass.
- `cargo check -p game_server`: pass.
- `cargo test -p game_server -- --nocapture`: not run because server request/result mapping was not changed.
