# game_resources_client

Client-only metadata for replay/FX.

- Authoring source: `meta.ron`
- Build artifact (generated): `client_meta.db` (do not hand-edit)

This dataset is intended to map stable server keys to Addressables addresses:
- Units: `base_uuid` (UUID) -> unit prefab + basic-attack VFX
- Skills: `skill_id` (string) -> cast/projectile/hit VFX + optional animation key
- Buffs: `buff_key` (string, stable) -> apply/tick/aura/expire VFX
- Equipments/Artifacts: `base_uuid` (UUID) -> prefab/VFX/icon

## Build `client_meta.db`

From repo root:

- `cargo run -p client_meta_builder -- --input game_resources_client/meta.ron --output game_resources_client/client_meta.db`

Notes:
- `--overwrite=true` is the default.
- The output DB is intentionally query-only (PK lookups).

Tip: You can run `client_meta_builder` from any working directory; if the input path isn't found relative to the CWD, it will fall back to resolving under the repo root.
