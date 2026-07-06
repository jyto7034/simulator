# Boss Omen Phase 5 Completion Hardening Plan

## Objective

Finish the remaining Phase 5 Boss Omen Chain hardening work found during goal-completion review.

This goal does not redesign Boss Omen. It closes the verified gaps in the current implementation:

- Live data must provide a natural way for Boss Omen to become eligible.
- Event choice resolution must be atomic: no partial reward/effect application when a later effect fails.
- Event-started combat behavior must be fixed by user-visible contract tests.
- Stable Boss Omen/Event policies must be absorbed into canonical documentation.

The skipped-boss reroll issue is intentionally out of scope for code changes in this goal. Current live data only has WhiteNight as a final-boss chain, and the accepted policy is to leave the existing pool-reset behavior until more final-boss chains exist.

## Source Of Truth Order

Verify facts in this order during implementation:

1. Runtime code for skill fragment progress, Boss Omen triggering, Event choice execution, Event-started combat, battle result handling, and node completion.
2. Live RON/data for skill fragments, Boss Omen chains, Event definitions, PVE encounters, rewards, and abnormalities.
3. Unity-facing snapshot/command DTO contracts and live JSON/probe output.
4. Current canonical docs and the active Boss Omen policy draft.

Do not trust docs over runtime behavior. If code reading shows a better long-term model, record the evidence in `EXPERIMENT_NOTES.md` and apply it unless it requires a user policy decision.

## Current Confirmed Findings

### 1. Boss Omen live trigger is weak

`BossOmen` currently requires at least one `SkillFragmentProgress.awakened == true`.

Skill fragment awakening is only valid for active skill fragments with `awakened_skill_id: Some(...)`.

Current live `skill_fragments/base.ron` has no `awakened_skill_id: Some(...)`, so normal live play has no clear path to satisfy the Boss Omen start condition.

### 2. Event choice effects are not atomic

`apply_event_choice_effects` applies `Grant` immediately while iterating effects.

A later `StartCombat` effect can still panic/fail on invalid data or duplicate combat effects. That can leave the run with already-applied rewards but no valid completed choice resolution.

Policy requires partial application to be impossible.

### 3. Event-started combat policy is under-tested

Basic Event scene progression is tested. Boss Omen overlay/forced boss node behavior is tested.

Missing contract tests include:

- Event choice starts combat.
- Event choice commitment survives retreat/re-entry.
- Victory/defeat/exhausted attempts consume the Event node according to normal combat attempt rules.
- BossOmen step is consumed only when the source node becomes `Completed`.
- Forced Boss Omen boss victory/defeat follows Endless final-boss policy.
- Run checkpoint preserves Boss Omen and Event session state where those states can matter.

### 4. Phase 5 policy is still draft-only

`docs/boss_omen_chain_policy_draft.md` remains the active policy draft.

Stable policies must be moved into canonical docs and the draft must be deleted or marked superseded.

## Policy To Preserve

- Boss omen chains are Endless-only and run-local.
- At most one boss omen chain is active at once.
- A source becomes eligible when:
  - mode is Endless;
  - current Floor is at least the live-RON minimum Floor;
  - the player owns at least one bloomed/awakened skill fragment according to the final trigger policy;
  - no chain is active.
- Omen chain definitions are data-authored.
- Event source steps overlay Event nodes. Combat source steps overlay normal Combat nodes.
- Event choices commit immediately and cannot be changed on re-entry.
- Event-started combat uses normal combat attempt rules.
- BossOmen steps are consumed when their source node becomes `Completed`.
- Completed chains force a separate Boss node.
- Forced Boss Omen boss defeat is run failure; SavePoint rollback is a UI/user choice when available.
- Data invariant errors are panic/fail-fast QA errors, not silent fallback.

## In Scope

- Audit the current skill fragment awakening and Boss Omen start-condition code.
- Choose a long-term live-data fix for the trigger gap, favoring data-authored awakenable fragments over hidden code fallbacks.
- Add or adjust live RON so at least one valid live fragment can become awakened through normal systems.
- Add validation/tests that prevent the live trigger path from silently disappearing again.
- Refactor Event choice execution to preflight or stage all effects before applying mutations.
- Keep `Grant`, `StartCombat`, and `ApplyBossOmenStepResult` semantics explicit instead of hiding them in compatibility paths.
- Add focused tests for Event-started combat, retreat/re-entry, node consumption, BossOmen step consumption, forced boss result behavior, and checkpoint persistence.
- Absorb stable rules from `docs/boss_omen_chain_policy_draft.md` into canonical docs.
- Update `docs/README.md` if document roles change.
- Run focused tests after each meaningful change and broad tests at the end.

## Out Of Scope

- Adding new final-boss content beyond minimal fixture/live-data work needed to make the trigger path real.
- Implementing Silent Orchestra or other additional Boss Omen chains.
- Changing the accepted skipped-chain pool-reset behavior while WhiteNight is the only live chain.
- Unity implementation.
- Full visual-novel presentation catalogs.
- Account-wide omen history.
- Reworking the entire reward system.
- Reworking map generation outside what the tests require.

## Implementation Plan

1. Re-audit runtime code and live RON.
   - Read skill fragment awakening, Boss Omen trigger, Event choice execution, Event-started combat, node completion, checkpoint save/load, and final boss result handling.
   - Record mismatches in `EXPERIMENT_NOTES.md`.

2. Fix Boss Omen live trigger path.
   - Prefer adding a live awakenable placeholder/temporary active skill fragment over changing the trigger into a hidden fallback.
   - Ensure validation catches active fragments that can be marked awakened but cannot resolve an awakened skill.
   - Add a test that proves live data contains at least one path to an awakened fragment that can satisfy Boss Omen eligibility.

3. Make Event choice resolution atomic.
   - Preflight all effects before mutating run state.
   - Reject/panic data-invariant failures before applying grants or committing choice state.
   - Apply mutations only after all effects are known valid.
   - Add a regression test where a choice with a valid grant and invalid combat effect cannot partially mutate state.

4. Strengthen Event-started combat tests.
   - Test StartCombat choice starts the live battle pipeline.
   - Test retreat preserves committed choice and pending combat state.
   - Test victory, defeat, and exhausted retreat attempts consume the Event node according to normal combat rules.
   - Test BossOmen step consumption happens when the source node becomes `Completed`.
   - Test forced boss victory/defeat behavior for Endless.
   - Test run checkpoint save/load preserves relevant `boss_omen` and `event_sessions` state.

5. Canonicalize docs.
   - Move stable policy from `docs/boss_omen_chain_policy_draft.md` into canonical docs.
   - Mark the draft superseded or remove it only after the stable policy has a canonical home.
   - Update document index/README if needed.

6. Final verification.
   - Run focused tests introduced or touched by this goal.
   - Run `cargo check -p game_server`.
   - Run broad core tests.
   - Record all commands and outcomes in `EXPERIMENTS.md`.

## Completion Criteria

- Boss Omen can become eligible from live data through a documented, tested, non-fallback path.
- Event choice effect execution is atomic.
- Event-started combat behavior is covered by user-visible contract tests.
- BossOmen step consumption and forced boss result behavior are covered by tests.
- Run checkpoint persistence covers Boss Omen/Event session state where relevant.
- Stable Phase 5 policy is canonicalized outside the draft document.
- No compatibility layer or dual schema is introduced.
- Focused tests and final broad checks pass.

## Stop Conditions

Complete/stop the goal and report questions instead of deciding silently if implementation discovers:

- The Boss Omen trigger should no longer be awakened-fragment based.
- The temporary awakenable fragment would require real boss/skill content decisions beyond placeholder data.
- Event-started combat needs a Unity DTO/UX shape not captured by existing contracts.
- Atomic Event execution requires broad reward/inventory transaction machinery beyond this goal.
- Forced Boss Omen defeat/SavePoint rollback semantics conflict with existing final-boss or checkpoint policy.
- Canonical docs disagree on a user-visible policy.

