# Step Targeting Contract Plan

## Objective

Document the live `StepTargetingMode::{ReuseCastTarget, RetargetOnStep}` contract in `docs/skill_target_contract.md` without changing runtime behavior, live RON schema, or Unity-facing DTO shape.

## Scope

In scope:

- Read runtime targeting code, validation, live RON, and existing contract docs.
- Clarify the responsibility split between `cast_targeting`, `target`, `targeting`, `defense_tile_range`, and `DeliveryDef::TileArea`.
- Record live `RetargetOnStep` usage evidence.

Out of scope:

- Runtime targeting code changes.
- Live skill data rewrites.
- New targeting modes.
- Unity-facing DTO changes.

## Source Of Truth

1. Runtime code:
   - `src/game/ability.rs`
   - `src/game/data/skill_data.rs`
   - `src/game/battle/core/sim.rs`
   - `src/game/battle/core/targeting.rs`
2. Live data:
   - `../game_resources/data/skills/base.ron`
3. Contract docs:
   - `docs/skill_target_contract.md`
4. Goal/design docs:
   - `docs/step_targeting_contract_goal.md`
   - `docs/component_refactor_master_goal.md`

## Plan

1. Confirm runtime meaning of `ReuseCastTarget` and `RetargetOnStep`.
2. Confirm live RON examples using `RetargetOnStep`.
3. Update `docs/skill_target_contract.md` with a step targeting section.
4. Run focused text checks to verify code/data/doc synchronization.

## Completion Conditions

- `docs/skill_target_contract.md` explicitly documents `StepTargetingMode`.
- The document separates `cast_targeting`, `target`, `targeting`, `defense_tile_range`, and `TileArea` responsibilities.
- Live RON `RetargetOnStep` usage is recorded and matches the documented contract.
- No Unity-facing DTO, live RON schema, or runtime behavior change is introduced.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- Runtime and live RON disagree on `StepTargetingMode` meaning.
- A new targeting policy or targeting mode is required.
- Unity-facing contract changes are required.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Verification Commands

- `rg -n "StepTargetingMode|RetargetOnStep|ReuseCastTarget|cast_targeting|defense_tile_range|TileArea" docs/skill_target_contract.md src/game/ability.rs src/game/battle/core/sim.rs src/game/data/skill_data.rs`
- `rg -n "targeting: RetargetOnStep" ../game_resources/data/skills/base.ron`
