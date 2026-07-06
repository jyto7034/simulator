# Boss Omen Chain Plan

## Objective

Design and implement the Endless boss omen chain system after the base Endless progression, abnormality research state, bonus objectives, and repeat encounter weighting are stable.

This goal turns Endless boss progression into a long-term chain system without making every boss a run-ending event. It is intentionally last in the Endless sequence because it depends on the preceding systems.

Use `docs/boss_omen_chain_policy_draft.md` as the active Phase 5 policy draft while implementing. After implementation and verification, absorb stable rules into canonical docs and delete or mark the draft superseded.

## Source Of Truth Order

Verify facts in this order during implementation:

1. Runtime Floor progression, map generation, encounter selection, battle result, and research state code.
2. Live RON/data for abnormalities, encounters, map generation, and rewards.
3. Unity-facing snapshot/result contracts and live WebSocket JSON.
4. Latest policy docs.

Do not trust this plan over runtime behavior. If code reading shows a better long-term model, record the evidence in `EXPERIMENT_NOTES.md` and apply it unless it requires a user policy decision.

## Dependencies

- Depends on `docs/goals/run_floor_progression_contract/`.
- Depends on `docs/goals/endless_abnormality_research_state/`.
- Depends on `docs/goals/pve_encounter_bonus_objectives/` if omen bosses use objective-driven research rewards.
- Depends on `docs/goals/endless_repeat_encounter_weighting/` if omen chains interact with normal candidate weighting.

## Current Policy To Implement

- Boss omen chains are Endless-only and run-local.
- At most one boss omen chain can be active at once.
- A boss omen source becomes eligible when the run is Endless, the current Floor is at least the live-RON minimum Floor, the player owns at least one bloomed skill fragment, and no chain is active.
- Candidate abnormalities opt into chains with `omen_chain_id`, which points to data-authored `BossOmenChain` definitions.
- Omen steps have explicit `source_kind`; first implementation supports `Event` and `Combat`.
- `Event` source steps overlay Event nodes. `Combat` source steps overlay normal Combat nodes.
- Gate, Boss, FinalBoss, Start, Maintenance, Support, Shop, Reward, and HeadquartersContact are not default overlay targets.
- Event nodes are independent node category/content and use scene graph definitions.
- Core sends event resource ids and state; Unity renders presentation from local catalogs.
- Unity advances non-choice Event scenes via `advance_event_scene`; choices use `select_event_choice`.
- Event choices are committed when selected and cannot be changed on re-entry.
- Event-started combat and Combat source omen use normal combat attempt rules: first attempt plus two retreats; victory, defeat, or exhausted attempts complete the node.
- Omen steps are consumed when their source node becomes `Completed`.
- If no eligible overlay node exists for the required source kind, keep the provisional boss/step and retry placement on the next Floor.
- If the first placed omen source is ignored and the player enters Gate, reroll another boss next Floor and push the provisional boss back into the pool.
- Completed omen chains force a separate boss node in front of the current position; other nodes are not selectable, but Safezone/loadout maintenance is possible.
- Boss node rewards, research, fragments, and dust are data-authored in RON. Core must not hard-code per-boss rewards.
- Boss node defeat is run failure with Unity options for main menu or SavePoint rollback when available.
- RON reference errors, event graph invariant errors, catalog/id mismatch, and similar production-data invariant violations are panic/fail-fast, not gameplay rejection.
- Boss omen chains may explicitly bypass normal non-omen repeat weighting.
- Balance and trigger values must be live-RON configurable.

## In Scope

- Re-audit current boss/elite encounter selection, map generation, final boss semantics, Event node absence/presence, battle result, reward, SavePoint rollback, and response research state.
- Define data-authored boss omen chain and Event definition models.
- Add Event node category/content and EventSessionState if runtime code confirms no equivalent exists.
- Add commands and DTOs for Event scene advancement and choice selection.
- Add run-local state for provisional/active omen chain, pending step, placed overlay, forced boss node, and Event session commitment.
- Add deterministic map-generation/encounter hooks for Event/Combat source overlays and forced boss node selection.
- Ensure omen chain candidate/bypass logic stays separate from normal repeat weighting.
- Ensure chain bosses do not accidentally reuse Standard final boss run-ending logic except where the policy says boss node defeat is run failure.
- Add tests for data validation, deterministic chain selection, overlay placement/deferral, Event command flow, attempt/result behavior, persistence, and forced boss node behavior.
- Update docs/probes after runtime shape is final.

## Out Of Scope

- First implementation of Endless Floor progression.
- Base abnormality research state.
- Base bonus objective evaluation.
- Base repeat encounter weighting.
- Unity implementation.
- New boss content beyond minimal fixtures needed for tests.
- Making every boss a Gate or run-ending event.
- Full Unity visual novel presentation catalogs.
- Account-wide omen history.
- Cyclic or condition-heavy Event scene graphs beyond the first acyclic graph implementation.

## Initial Design Constraints

- Omen chain state should be run-local.
- Omen chains should not be inferred from node names or Unity presentation.
- Chain triggers should be explicit and data-authored.
- Chain progress should persist through run checkpoint/save if run state is persisted.
- Omen chain rewards should use existing reward/research systems where possible.
- Event definitions must validate acyclic scene graphs, reachable scenes, valid scene references, and at least one valid terminal/resolution path.
- Event resolution must be atomic. Partial effect application is not allowed.
- Similar production-data invariant failures must panic/fail-fast during QA rather than degrade into fallback behavior.

## Implementation Sketch

1. Re-audit all prerequisite systems after they are complete.
2. Align this goal's notes with `docs/boss_omen_chain_policy_draft.md`.
3. Draft exact Rust/RON data model for Event definitions and BossOmenChain definitions.
4. Add runtime Event node/session support and command handling.
5. Add runtime omen chain state.
6. Add deterministic selection/injection logic for Event/Combat overlays and forced boss nodes.
7. Add result handling, attempt handling, persistence, and SavePoint rollback integration.
8. Add focused tests and live RON validation.
9. Update docs/probes.

## Completion Criteria

- Boss omen chains are data-authored.
- Event definitions are data-authored and validated.
- Chain state is run-local and deterministic.
- Event scene advancement and choice commitment are command-driven and persisted.
- Chain-driven encounters can appear in Endless without ending the run automatically.
- Normal repeat weighting and special chain bypass rules are clearly separated.
- Missing overlay candidates defer to the next Floor instead of falling back silently.
- Completed chains force a separate boss node and restrict selection to that boss node.
- Tests cover chain selection, persistence, Event command flow, overlay placement/deferral, result behavior, forced boss behavior, and validation.

## Stop Conditions

Stop and report questions instead of deciding silently if implementation discovers:

- The active policy draft lacks a rule for a user-visible behavior.
- A chain would require a Unity DTO/UX shape not captured by the draft.
- Existing encounter data cannot express chain candidates cleanly.
- Chain implementation would require broad map generator redesign beyond this goal.
