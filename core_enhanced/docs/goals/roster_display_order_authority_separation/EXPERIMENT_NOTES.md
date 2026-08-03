# Roster Display Order Authority Separation Notes

Status: complete

Created: 2026-07-15

Last updated: 2026-07-15

## Structural Judgments

- Core display order is not employee ownership or battle deployment state.
- Current Core sync uses combat-available UUIDs despite its “owned units” name; this is another reason not to preserve it as the full employee-card order.
- Core producer, game_server forwarding, Unity old-contract consumption, Unity local replacement, and the formation interaction change for the same reason and form one complete display-order authority boundary.
- Unity must store UUID order separately from authoritative snapshot JSON and request a display-ordered view explicitly rather than mutate `RosterModel` gameplay facts.
- Formation badge numbers are derived from one ordered UUID draft; they are not separately mutable per-unit state.
- The user superseded both the original Rest placement and the intermediate Loadout placement after completion. The current formation surface is the SafeZone Item `ItemUseEmployees` action-card panel; Rest retains read-only ordered status and Loadout retains equipment editing only. The Item employee hit targets reuse the same action metadata, outlines, and badges without changing authority.
- Automatic reconciliation is committed local-settings maintenance, not a formation draft: removed/unavailable UUIDs are pruned and newly eligible UUIDs append immediately. User-driven reordering remains atomic and persists only on Complete.
- Eligible employees are ordered first; dead/unavailable employees remain visible after them in deterministic UUID order and never receive an active formation number.

## Source-Of-Truth Conflict

Completed G08 documents and current canonical wording describe Core `roster_order` and the following root snapshot as authoritative. That was valid under the former policy.

The 2026-07-15 user decision supersedes only that roster-order conclusion:

- Core order and `MoveRosterUnit` disappear,
- Unity-local UUID order becomes presentation authority,
- checkpoint and battle abandonment do not restore it,
- Node Preview controls are removed.

G08's `RecoverBattleSetupLoss` and `LoadRunCheckpoint` behavior remains unchanged.

## Resolved Persistence Decision

The user approved device-local, restart-persistent, player-scoped storage on 2026-07-15. The stable player/account UUID used for `/game` authentication scopes a versioned JSON settings document. Identity must be injected explicitly; settings must not reach into a private transport field or use a device-global key.

Formation drafts remain transient. Only complete commits; cancel and interrupted editing preserve the previous committed JSON. Storage remains Unity-only and cross-device sync is outside scope.

## Direct Dependencies

- Core, game_server, and Unity repositories must be modified as one coordinated contract cut; strict Unity capability parsing makes an uncoordinated release unsafe without compatibility.
- Actual Rust fixture generation remains the producer proof for Unity contract tests.
- Functional formation interaction uses the production SafeZone Item employee panel with unit hit targets and highlight/badge anchors. Final art creation is not part of this Goal.

## Alternatives Not Applied

- Keep Core order but hide its UI: checkpoint/retreat would still restore presentation state.
- Store replacement order in game_server: preserves remote authority and adds unnecessary persistence.
- Keep a deprecated command or dual schema: no compatibility approval.
- Use employee names or server array indices as keys: unstable identity.
- Store committed order in transient `GameUiSelectionState`: wrong lifetime.
- Pair swapping, adjacent arrows, or character dragging: not the approved sequential interaction.
- Sort Event candidates or result statistics: separate domain ordering and outside scope.
- Build final illustrated art inside this Goal: separate asset/content boundary.

## Follow-Up Candidates

- optional sorting presets,
- controller/keyboard-specific ordering controls,
- cloud/cross-device synchronization,
- final illustrated Rest scene/art production if it remains unavailable.

These are not implemented automatically.

## Dirty Worktree

- Goal path had no name collision beyond the earlier documents explicitly replaced by the user.
- Core, game_server, and Unity were all heavily dirty at initialization; nearly every target file already had user-owned changes and some files/fixtures were untracked.
- Before production edits, re-check repository/submodule state and every target diff. Never reset, restore, clean, stash, or overwrite unrelated work.
- Completion audit found that existing screenshot tests delete and regenerate two tracked PNG validation artifacts which were already dirty at Goal start. Those tests were run before this side effect was recognized, and the original uncommitted binary bytes are not recoverable from the workspace, Unity artifacts, or duplicate-path search.
- Required live-server validation appended to the already-untracked `game_server/logs/app.log.2026-07-15`. It remains a generated log, but its exact start-of-Goal contents were not captured.
- On 2026-07-15 the user confirmed that the earlier generated artifacts were not important and explicitly accepted retaining the regenerated PNGs and appended generated log. This resolves the preservation exception without attempting an unsafe reconstruction.

## Unverified Consumers And Skipped Validation

- Deployed client/server version inventory and any inaccessible external consumer are unknown. No compatibility need has been proven.
- Core, server, Unity, live-flow, formatter, and live RON validation are now recorded in `EXPERIMENTS.md`.
- Final illustrated Rest scene assets remain unavailable and outside scope.
- The real current UGUI SafeZone Item action-card surface is covered by PlayMode input/render evidence; final illustrated scene assets remain a follow-up rather than a blocker for the approved interaction semantics.

## Remaining Risks

- A coordinated contract cut may affect unknown deployed versions.
- Heavy dirty worktrees make overlap preservation a material implementation risk.
- Functional implementation and non-mock live evidence are complete. The previously unverifiable binary/log content is an explicitly accepted generated-artifact exception and no longer blocks completion.
