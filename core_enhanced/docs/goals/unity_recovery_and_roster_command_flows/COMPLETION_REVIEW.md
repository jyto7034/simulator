# Unity Recovery And Roster Command Flows Completion Review

Goal: G08

Review date: 2026-07-14

Guide: `docs/goal_completion_review_guide.md`

Overall verdict: Complete

Overall long-term fit: High

Immediate correction: none

## Audit Table

| Item | Verdict | Fit | Immediate correction |
| --- | --- | --- | --- |
| DCA-H-13 / CPR-H-07 missing Unity recovery/checkpoint/roster flows | Complete | High | none |

## DCA-H-13 / CPR-H-07 — Player-Reachable Typed Command Flows

Master item:

Core and the canonical contract already defined `RecoverBattleSetupLoss`, `LoadRunCheckpoint`, and
`MoveRosterUnit`, but ArkClient had no dedicated typed sender/dispatcher/UI path for any of them.
The roster action was additionally absent from the G06 common capability projector, so its execution
had no current-context gate.

Expected behavior:

- Setup reconstruction consumes only current revisioned recovery capability and remains read-only.
- Checkpoint load appears only from Core's root recovery capability, including the qualified G14
  forced-BossOmen `RunFailed` exception and excluding ordinary terminal states.
- Roster organization appears in `ViewingMap`/`NodeConfirm`, derives concrete slot/swap identities
  from `roster_order`, and commits only through the existing typed command and following snapshot.

Runtime evidence:

- `src/game/world/action_capability.rs` enumerates `MoveRosterUnit`, evaluates it only for a nonempty
  roster in `ViewingMap`/`NodeConfirm`, and owns its execution preflight. The existing domain handler
  still validates employee identity, destination bounds, and swap occupant before mutation.
- `GameCommandBus.cs` has explicit exact-shape methods for all three behaviors. Setup recovery does
  not add `battle_authority`; checkpoint and roster methods use their projected action gates.
- `GameSnapshotStore.cs` uses current battle capability whenever the runtime is complete. When setup
  is missing but a current update remains, only query/recovery scope is reachable; gameplay never
  falls back to root or leaks before reconstruction.
- `BattleScreenComposer`, `RunFlowScreenComposer`, and `NodeMapScreenComposer` add controls only
  through `GameScreenCompositionContext`/typed capability lookup. Ordinary GameOver receives no
  checkpoint button. Node Map and NodeConfirm share the existing two-panel layout and add roster rows
  and adjacent move controls to the detail panel.
- `MapNodeActionHandler` rechecks the roster action gate before dispatch. It does not reorder local
  cards; the server's following root snapshot remains the final source.

Data and public-contract evidence:

- No new DTO, save, RON, balance, result variant, compatibility alias, or generic payload bag was
  added. Existing `ActionKind`, action scope, request fields, `roster_order`, `roster_slots`,
  checkpoint facts, battle update, and setup snapshot shapes are reused.
- Core rulebook/runtime docs and Unity canonical/implementation docs now describe the exact
  capability-driven controls, incomplete-runtime recovery boundary, and snapshot ownership.

Test evidence:

- Core red tests first proved both defects: no projected roster capability and successful roster
  movement from GameOver. Final focused roster matrix passed 3/3; map flow passed 28/28.
- Unity typed wire tests verify exact field names, JSON null for empty swap, read-only recovery without
  mutation authority, and checkpoint command shape.
- Presenter/handler tests cover ViewingMap roster controls, current destination occupant, empty
  destination, active-battle recovery, qualified RunFailed load, ordinary GameOver omission, and
  capability recheck.
- Incomplete-runtime regression proves RequestBattleState/Recover remain enabled while
  Deploy/Pause/Retreat are absent.
- The explicit live Rust-server test performs actual roster movement and setup reconstruction before
  continuing through deployment, playback, withdrawal, and exact retreat.

Legacy/fallback audit:

- There is no route-name, BossOmen reason, checkpoint counter, roster count, or kind-id inference in
  Unity command authorization.
- There is no static recovery list, root battle fallback, locally reordered roster, generic
  `SendBehavior` UI escape hatch, duplicate command, ignored focused test, or third-panel layout
  fallback.

Long-term direction review:

- Fit: High.
- Improvement class: none required for completion.
- Reason: the implementation extends the existing typed capability/preflight and domain UI seams,
  preserves Phase 1 revision authority and G14/G15 transitions, and keeps Core as the only execution
  authority without widening public schema.

Verdict:

- Complete.

Remaining risk:

- The explicit live test creates ordinary map/battle states rather than manufacturing the rare
  forced-BossOmen RunFailed state. That exact load transition is already covered by the current Core
  G14 test and Unity's typed command/presenter tests; no separate Unity inference exists.

## Cross-Component Findings

- G08 exposed one real G06-era common projection omission for roster movement and repaired projection
  and execution together rather than adding a Unity exception.
- Read-only setup recovery and mutation resync remain distinct. Losing setup narrows the client to
  query/recovery scopes until the same battle is reconstructed.
- G14's dedicated Gate requires no special Unity action: it remains an ordinary projected map node
  selected and confirmed through the existing map flow.
- G15 no-deployable restriction remains authoritative: disabled checkpoint load and restricted map
  candidates are consumed as projected, never recalculated.

## Validation Results

- Core library: 681 passed, 0 failed/ignored.
- Core all-targets: 722 passed (`681 + 3 + 3 + 18 + 13 + 4`).
- game_server check: passed; all-targets 28 passed, 0 failed/ignored.
- Unity EditMode job `b856005e883f494c9c3258b5c47616a2`: 193 passed, 0 failed/skipped.
- Unity ordinary PlayMode job `9602b6392ae84599bdff15107a8a17d2`: 51 passed, 0 failed,
  2 explicit live-server skips.
- Unity explicit live job `15366d3ec43b4fabadacb2ccdc14e602`: 1 passed, 0 failed/skipped.
- Unity post-run error console after clear: 0 entries.
- Core formatting, scoped Core/Unity diff checks, server shutdown, and legacy/ignored searches passed.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

None inside G08. Future visual drag-and-drop polish may reuse the same typed payload, but it is not a
contract correction and must not introduce local roster legality.

## Policy Questions

None.

## Review Conclusion

DCA-H-13 / CPR-H-07 is Complete with High long-term fit and no immediate correction. G08 may close
and R02 may independently review the complete Phase 2 transaction/recovery integration.
