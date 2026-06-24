# PVE Enemy Wave Model Refactor Notes

## Working Decisions

### Enemy Entry Shape

The current `PveWaveEnemyData` struct is the main design smell.

It makes `abnormality_id` structurally mandatory even for `CorrodedEmployee` entries. This is not just naming polish; it weakens validation because code can accidentally fall back from `profile_id` to `abnormality_id` and still appear to work.

Preferred direction:

- Replace the struct with explicit enemy variants.
- Keep common behavior behind helper methods only where it is actually common.
- Do not preserve abnormality-specific identity fields on corroded employee entries.

### Generated Corroded Waves

Generated corroded waves are not arbitrary random encounters. They are deterministic authored pools:

- authored preset
- authored role mix
- budget/count constraints
- deterministic seed inputs

This behavior should be preserved. The refactor should move the code, not redesign the gameplay.

### Legacy `PveWaveData.enemies`

The top-level `enemies` field is now a legacy path beside `source`.

Preferred direction:

- Convert all live RON to `source: Manual(...)` or another explicit source variant.
- Remove top-level `enemies`.
- Avoid a dual schema.

If this breaks saved content or external tools, stop and ask the user instead of introducing compatibility silently.

### Preview vs Runtime

Preview currently owns generated wave resolution. That is backwards for long-term architecture.

Preview should ask the battle/domain layer what enemies a wave contains, then display that result. It should not be the module that defines the generation algorithm.

Follow-up code review refined this slightly:

- Current battle setup does not independently resolve raw `PveWaveData`.
- It consumes `CombatPreview.spawn_waves[*].enemy_entries`.
- Therefore, the goal is not to make preview and battle setup both call generation separately.
- The goal is to move generation out of preview internals while preserving one resolved composition that preview, setup snapshot, and runtime spawning all share.

This matters because generated corroded waves are seed-based. If battle setup re-runs generation with a different seed/context from preview, Unity can preview one wave and runtime can spawn another.

### SpawnWaveEnemyEntry Contract Risk

`SpawnWaveEnemyEntry` mirrors the same mixed identity shape as `PveWaveEnemyData`:

- `kind`
- `profile_id: Option<String>`
- `abnormality_id: String`

This may be Unity-facing through combat preview or battle setup snapshot paths. Treat it differently from `PveWaveEnemyData`:

- `PveWaveEnemyData` is the RON/internal authoring model and should be refactored first.
- `SpawnWaveEnemyEntry` can be cleaned up only after verifying Unity contract impact.
- If Unity depends on this shape, stop and report the contract change instead of silently changing it.

### Preview Briefing Legacy Read

`primary_enemy_kind()` currently reads `wave.enemies.first()` directly. That ignores `source: Manual(...)` and `GeneratedCorroded`, and it will become wrong once top-level `enemies` is removed.

Fixing this is part of the goal because it is a user-visible preview behavior, not just internal cleanup.

### FacilityEntity

`EnemyKind::FacilityEntity` exists, but runtime spawning currently returns `MissingResource("FacilityEntityProfile")`.

Do not invent a facility/entity profile data model in this goal. Keep the variant unsupported until a real facility entity policy and data source exist.

### Implemented Contract

Implemented result:

- `PveWaveEnemyData` is now the explicit authoring/data variant model.
- `PveWaveData.source` is the only authored wave composition source.
- Live RON no longer uses top-level `enemies`.
- Manual corroded employee waves use `CorrodedEmployee(profile_id: ...)`.
- Manual abnormality waves use `Abnormality(abnormality_id: ...)`.
- Generated corroded waves are resolved by `game::wave_resolution`, not by `combat_preview` internals.
- `SpawnWaveEnemyEntry` remains unchanged as an adapter/DTO shape because it may be Unity-facing.
- Corroded employee logic must use `profile_id`; it must not fall back to `abnormality_id`.

## Policy Questions To Stop For

No immediate user question is required before starting.

Stop if any of these become necessary:

- Should `FacilityEntity` be implemented now, removed for now, or left as an unconstructable variant? Default answer for this goal: leave it unsupported unless code proves that impossible.
- Should generated corroded wave seed inputs change from preview seed to battle/session seed?
- Should RON schema use externally tagged, internally tagged, or adjacent tagged enum entries if readability and serde compatibility conflict?
- Should Unity receive a changed wave enemy DTO shape, or is this strictly internal/live RON?

## Follow-Up Candidates Outside This Goal

- Add role/archetype tags to `CorrodedEmployeeProfileMetadata`, such as guard, rusher, marksman, bruiser, medic, veteran.
- Consider using those tags for threat preview text and wave preset authoring.
- Add a future facility entity profile database if tool/object enemies become real combat actors.
- Review `EnemyBriefing` fields after this refactor; it may also contain abnormality-biased naming.
