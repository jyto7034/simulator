# R03 world leaf ownership investigation011

Bounded read-only design investigation under AUTORUN §8; not a completion audit. No repository/source/Git changes or cargo/build/test execution. Scope is the requested GameCoreState roots and direct ownership dependencies. RunState, ActiveBattleSession and RunCheckpointState are not reviewed as whole roots; their shared child types appear only where a pending sweep owns another copy.

Primary capture: 2026-09-10T09:31:30.245403Z; direct dependency capture: 09:34:23.577689Z; final support capture: 09:36:29.445600Z. These are non-atomic reads in a concurrently edited workspace. The JSON records every initial/close SHA256, drift and inspected span; successor implementations are not silently treated as reviewed.

## Recommended split

Use explicit native field/variant visitors for the owner wrappers, preserving current public types and ordinary Clone. The closed serializer can cover several small fully owned children; it must not cover an entire owner containing sets, maps, shared metadata or skipped native state.

The two concrete additional table boundaries are ShopSessionState.purchased_merchandise and MonolithRunState.remanifestation_nodes. The Shop set is insert-only in the inspected opaque application flow. The Monolith map is not: ordinary gameplay inserts, removes and drains it, and no retained storage-history counter was found.

All paths below are relative to core_enhanced/src/.

## Requested roots

| Root | Ownership / Serialize | Minimal connection |
| --- | --- | --- |
| PlayerInfo | Debug/Clone/Serialize/Deserialize; id:Uuid, name:String. No table/Arc/skip. game/world/state.rs:53–66. | Eligible closed leaf, or struct fields id,name. |
| LifecycleContext / GameState | LifecycleContext has only private phase and no Serialize (state.rs:113–134). GameState derives Clone/Serialize but **not Copy**; NodeConfirm/InNode own MapNodeKindId(String). Other variants hold fixed enums/UUIDs. resources/state.rs:16–62; map/types.rs:17–19. | Native LifecycleContext {phase}; phase may use a closed GameState leaf. Visit by borrowing phase(), never by snapshot() which clones it. Do not account the whole enum as a Copy scalar. |
| NodeSession | Clone/Serialize; node_id,kind_id,category,payload. map/session.rs:10–16. Payload variants contain fixed records and optional owned IDs/Strings, no loaded definition owner (map/types.rs:56–80). | Eligible closed leaf, or these four native fields. Keep kind_id and payload String costs. |
| VisualNovelRuntimeState | Clone/Serialize; active:Option<VisualNovelSession>, queue:Vec<VisualNovelSession>. Session owns session_id/source/vn_id/current_scene_id/committed_choice_id/blocking/presentation_mode (visual_novel.rs:13–44). | Eligible closed leaf or native active,queue. Source carries only fixed IDs/UUIDs; data IDs are String newtypes, not pointers to VN definition graphs. |
| VisualNovelTriggerRuntimeState | Clone/Serialize; **two public raw HashSets** (visual_novel.rs:55–75). | Native field handling for persistent_consumed and checkpoint_rewind_consumed with the separately proven opaque tombstone-free envelope; then visit every consumed key. Consumed key {trigger_id,scope} is a closed leaf; scope is Copy. Preserve raw public fields. |
| RunPowerAuthorityState | Copy/Clone/Serialize; power_model_version:u32, run_peak_certified_power:u64,current_floor_power_anchor:u64 (state.rs:325–331). | Existing Copy-bounded helper is sufficient; alternatively explicit three fields. |
| pending MasteredEliteSweepCommitPlan | Clone only; result,grant_state,encounter_progression_after,node_completion (world/mastered_elite_sweep.rs:21–27). | Explicit four-field visitor, with the native boundaries below. It is substantially more than a result DTO. |

VN/event ID definitions are transparent String newtypes (data/visual_novel_data.rs:10–40; visual_novel_trigger_data.rs:14–16; event_data.rs:14–58). OnceLock/HashMap indexes elsewhere in those database modules are not owned by these IDs. No definition lookup or graph expansion is part of their Clone.

The simple VN/Event children only skip serialization of absent Options; no present heap owner is hidden by those attributes. This does not justify accepting arbitrary future skipped fields into the whitelist.

## ActiveNodeContent: all seven variants

ActiveNodeContent derives Debug/Clone, not Serialize (resources/selection.rs:223–232). Use an exhaustive native enum visitor.

- **Shop:** native ShopSessionState fields are id,name,uuid,context,node_id,offer_count,refresh_policy,sell_policy,refresh_uses,page_index,offers,purchased_merchandise (selection.rs:37–51). All except the final set have a closed owned shape. ShopOfferState (26–35) contains merchandise and tags:Vec<String>; ShopMerchandise is a String-bearing enum, ShopRefreshPolicy is Copy, and ShopSellPolicy owns Option<Vec<String>> (data/shop_data.rs:14–27,77–89). Reuse these as small leaves; give purchased_merchandise native set accounting.
- **Reward:** RewardSessionState has no Serialize. Visit stage_uuid,mode,rewards,selected_reward_uuid,can_skip (selection.rs:64–71). RewardOption owns four Strings, one UUID and Vec<RewardEffect>; effects own optional-by-variant String identifiers and fixed quantities/policies, not item metadata (reward.rs:19–67). RewardOption/RewardEffect are closed-leaf candidates.
- **Event:** EventSessionState is a closed-leaf candidate: node_id,event_id,current_scene_id,committed_check,started_combat. Committed check owns IDs, fixed numbers/enums, Vec<EventCheckModifierComponent> and optional EventChoiceKeyEvidence; equipment evidence stores UUID/definition-id Strings rather than EquipmentItem/Arc metadata. Its directly nested graph has no map/set/Arc (selection.rs:123–187).
- **SavePoint:** no Serialize; explicit node_id (80–88).
- **Maintenance:** no Serialize; explicit node_id (91–99).
- **HeadquartersContact:** no Serialize; explicit node_id,options,shop_pool_id (102–107). options is Vec<HeadquartersContactOption>, a Copy enum (map/types.rs:45–50).
- **CombatBattle:** no Serialize. Visit primary_abnormality_id,encounter_id,node_type,mission_variant,abnormality_uuid,outcome_id,verdict,winner, the conditionally present event_log, result_stats,reward_mode,rewards,grant_node_bonus,advance_encounter_progression,participant_results,employee_conditions,compendium_candidates (selection.rs:201–221). Preserve the exact cfg(any(test,debug_assertions,feature="test-fixtures")) on event_log.

CombatBattle's children divide as follows:

- BattleOutcomeId is a String newtype; ParticipantBattleResult is Serialize with fixed fields; BattleResultStatsDto and its DTO descendants are Strings/Vec/fixed records (battle/types.rs:35–81; battle/result_stats.rs:20–80). These are closed-leaf candidates.
- employee_conditions is EmployeeRoster and must reuse its native owner visitor, not serialize its metadata graph.
- CompendiumCandidate has no Serialize; visit id,kind,base_uuid (core/spawn_kernel.rs:722–733). Its two-variant kind is not currently Copy either: use an exhaustive unit-variant visitor, not a Copy-bound invocation.
- event_log must use its native log visitor. BattleEventEntries is Arc<Vec<BattleEventLogEntry>> (battle/event_log.rs:76–101): ordinary Clone retains an Arc handle. BattleEventLog also owns version and nine serde-skipped fields: floor_seq,latest_seq,earliest_time_ms,latest_time_ms,encoded_entry_array_bytes,accounted_entry_count,observed_entry_count,unaccounted_encoded_upper_bytes,sequence_exhausted (397–419). The source Arc inside an entry is also skipped (804–808), but cloning the entries Arc does not clone those entries/pointees. COW preparation is a separate path; do not force it during clone measurement.

## Pending sweep: nested owner boundaries

Use these exact immediate owner fields rather than adding Serialize to production owner structs:

1. **EconomyGrantState** {inventory,skill_fragments,roster,uuid_manager,enkephalin}; Clone only, world/grant_state.rs:20–36. Inventory/EmployeeRoster/SkillFragmentInventory/UuidManager already have native field visitors in the captured source. Inventory owns three private OwnedHashMap wrappers and shared Arc<EquipmentItem>/Arc<ConsumableItem> metadata; SkillFragmentInventory and UuidManager own BTreeMaps. Enkephalin is just amount:u32 and Serialize (resources/economy.rs:3–6). Visit each retained grant copy, even if its contents equal current state.
2. **result:MasteredEliteSweepResultDto** is a closed-leaf candidate at its current shape (behavior.rs:1203–1217): sweep_uuid,node_id,encounter_id,reward_source_id,fragment_id,reward_branch,base_rewards,exclusive_effects,enkephalin_after,inventory_diff,skill_fragment_diffs,employee_experience_diffs. InventoryDiffDto has Vecs of owned item/material DTOs, not Arc metadata (inventory.rs:30–43,79–93,117–146,167–174); its weapon_profile uses the already-reviewed WeaponCombatProfile leaf. ConsumableEffect is a fixed-field enum (data/consumable_data.rs:35–66). Reward/diff leaves are at reward.rs:19–91.
3. **EncounterProgressionState** needs native fields completed_reward_source_ids,mastered_elite_cycle,normal_boss_cycle,monolith,boss_omen (encounter_progression.rs:127–135). completed_reward_source_ids is BTreeSet. Each EncounterCycleState is cycle_index plus two Vec<String> and can be a closed leaf (19–25). Monolith and BossOmen are not whole closed leaves.
4. **CombatResultNodeCompletion** is Clone only and needs an exhaustive native enum match (world/node_flow.rs:89–112):
   - NodeCompleted {map,progression,completing_node_id,boss_omen,view}
   - FloorAdvanced {next_map,next_progression,run_progression,facility_run_state,encounter_progression,view}
   - RunComplete {map,progression,run_progression,view}

The last enum owns independent RunMap/MapView/MapProgression values; it does not borrow the live RunState. Their inspected definitions are String/Vec/fixed graphs (map/types.rs:164–190,347–361,591–608,686–699; map/progression.rs:18–42). They are suitable candidates for their own reviewed leaves, rather than grounds to include the excluded whole RunState. FacilityRunState is a BTreeMap<String,FacilityPoolRunState>; the pool has cycle,remaining_template_ids:Vec,used_template_ids:BTreeSet (map/facility.rs:209–219), requiring native table visitors.

BossOmenRunState must visit provisional,active,placed_source,forced_boss,cycle_index,remaining_chain_ids,completed_chain_ids. The final field is BTreeSet<BossOmenChainId>; its other children are String/Vec/fixed records (boss_omen.rs:22–73). Do not send this whole owner through a serializer sequence path.

MonolithRunState must visit progress_by_abnormality,remanifestation_nodes,residuals,next_residual_sequence,exchange_state,exchange_shop_guarantee_issued,next_floor_shop_guarantee_pending,corrupted_monolith_acquired (monolith.rs:91–102). The first field is BTreeMap; the second is a public raw HashMap. MonolithProgress, MonolithRemanifestationNode and MonolithResidual are individually closed leaves (17–27,66–79); the owner is not.

## Shop set provenance: bounded confirmation

Whole-source field-reference searches found:
- fresh HashSet::new in build_shop_session (world/shop.rs:53–71);
- inserts at 557–559 and 597–599;
- shared contains reads; no ordinary remove/retain/clear/reserve/drain or returned mutable-set method;
- commit copies the session into run.shop_sessions and moves it into active content (31–49). remove_offer deletes from offers:Vec, not from the purchased set (selection.rs:53–60).

Thus fresh opaque sessions, their native copies, and fresh serde-deserialized checkpoint sessions preserve the insert-only invariant. Checkpoint payload includes ShopSessionState values (state.rs:471); its byte parser constructs fresh values (world/active_run.rs:115–124). This narrow link does not constitute a review of the full checkpoint graph.

ShopSessionState and its set field are public, so external code can still mutate raw storage arbitrarily. The capacity proof is restricted to the closed CoreApplication construction family; preserve the field/API and do not declare a blanket HashSet CloneShape implementation.

## Monolith map: actual unresolved storage credit

begin_remanifestation inserts at monolith.rs:190; resolve_remanifestation_victory removes at 213; resolve_remanifestation_failure removes at 266; clear_floor_local_remanifestations drains at 282. These are real opaque gameplay paths: world/node_flow.rs:549–552; world/combat.rs:3017–3020,3063–3070; world/state.rs:1491–1494.

No retain/reserve/direct map.clear call or raw mutable-returning method was found in the inspected field references. The public raw field itself permits arbitrary mutation/import in the public type, and whole Clone/Deserialize are derived. Native drain/removal means current capacity is not a storage-history proof even in the fresh application family.

No existing lifetime insertion/reservation counter or capacity high-water exists among the owner's fields. uses_remaining is set by purchase's initial_uses argument (160), decremented before insertion (183), and overwritten to zero on failure (272). residuals/next_residual_sequence track selected final-stage rewards (221–239), not all table insertions. Neither residual count nor progress map length is an established storage credit.

A policy-sensitive semantic bound involving initial_uses and import reservation might be investigated separately; it is not proved here. This report recommends no arbitrary multiplier. Preserve the public raw field/struct-literal contract and supply a separately proved external owner-history/provenance seam before relying on this table in bounded whole clone. That seam must accompany owned plan/capture copies and validated imports; measuring serialized shape or rebuilding the map does not resolve the native-clone requirement.

## Serializer gate and practical limits

The captured clone_shape.rs has native Vec/Arc/OwnedHashMap/BTree visitors and an explicit closed-leaf whitelist. It rejects SerializeMap and collect_str (earlier read spans 404–427). **HashSet and BTreeSet serialize as sequences**, so that rejection alone does not prevent set storage undercount. Pinned serde_core-1.0.228 ser/impls.rs:221–225 explicitly applies its sequence implementation to HashSet. Whitelist review must exclude these owners by actual type ownership. Root confirmed neither ShopSessionState nor MonolithRunState had been added as whole leaves.

Use the existing explicit struct/enum/newtype patterns at private owner boundaries; their exhaustive patterns expose new direct fields to compilation. Keep non-Serialize owners non-Serialize; no wire-format change is needed. Preserve conditional fields and skipped native state, and count Arc handles without resolving or serializing pointees.

Eligibility above is an ownership-shape proposal, not proof that newly registered leaf serialization or all derived Clone paths have passed tests. The existing WeaponCombatProfile leaf and other previously reviewed scalar leaves are reused rather than independently re-audited in full. Omitted whole RunState/ActiveBattle/Checkpoint graphs, total heap coexistence and all runtime paths remain outside this investigation.


Close rehash: 2026-09-10T09:38:55.702816+00:00. Of 37 inputs, three drifted during root work; their successor implementations were not reviewed:

| Source | Initial SHA256 | Close SHA256 |
| --- | --- | --- |
| authoring/clone_shape.rs | 128a52d55e3bba9800039128d209b1e50ccd9f87e51d68d2d64f84c49fb0229e | b46e1c87e744adc5515ab434a0f8def82eb501063a2ff14ec58a45738ded397a |
| game/battle/event_log.rs | 82975ec82b87e469eaa206e46a2042a51a12ee4b6adf0375148ab64ace97d7b8 | 49ddbf7545b8939ab4ae2b3cd42a87e1b42fbe97c7dbdfec67201daf33b63619 |
| game/battle/result_stats.rs | bd0259e0daf942a9dca490962b40fb374ce5c7891fccf14f5754e95bac10c543 | dab57d5d702c078ee000c7ac40933982471ce7e42001ec2d9cdc7c288732ceea |

The remaining 34 inputs had identical initial and close hashes. Exact hashes and scoped spans for all 37 inputs are in the JSON companion.
