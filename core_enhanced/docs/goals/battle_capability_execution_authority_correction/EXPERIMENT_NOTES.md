# Battle Capability Execution Authority Correction Notes

Status: complete

## Active Technical Judgments

- The Core correction must be one battle-context evaluator shared by projection and execution. Do
  not search the projected DTO as a separate authority if that would duplicate the action set or
  make serialization shape control execution.
- Context preflight and concrete payload validation are distinct layers. Passing context preflight
  does not authorize an employee/tile/skill/target payload.
- Authority validation remains first so missing/stale requests reject before context/payload work.
- `RequestBattleState` and `RecoverBattleSetupLoss` remain available during blocking VN.
- Missing and stale errors keep distinct codes; only their server recovery composition is shared.
- Server proactive resync is preferred over adding a second Unity auto-request fallback.

## Dirty-Worktree Classification

The following likely G30 files are already dirty and require scoped diff inspection before editing:

- `src/game/world.rs`
- `src/game/world/action_capability.rs`
- `src/game/world/battle_capability.rs`
- `src/game/world/tests/combat.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`
- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameSnapshotStore.cs`
- `/mnt/f/unity projects/ark/Assets/Tests/EditMode/ArkClientContractTests.cs`

- `src/game/world.rs`: mixed G01/G06/G07 plus separate VN/RunStart history. G30 owns only composing
  the new battle preflight in `gate_behavior_action`; authority-first order is preserved.
- `src/game/world/action_capability.rs`: untracked completed G06 common seam; no G30 behavior rules.
- `src/game/world/battle_capability.rs`: untracked completed G07 projector and the G30 shared-
  evaluator owner.
- `src/game/world/tests/combat.rs`: mixed historical test work; G30 adds focused coverage adjacent to
  the existing blocking-VN test without rewriting other hunks.
- server `handlers.rs`: mixed history; G30 owns only the stale-only recovery match/helper and paired
  authority recovery test.
- Unity `GameSnapshotStore.cs`: completed G06/G07 store behavior; no G30 production edit required.
- Unity `ArkClientContractTests.cs`: large mixed test history; G30 adds one test adjacent to the
  stale authority test only.

No unrelated or unclassifiable hunk must be overwritten for the planned correction. Continue using
small contextual `apply_patch` edits and recheck scoped diffs after every change.

## Out-Of-Scope Follow-Up

- Deep atomic staging of `BattleCore::deploy_player_unit` after validated world input remains a
  follow-up candidate from R01. It is not required to fix context authority or missing resync.

## Policy Questions

None currently. A DTO/RON/save/UX/gameplay meaning change would be a new policy question and must
stop this Child.
