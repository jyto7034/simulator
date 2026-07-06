# Endless Mode Progression Master Notes

## Initial Notes

- The original policy discussion had about eight items. They were grouped into five implementation goals by shared code surface and dependency order.
- The largest shared surface is `RunProgression`, Gate transition, map terminal selection, and snapshot DTO, so it is one first-phase goal.
- Research state, bonus objectives, repeat weighting, and boss omen chains are separated because they touch different runtime domains and have different policy maturity.

## Dependency Rationale

`run_floor_progression_contract` comes first because every later Endless policy needs a reliable `floor_index` and mode-specific progression semantics.

`endless_abnormality_research_state` comes second because both bonus objectives and repeat weighting need response-complete state.

`pve_encounter_bonus_objectives` comes before or alongside repeat weighting only after research state exists, because objective rewards mutate research progress.

`pve_encounter_classification_floor_scaling_contract` is Phase 4A, the prerequisite repair discovered during the original repeat-weighting startup audit. It comes before repeat weighting because `PveEncounter.difficulty` currently controls map encounter candidate eligibility and can empty Endless candidate pools before recent-Floor exclusion runs. Removing it and adding explicit `Normal`/`Elite`/`NormalBoss`/`FinalBoss` classification lets Phase 4B operate on encounter identity, current Floor scaling, recent Floor history, and response completion state instead of a bounded authored difficulty range.

`endless_repeat_encounter_weighting` needs research state and Floor history.

`boss_omen_chain` is last because it may bypass normal weighting, use research state, and require additional user policy decisions.

## Shared Risk Areas

- `run_progression` snapshot DTO.
- `MapViewDto`.
- `map_encounters.rs`.
- Combat result/reward resolution.
- Live RON schema and validation.
- Unity probe/docs sync.

When a subgoal touches one of these, update its `EXPERIMENT_NOTES.md` and check whether this master plan needs a dependency note.

## Deferred Policy Questions

These should not block Phase 1:

- Exact boss omen trigger policy.
- Omen chain length.
- Forced omen node placement.
- Multiple simultaneous omen chains.
- Account-wide abnormality encyclopedia research.

These may block later phases if encountered:

- Whether Standard mode should also display response research state.
- Exact Unity DTO shape for response research progress.
- Empty candidate fallback when repeat exclusion removes all encounters.
- Multi-primary-target bonus objective policy.

## Master Follow-Up Candidates

- Merge `docs/endless_mode_research_policy_draft.md` into canonical docs after the implementation sequence stabilizes.
- Update `docs/README.md` after new canonical docs are created or merged.
- Add a final contract audit goal after the five phases if DTO changes are broad.
