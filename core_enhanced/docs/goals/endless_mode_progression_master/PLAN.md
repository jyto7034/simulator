# Endless Mode Progression Master Plan

## Objective

Coordinate the long-term Endless mode progression work across the dependent subgoals in the correct order.

This master goal does not directly implement runtime behavior. It defines the dependency order, review gates, completion criteria, and stop conditions for the subgoals that convert the game from an Act-capped run model into a Floor-based Standard/Endless progression model with abnormality response research.

## Source Of Truth Order

For every subgoal, verify facts in this order:

1. Runtime code.
2. Live RON/data.
3. Unity-facing snapshot/command/result contracts and live WebSocket JSON.
4. Latest policy docs and this master plan.

Do not trust this master plan over runtime behavior. If a subgoal discovers a better long-term structure, record the evidence in that subgoal's `EXPERIMENT_NOTES.md` and update this master plan only after the subgoal policy is stable.

## Temporary Policy Drafts

- Phase 5 boss omen/event-node policy is currently gathered in `docs/boss_omen_chain_policy_draft.md`.
- Keep that draft during Phase 5 implementation. After implementation and validation, absorb stable rules into canonical docs and delete or mark the draft superseded.

## Subgoal Order

Run the subgoals in this order:

1. `docs/goals/run_floor_progression_contract/`
2. `docs/goals/endless_abnormality_research_state/`
3. `docs/goals/pve_encounter_bonus_objectives/`
4A. `docs/goals/pve_encounter_classification_floor_scaling_contract/`
4B. `docs/goals/endless_repeat_encounter_weighting/`
5. `docs/goals/boss_omen_chain/`

Dependency graph:

```text
run_floor_progression_contract
  -> endless_abnormality_research_state
      -> pve_encounter_bonus_objectives
      -> pve_encounter_classification_floor_scaling_contract
          -> endless_repeat_encounter_weighting
              -> boss_omen_chain
```

`boss_omen_chain` may also depend on `pve_encounter_bonus_objectives` if omen bosses use objective-driven research rewards.

## Master Policy Summary

- `Standard` is a fixed 3-Floor mode.
- `Endless` is a max-free Floor progression mode.
- Gate advances to the next Floor and is independent from boss/elite nodes.
- Standard final boss is run-ending.
- Endless boss/elite encounters are not automatically run-ending.
- Endless completion is based on suppression-target abnormality response completion.
- Response research is run-local.
- PveEncounter bonus objectives can add research on victory.
- Repeat-complete abnormality suppression grants fragment dust.
- `PveEncounter.difficulty` is removed.
- `PveEncounter` uses explicit non-numeric `Normal`/`Elite`/`NormalBoss`/`FinalBoss` classification.
- `Normal` has no primary abnormality. `Elite`, `NormalBoss`, and `FinalBoss` require `primary_abnormality_id`.
- Current Floor drives encounter intensity through explicit scaling policy, not candidate eligibility.
- Repeat encounter candidate weighting uses recent Floor history and response-complete state.
- Boss omen chains are a later expansion after the base systems are stable.
- All balance values must be live-RON configurable.

## Phase 1: Run/Floor Progression

Subgoal: `run_floor_progression_contract`

Status: complete on 2026-06-26.

Must finish before other subgoals that use Floor index or mode-specific progression.

Completion gate:

- Runtime progression is Floor-centric and mode-aware.
- `Standard` has fixed 3-Floor behavior.
- `Endless` can advance by Gate without a max cap.
- Gate transition applies trauma recovery and creates the next Floor map.
- Standard final boss win/loss semantics remain correct.
- Unity-facing `run_progression` snapshot is Floor-centric.
- Act-centric DTO/result fields are removed or have a user-approved short transition plan.
- Live RON loading and focused gameplay flow tests pass.

Do not start Phase 2 until this gate is satisfied or the user explicitly approves proceeding with a known limitation.

## Phase 2: Run-Local Abnormality Research State

Subgoal: `endless_abnormality_research_state`

Status: complete on 2026-06-26.

Must finish before bonus objective research grants or repeat encounter weighting.

Completion gate:

- `RunState` owns a run-local abnormality response/research state.
- Suppression target inclusion/exclusion policy is implemented.
- Base victory research gain works from live RON values.
- Response completion caps progress and records completion metadata.
- First response completion grants the data-authored unique skill fragment.
- Repeat response-complete suppression grants fragment dust instead of duplicate unique fragments.
- State persists through the existing run checkpoint/save boundary.
- Focused reward/state/snapshot tests pass.

Do not start Phase 3 or Phase 4 until this gate is satisfied.

## Phase 3: PveEncounter Bonus Objectives

Subgoal: `pve_encounter_bonus_objectives`

Can run after Phase 2. It should not change repeat encounter selection.

Status: complete on 2026-06-27.

Completion gate:

- Live RON can author `ClearWithin` and `DecisiveDamage`.
- Invalid bonus objective data fails validation.
- `ClearWithin` uses battle start as origin.
- `DecisiveDamage` counts one non-DoT effective damage source against the primary abnormality target.
- Defeat, retreat, and node failure grant no bonus objective research.
- Victory grants base research plus satisfied bonus objective research.
- Result/log DTOs expose enough information for presentation.
- Focused combat/result/live RON tests pass.

## Phase 4A: PVE Encounter Classification And Floor Scaling Contract

Subgoal: `pve_encounter_classification_floor_scaling_contract`

This is the prerequisite repair discovered during the original Phase 4 startup audit.

Status: complete on 2026-06-27.

Must finish before repeat encounter weighting because the old difficulty range filter can empty Endless candidate pools before recent-Floor exclusion runs. This phase prepares the encounter data model so repeat weighting can operate on explicit encounter identity and response state instead of bounded authored difficulty windows.

Completion gate:

- `PveEncounter.difficulty` is removed from runtime schema and live RON.
- `PveEncounter` has an explicit normal/elite/normal-boss/final-boss classification in runtime schema and live RON.
- Encounter candidate selection no longer uses difficulty ranges.
- Normal/elite/normal-boss/final-boss selection uses explicit role semantics, not maximum difficulty.
- Floor scaling stages are authored in live RON.
- Floor scaling applies max health, attack, defense, generated wave budget scaling, and explicitly authored extra waves according to policy.
- High Endless Floor map assignment does not fail because authored encounter difficulty ran out.
- Focused map encounter and live RON tests pass.

## Phase 4B: Endless Repeat Encounter Weighting

Subgoal: `endless_repeat_encounter_weighting`

Can run after Phase 4A. Requires Phase 2 research state and should run after Phase 3 if bonus objectives affect encounter evaluation or completion pacing.

Status: complete on 2026-06-27.

Completion gate:

- Endless normal encounter selection uses recent Floor exclusion.
- Recent Floor exclusion window comes from live RON.
- Response-complete candidate weight multiplier comes from live RON.
- Candidate selection is deterministic for fixed seed and run state.
- Standard mode encounter selection remains policy-compatible.
- Empty candidate fallback policy is explicit and tested.
- Focused map encounter and live RON tests pass.

## Phase 5: Boss Omen Chains

Subgoal: `boss_omen_chain`

This is intentionally last.

Completion gate:

- Omen chains are data-authored.
- Chain state is run-local and deterministic.
- Chain-driven encounters can appear in Endless without automatically ending the run.
- Omen chains have explicit bypass rules for normal repeat weighting.
- Trigger policy, chain length, forced-node behavior, and rewards are user-approved.
- Tests cover chain selection, persistence, result behavior, and validation.

Do not begin implementation if the policy questions in `docs/goals/boss_omen_chain/EXPERIMENT_NOTES.md` remain unanswered.

## Cross-Goal Rules

- Do not implement compatibility layers, fallback paths, or dual schemas unless the subgoal documents the reason and the user approves it.
- Do not keep old Act terminology as public contract after Phase 1 unless it has a short removal condition.
- Do not change Unity-facing DTO shape in multiple subgoals at once without updating docs/probes.
- Do not silently move balance values into hard-coded constants.
- Do not preserve legacy tests by marking them ignored. Replace them with latest policy tests.
- If a subgoal finds a better long-term design, update that subgoal first, then update this master plan.
- If a subgoal needs a user decision, complete/stop that subgoal with questions rather than guessing.

## Master Completion Criteria

The master goal is complete when:

- All active phases are complete or explicitly deferred by user decision.
- `Standard` and `Endless` mode progression are cleanly separated.
- Endless can progress through Floors by Gate.
- Run-local abnormality response completion exists.
- Bonus objectives can grant response research.
- Repeat encounter selection respects recent Floor and completion state.
- Boss omen chain policy is either implemented or explicitly deferred with a stable follow-up plan.
- Canonical docs and probes match runtime JSON.
- Focused tests for each phase and final broad checks are recorded.

## Master Stop Conditions

Stop and report instead of proceeding if:

- Unity requires a transition schema not captured in the active subgoal.
- Save data migration policy becomes necessary.
- Live RON content lacks required data for a phase and adding defaults would hide invalid content.
- Encounter difficulty or reward pacing needs a new balance decision.
- A subgoal would require deleting or replacing user-authored content outside its scope.
- Two subgoals would need to modify the same public DTO in conflicting ways.
