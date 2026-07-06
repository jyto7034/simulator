# Refactor Low-Risk Boundary Cleanup Notes

## Initial Judgment

These items are grouped because they are low-risk, mostly local, and already have user policy decisions.

- Shop hidden items: internal state stays; Unity-facing DTO hides reserve items.
- Boss omen distance: implementation correctness cleanup, BFS shortest path.
- `ApplyBossOmenStepResult`: fail-fast until real mechanics are designed.
- Battle records: current behavior is codex-oriented and should be documented, not changed.

## Guardrails

- Do not implement real boss omen step-result mechanics in this goal.
- Do not rename persisted fields unless the blast radius is reviewed.
- Do not remove `ShopSessionState.hidden_items`; only remove client exposure.
- Do not convert abnormality codex records into per-battle timeline archives.

## Implementation Notes

- Live `white_night_confession_01` event data used `ApplyBossOmenStepResult`, but current boss omen step consumption is already tied to source node completion. The event-choice effect marker had no real mechanics and would become an intentional fail-fast panic, so the live RON marker was removed instead of inventing partial boss omen result behavior.
- `ApplyBossOmenStepResult` remains in the schema as a future explicit effect marker, but runtime use now panics until the mechanics are designed.
- `battle_records` naming remains unchanged in this goal. Comments and canonical rulebook text clarify that current records are abnormality codex/observation records keyed by `abnormality_uuid`, not per-battle timeline archives.

## Follow-Up Candidates

- Real boss omen step-result mechanics once boss-specific omen effects are designed.
- A separate per-battle timeline archive if product needs every combat replay.
