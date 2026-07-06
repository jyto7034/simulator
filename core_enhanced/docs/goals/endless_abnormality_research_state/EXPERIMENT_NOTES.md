# Endless Abnormality Research State Notes

## Initial Notes

- This goal should start only after the Floor-centric run progression goal is complete or stable enough to provide reliable `floor_index`.
- Existing skill fragment inventory already has fragment dust support.
- Existing reward execution already supports skill fragment and fragment dust effects; verify before adding new paths.
- Current abnormality metadata includes threat class. Validate whether it can distinguish Elite/NormalBoss/FinalBoss for reward amounts.
- Current PvE encounter data has `abnormality_id`; verify whether every suppression encounter references a valid abnormality.
- Existing run checkpoints store `RunProgression`, roster, inventory, skill fragments, and abnormality attempts. Add research state to the same persistence boundary.

## Policy Clarifications Already Resolved

- Response progress is run-local.
- Account-wide encyclopedia research is future work.
- Abnormality parts are not inventory items.
- Repeat response-complete suppression gives fragment dust, not another unique skill fragment.
- Balance values must be live-RON configurable.

## Out-Of-Scope Follow-Up Candidates

- Account-wide abnormality encyclopedia.
- Tool abnormality research events.
- Abnormality part result presentation.
- Boss omen chains.
- Repeat encounter weighting.

## Questions To Escalate If Found

- Where should first-completion unique fragment ids live if abnormality metadata and PvE encounter both have plausible ownership?
- Should Standard mode also use the same response research state, or only Endless?
- Should response progress appear in every snapshot, only map/result snapshots, or only a future encyclopedia screen?

## 2026-06-26 Startup Audit Resolution

Phase 2 initially could not safely proceed to implementation.

The runtime surfaces are otherwise compatible with the plan:

- `RunState` is the correct owner for run-local abnormality research state.
- `RunCheckpointPayload` is the persistence boundary that must carry the new state.
- Combat result completion already stages inventory, skill fragments, roster, UUID manager, and enkephalin together.
- `RewardEffect::GrantSkillFragment` and `RewardEffect::GrantFragmentDust` already exist and can be routed through `GrantExecutor`.
- `AbnormalityMetadata.threat_class` can distinguish suppression target threat classes at runtime.
- `PveEncounter.abnormality_id` provides the encounter-to-abnormality link.

However, the subgoal requires:

- "First response completion grants the abnormality's data-authored unique skill fragment."
- "Missing or ambiguous unique fragment references are validation errors when an abnormality is expected to grant a unique fragment."

Current live data does not satisfy that requirement:

- 30 live abnormalities are present.
- 22 `fragment_*` skill fragments are present.
- Only 15 abnormalities currently have a `SkillFragmentOrigin::Abnormality` mapping.
- 15 abnormalities have no abnormality-origin unique fragment mapping:
  - `f-01-87_snow_queen`
  - `f-02-49_rudolta`
  - `d-04-108_parasite_tree`
  - `o-04-66_porcubbus`
  - `f-01-18_scarecrow`
  - `o-02-63_apocalypse_bird`
  - `o-01-73_knight_of_despair`
  - `o-01-64_king_of_greed`
  - `f-02-58_big_bad_wolf`
  - `t-06-27_moonlit_wail`
  - `t-04-50_queen_bee`
  - `o-01-15_nameless_fetus`
  - `f-02-70_black_swan`
  - `o-05-76_schadenfreude`
  - `d-01-110_clouded_monk`

This is a user/content decision, not an implementation detail. Do not infer fragment ids by name, reverse lookup, concept origin, or placeholder naming.

Recommended long-term decision:

1. Add an explicit optional `response_complete_skill_fragment_id` field to `AbnormalityMetadata`.
2. Validate that every suppression-target abnormality has this field and that it references a real skill fragment.
3. Keep `SkillFragmentOrigin` as provenance metadata only, not as the reward source of truth.
4. Add or author placeholder fragments only by explicit content decision.

User decision on 2026-06-26:

- Use option 1 plus option 2.
- Not every abnormality's final skill fragment is designed yet.
- Add temporary placeholder skill fragments now and make abnormalities explicitly grant those fragments.

## Implementation Notes

- `RunState` is now the owner of `RunAbnormalityResearchState`.
- `RunCheckpointPayload` carries abnormality research state through the existing checkpoint boundary.
- `RunSnapshotDto.abnormality_research` exposes run-local response progress for presentation.
- `AbnormalityMetadata.response_complete_skill_fragment_id` is the source of truth for the first response-completion skill fragment reward.
- `SkillFragmentOrigin` remains provenance metadata and is not used to infer response-completion reward identity.
- Live RON `run/policy.ron` owns default required research, base victory gain, and repeat-complete fragment dust amounts.
- Partial battle-only `GameData` fixtures with an empty skill fragment catalog are allowed to omit response-completion fragment ids. Complete/live data and fixtures with a non-empty skill fragment catalog must validate the field.
- Endless combat victory applies research updates only for Endless mode; Standard final boss semantics remain in the Floor progression goal.
- When all initialized response entries are complete in Endless, combat result completion transitions to `RunComplete`.
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md` now documents `state_snapshot.state.abnormality_research` as the Unity-facing progress/encyclopedia/result UI source. Unity must not recompute research gain, first-completion fragment grants, repeat-complete dust grants, or run completion from this field.

## Follow-Up Candidates

- Merge `docs/endless_mode_research_policy_draft.md` into canonical docs after Phase 3 and Phase 4 settle the bonus objective and repeat weighting details.
- Replace placeholder response-completion fragments with final abnormality-specific fragment designs as content matures.
