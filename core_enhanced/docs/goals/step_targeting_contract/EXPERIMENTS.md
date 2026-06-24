# Step Targeting Contract Experiments

## 2026-06-13

### Read runtime and live data

Command:

```bash
rg -n "RetargetOnStep|StepTargetingMode|targeting_mode|CastTarget|StepTarget|defense_tile_range|targeting" src/game tests docs/skill_target_contract.md
```

Result:

- `StepTargetingMode` is declared in `src/game/ability.rs`.
- Runtime step context resolution lives in `src/game/battle/core/sim.rs`.
- Validation for tile-area/range contracts lives in `src/game/data/skill_data.rs` and related world setup checks.
- Existing tests already cover RetargetOnStep behavior in `tests/skill_refactor_validation.rs`.

### Check live RetargetOnStep usage

Command:

```bash
rg -n "targeting: RetargetOnStep" ../game_resources/data/skills/base.ron
```

Result:

- Live skills use `RetargetOnStep` at lines 246, 330, 420, 596, 821, 957, and 1748.
- Observed examples are follow-up or conditional attack steps that should select a fresh valid unit at step execution time.

### Decision

No runtime code change is needed for this goal. The code/data contract is already implemented; the missing piece is contract documentation.

### Verify documentation synchronization

Command:

```bash
rg -n "StepTargetingMode|RetargetOnStep|ReuseCastTarget|cast_targeting|defense_tile_range|TileArea" docs/skill_target_contract.md src/game/ability.rs src/game/battle/core/sim.rs src/game/data/skill_data.rs
```

Result:

- `docs/skill_target_contract.md` now explicitly documents both `ReuseCastTarget` and `RetargetOnStep`.
- The document names the same runtime fields used by `resolve_skill_step_context`.
- The document keeps `TileArea` as a hit/filter policy over `defense_tile_range`, not a separate geometric shape.

Command:

```bash
rg -n "targeting: RetargetOnStep" ../game_resources/data/skills/base.ron
```

Result:

- Live RON still has 7 `RetargetOnStep` usages.
- No live RON or Rust code change was required.
