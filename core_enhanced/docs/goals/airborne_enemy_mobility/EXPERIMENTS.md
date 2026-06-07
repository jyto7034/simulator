# Airborne Enemy Mobility Experiments

## Log

- Started after Movement Backend Policy Refactor completed.
- Read `docs/airborne_enemy_mobility_goal.md`, master plan, and current policy decisions.
- Added `mobility_kind` through abnormality metadata, combat profile, runtime unit, unit snapshot, and `UnitSpawned`.
- Wired airborne runtime units into `MovementTerrainPolicy::Airborne`.
- Excluded airborne enemies from block matching.
- Split airborne enemy basic attack target priority from normal ground targeting: protected target in range first, otherwise nearest player combatant.
- Added `air_capable` to cast target and step DTO/schema so single-target cast and step resolution can independently gate airborne targets.
- Renamed the shared targetability helper from basic-attack-specific wording to `single_target_can_target_unit`.
- Added focused runtime tests for airborne blocking, route attack/continue behavior, protected-target priority, nearest combatant fallback, basic attack air capability, skill cast/step air capability, area hit inclusion, and preview warning generation.
- Marked live `Punishing Bird` as `mobility_kind: airborne`.
- Added a focused `ron_loading` test for live abnormality mobility schema because full live GameData loading is currently blocked by weapon-profile data debt.
- Updated canonical Unity docs under `F:\unity projects\ark\docs`.

## Verification

- `cargo test -p game_core airborne -- --nocapture`: passed.
- `cargo test -p game_core movement -- --nocapture`: passed.
- `cargo test -p game_core blocking -- --nocapture`: passed.
- `cargo test -p game_core combat_preview -- --nocapture`: passed.
- `cargo test -p game_core --test ron_loading live_abnormality_ron_reads_airborne_mobility_kind -- --nocapture`: passed.
- `cargo check -p game_core`: passed.
- `cargo check -p game_server`: passed.
- `cargo test -p game_core --test ron_loading -- --nocapture`: failed because live weapon equipment still lacks `weapon_profile`.
- `cargo test -p game_core -- --nocapture`: failed because several legacy/test weapon equipment fixtures and live `justitia` still lack `weapon_profile`; one newly added step revalidation fixture failed once and then passed after adding battlefield placement.
