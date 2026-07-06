# PVE Encounter Classification And Floor Scaling Contract Plan

## Objective

Refactor PVE encounter authoring so it matches the current game flow:

```text
facility Floor node
-> explicit encounter classification
-> selected encounter scenario
-> Floor scaling transforms combat intensity
-> DefenseRoute battle construction
```

This goal replaces `PveEncounter.difficulty` with explicit non-numeric encounter classification and introduces live-RON Floor scaling policy for enemy stat growth, generated wave budget growth, and explicitly authored extra reinforcement waves.

The goal must keep the long-term separation clear:

- encounter classification decides what kind of encounter can appear;
- current Floor decides how strong the encounter becomes;
- wave data decides what actually spawns;
- abnormality metadata decides research/response identity for elite, normal boss, and final boss suppression.

## Source Of Truth Order

Verify facts in this order during implementation:

1. Runtime map encounter assignment, battle construction, wave resolution, and PVE encounter loading code.
2. Live RON/data for PVE encounters, run policy, map generation, abnormalities, and corroded wave presets.
3. Unity-facing snapshot/command/result contracts and live probe JSON if any DTO shape changes.
4. `docs/game_rulebook.md`, `docs/skill_target_contract.md`, the Endless master goal, and other latest policy docs.

Do not trust this plan over runtime behavior. If code reading shows a better long-term model, record the evidence in `EXPERIMENT_NOTES.md` and apply it unless it requires a user policy decision.

## Dependency And Placement

This goal is the canonical replacement for the earlier difficulty-removal planning target. The useful audit findings from that target have been merged into this goal's notes and experiment log.

Run order:

```text
pve_encounter_bonus_objectives
  -> pve_encounter_classification_floor_scaling_contract
      -> endless_repeat_encounter_weighting
      -> boss_omen_chain
```

This must finish before repeat encounter weighting because recent-Floor exclusion and response-complete weighting need a stable encounter classification and cannot build on `difficulty` windows.

## Current Policy To Implement

### Encounter Classification

Remove `PveEncounter.difficulty`.

Do not replace it with another encounter-authored numeric difficulty field.

Add an explicit non-numeric encounter classification field to `PveEncounter`:

```text
Normal
Elite
NormalBoss
FinalBoss
```

Meaning:

- `Normal`: ordinary combat with only corroded employees. `primary_abnormality_id = None` is expected and allowed.
- `Elite`: corroded employees plus an elite abnormality monster. `primary_abnormality_id` is required.
- `NormalBoss`: corroded employees plus a non-final boss abnormality monster. `primary_abnormality_id` is required.
- `FinalBoss`: corroded employees plus a run-ending final boss abnormality monster. `primary_abnormality_id` is required.

Encounter classification is not encounter difficulty. It answers "what kind of scenario is this?"

Current Floor answers "how strong is it?"

### Encounter Identity

Rename or replace the old ambiguous `abnormality_id` field with a clearer primary identity field if code reading confirms the schema can be changed cleanly.

Target meaning:

```text
primary_abnormality_id
```

- `None` for pure Normal corroded-employee encounters.
- `Some(id)` for Elite/NormalBoss/FinalBoss encounters.
- Used for run-local abnormality response research, bonus objective target identity, repeat encounter history, and boss/final boss identity.

Waves remain the source of truth for actual spawned units.

`primary_abnormality_id = None` is not a fallback placeholder. It means the encounter has no primary abnormality target, does not drive abnormality response research, and does not grant abnormality response completion or abnormality fragment rewards. This is valid for `Normal` only.

`Normal` encounters must not define abnormality suppression research or abnormality-targeted bonus objectives. If a normal corroded-employee battle needs extra rewards later, those rewards must be authored as ordinary encounter rewards, not abnormality response progress.

`Elite`, `NormalBoss`, and `FinalBoss` encounters must identify exactly one primary abnormality target for the current implementation. Multi-primary encounters are a future policy topic and must not be inferred from wave contents.

### Candidate Selection

Encounter candidate selection must not use difficulty ranges.

Candidate selection should use:

- map node category;
- map node kind;
- explicit encounter classification;
- `PveEncounter.node_type`;
- current `GameMode`;
- final Standard Floor policy;
- future repeat weighting and boss omen state.

Required selection behavior:

- normal combat nodes select `Normal` encounters.
- elite combat nodes select `Elite` encounters.
- non-final boss nodes select `NormalBoss` encounters.
- Standard final boss node selects `FinalBoss` encounters.
- non-boss nodes never select `NormalBoss` or `FinalBoss` encounters.
- `NormalBoss` selection must not accidentally select `FinalBoss` unless the mode/floor policy marks the node as the run-ending final boss.
- if a role-compatible pool is empty, fail loudly or stop for policy; do not silently select all encounters.

### Floor Scaling Stages

Floor scaling uses live RON policy, not hard-coded constants.

Initial live policy:

```text
Stage 1: Floor 1-2
Stage 2: Floor 3-4
Stage 3: Floor 5-6
Stage 4: Floor 7-8
Floor 9+: clamp to Stage 4
```

RON authoring uses human-facing Floor numbers starting at 1.

Runtime `floor_index` remains zero-based internally:

```text
runtime floor_index 0 -> authored Floor 1
runtime floor_index 1 -> authored Floor 2
```

### Stat Scaling

Floor scaling may adjust only:

- max health;
- attack;
- defense.

Do not scale in this goal:

- magic resistance;
- movement speed;
- attack interval;
- windup;
- resonance;
- skill cooldown;
- deployment cost;
- reward payout.

Balance values must remain live-RON configurable. The user will tune exact values later.

### Generated Wave Budget Scaling

Generated corroded waves can receive a budget multiplier from the active Floor scaling stage.

Rules:

- apply multiplier to generated wave budget;
- use ceiling after percent multiplication;
- enforce minimum budget 1;
- do not mutate the source RON object;
- the resolved `SpawnWave`/battle scenario should contain the scaled result for that battle only.

Manual waves do not receive quantity/budget scaling.

Manual wave units still receive stat scaling.

### Extra Waves

Floor scaling can insert explicitly authored extra waves.

Rules:

- no automatic repetition of all existing waves;
- no implicit cloning of boss phase waves or scripted objective waves;
- only waves listed in the active stage's `extra_waves` are inserted;
- inserted extra waves must use the same validation as normal `PveWaveData`;
- extra waves may use `GeneratedCorroded` or manual enemy entries, but normal live tuning should prefer generated corroded reinforcements.

### Scaling Scope

Floor scaling is applied when building the actual battle scenario/spawn waves.

It is not used for encounter candidate eligibility.

It is not used by Unity to calculate combat values.

If preview and actual battle need the same scaled wave/stat information, core must expose preview data from the same scaling resolver rather than duplicating scaling logic in Unity.

## In Scope

- Audit current `PveEncounter` schema, live RON, map encounter assignment, battle construction, and wave resolution.
- Remove `PveEncounter.difficulty` from schema, runtime code, tests, and live RON.
- Add explicit encounter classification to schema, runtime code, tests, and live RON.
- Clarify `primary_abnormality_id` semantics. Rename the old field if the blast radius is acceptable; otherwise document a short, user-approved transition before implementation.
- Rewrite encounter candidate selection to use classification, node category/kind, node type, game mode, and final Standard Floor policy.
- Remove silent all-pool fallback paths from encounter selection.
- Add run policy RON schema for Floor scaling stages.
- Implement active Floor scaling stage selection with clamp-to-last-stage overflow.
- Apply Floor stat scaling to enemy units during battle construction.
- Apply generated corroded wave budget scaling during generated wave resolution.
- Insert explicit stage extra waves into final battle wave definitions.
- Ensure combat preview and battle setup use the same scaled wave source if they both expose wave information.
- Update live RON.
- Update canonical docs if they describe the old `PveEncounter.difficulty`, encounter identity, or Floor scaling policy.
- Add focused tests and broad validation.

## Out Of Scope

- Exact long-term balance tuning beyond the first live-RON stage table.
- Endless repeat encounter weighting.
- Boss omen/provisional boss chains.
- Corroded wave preset redesign unless required to avoid incorrect runtime selection.
- Reward scaling.
- Magic resistance scaling.
- Equipment/skill scaling.
- Unity UI implementation.
- New encounter content beyond what is required to migrate existing live RON cleanly.

## Suggested Runtime Shape

The exact Rust names may change after code reading, but the domain shape should resemble:

```rust
pub enum PveEncounterClass {
    Normal,
    Elite,
    NormalBoss,
    FinalBoss,
}

pub struct PveEncounter {
    pub id: String,
    pub encounter_class: PveEncounterClass,
    pub primary_abnormality_id: Option<String>,
    pub node_type: Option<CombatNodeType>,
    pub mission_variant: Option<CombatMissionVariant>,
    pub survive_timer_ms: Option<u64>,
    pub reward_mode: RewardMode,
    pub reward_uuids: Vec<Uuid>,
    pub suppression_research: Option<PveSuppressionResearchData>,
    pub battlefield: Option<PveBattlefieldOverrideData>,
    pub tactical_plan: Option<PveTacticalPlanData>,
    pub win_condition: Option<PveWinConditionData>,
    pub waves: Vec<PveWaveData>,
    pub static_obstacles: Vec<PveStaticObstacleData>,
}
```

Run policy should express stage-based scaling, for example:

```ron
floor_combat_scaling: (
    stages: [
        (
            min_floor: 1,
            max_floor: 2,
            max_health_multiplier_percent: 100,
            attack_multiplier_percent: 100,
            defense_multiplier_percent: 100,
            generated_wave_budget_multiplier_percent: 100,
            extra_waves: [],
        ),
        ...
    ],
    overflow_policy: ClampToLastStage,
)
```

Exact field names may be changed during implementation if code reading shows a cleaner local pattern.

## Test Plan

Focused tests should cover user-visible contracts and live RON behavior rather than implementation details.

Required tests:

- live RON loading succeeds without `PveEncounter.difficulty`.
- every live encounter has explicit encounter classification.
- `Normal` encounters may omit `primary_abnormality_id`.
- `Elite`, `NormalBoss`, and `FinalBoss` encounters require `primary_abnormality_id`.
- `primary_abnormality_id`, when present, references a known abnormality.
- `Elite` encounters reference an Elite abnormality.
- `NormalBoss` and `FinalBoss` encounters reference Boss abnormalities.
- normal combat nodes select `Normal` encounters.
- elite nodes select `Elite` encounters.
- non-final boss nodes select `NormalBoss` encounters.
- Standard final Floor boss node selects `FinalBoss` encounters.
- non-boss nodes never select `NormalBoss` or `FinalBoss`.
- high Endless Floors do not empty encounter pools due to difficulty.
- selection is deterministic for fixed seed and run state.
- active Floor stage maps Floor 1-2, 3-4, 5-6, 7-8 correctly.
- Floor 9+ clamps to the last stage.
- stat scaling affects max health, attack, and defense only.
- generated corroded wave budget scaling uses ceiling and minimum 1.
- manual wave counts are not scaled.
- explicit extra waves are inserted only when the active stage lists them.
- combat preview and actual battle setup do not disagree on scaled wave identity/count if both expose wave data.

Suggested commands:

```bash
cargo test pve_encounter --lib
cargo test map_encounter --lib
cargo test floor_scaling --lib
cargo test generated_map --lib
cargo test --test ron_loading
cargo check --lib
cargo check -p game_server
```

At the end, run broader tests appropriate to the blast radius.

## Completion Criteria

- [x] `PveEncounter.difficulty` is fully removed from schema, runtime code, live RON, tests, and docs.
- [x] `PveEncounter` has explicit `Normal`/`Elite`/`NormalBoss`/`FinalBoss` classification.
- [x] Encounter identity clearly separates primary abnormality research target from actual spawned wave units.
- [x] Candidate selection no longer uses difficulty ranges or maximum difficulty.
- [x] Candidate selection respects normal/elite/normal-boss/final-boss role semantics.
- [x] Empty role-compatible pools fail loudly or are recorded as policy blockers; they do not silently fall back to all encounters.
- [x] Floor scaling stage policy is authored in live RON.
- [x] Floor 1-8 stage table and Floor 9+ clamp behavior are implemented.
- [x] Floor scaling applies max health, attack, and defense only.
- [x] Generated corroded budget scaling works with ceiling and minimum 1.
- [x] Manual waves do not scale count/budget.
- [x] Explicit extra waves are inserted by stage.
- [x] Combat preview and battle construction use the same scaled encounter/wave resolver where needed.
- [x] Live RON validation catches invalid encounter classification and invalid scaling stage data.
- [x] Focused tests and live RON loading pass.
- [x] Final broad validation commands are recorded in `EXPERIMENTS.md`.

## Stop Conditions

Stop and report questions instead of deciding silently if implementation discovers:

- Existing live encounters cannot be classified as `Normal`, `Elite`, `NormalBoss`, or `FinalBoss` without deleting/replacing content.
- Renaming `abnormality_id` to `primary_abnormality_id` requires a Unity-facing DTO or save migration decision.
- Current node kinds cannot reliably distinguish normal, elite, normal boss, and final boss selection.
- Combat preview and battle construction use separate wave paths that cannot be unified without a larger contract change.
- Floor stat scaling requires new balance values not covered by the live-RON stage policy.
- Extra waves require new authoring semantics beyond `PveWaveData`.
- Corroded wave preset `difficulty` is actually used as runtime selection and must be redesigned together with this goal.
- Unity-facing DTO shape changes are required.
- Save data migration policy becomes necessary.
