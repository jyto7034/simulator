# R03 bounded world capture source assistance 008

Status: read-only design assistance under the assigned AUTORUN §8 scope, not a completion audit. No repository mutation, build, test, runtime, or Git command was performed. Root-reported INPUT053 results are historical context, not my executions. Public Session/final receipt commit remains a separate implementation obligation.

Sources were discovered with rg, then read with Python read_bytes; each full-file SHA256 and numbered inspected span came from those same bytes. JSON preserves all 22 reads of 16 paths. Final rehash at 2026-09-10T10:12:23.996729+00:00 found no drift against these reads; root can subsequently change them. Line references below are to these captured versions. Prefix W means core_enhanced/src/game/world/.

## Minimum integration

All five helpers can become fallible internally: game/mod.rs:29 declares world pub(crate); world.rs:19–44 keeps state and grant_state private. A spelled pub capture is therefore not an external public API. Only the active-run rollback helper discussed below is cfg(test).

Use an owner-local borrowed projection, not controlled_clone(&capture(...)): the latter copies before admission. Follow existing authoring/clone_shape.rs:24–34: inactive scope executes the original native construction; active scope polls, visits every actually copied field into one CloneMeter, completes finish before the FIRST native field clone, constructs the unchanged captured value, polls, then returns Ok(value). A failed final poll drops that local value. Do not retain references/owners in the meter or reset attempt accounting. control.rs:373–411 already admits scans and charges the final copy.

Charge projected output structure and fixed/default fields consistently, but never traverse omitted owners. Reuse native CloneShape visitors, including OwnedHashMap storage proofs, native-set visitors and Arc handles. Serde would miss RunCheckpointState.encoded_core_payload (Wstate.rs:380–386) and cannot substitute for this ownership projection. Generate checkpoint field visitation from its existing capture macro list.

## Capture map and callers

1. **BattleEntrySnapshot** — Wstate.rs:544–615, current pub capture(&GameCoreState) -> Self; propose Result<Self, GameError>. Copies lifecycle phase as game_state; active_node_content, node_session, run, visual_novel, visual_novel_triggers, uuid_manager, enkephalin, inventory, roster, starter_candidates, skill_fragments and run_checkpoint; pending_game_mode is Copy. Its first copy is lifecycle.snapshot(), so visit borrowed lifecycle.phase() before calling snapshot(). It excludes player, pending sweep, fragment policy, player profile, active battle and power authority.

   Four call sites: Wnode_flow.rs:628 handle_confirm_enter_node_inner; Wcombat.rs:285 handle_event_combat_node; Wcombat.rs:660/667 commit_planned_live_battle_start. All return Result<BehaviorResult, GameError>. Conditional capture requires .then(...).transpose()? or an explicit branch; event reentry uses capture(...)?.as_event_combat_reentry(...). The latter consumes the snapshot without cloning it. Commit's supplied snapshot is already moved from planned.entry_snapshot. New captures finish before start_abnormality_attempt, next_battle_session and active-session installation (669–687). Node confirmation has earlier map/progression copies and is wrapped by with_state_transaction (604–605). Snapshot restore moves these selected owners and clears active_battle; retreat takes the active session and consumes its snapshot (2794–2801).

2. **RunCheckpointPayload** — Wstate.rs:388–490, generated pub(super) capture(&GameCoreState, &RunState, &ContentIdentity) -> Self; propose Result. Before content_identity.clone(), visit that identity; the two power scalars; all eleven macro run fields: map, map_progression, run_progression, facility_run_state, encounter_progression, event_sessions, shop_sessions, combat_previews, abnormality_attempts, recruitment, record_progress; checkpoint_rewind_consumed native HashSet; enkephalin, inventory, roster, skill_fragments and uuid_manager. Do not recursively capture the previous checkpoint or all RunState.

   Production caller is Wnode_flow.rs:1237–1248 save_run_checkpoint -> Result<(), GameError>. Capture and encode are local before replacing payload/cache together. A control poll after encoding and before the first assignment handles late cancellation without splitting that replacement; capture admission does not price encoding. Three test construction sites were discovered (tests/mod.rs:139, tests/mastered_elite_sweep.rs:389, tests/combat.rs:3210), not executed. Restore consumes payload, preserves the visual-novel-session UUID counter, and clears transient owners (state.rs:439–473). Existing checkpoint-load payload.clone at node_flow.rs:1264–1269 is a separate copy seam.

3. **clone_non_battle_transaction_state** — Wstate.rs:168–196, pub(super) -> Option<Self>. Preserve the active_battle early None before any copy; use Result<Option<Self>, GameError> so control failure cannot become domain InvalidAction. Visit exactly cloned fields: player, lifecycle, active_node_content, pending_mastered_elite_sweep, node_session, run, visual_novel, visual_novel_triggers, uuid_manager, enkephalin, inventory, roster, starter_candidates, skill_fragments, skill_fragment_policy and run_checkpoint. pending_game_mode/run_power_authority are Copy. player_profile becomes Default and active_battle becomes None; neither source owner is cloned.

   Production callers are Wnode_flow.rs:783–803 with_state_transaction<T> -> Result<T, GameError>, and Wmastered_elite_sweep.rs:150–172 handle_sweep_mastered_elite_inner -> Result<BehaviorResult, GameError>. Both can use clone_non_battle_transaction_state()?.ok_or(InvalidAction)?. Wactive_run.rs:170–179 capture_active_run_transaction_rollback -> Option<ActiveRunTransactionRollback> is cfg(test), not a third production caller; its adapter/tests need explicit error handling, never silently .ok()/None. The sweep's earlier pristine_profile.clone at 168 needs independent controlled admission before copying. Its successful staging restores pristine/profile before retaining the plan (324–327).

4. **EconomyGrantState** — Wgrant_state.rs:15–46, pub(super) capture -> Self; propose Result. First copy inventory; projection is exactly inventory, skill_fragments, roster, uuid_manager, enkephalin.

   Eight production sites: Wreward.rs:95 apply_reward_session_with_context -> Result<reward tuple, GameError>; :174 apply_grant_effects_with_context and :193 preview_grant_effects -> Result<GrantExecutionResult, GameError>; Wmastered_elite_sweep.rs:226 -> Result<BehaviorResult, GameError>; Wevent_node.rs:529 plan_event_grant -> Result<EconomyGrantState, GameError>; Wshop.rs:582 purchase_offer -> Result<BehaviorResult, GameError>; Wadmin/grants.rs:169 admin_apply_grant_effects -> Result<GrantExecutionResult, GameError>; Wcombat.rs:715 staged_combat_result_state currently -> StagedCombatResultState. Convert only that wrapper to Result; its direct callers stage_committed_combat_resolution (:707) and plan_player_victory_combat_result_completion (:3169) already return Result. All other sites accept ? directly.

   Local grant plans own mutations until commit_into consumes and moves the five roots. Preview drops its plan. A poll belongs before the first commit move, not between moves; errors after an earlier inner commit require the outer draft owner. Shop currently commits grant state before fallible commit_shop_session (:601–605), so capture admission alone does not guarantee whole-shop atomicity. Reward vectors, shop offer/effect/session copies, and earlier combat planning copies are separate seams.

5. **BattleEmployeeAttemptState** — Wstate.rs:618–631, pub new(&EmployeeRoster) -> Self; propose Result. Meter only the borrowed roster plus output structure/empty processed_incapacitations set before roster.clone(). Never copy an existing processed set. Sole production caller Wcombat.rs:515 is plan_explicit_combat_node_with_state (&self, ..., roster) -> Result<PlannedLiveBattleStart, GameError> (:446–455). Adding ? drops the already constructed local battle/execution on failure. It does not consume a live attempt/UUID; those are committed later at :669–686. Earlier setup work remains consumed in the operation budget.

## Cancellation and rollback ownership

Each helper is read-only until returning an owned capture. Cancellation after native construction drops its result; native Clone/drop is not internally interruptible. Pre-admission bounds its selected owners, but this report proves no broader latency or imported-state safety.

The rollback at Wnode_flow.rs:793–800 currently clones live player_profile after an operation error, restores before, then merges profile. Do not replace that clone with fallible controlled_clone followed by ?: sticky failure could exit before restoration. Because the captured before.profile is exactly Default, move the live profile with mem::take, restore before, and assign the moved profile directly. This is value-equivalent here: PlayerProfileState::merge uses BTreeSet extend, OR and maxima (game/encounter_compendium.rs:21–56,130–132). It also avoids fresh merge allocations during rollback. Do not generalize direct assignment to a rollback whose saved profile is populated.

No capture rollback should refund shared accounting or poll midway through mandatory restoration. The existing application transaction clones a draft, executes the operation, polls and validates, then moves all roots at commit (Wapplication.rs:172–217). This owns failures after inner mutations; capture helpers cannot independently restore earlier caller changes. It does not implement the future tool Session's final output/receipt atomicity.

## Focused validation proposals, not executions

- Exercise exact/insufficient CloneSlots and work at every capture; failure must precede native copying, preserve borrowed roots and retain consumed usage.
- Vary omitted profile/battle owners while holding projected fields fixed; verify they are not traversed/copied. Active-battle non-battle capture retains its None semantics.
- Trigger cancellation after construction and operation failure after profile registration; ensure local plans drop and rollback preserves live profile without another fallible copy.
- Verify all four snapshot branches, both combat staging callers, checkpoint payload/cache pair preservation, and empty attempt incapacitation set.
- Compare ordinary/no-scope capture values and existing rollback/grant/retreat behavior; checkpoint serialization and prior copies need their own bounded checks.

## Source fingerprints

Inspected spans, sizes, timestamps, caller records and final rehash appear in the adjacent JSON. Full-file SHA256:
- core_enhanced/src/game/world/state.rs: c267443c9146bfc7d14b9bc0d959c34ef333c4bf52867a20c52d77fae09c721d
- core_enhanced/src/game/world.rs: 496b24ed05866ffc88baebaa0f7076ad46569af7b3386b7727d43bc922b72a91
- core_enhanced/src/game/mod.rs: 8bfe8d164a9b2ca9339293d3cb4b1162d43b8bb82aa18e23c87ac11e5887be5a
- core_enhanced/src/game/world/grant_state.rs: db875605a3f13331488faab157a724ae9397e5e07ce703f1d80a43c478b06ed5
- core_enhanced/src/game/world/node_flow.rs: af2248eda5e4e18bfbcc0397db6d90f62b4d79b31cbdb4672f59d758cfc91475
- core_enhanced/src/game/world/combat.rs: f43f1b97501ef60207d4a316eaa09c8d157bd4af344c9f9d81a0d686d01a3bcb
- core_enhanced/src/game/world/reward.rs: 2b5d853e59c55cb809e1925b521a36dfef45703360a88c923391497e699e4ef7
- core_enhanced/src/game/world/mastered_elite_sweep.rs: ef0a7c99c29122c81e1d6426f74ad6244ddb45f5256afff845c9c2f6a74fa4e2
- core_enhanced/src/game/world/active_run.rs: 12fd89ebe68e4949de5ca4a04a1a52270b4ae058996dadae4da7eafb8f5dd18a
- core_enhanced/src/game/world/event_node.rs: 947ceb5f2c4735846ef9df3ed9982da27884301284a7feb8c236515f73eeb861
- core_enhanced/src/game/world/shop.rs: 22c5403b0de7169d47d62f4372ae65752998c122fc0fd5a0181bce6282a45d81
- core_enhanced/src/game/world/admin/grants.rs: d89359a6b8c77dc902fb5f511adf20adb40863db1d82516f6d5a2f03cc02375b
- core_enhanced/src/authoring/clone_shape.rs: eda9a955a17dae517e3dbd5a6952cff78c65e9297d25af946b414e72b6f7481c
- core_enhanced/src/game/world/application.rs: a13098c116e6db6f2e19f390306b8b8e8915d9b3d03e4d7a425675cef404731b
- core_enhanced/src/game/encounter_compendium.rs: a340cd035b775a4c56ea6cbcfb3900098c9d3147da94993521788d4571fbde4c
- core_enhanced/src/authoring/control.rs: 8230d912809e601520f9c2acd7c4dadf0f67a186634309617bd8fa34b4173841
