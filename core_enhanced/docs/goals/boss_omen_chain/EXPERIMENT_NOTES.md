# Boss Omen Chain Notes

## Initial Notes

- This is intentionally a later goal.
- First Endless implementation should remain infinite Floor + Gate progression without omen chains.
- Omen chain policy likely requires user decisions around triggers, chain length, forced node placement, and rewards.
- Do not let boss omen work leak into base Floor progression or base repeat weighting.
- Phase 5 policy is now gathered in `docs/boss_omen_chain_policy_draft.md`. Treat that draft as the active policy source during implementation.

## Known Dependencies

- Floor index and mode-specific progression.
- Run-local abnormality response state.
- Stable encounter candidate selection.
- Data-authored suppression target and reward policies.

## Out-Of-Scope Follow-Up Candidates

- New final boss content.
- Unity omen presentation.
- Special event chains for tool abnormalities.
- Account-wide omen history.
- Additional boss chains beyond WhiteNight. Silent Orchestra needs authored symphony source encounters/events before it should be added to live `boss_omen/chains.ron`.

## Questions To Ask Before Implementation

The original questions are answered in `docs/boss_omen_chain_policy_draft.md`. If code reading reveals a behavior not covered by that draft, stop and ask instead of guessing.

## User-Decided Policy

- Boss omen chains apply to Endless mode.
- A boss omen source becomes eligible after the player has at least one bloomed skill fragment and the run has reached the configured minimum Floor.
- Boss omen nodes are optional, not mandatory path blockers.
- Omen steps are not limited to normal combat nodes. Each step has a data-authored `source_kind`; first implementation supports `Event` and `Combat`.
- Event source steps overlay Event nodes.
- Combat source steps overlay normal Combat nodes.
- The overlay keeps the original node slot, graph connectivity, and selection structure, but changes the node's presentation/content source into a boss omen source.
- Gate, Reward, Shop, Rest, Support, HeadquartersContact, Maintenance, Elite, Boss, FinalBoss, Start, and Maintenance nodes are not default overlay targets.
- Event nodes are independent visual-novel-style nodes with scene graph definitions.
- Core owns event state and choice results; Unity owns presentation resources and reads them by ids supplied by core.
- Unity advances non-choice Event scenes with `advance_event_scene` and confirms choices with `select_event_choice`.
- Event choices are committed immediately and cannot be changed on re-entry.
- Event-started combat and Combat source omen use normal combat attempt rules.
- BossOmen step is consumed when the source node becomes Completed.
- If no eligible overlay node exists for the required source kind, keep the same provisional boss/step and retry on the next Floor.
- If the first placed omen source is ignored and the player enters Gate, push the provisional boss back into the pool and reroll another boss next Floor.
- Completed chains force a separate boss node. This is not a Gate replacement and not an existing-node overlay.
- Boss node rewards are data-authored in RON. Boss node defeat is run failure with main-menu or SavePoint rollback options.
- Similar RON/catalog/event graph/core invariant violations are panic/fail-fast QA errors, not gameplay rejections.

## Implementation Notes

- `EventChoiceEffect::StartCombat` reuses the live battle pipeline through an explicit encounter helper. It does not mutate the map node into a Combat node.
- Event choice commitment is stored in run-local `event_sessions`, not only in `active_node_content`, so retreat/retry cannot reset the selected choice.
- Completed boss omen chains create a separate dynamic Boss node instead of replacing Gate or overlaying an existing node.
- Forced boss insertion sets that dynamic Boss node as the map terminal and makes it the only selectable node.
- Forced boss victory clears the active boss omen state before the next Floor map is generated.
- Event `StartCombat` encounter references are validated against PVE encounter data during data validation; missing references are QA/data errors.
