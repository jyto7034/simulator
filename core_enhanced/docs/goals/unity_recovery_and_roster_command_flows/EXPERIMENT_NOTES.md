# Unity Recovery And Roster Command Flows Notes

Goal: G08

Status: complete

## Current Position

- Isolated goal documents are initialized.
- Source discovery is active; no G08 implementation edit has been made yet.
- Discovery found one bounded Core parity gap: `MoveRosterUnit` has an existing typed contract but is
  absent from the common capability projector/preflight, so it currently lacks the required
  `ViewingMap`/`NodeConfirm` execution gate.

## Direct Predecessor Handoff

- G07 is the sole action-capability and battle-revision authority. Unity renders and submits; it
  does not authorize.
- G15 exact abandonment and typed no-deployable recovery restrictions remain unchanged.
- G14 dedicated Gate and qualified RunFailed checkpoint capability remain unchanged.

## Owned Command Set

- `RecoverBattleSetupLoss`: read-only battle recovery capability; its response reprojects current
  revisioned authority but the request is not a mutation-authority command.
- `LoadRunCheckpoint`: root recovery capability, including the G14 RunFailed exception.
- `MoveRosterUnit`: root gameplay capability plus typed roster identities/slots.

## Dirty Worktree Rule

Inspect every target file's current diff before editing. Preserve all unrelated Core, server, Unity,
test, documentation, scene, and asset changes. Never stash, reset, restore, clean, or baseline commit.

## Open Questions To Resolve By Testable Discovery

- Which existing route should own the qualified RunFailed checkpoint control without creating a
  static GameOver rule?
- Whether battle setup recovery belongs in the battle screen command model or a dedicated recovery
  affordance built from the battle capability.
- Which existing maintenance roster interaction can express destination and optional swap identity
  accessibly without inventing roster validity rules.
- Whether the current command-result store path already consumes `roster_slots` or needs a typed
  result reader update using the existing wire shape.

## Implemented Flow

- Core now projects/revalidates existing `MoveRosterUnit` in `ViewingMap` and `NodeConfirm` only.
- ArkClient command bus has explicit methods for all three owned commands.
- Battle UI exposes setup reconstruction only from the battle recovery capability.
- Node Map exposes checkpoint load only from the root recovery capability; GameOver exposes the same
  command only when the qualified G14 RunFailed snapshot includes it.
- Node Map/NodeConfirm render authoritative roster-order rows and adjacent move controls inside the
  existing detail panel. Swap UUID is copied from the destination slot; an empty destination sends
  JSON null. The fixed two-panel Node Map layout is preserved.
- Non-battle command success is followed by the existing server root snapshot, so `roster_order`
  remains the final presentation source rather than local card reordering.
- If setup is missing but the current battle update remains, `GameSnapshotStore` exposes only that
  update's query/recovery capabilities. It does not fall back to root or expose battle gameplay.

## Final Validation

- Core all-targets: 722/722.
- game_server all-targets: 28/28; check passed.
- Unity EditMode: 193/193, job `b856005e883f494c9c3258b5c47616a2`.
- Unity PlayMode: 51 passed and 2 explicit skips, job `9602b6392ae84599bdff15107a8a17d2`.
- Unity explicit live server: 1/1, job `15366d3ec43b4fabadacb2ccdc14e602`.
- Unity error console: zero after clear.
- Formatting, scoped diff, command-surface, ignored-test, and local-server shutdown checks passed.

## Completion Review

- Review: `COMPLETION_REVIEW.md`.
- Verdict: Complete.
- Long-term fit: High.
- Immediate correction: none.
- R02 is unlocked for independent Phase 2 integration review.
