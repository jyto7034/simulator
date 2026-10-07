# R03 inner clone site investigation 007

Bounded AUTORUN §8 read-only source/design assistance. No repository/source/doc/Git mutation, build or test execution. Only this new /tmp report pair is written. This is not a completion audit. Parent-reported INPUT051 results are not this helper's execution.

**Whole-Core clone preflight does not price later inner copies.** The main actionable boundaries are request normalization/cache return before the checkpoint, per-microstep BossEncounterRuntime/ActiveBossPattern copies, and copies after summon/spawn mutation which need explicit error restoration. The companion JSON gives 25 site groups, exact return types, ownership and caller paths.

The requested simulation.rs does not exist; the actual file is core/sim.rs. References below are relative to game/battle unless otherwise stated and use the first recorded source version. Full-file hashes and inspected spans are included below/JSON. Reads are per-file snapshots, not an atomic tree snapshot.

## Existing helper and concurrent changes

authoring/clone_shape.rs:24–34 now provides controlled_clone<T:Clone+CloneShape>. Inactive scope immediately executes native clone. Active scope polls, visits borrowed shape, finishes admission/copy charge, clones, then polls. Its failed post-copy poll drops the local clone; the helper does not restore earlier caller mutations. Do not call it on an already cloned temporary.

At 09:59:28Z, four inputs had drifted. Narrow follow-up at 09:59:53Z observed root replacing plan_next's self.clone with controlled_clone, removing the boundary's extra runtime clone, adding BuffDef native shape, and connecting all three buff handlers. These are recorded implementation observations, not fresh coverage/test approval. Earlier site evidence remains in JSON.

## Boss runtime and definitions

| Sites | Actual ownership / return | Smallest connection |
|---|---|---|
| plan_next:395; boundary:1876 | Two deep runtime copies each iteration; both functions return Result | Borrow boundary runtime for plan_next(&staged); control only plan's mutable next. Root subsequently made this change. |
| enter_phase:526 | CompiledBossEncounterPhase: String + Vec of Arc bindings + transition Vec; Result<()> | Keep definition Arc locally, borrow phase; alternatively meter exact phase copy. |
| advance_actor_once:1082 | CompiledBossControlNode with owned task/transition/candidate/route Vec; Result<bool> | Node can borrow from already local behavior Arc. |
| advance_pattern_once:1242,1291 | ActiveBossPattern: state/tasks/native maps/sets plus definition/frame Arcs; Result<bool> | Control both copies, or retain only needed Arcs/scalars and borrow decision state. |
| transition:442,1266 | Transition IDs/condition/loop guard own Strings; Result callers | At 442 only timing/target index are needed: copy scalars. At 1266 borrow from stable definition Arc or control exact record. |
| admit_state_tasks:639–786 | Task IDs, summon definition/batch IDs, placement zone String, occupancy Vec, route/removal-signal Strings; Result<ActiveBossState> | Admit output Vec before constructing records; copy source fields through control. |
| resolve_target_query:2169,2172 | Query filters/order_by Vec<Copy>; Result<Query> | Borrow query from literal/frame with an explicit lifetime, or control query copy. |

Compiled records must not be confused with their Arc handles. Definition getters return &Arc for boss/summoned definitions (boss_runtime_data.rs:844–884). behavior clones at :535/549/585/831/849/1077/1323 and pattern/frame clones at :1219–1220 retain handles only. CompiledBossPatternCandidate itself also owns optional cooldown.group_id:String, so copying a whole candidate is not Arc-only. CompiledBossPatternTransitionTarget contains only usize/unit values; its clones :1358/1370/1373 are fixed despite lacking Copy derive. BossTargetLock:945 likewise has fixed ID/owner/policy.

from_setup:348 returns Self and copies actor unit_ref Strings plus spawn-zone/route vectors (:355,380–381), before any Core checkpoint. BattleCore::new_from_database also returns Self (core/mod.rs:438–446). A production fallible constructor or projected borrowed admission in events/combat.rs before :219 is required; that parent returns Result<ResolvedCombatSetupPlan,GameError>. Existing spawn counts/board/obstacle charges :196–215 do not measure these nested copies. Do not return a missing/default boss runtime on control failure.

route_cells_for_actor:1431 returns an owned Vec. Its three callers only inspect cells: poll_actor_once:968, select_rejoin_route_index:1408, boss_movement_directive:2001. Returning &[Position] avoids a deep copy and avoids propagating Result through Option/MovementDirective APIs. Keep the route selection/order unchanged.

summon_task_views:1494 deep-clones every live summon task and sorts the output. capture_boss_summon_requests already returns Result, so a private fallible materializer fits. summon_request_is_active:1529 returns bool but unnecessarily builds that entire Vec; use a direct borrowed identity scan. Control failure must not become false and cancel a valid request.

Remaining ID copies into cooldown/batch/signal tables (:1209,1318,1367,1583,1614,1628) need admission before insertion. settle_summon_task returns Result; note_summon_batch_cleared and note_signal_emitted return (). Private Result propagation through reconcile_summon_batches and BossRuntimeTransitionPlan::commit is direct. commit currently assigns next_runtime before applying effects (:1759–1785), so a new signal-copy failure must escape to the staged boundary before its final swap. boss_actor_adapter_view copies two Strings into an Option DTO (:1810–1811); borrowing IDs is smaller than a control-to-None fallback through Wave callers.

The boundary's committed self is untouched until :1889; next-runtime/effects/staged copies can be discarded on Result failure. This protection applies to that actual caller path, not every direct fixture call.

## Spawn normalization, reservation and settlement

SpawnRequest owns ID/source Strings, optional occupancy Vec, and placements:Vec<SpawnPlacement>; each placement owns occupants:Vec even though each occupant is Copy. Request/placement copies are therefore deep. SpawnRequestSettlement may own unit_ids:Vec, MutationFailed.game_error_code:String, or cancellation ID Strings.

normalize_request:354 returns Result<SpawnRequest,SpawnRequestSettlement>. Its request.clone at :367 occurs before execute_spawn_request's whole-Core preflight at :622. identity_request:350 returns SpawnRequest and falls back to request.clone for every normalization error. **Do not insert control failure into this domain Err channel.** Use an outer control Result or a dedicated internal error distinction; fallback remains only for ordinary invalid requests. Invalid request identity, canonical order and cached domain rejection must stay unchanged.

Other copies before that checkpoint include cached settlement return :608 and reservation request arguments :615/:620. reserve_request:398 already returns Result<(),SpawnKernelError>; control failures can use existing ExecutionInterrupted before key insertion. bind/cancel/reject request duplicates and key Strings (:408,471,513,530,545) can be controlled or replaced with ownership moves plus a pre-copied ID. Private settlement:417 currently returns Option; use borrowed settlement plus a fallible copy, not failure-as-None.

settle_trigger:477 returns Result and duplicates trigger/request IDs and the cached result. Control each real duplicate before its ledger update. Public compendium recording returns CompendiumCandidateError, which has no control variant, and its getter returns Vec (:554–579,733–742). A private fallible adapter or preadmission at its Result caller is needed; fabricating IdentityConflict/OrderConflict is wrong.

execute_spawn_request performs the final settlement.clone at :660 **after placement callbacks mutate self**. A newly fallible copy there must restore before explicitly. Retain the checkpoint until the cache/result copy succeeds; do not add a bare ? after mutation. The domain rejection branch already moves before back into self at :655, so the implementation must handle checkpoint ownership without an unmetered extra clone.

The existing checkpoint is deliberately taken after reservation. Source test authoring_spawn_interrupt_restores_mutations_and_stops_before_next_placement (:1027–1066) expects the ledger entry to remain Pending after interruption. Therefore inner rollback is not the pre-request state; the authoring outer candidate owns restoration of the complete prior session. Preserve this distinction and keep control interruption out of cached rejected settlements.

## Summon transaction

settle_boss_summons:130, capture:152 and settle_one:177 all return Result. Before commit_summon_batch's Core preflight, they copy sorted request IDs (:141), trigger/key Strings (:167/170/232), pending/view (:182/185), and request arguments (:196/245/274). Control source copies before collect/push/argument evaluation. Destructuring an already owned pending record may remove the second view clone.

Definition retrieval :225 and behavior retrieval :473 clone Arcs only; definition.combat_profile.clone at :365 is a separate deep copy per member. commit_summon_batch also copies route cells :330, request :390, returned unit IDs :404, and counter/signal/batch ID Strings :419/431/443/445. Its spawns/member vectors are fresh allocations and still need admission before construction; a clone helper alone does not cover them.

commit_summon_batch returns Result and creates before at :309. Explicit restore branches protect bind/execute/settlement/finish-trigger failures. After execute_spawn_request succeeds, bare ? for new controlled copies can bypass restoration. A fallible body with one restore-on-error exit, or explicit restoration at each new failure, preserves the checkpoint. Existing task settlement :465, behavior lookup :479, attach :482 and recording :491 also have bare ? paths; current outer boss staged boundary protects the session, but do not advertise complete local rollback from the mere presence of before.

reconcile_summon_batches:505 copies each batch snapshot (:514) and later a counter-key String (:600) after mutations. It runs inside the staged boss boundary. Spawn-zone candidate copy :705 is in Result<Vec<Placement>>; connect directly. summon_spawn_request:764 and summon_occupancy_policy:845 are infallible materializers of Strings/Vec and need private Result plumbing or caller preadmission. The insufficient-space branch recomputes all placements at :281; reusing the already known count can eliminate that work.

## Actual sim.rs sibling sites

All three direct BuffDef copies are in process_event_inner -> Result<()>: ApplyBuff :3196, Tick :3305, Expire :3395. BuffDef owns name:String, with other fields Copy. Use get().map(controlled_clone).transpose()?; root subsequently connected all three. Copying only used fixed definition fields is another allocation-free option. Missing definition remains ordinary Ok(()), distinct from control failure.

Two other direct fallible seams are AutoCastStart's full SkillDef clone :2880 and build_skill_step_commands' DamageSourceSnapshot template clone :2098. The former bypasses cloned_skill_definition (core/mod.rs:322–330), which already charges before clone. SkillDatabase::get_by_id returns &SkillDef, not &Arc. The latter needs the existing native snapshot visitor (bonus Vec plus source Arc).

enqueue_scenario_events:1520 clones deep events after only outer-length work charge; activation is cloned again :1524. Startup obstacle vector :1831 is copied before per-obstacle charges. Both functions return Result and can control source copies directly.

record_spawned_units:1395 returns Result and materializes identities/compendium IDs into a Vec before logging; its closure can propagate fallible element copies. Terminal helpers copy outcome/finish IDs and result-stat owner (:1278/1308/1336/1709); some return Option/plain values and require explicit caller plumbing. Do not set finalized then return an uncaught controlled-copy failure.

Skill/cast/deferred/log argument Strings and presentation Options remain actual owned copies. In particular register_persistent_tile_area receives already cloned footprint/IDs (:2612–2617), and impact context receives targets.clone (:2544): callee admission happens too late to price these argument copies. area.clone is fixed-value data; cast_origin.clone and effect.source().cloned() retain Arcs only. Private infallible leaves may stop on a latched control error only when their actual caller polls before acceptance; process_event explicitly polls after its inner body (:2655–2656). This report does not establish every command/observer caller's polling coverage.

## Focused validation candidates (not executed)

- Exact budget versus one-less at normalization, cached replay and runtime planning: no native copy/callback after failed admission; same ordinary normalization/replay results.
- Long nested placement occupants/IDs and large cached Spawned.unit_ids: limits fail before first allocation, not at later Core checkpoint.
- Cancellation or work exhaustion during final settlement copy after a successful placement: restore units/log/IDs/queue; retain only the documented Pending baseline; no cached rejection.
- Summon failure after spawn success but before unit-ID/result copy: restore UUID continuation, counters, members, triggers and parent runtime through the actual transaction owner.
- Large compiled phase/control children versus large shared Arc pointees: charge owned children only; borrowed alternatives preserve chosen transition/route/target.
- Long buff names, AutoCastStart definitions, repeated damage templates and persistent-area argument copies exercise the actual first-copy seams.

Root owns implementation choices, actual tests and acceptance. No optional trace setting may change these mandatory charges; trace overflow remains gap-only. This bounded inventory does not establish broad runtime/R23 safety or whole-implementation completion.

## Source hashes and inspected spans

Hashes cover complete source files. Separate versions preserve concurrent root edits. Line spans refer to their own version.

| File (under core_enhanced/src) and spans | SHA256 |
|---|---|
| game/battle/core/boss_runtime.rs 1–470, 478–641, 730–880, 930–950, 1061–1095, 1167–1380, 1430–1530, 1748–1897, 2140–2178, 641–730, 1089–1167, 1400–1431, 1529–1639, 1947–2020 [v1] | `9e3b7232449ffa7bf6156341ffed663d3f839d116f1278e677a5fa0e6c292c5a` |
| game/battle/core/boss_runtime.rs 389–400, 1871–1880 [v2] | `8957d1fab18b35ff72a0a34bdb930d8daed259fac1a4164df64e9d3b65bd0705` |
| game/battle/core/spawn_kernel.rs 1–350, 349–724, 725–748, 935–968, 1010–1070 | `d36bbb0156b8a9fb1bc1a34483b4b2eb3d87e05c322d6fef636ea831219d871b` |
| game/battle/core/summon_runtime.rs 1–295, 296–504, 505–640, 661–813, 840–892 | `9947f790d3ee32a47c863d32e5d4dcfb800bc6ab6e2fb84e3129ccb8c330a957` |
| game/battle/core/sim.rs 3150–3225, 3270–3305, 1254–1339, 1516–1546, 1819–1860, 2645–2692, 3390–3420, 3298–3314, 3387–3442, 330–410, 458–527, 695–716, 895–930, 1394–1475, 1698–1722, 2041–2118, 2280–2375, 2838–2895 [v1] | `544b88ba72e8184648958b2042e5e38f319a7a83a708484ca1d1f97505c71e8b` |
| game/battle/core/sim.rs 3193–3210, 3308–3325, 3404–3421, 587–643, 2234–2270, 2397–2418, 2508–2568, 2600–2635 [v2] | `04ac837994c1cdebccf3b01177d058ea88f73491fca14be0e55818f326ccd302` |
| game/data/boss_runtime_data.rs 99–153, 513–781, 844–895, 155–165, 228–285, 320–371, 435–452, 490–513 | `3df26fd5e0950e4bb1fd76ae0ba766871717fe8adeb5c37466301a2aac88ef8d` |
| game/battle/core/mod.rs 370–460, 306–349 | `d9b6f0b7e48cdd9a8d0cf2e72ddb678ac58c2a260e6cc7cb24d3dcacdae276ec` |
| game/battle/core/wave_runtime.rs 232–252, 341–370 | `b2b76f5a779f2949ba1c6beb22442c4007757eeaf6bddd581518f49cb392e427` |
| authoring/clone_shape.rs 1–35 [v1] | `de81066710b1235a0e1a6bda0747ba04e52b1faac99c4d5af0dc6e51b721c8ca` |
| authoring/clone_shape.rs 21–35 [v2] | `cb9b64ad8dd10c5f2fdaebbddf3d7c5c64ac47dfd61603521f3aeaac6a406395` |
| game/events/combat.rs 170–231, 255–305 | `d88719417886eea018ae08a66c750884f163051f160d0b9b21ece3c1eddffc66` |
| game/battle/buffs.rs 58–91, 173–179 [v1] | `4b501650a51b5835814fa3a2e689ba19e8de1780144f1c29ddf07f4b28ec783b` |
| game/battle/buffs.rs 64–86 [v2] | `742f04a540e0794e52479f2b34735ee72e9d5ca3fc4239a760af4482fd3aad19` |
| game/data/skill_data.rs 84–99, 128–143 | `8a1cccf346cc3b45755a92a45e41be92deb5a0e641a0f519977dc6a2c7c62fb5` |
| game/ability.rs 614–638, 723–765 | `c7cec94493e6de6d2ebb005941f3f80a276396e3803afb07f5d5a884d65ab195` |
| game/ability/authoring.rs 43–75 | `c4ca67ceac4502c2c31c8c91a584e09dff3c87fd8976269d436dbddbc2e08e1a` |
