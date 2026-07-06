# Boss Omen Phase 5 Completion Hardening Notes

## Starting Context

This goal follows the Phase 5 `boss_omen_chain` implementation and its goal-completion review.

The review found that the core skeleton is present, but several contract-level gaps remain:

- Boss Omen trigger relies on awakened skill fragments, while live data currently has no awakenable active fragments.
- Event choice execution can apply `Grant` before a later effect fails.
- Event-started combat behavior is structurally present but insufficiently tested.
- Phase 5 policy remains in a draft document instead of canonical docs.

## User-Decided Policy

- Do not change skipped-chain reroll behavior in this goal.
- WhiteNight is currently the only authored final-boss chain, so resetting the candidate pool and possibly seeing WhiteNight again is acceptable for now.
- The Boss Omen trigger problem, Event atomicity, Event-started combat tests, and canonical documentation should be fixed as a goal.

## Initial Technical Observations

- `apply_boss_omen_to_map` requires `has_awakened_skill_fragment`.
- `has_awakened_skill_fragment` checks `SkillFragmentProgress.awakened`.
- Skill fragment awakening is only meaningful for active skill fragments with `awakened_skill_id: Some(_)`.
- Live `skill_fragments/base.ron` currently has no `awakened_skill_id: Some(_)`.
- `apply_event_choice_effects` applies `Grant` immediately and only later validates `StartCombat` effects in the same loop.
- Basic Event scene progression is tested, but Event-started combat and retreat/re-entry behavior are not sufficiently fixed by tests.
- `docs/goals/boss_omen_chain/PLAN.md` instructs implementers to absorb the draft policy into canonical docs after implementation.

## Implementation Decisions

- Boss Omen eligibility remains awakened-fragment based. The long-term fix is live data authoring (`awakened_skill_id: Some(...)`) plus tests, not a hidden code fallback.
- The first live awakenable fragments use existing live skills. This avoids inventing new final-boss content while making the trigger path real.
- Event choice atomicity is implemented through staged clones of mutable reward/run resources. Only after all effects validate does the code commit inventory, skill fragments, roster, uuid manager, and enkephalin back to core state.
- Event-started combat keeps the node category as Event. The Event choice explicitly starts combat by encounter id, and the stored explicit combat preview is reused for retry/re-entry.
- `CompleteCombatResult` remains reward-first on victory. Tests should assert `CombatRewardsGranted { completion: ... }` rather than expecting direct node completion variants.
- BossOmen source step consumption belongs to node completion, not node entry. Terminal source completion must consume the step before next-floor generation so completed chains create forced boss nodes instead of trying to overlay a missing next step.
- Forced BossOmen boss handling must run before normal BossOmen overlay placement during next-floor map generation. Otherwise a completed chain can look like an active chain with no remaining step.
- The skipped-chain reroll edge case remains unchanged per user policy while WhiteNight is the only authored final-boss chain.
- Stable policy now lives in `game_rulebook.md` and `skills/skill_fragment_system.md`; the old draft is reference-only.

## Long-Term Direction

- Prefer data-authored trigger eligibility over code-side special cases.
- Prefer explicit preflight/staged execution over ad hoc rollback.
- Prefer behavior/DTO/live-RON tests over implementation-shape tests.
- Keep Boss Omen chain policy separate from normal encounter weighting and map generation policy unless code reading proves they must share a boundary.

## Follow-Up Candidates Outside This Goal

- Add additional final-boss chains beyond WhiteNight.
- Add Silent Orchestra source steps after its symphony encounters/events are authored.
- Add full Unity visual-novel presentation catalogs.
- Add account-wide omen history or long-term codex integration.
- Revisit skipped-chain reroll policy once at least two final-boss chains exist in live data.

## Policy Questions To Raise If Found

- Should Boss Omen eligibility be based on awakened fragments, bloom availability, owned unique boss fragments, or another progression milestone?
- Should event choice effects support non-reward roster condition changes in this goal, or only validate the current effect set?
- Should `ApplyBossOmenStepResult` remain an explicit effect marker if step consumption actually happens on node completion?
- Should forced Boss Omen boss victory grant special guaranteed rewards distinct from normal final boss rewards?
