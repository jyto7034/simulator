# PVE Enemy Wave Model Refactor Plan

## Objective

Refactor PVE wave enemy data so the code model matches the domain model:

- Abnormality enemies are identified by `abnormality_id`.
- Corroded employees are identified by `profile_id`.
- Future facility entities / tool-like battlefield entities should not inherit abnormality-only field names.
- Generated corroded waves should be resolved by shared battle-domain wave generation code, not preview-local helper logic.

The current code works, but it encodes different enemy identities through one shared `PveWaveEnemyData` struct:

```rust
pub struct PveWaveEnemyData {
    pub kind: EnemyKind,
    pub profile_id: Option<String>,
    pub abnormality_id: String,
    pub tier: Tier,
    pub count: u32,
}
```

That shape forces `CorrodedEmployee` entries to carry an `abnormality_id` field, and generated corroded waves currently emit `abnormality_id: profile_id`. This is easy to misuse as enemy categories expand.

Target shape should be a discriminated enemy entry model, for example:

```rust
pub enum PveWaveEnemyData {
    Abnormality {
        abnormality_id: String,
        tier: Tier,
        count: u32,
    },
    CorrodedEmployee {
        profile_id: String,
        tier: Tier,
        count: u32,
    },
    FacilityEntity {
        profile_id: String,
        tier: Tier,
        count: u32,
    },
}
```

The exact Rust/RON representation may differ if runtime code reveals a cleaner long-term design, but the final model must not require corroded employees to masquerade through abnormality-specific fields.

## Source Of Truth Order

Verify facts in this order during implementation:

1. Runtime code.
2. Live RON/data.
3. Unity-facing snapshot/command/update contracts.
4. Latest policy docs.

Do not trust this plan over runtime code. If code reading shows a better long-term shape, record the evidence in `EXPERIMENT_NOTES.md` and apply it unless it requires a user policy decision.

## Current Observed Structure

Code reading before this goal found:

- `src/game/data/pve_data.rs`
  - `PveWaveEnemyData` mixes `kind`, `profile_id`, and `abnormality_id`.
  - `PveWaveData` has both `source: Option<PveWaveSource>` and legacy `enemies: Vec<PveWaveEnemyData>`.
  - Validation prevents `source: Manual` and legacy `enemies` from being used together, but the legacy field still exists.
- `src/game/data/corroded_wave_data.rs`
  - `CorrodedWaveRoleWeight::to_enemy_data()` creates a `CorrodedEmployee` entry but fills `abnormality_id` with `profile_id`.
- `src/game/combat_preview/mod.rs`
  - `resolve_wave_enemy_data()` and `generate_corroded_wave()` live in preview code even though they decide actual wave composition.
  - Generated waves are deterministic pseudo-random mixes based on preset, budget, count range, weights, preview seed, seed salt, wave index, and pick step.
  - `primary_enemy_kind()` still inspects only legacy `wave.enemies.first()`, so `source: Manual(...)` and `GeneratedCorroded` waves can be misclassified in preview briefing.
- `src/game/combat_preview/types.rs`
  - `SpawnWaveEnemyEntry` also carries both `profile_id: Option<String>` and `abnormality_id: String`.
  - This may be Unity-facing through combat preview / setup snapshot paths, so changing this DTO shape requires an explicit contract check before editing.
- `src/game/combat_setup/enemy_spawns.rs`
  - Runtime battle setup consumes `CombatPreview.spawn_waves[*].enemy_entries`; it does not currently resolve raw `PveWaveData` independently.
  - Runtime spawning already treats `EnemyKind::CorrodedEmployee` as profile-based and creates `BattleUnitSource::CorrodedEmployee`.
- `src/game/data/validation.rs`
  - Manual corroded entries require `profile_id`.
  - Generated corroded presets validate profile references.
- `src/game/world/state.rs`
  - `battle_setup_snapshot` currently collects `abnormality_ids` from every `SpawnWaveEnemyEntry.abnormality_id`, which is suspicious for corroded employee entries and must be checked against Unity setup contract semantics.

## In Scope

- Replace ambiguous PVE wave enemy entry shape with explicit enemy variants.
- Remove or migrate the legacy `PveWaveData.enemies` path so `source` is the single wave composition entry point.
- Move generated corroded wave resolution out of `combat_preview` into a shared domain module.
- Keep one wave expansion source of truth. In the current architecture this likely means combat preview expands waves once, and battle setup consumes the resolved `SpawnWave` without regenerating a different composition.
- Update validation for live RON and generated presets.
- Update tests that inspect wave enemy identity.
- Update docs that describe PVE encounter/wave schema, corroded employee representation, or field enemy policy.

## Out Of Scope

- New enemy categories beyond keeping the existing `FacilityEntity` placeholder safely unsupported.
- New corroded employee profiles or balance changes.
- New wave pool policy beyond preserving the current deterministic generated-corroded behavior.
- Unity visual implementation changes.
- Boss/elite HUD policy changes unless a DTO shape conflict is discovered.

## Target Policy

### Enemy Identity

Enemy identity must be variant-specific:

- `Abnormality` entries use `abnormality_id`.
- `CorrodedEmployee` entries use `profile_id`.
- `FacilityEntity` entries remain unsupported until a facility/entity profile data source exists.

Do not keep an `abnormality_id` field on non-abnormality enemy entries as a compatibility alias.

### Wave Source

`PveWaveData` should have one authoritative composition source.

Preferred target:

- `source: PveWaveSource`
- `PveWaveSource::Manual(Vec<PveWaveEnemyData>)`
- `PveWaveSource::GeneratedCorroded { ... }`

Avoid keeping both `source` and legacy top-level `enemies`. If removing `enemies` creates a real migration hazard, stop and record the evidence; do not add a silent dual schema without user confirmation.

### Generated Corroded Waves

Generated corroded waves remain deterministic, constrained generation:

- profile pool comes from `corroded_wave_presets.ron`
- each role references `corroded_employees.ron`
- count is selected within `count_range`
- min/max/count/cost/weight/budget remain enforced
- same seed input must produce the same generated enemy list

The resolver should live in a shared domain location, not preview-only code.

### Preview / Runtime Consistency

Combat preview, setup snapshot, and actual battle setup must use one resolved wave composition.

Current code path:

1. `PveWaveData` is expanded while building `CombatPreview.spawn_waves`.
2. `enemy_spawns.rs` consumes those resolved `SpawnWave.enemy_entries` to build runtime enemy drafts.
3. Setup snapshot also exposes preview-derived wave/setup data.

The refactor should preserve that single-composition behavior. Do not make battle setup re-run generated wave selection separately from preview unless the architecture is deliberately changed and deterministic equality is tested.

### Unity-Facing Shape

`PveWaveEnemyData` is live RON/internal authoring data. `SpawnWaveEnemyEntry` may be Unity-facing. Before changing `SpawnWaveEnemyEntry` fields, verify the current Unity setup/preview contract.

Preferred implementation order:

- First remove ambiguity from the RON/internal wave model.
- Then remove fallback reads such as `profile_id.unwrap_or(abnormality_id)`.
- Only change `SpawnWaveEnemyEntry` shape if the Unity-facing contract docs/code are updated in the same goal, or stop and report the needed contract decision.

## Initial Code Reading Targets

Start by reading:

- `src/game/data/pve_data.rs`
  - `PveWaveEnemyData`
  - `PveWaveSource`
  - `PveWaveData`
  - encounter validation helpers
- `src/game/data/corroded_wave_data.rs`
  - `CorrodedWavePreset`
  - `CorrodedWaveRoleWeight::to_enemy_data`
- `src/game/combat_preview/mod.rs`
  - `resolve_wave_enemy_data`
  - `generate_corroded_wave`
  - `appearance_seeds_for_enemy`
  - enemy briefing generation
- `src/game/combat_preview/types.rs`
  - `EnemyKind`
  - `SpawnWaveEnemyEntry`
  - `EnemyBriefing`
- `src/game/combat_setup/enemy_spawns.rs`
  - `build_enemy_drafts`
- `src/game/data/validation.rs`
  - PVE encounter validation
- `tests/ron_loading.rs`
  - live RON schema and reference tests
- live RON:
  - `/mnt/f/work/simulator/game_resources/data/pve/encounters.ron`
  - `/mnt/f/work/simulator/game_resources/data/enemies/corroded_employees.ron`
  - `/mnt/f/work/simulator/game_resources/data/enemies/corroded_wave_presets.ron`

## Implementation Plan

1. Audit current PVE wave entry usage.
   - Search all `PveWaveEnemyData`, `EnemyKind`, `profile_id`, and `abnormality_id` reads.
   - Separate public DTO / Unity-facing usage from internal RON-only usage.
   - Record findings in `EXPERIMENT_NOTES.md`.

2. Design the explicit wave enemy variant model.
   - Prefer an enum or internally tagged RON shape.
   - Preserve authored readability in RON.
   - Keep `tier` and `count` semantics stable.
   - Provide helper methods only for common fields that are truly common, such as `count()` and `tier()`.

3. Migrate runtime code to variant-based matching.
   - Update validation to match per enemy variant.
   - Update preview wave expansion.
   - Update spawn wave DTO construction only after checking whether `SpawnWaveEnemyEntry` is Unity-facing.
   - Update combat setup draft construction to consume the resolved entry shape without abnormality/profile fallback.
   - Fix preview briefing code such as `primary_enemy_kind()` so it uses resolved wave entries or the shared resolver rather than legacy `wave.enemies`.
   - Remove fallback logic that treats `profile_id.unwrap_or(abnormality_id)` as a normal path.

4. Move generated corroded wave resolution into shared domain code.
   - Create a focused module such as `src/game/combat_setup/wave_resolution.rs` or `src/game/waves.rs` if that better matches current structure.
   - Preview should call the shared resolver when building `SpawnWave`.
   - Live battle setup should consume the preview-resolved `SpawnWave` unless the current preview-first architecture is intentionally changed.
   - Keep deterministic seed behavior unchanged unless current code proves it is wrong.

5. Remove legacy `PveWaveData.enemies`.
   - Convert live RON to explicit `source: Manual(...)` where needed.
   - Do not keep silent compatibility for both `source` and `enemies`.
   - If removing `enemies` requires broad save/migration policy outside live RON, stop and ask the user.

6. Update generated corroded preset output.
   - Generated roles should produce `PveWaveEnemyData::CorrodedEmployee { profile_id, tier, count }`.
   - No generated corroded output should carry abnormality identity fields.

7. Update tests around user-visible behavior.
   - Live RON loading validates all manual and generated wave references.
   - Generated corroded wave preview produces only corroded employee entries.
   - Same seed and preset produce stable generated composition.
   - Combat setup can spawn generated corroded employees from resolved wave entries.
   - Preview briefing / primary enemy kind works for `source: Manual(...)` and `GeneratedCorroded`, not only legacy `wave.enemies`.
   - Abnormality and corroded employee entries fail validation when they use the wrong identity field.
   - Existing tests that only codify legacy struct shape should be deleted or rewritten.

8. Update docs.
   - `docs/game_rulebook.md`: field enemies and corroded employee wave policy if applicable.
   - `docs/skills/lobotomy_content_catalog.md` only if enemy category language is affected.
   - `docs/README.md` if it indexes the relevant domain docs.
   - External Unity docs only if Unity-facing DTO shape changes.

9. Verify incrementally.
   - Run a focused test after each small structural slice.
   - Run `cargo check -p game_core` after model migration.
   - Run live RON validation tests before broad tests.
   - Finish with broad relevant tests.

## Completion Conditions

- `PveWaveEnemyData` no longer mixes abnormality and corroded employee identity fields.
- Corroded employee wave entries use `profile_id` only.
- Manual wave composition uses the same explicit enemy entry model as generated output.
- Top-level legacy `PveWaveData.enemies` is removed, or a user-approved removal condition is documented if impossible.
- Generated corroded wave resolution is no longer preview-only logic.
- Preview and battle setup use one resolved generated-corroded composition; battle setup must not accidentally regenerate a different wave.
- Preview briefing no longer depends on legacy `PveWaveData.enemies`.
- Live RON loads and validates with the new schema.
- Tests cover runtime behavior and data validation, not merely old internal struct layout.
- No compatibility layer, fallback path, or dual schema remains unless explicitly approved and documented with removal conditions.

## Stop Conditions

Stop and report questions if implementation discovers any of these:

- Unity-facing DTO shape would change in a way not covered by current contract docs.
- `SpawnWaveEnemyEntry` is confirmed Unity-facing and cannot be changed without a broader Unity contract update.
- Saved player data or migration policy is needed.
- Existing live RON relies on mixed abnormality/corroded identity in a way that cannot be cleanly migrated.
- Facility entity profile policy must be decided before the enemy entry enum can be finalized. If this happens, keep `FacilityEntity` unsupported rather than inventing a profile source.
- Generated wave seed semantics would change in a gameplay-visible way.
- A compatibility layer appears necessary.

## Verification Plan

Run focused commands as slices progress:

```bash
cargo check -p game_core
cargo test -p game_core ron_loading -- --nocapture
cargo test -p game_core combat_preview -- --nocapture
cargo test -p game_core combat_setup -- --nocapture
```

Final verification should include:

```bash
cargo fmt
cargo test -p game_core
```

Record every failed command, cause, fix, and re-run result in `EXPERIMENTS.md`.
