# Contextual Action Capability Projection Experiment Notes

Status: active

Implementation complete: false

Created: 2026-07-13

## Binding Structural Decisions

- Root `allowed_actions` is removed, not reinterpreted or aliased.
- Core owns typed context capability projection; domain pure validators own concrete payload eligibility.
- Projection/preflight and execution share validators and typed reasons; execution revalidates atomically.
- Action scope is explicit: gameplay, query, or recovery.
- Irrelevant actions are omitted; relevant blocked actions retain typed disabled reasons.
- No generic payload bag, unbounded eligibility product, compatibility field, or Unity inference is allowed.
- G01 battle revision/cursor/order/reconnect ownership is immutable input to G06.

## Cross-Goal Boundaries

- G07: battle capability, battle payload eligibility, expected revision, reconnect/resync capability, Unity battle whitelist removal.
- G03: Event choice and employee eligibility.
- G21: Safe Zone research delivery eligibility.
- G24: ordinary inventory/consumable preview and eligibility; consumes G06 shared preflight seam.
- G28: Shop offer/sale/refresh eligibility.
- G08: player-facing recovery/roster flows consuming G07.
- G12: strict generated live-data loading; G06 must not invent a domain RON validator.

## Questions To Answer From Current Code

- Which current gate is authoritative: cached `ActionValidator`, `ActionScheduler`, `get_allowed_actions()`, handler validation, or some combination?
- Which actions are truly payload-free and already have reusable pure validation?
- Which parameterized actions can honestly report “at least one payload executable” without absorbing later domain policy?
- Which stable common disabled reasons already exist and which are string-only or conflated?
- Which root/server/Unity consumers require an atomic schema cutover?
- Does any saved/checkpoint state persist action cache or allowed actions?
- What smallest actual Rust root fixture can Unity consume without coupling to battle-specific G07 fields?
- Which existing tests protect user-visible context behavior versus coarse table cardinality?

## Known Dirty Worktree Risk

G01 changed Core root/battle DTOs, game_server actor mapping, Unity root/battle runtime, tests, fixtures, and canonical documents. Inspect exact current diffs before G06 edits and preserve G01 plus unrelated user changes.

## Predecessor Invariants To Preserve

- `battle_state_revision` is Core-owned, checked, and scoped by fresh battle UUID.
- Server connection owns event cursor and ordered actor emission.
- Unity ordinary battle freshness is UUID/revision-only; seq is continuity and time is presentation.
- Reconnect/setup recovery preserves the active battle.
- G06 common capability cannot become a second battle commit or reconnect authority.

## Provisional Long-Term Seam

1. A pure Core context projector enumerates only relevant typed action families and calls shared validators for enabled/reason state.
2. Payload-free commands use one pure validator directly from both projection and execution.
3. Parameterized action families use a bounded domain-provided “any executable payload” query for context capability and retain detailed eligibility in their typed domain projection/preflight.
4. Root snapshot serializes typed `action_capabilities`; server preserves it; Unity stores/renders it without substitution.
5. Atomic command execution calls the same validator again immediately before mutation.
6. Old cached/table authorities are deleted in the same schema cutover.

This is a technical interpretation of accepted PD-02. It must be revised if current-source evidence exposes an invariant gap; it must not absorb another child's policy to obtain a superficially complete list.

## Follow-Up Candidates

None yet. Record only after current-source discovery.

## Revalidated Authority Split

- The cached `ActionValidator` is now strictly dead derived state: transition and refresh paths write it, but live gate/snapshot paths recompute the scheduler. Delete it rather than adapting it into a capability cache.
- `ActionScheduler` is still the live coarse policy authority and cannot be renamed into the accepted projector because it does not call payload/domain validators or produce typed disabled reasons.
- `PlayerBehavior::kind()` remains a useful exhaustive mapping from concrete command to stable action identity; retaining that mapping does not imply retaining coarse execution policy.
- game_server is currently a transparent typed root wrapper. G06 should preserve that property and add exact serialization tests rather than insert server policy.
- Unity common consumption is centralized enough for an atomic root cutover through `GameSnapshotStore`/`GameCommandBus`; individual composers mostly depend on `HasAllowedAction` rather than parsing JSON themselves.
- `battleRuntimeActions` is a deliberate G07 boundary. G06 may change the shared capability model/API but must not claim the battle whitelist removed or battle eligibility complete.
- Admin `dump_allowed_actions` is an old public/debug name tied to the removed field. Keeping it would leave a stale contract surface; migrate it to typed capabilities in the same cutover unless current server evidence shows it is intentionally historical.

## Initial Action Ownership Matrix

| Family | Common G06 responsibility | Later payload owner |
| --- | --- | --- |
| Start/pre-run/starter selection | Context/scope DTO; payload-free start and finite candidate availability parity where current validators permit | None currently registered |
| Map request/select/confirm/cancel | Query/gameplay scope and existing map-navigation validator composition | Common G06 unless a current domain boundary disproves it |
| Loadout/inventory/consumable | Context family only; do not invent exact item/employee eligibility | G24 |
| Support/Headquarters/Safe Zone | Context family and existing selected-content relevance only | G21 for research delivery; other support payload behavior stays current unless exact eligibility is safely reusable |
| Shop | Shared DTO/projector/preflight interface only | G28 |
| Reward | Common context and existing finite reward/skip validators where reusable | No separate payload child; coordinate with current reward domain without changing reward policy |
| Event | Shared DTO/projector interface only | G03 |
| Visual Novel | Common progression shape can use current pure scene-next lookup; battle revision participation remains G07 | G07 for battle projection integration |
| Combat result | Payload-free local completion parity via existing `can_complete_combat_result_locally` | Later result policy remains G15/G26 |
| Battle | Shared types/API only | G07 |
| Maintenance | Context family only; no new content/balance semantics | Deferred G22 and existing equipment validators |

## Baseline Classification

- Scheduler 14/14 is expected legacy evidence. Tests requiring four Shop actions, ten battle actions, or both VN progression actions conflict with accepted PD-02 and must not survive as renamed completeness tests.
- Core state-transition gate 1/1 protects rejection outside coarse context but does not prove selected payload eligibility or typed reason parity.
- Server root preservation 1/1 is a useful transport seam to migrate to exact typed capability JSON.
- Unity EditMode 182/182 is the external baseline; many hand-authored `allowed_actions` fixtures and the battle whitelist test will require owner-aware migration across G06/G07.

## Reconciled Projector Boundary

- `ActionKind` remains the stable identity vocabulary, not an authorization object.
- Common evaluation is tri-state: irrelevant actions are omitted, relevant executable actions are enabled, and relevant blocked actions carry a stable typed reason.
- The common execution gate calls the same evaluator/domain preflight and re-evaluates immediately before mutation. A projected vector is never trusted as a token.
- Deferred-domain actions are not copied into a new generic state table merely to preserve old cardinalities. Their existing handlers remain atomic authority until the child owning their typed eligibility migrates them.
- G06 common query: `RequestMapData`. G07 queries: `RequestBattleState`, `RequestDeploymentRangePreview`.
- G06 recovery: `LoadRunCheckpoint`. G07 recovery: `RecoverBattleSetupLoss`. `RetreatBattle` is gameplay, not recovery.
- The initial parity slice is `SelectMapNode` and `LoadRunCheckpoint`. Map selection uses authoritative selectable node state; checkpoint loading uses `RunCheckpointState::can_load()` and ViewingMap context.

## Checkpoint DTO Decision

- `RunCheckpointState::can_load()` remains the Core predicate and load execution preflight.
- `RunCheckpointSnapshotDto.can_load` is removed because it duplicates command executability, cannot express context, and has no Unity consumer.
- `exists`, `loads_used`, `max_loads`, and `remaining_loads` remain factual checkpoint data.
- Current executable availability is expressed only by the `LoadRunCheckpoint` recovery capability and its typed disabled reason.
- This changes no load limit, checkpoint scope, rollback meaning, or save format; it only removes an ambiguous derived presentation boolean.

## Unity MCP Verification Contract

- Active instance: `ark@b49d32f6f2f70bbe`, Unity 6000.3.4f1.
- After each Unity script/fixture cutover, refresh and wait for domain reload, read compilation errors from the Console, then run the focused EditMode class through Unity MCP.
- Final verification includes nonzero EditMode and PlayMode runs through the live Editor; shell-only C# inspection is not accepted as Unity completion evidence.

## Initial Typed Reason Vocabulary

- Common technical codes introduced by the first slice: `context_irrelevant`, `no_eligible_payload`, `checkpoint_unavailable`, `progression_unavailable`, `content_unavailable`, and `completion_unavailable`.
- These are stable localization keys, not user-facing English. G06 uses only codes justified by common validators; later children may extend the enum through their own reviewed DTO migration.
- Enabled entries omit `disabled_reason_code`; disabled entries always use the typed constructor and include it.

## First Vertical Slice Status

- `RequestMapData`, `SelectMapNode`, and `LoadRunCheckpoint` are the only actions temporarily routed through the new evaluator during the reversible trial.
- Other actions still use the legacy gate until the projector is complete; this is an intermediate implementation state, not an accepted compatibility endpoint.
- The mutation handlers no longer duplicate the two migrated domain conditions. The gate/preflight owns the check and is invoked again for every command.
- After adding test-only imports, the focused slice passed 1/1 and proves typed scope/reason projection, selected-map payload rejection, checkpoint disabled parity, and stale-state atomic revalidation.

## Root Cutover Status

- Rust now emits only `action_capabilities`; there is no serde alias/default for `allowed_actions`.
- Checkpoint capacity facts remain in `run_checkpoint`; current command executability is no longer duplicated there.
- This is an intentionally incomplete cross-layer checkpoint: server compilation/tests and Unity still require migration before the schema is usable end-to-end.

## Unity Typed Consumer Decision

- Unity rejects non-object entries, unknown Core action names, unknown scopes/reasons, enabled entries carrying a reason, disabled entries lacking a reason, and duplicate action identities.
- `TryGetActionCapability` distinguishes absence from disabled. `IsActionEnabled` is the command gate convenience API.
- Common presenter command creation returns without adding a button when capability is absent. Disabled capabilities create disabled buttons with localized reason text.
- No `HasAllowedAction` compatibility method is retained. Test code must migrate with production.
- The static battle compatibility path is renamed internally but intentionally remains G07-owned; it is not evidence that G06 completed battle authority.
- Unity MCP must explicitly import newly created external C# files before compilation if the first refresh reports the type missing.

## Unity Failure Classification

- 160/182 passing immediately after the API cutover shows most snapshot readers are independent of action availability.
- The 22 failures are not production compiler defects. They cluster where old fixtures omit the new field and therefore correctly cause commands to be absent.
- Replace the old `SnapshotStore_UsesContextTypeAndAllowedActions` test with typed scope/enabled/reason/absence coverage.
- A reusable fixture builder is acceptable only in test/mock code if it emits the exact production DTO objects. It must not parse or translate old `allowed_actions` at runtime.
- Explicit local placeholder commands are not Core gameplay/query/recovery capabilities. Their dedicated method remains visible/disabled by local UX policy; known Core placeholders are still omitted when the capability is absent. Ordinary Core `AddCommand` always omits absent actions.
- VN command availability is snapshot-specific and mutually exclusive. Tests must apply a new capability snapshot between Advance and Choice.
- Map Confirm does not exist as a disabled action while still in ViewingMap; it appears only after Core enters NodeConfirm.

## Expanded Common Projector

- Common context projection now uses active session variants rather than `InNode`-wide unrelated actions.
- Support choice is relevant only for choice-mode support content; completion is disabled until a choice is committed.
- HQ actions are derived from current headquarters options; recruitment is disabled when no candidate exists.
- Shop context capability performs finite current-state checks for at least one purchasable/sellable payload and exact reroll/exit predicates. G28 still owns offer-level eligibility DTOs and final shop refinement.
- Reward Select/Claim/Exit reflects reward mode, selected reward, and skip policy.
- Non-battle VN Advance versus Choice follows the actual current scene next type and is mutually exclusive. Battle integration remains G07.
- The failed initial test exposed an obsolete temporary assertion, not a production regression.

## Legacy Removal Boundary

- Deferred-domain commands now reach their existing domain handler preflight directly; they are not authorized by a projection snapshot and are not copied into a replacement global table.
- This preserves atomic validation while later child goals add their typed domain capability/eligibility projections.
- `ActionScheduler`, `AllowedActionContext`, `ActionValidator`, refresh calls, `get_allowed_actions`, and `is_action_allowed` are deleted from production.
- Tests asserting battle/inventory presence in a global root list are obsolete by ownership, not candidates for a mechanical rename.
- The 33 test compile errors are the complete current stale API inventory and must reach zero before broad tests.
- All 33 are now resolved without reintroducing a compatibility helper. Common tests assert typed projection; deferred-domain tests assert actual behavior and leave capability projection to their owning child.
- A subsequent runtime failure found three separate `run_checkpoint.can_load` JSON assertions. They are DTO legacy, distinct from the retained `RunCheckpointState::can_load()` domain predicate.

## Broad Boundary Corrections

- Projecting a domain context capability does not automatically make G06 the selected-payload execution owner.
- Shop context projection remains useful to Unity, but G06 must not mask specific Shop handler errors; G28 will compose Shop preflight fully.
- Battle VN remains entirely under G07 during execution even though common non-battle VN uses the shared types.
- Generic `InvalidAction` expectations for G06-owned common preflight are obsolete and should assert the stable action/reason pair.
- The corrected boundary is supported by 628/628 Core tests: battle VN and Shop selected payload remain domain-owned, while common map/checkpoint/support/start parity is typed.

## Fixture And Error Provenance

- `game_server/examples/generate_action_capability_fixture.rs` constructs a real Core root snapshot and is the provenance authority for Unity's checked-in `action_capability_contract.json`.
- The generator compare mode checks JSON values exactly; Unity reads that file directly in EditMode.
- Unity negative tests reject unknown action/scope/reason, missing disabled reason, enabled reason presence, non-object entries, and duplicates through the strict model (representative cases are fixed in tests; implementation enforces the full set).
- Server error `code` carries `ActionDisabledReasonCode::as_str()`, so projection and failed execution share the same stable machine-readable reason across transport.

## PlayMode Failure Audit

- The first broad run is 45/53, not completion evidence.
- Do not change eight expectations independently: missing NodeMap utility availability can prevent actions that later tests rely on and leave shared UI state behind.
- Focused isolation must establish whether each failure reproduces alone before deciding production versus test-fixture correction.
- Isolation confirmed a two-test root cause: old disabled-Confirm expectations contradicted irrelevant-action omission. Fixing them restored root cleanup and all 53 PlayMode tests passed.

## Final Authority And Handoff

- Common/root capability authority is now Core `GameCore::get_action_capabilities()` plus the
  shared current-state preflight; serialized capability vectors are never authorization tokens.
- game_server remains a transparent transport mapper and preserves exact stable reason codes.
- Unity common UI consumes absence/enabled/disabled/scope directly and localizes typed reasons; it
  does not reconstruct common capability from context.
- `run_checkpoint` is factual state only. `LoadRunCheckpoint` recovery capability is the sole
  presentation affordance for current executability.
- Runnable WebSocket probes enforce the same contract and no longer describe or parse the removed
  root list/derived checkpoint boolean.
- The completion audit found no common/root compatibility path or duplicate authority.
- G07 must remove `GameSnapshotStore`'s explicitly battle-scoped static action set and integrate
  revisioned battle capability/recovery. G03/G21/G24/G28 and deferred G22 own their concrete domain
  eligibility; this is a planned ownership boundary, not unfinished G06 work.
