# Experiment Notes

## Initial Context

- `behavior_execution_boundary_refactor` made `execute_with_source_command_id(...)` read as gate, validation, dispatch, postprocess.
- The remaining issue is duplicate whole-`PlayerBehavior` matching: one match for validation and another match for dispatch.
- The intended long-term direction is domain ownership: shop behavior should be validated and dispatched in the shop behavior entry, battle behavior in the battle entry, and so on.

## Policy Notes

- Preserve user-visible behavior.
- Preserve Unity/server command DTOs.
- Preserve allowed-action gate behavior.
- Preserve `source_command_id` handoff for live battle commands.
- Preserve one roster sync after successful behavior execution.
- Prefer explicit domain functions over a generic framework.

## Domain Buckets To Verify Before Editing

- Map/run flow: start game, starter selection, request map, select node, confirm/cancel node, complete node, checkpoint loading.
- Support/headquarters: support choice, recruitment, emergency supplies, headquarters shop opening.
- Shop: purchase, sell, reroll, exit.
- Reward: select reward, claim, exit.
- Event: advance scene, select choice.
- Equipment/maintenance: equip/unequip item, consumable use, skill fragment equip/unequip/upgrade/awaken/dismantle, equipment dismantle/enhance, roster movement.
- Battle: request battle state, recover setup loss, deployment range preview, deploy, withdraw, activate skill, retreat, pause/resume/speed.
- General bucket: only use if a behavior does not yet have a clean domain boundary.

## Verified Domain Mapping

After re-reading the current runtime code and handler files, use these execution buckets:

- `execute_map_run_behavior(...)`
  - `StartNewGame`
  - `SelectStarterEmployees`
  - `RequestMapData`
  - `SelectMapNode`
  - `ConfirmEnterNode`
  - `CancelSelectedNode`
  - `CompleteNode`
  - `LoadRunCheckpoint`
- `execute_support_headquarters_behavior(...)`
  - `ChooseSupport`
  - `RecruitEmployee`
  - `RequestEmergencySupplies`
  - `OpenHeadquartersShop`
- `execute_equipment_maintenance_behavior(...)`
  - `EquipItem`
  - `UnEquipItem`
  - `UseConsumableItem`
  - `EquipSkillFragment`
  - `UnequipSkillFragment`
  - `UpgradeSkillFragment`
  - `AwakenSkillFragment`
  - `DismantleSkillFragment`
  - `DismantleEquipment`
  - `EnhanceEquipment`
  - `MoveRosterUnit`
- `execute_shop_behavior(...)`
  - `PurchaseItem`
  - `SellItem`
  - `RerollShop`
  - `ExitShop`
- `execute_reward_behavior(...)`
  - `SelectReward`
  - `ClaimReward`
  - `ExitReward`
- `execute_event_behavior(...)`
  - `AdvanceEventScene`
  - `SelectEventChoice`
- `execute_battle_behavior(...)`
  - `CompleteCombatResult`
  - `RequestBattleState`
  - `RecoverBattleSetupLoss`
  - `RequestDeploymentRangePreview`
  - `DeployUnit`
  - `WithdrawUnit`
  - `ActivateSkill`
  - `RetreatBattle`
  - `PauseBattle`
  - `ResumeBattle`
  - `SetBattleSpeed`

No general bucket is needed in the current enum: every `PlayerBehavior` variant has a clear domain owner.

## Validator Movement Plan

- Remove the top-level `validate_behavior_payload(...)` call.
- Keep the top-level allowed-action gate and successful-command postprocess unchanged.
- Move validation calls into the domain function that dispatches the same behavior.
- For handlers that already validate internally, do not add duplicate validation.
- Keep battle `source_command_id` handoff only inside the battle domain entry.

## Implementation Notes

- `execute_with_source_command_id(...)` now reads as:

```text
derive action kind
-> gate_behavior_action(...)
-> execute_validated_behavior_domain(...)
-> postprocess_behavior_execution(...)
```

- `execute_validated_behavior_domain(...)` is the only full-enum behavior router.
- Domain helpers own the subset match for their own behavior group.
- Removed the old `validate_behavior_payload(...)` full-enum wrapper.
- No behavior is currently left in a general bucket.
- Did not change `PlayerBehavior`, `ActionKind`, Unity/server DTOs, allowed-action policy, or roster sync timing.
- `unreachable!` arms are internal routing invariants after the top-level domain router has already selected the matching domain.

## Follow-Up Candidates Outside Scope

- Splitting `PlayerBehavior` into domain-specific enums.
- Reworking server command envelopes.
- P-008 battle core decomposition.
