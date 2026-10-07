# R03 owned leaf clone shapes 005

Bounded read-only source/design assistance under AUTORUN §8. Root remains sole implementation/artifact writer. This helper ran no source, builds, tests or Git operations and changed no repository files. Only the new /tmp report pair is written. This is not a completion audit. The JSON companion contains a 30-type exhaustive direct-field map.

The minimum split is: native owner visitors for four OwnedHashMap tables; handle-only treatment for inventory metadata Arcs; bounded borrowed serialization for closed employee/profile/loadout leaves; separate admission for GrowthStack's raw HashMap. Serialization alone is not a table-storage bound.

## Inventory: exact owned boundary

| Type/field | Actual clone owner | Minimum visitor |
| --- | --- | --- |
| Inventory.equipments | EquipmentInventory | Native child visit |
| Inventory.equipment_materials | EquipmentMaterialInventory | Native child visit |
| Inventory.consumables | ConsumableInventory | Native child visit |
| Inventory.reserved_slot_count | private usize | Fixed scalar; not actual reserved Vec storage |
| EquipmentInventory.items | private OwnedHashMap<Uuid,OwnedEquipment> | Admit clone_bucket_bound before scanning; scalar UUID keys; visit values |
| EquipmentMaterialInventory.stacks | private OwnedHashMap<String,u32> | Admit table bound; each key's String bytes; scalar amount |
| ConsumableInventory.items | private OwnedHashMap<String,ConsumableStack> | Admit table bound; each owned definition-ID key; visit values |

Sources: inventory.rs219–226,457–460,508–571,587–590,702–713. The three child maps are private in the same inventory module, so a crate-private Inventory visitor can reach their native map histories without exposing a mutable/raw table. Existing public iterators expose live entries, not clone bucket bounds.

OwnedEquipment has instance_uuid:Uuid, meta:Arc<EquipmentItem>, equipped_to:Option<Uuid>, enhancement_level:u8. EquipmentItem is EquipmentMetadata. ConsumableStack has meta:Arc<ConsumableItem> and quantity:u32; ConsumableItem is ConsumableMetadata. Their derived Clone copies fixed records and increments the metadata Arc reference. It does **not** clone definitions.

Do not call Serialize on Inventory/EquipmentInventory/ConsumableInventory/OwnedEquipment/ConsumableStack as a clone leaf: serde's Arc serialization traverses metadata pointees. Equipment metadata owns id/name/cannot_unequip_reason strings, triggered_effects, ability_activations, optional weapon profile, event modifiers and event-choice keys. Traversing these would charge the shared definition as if independently copied. Consumable metadata similarly has strings/effect but the owned stack clones only its handle. Materials retain only String→u32; EquipmentMaterialMetadata is not an Inventory field.

Count each map key separately even when equal to an ID inside shared metadata. Do not invoke DTO builders: EquipmentItemDto/ConsumableItemDto/material DTO factories deliberately clone definition display fields and weapon/effect values and are a separate output boundary. ConsumableRemoval, when separately cloned, is one Arc plus removed_quantity/remaining_quantity u32; it is not retained inside Inventory.

## Employee root: all fields

EmployeeRoster has one private employees:OwnedHashMap<Uuid,Employee> (employee.rs636–685). Admit its clone_bucket_bound before values(). Its public len()/iter() cannot expose deleted-table history. Root can place the native visitor beside this owner.

Employee's twelve fields (356–373):

| Fields | Type/storage |
| --- | --- |
| uuid | Uuid |
| employee_definition_id, name | Two independently owned Strings |
| profile | EmployeeProfile |
| level, experience | Two u32 |
| condition | EmployeeCondition |
| loadout | EmployeeLoadout |
| skill_fragments | SkillFragmentLoadout |
| active_consumable_modifier | Option<ActiveConsumableModifier> |
| combat_profile | EmployeeCombatProfile |
| psychology | EmployeePsychology |

Employee has no metadata Arc. It owns copied identity/profile/skill strings. A whole-Employee Serialize walk would still miss native bucket storage under GrowthStack, so split that owner before applying safe leaf traversal.

EmployeeProfile (98–106) owns role/background:String; personality:EmployeePersonality{risk,social}; combat_aptitude:CombatAptitude; event_aptitude:EventAptitude; base_combat_stats:{max_hp:u32,attack:u32,defense:i32,magic_resist:i32}; basic_attack_speed_percent:u32. Everything except the two Strings is fixed scalar/Copy. EmployeeCondition contains physical_damage:u8 and trauma:u32.

EmployeeDefinition (127–133), used as an owned starter-candidate value, contains employee_definition_id/name:String, starter_eligible:bool, initial_loadout:EmployeeInitialLoadout and profile:EmployeeProfile. InitialLoadout owns equipment_ids:Vec<String> and baseline_skill_fragment_ids:Vec<SkillFragmentId>. SkillFragmentId and SkillId are transparent String wrappers, with borrowed as_str() access. These are safe borrowed-Serialize leaves after sequence admission; there is no implicit database traversal.

## Loadout, fragments, consumable modifier and psychology

EmployeeLoadout contains only item_slot:ItemSlot. ItemSlot's private layout is exactly:

- ByType { weapon, armor, accessory: Option<EquippedRef> }.
- Any3 { items: Vec<EquippedRef> }.

EquippedRef is Copy: instance_uuid/base_uuid:Uuid and equipment_type:EquipmentType. ItemSlot Serialize is a safe owned-shape leaf. Native matching can avoid serialization overhead and count Any3's actual len. Do not assume len≤3 solely from the type: checkpoint validation checks that rule separately. ItemSlot::iter() returns a heap-allocated Box iterator (item_slot237–246); a preflight should use owner-local layout matching or borrowed serde rather than allocate that adapter.

SkillFragmentLoadout owns baseline_ids:Vec<SkillFragmentId> and active_fragment_id:Option<SkillFragmentId> (skill_fragment576–604). Use baseline_ids()/active_fragment_id(), which borrow. equipped_ids() allocates/clones a combined Vec before returning and is unsuitable for a pre-copy meter. Count repeated ID occurrences, including a repeated active ID, as separate owned copies.

ActiveConsumableModifier (employee317 onward) owns source_item_uuid:Uuid, definition_id/name:String, tier:ConsumableTier, duration_policy:ConsumableDurationPolicy, remaining_combat_nodes:u32 and effect:ConsumableEffect. All eight current ConsumableEffect variants are scalar-only: DeathPrevent, ReducePhysicalDamage{amount:u8}, TraumaMitigation{percent:u32}, BattleHpSetup{bonus_percent:u32}, DeployCostReduction{percent:u32}, DefenseMitigation{percent:u32}, InitialSkillCharge{percent:u32}, OffenseBoost{attack_bonus_percent:u32}. Duration is NextCombatNode or CombatNodes(u32). This modifier is a safe borrowed-Serialize leaf; unlike a stack it owns its two strings and effect.

EmployeePsychology (employee_psychology47–51) owns trust_score:i16, memories:Vec<PsychologyMemory>, recent_events:Vec<PsychologyRecentEvent>. Memory fields: kind, subject, occurrence_count:u32, intensity:u8, first_source, last_source, remaining_nodes:Option<u32>. Recent-event fields: kind, source, trust_delta:i16. Subject is EmployeeSelf; source is NodeCompletion{node_id:Uuid}. These record children own no Strings/maps/Arcs. PsychologyEventKind is currently empty; no future event semantics should be invented. Meter the actual two Vec lengths rather than replacing the real fields with an assumed zero. Borrowed serialization is safe.

The sibling SkillFragmentInventory (skill_fragment20–40) is not inside Employee's loadout, but is a directly relevant run owner: private stacks:BTreeMap<SkillFragmentId,SkillFragmentStack{count:u32}>, progress:BTreeMap<SkillFragmentId,SkillFragmentProgress{upgrade_level:u8,awakening_progress:u32,awakening_available:bool,awakened:bool}>, fragment_dust:u32. Both maps own separate key strings even for the same ID. Owner-local map lengths precede traversal. stacks()/progress_entries() borrow keys and return references/scalar copies; they do not expose O(1) map length through their declared opaque Iterator API. Whole-value borrowed Serialize is safe for work/shape because it contains no Arc, hidden skipped owner or database.

## Combat profile: closed owned leaves and the one table exception

EmployeeCombatProfile has battle_profile:UnitCombatProfile and growth_stacks:GrowthStack (employee311–314). UnitCombatProfile has all fifteen fields below (battle/types393–408):

| Field | Type |
| --- | --- |
| stats | UnitStats, fixed Copy |
| basic_attack | BasicAttackDef |
| basic_attack_speed_percent | u32 |
| weapon_profile | Option<WeaponCombatProfile> |
| movement | MovementDef{speed_units_per_ms:u32,radius_units:u32} |
| resonance | ResonanceDef{start:u32,max:u32,gain_lock_ms:u64} |
| skill_id | Option<SkillId(String)> |
| skill_activation_mode | SkillActivationMode, scalar enum |
| deployment_affinity | DeploymentAffinity, scalar enum |
| block_capacity, block_radius_units, blockable | u32, f32, bool |
| mobility_kind | MobilityKind, scalar enum |
| target_traits | Vec<UnitTargetTrait>, current scalar Airborne enum |
| incoming_damage_modifiers | DamageModifiers, fourteen i32 fields |

UnitStats contains max_health/current_health/attack:u32, defense/magic_resist:i32, attack_interval_ms:u64, move_speed_units_per_ms:u32. DamageModifiers contains armor/magic-resist penetration flat and percent; generic/physical/magic/true damage amplification and reduction; crit chance and crit damage percent. Neither type owns heap children.

BasicAttackDef owns range_policy:TileRangePolicy, defense_tile_range:Option<TileRangePattern>, damage_type:DamageType, targeting_profile:TargetingProfile, air_capable:bool, range_role:WeaponRangeRole, interval_ms:u64, windup_ms:u32, ranged_reposition_ms:u64, delivery:DeliveryDef.

WeaponCombatProfile owns range_role, weapon_archetype, deployment_affinity, block_capacity:u32, ground_defense_focus:Option<{block_capacity_bonus,incoming_damage_reduction_percent:u32}>, platform_rest:Option<{hp_recovery_per_tick,resonance_recovery_per_tick:u32}>, damage_type, targeting_profile, air_capable:bool, defense_tile_range:TileRangePattern, interval_ms:u64, windup_ms:u32, ranged_reposition_ms:u64, delivery:DeliveryDef.

TileRangePattern contains include_anchor_tile:bool and rows:Vec<String>. DeliveryDef is Instant; Projectile with scalar speed/hit-policy/allow-targetless/range/lifetime/kills/pierces and scalar SkillProjectileCollisionDef; or TileArea{footprint:TileRangePattern,area:SkillTileAreaDeliveryDef}. The area object contains only scalar anchor/tile-origin/tracking/hit-target/include-caster/tick-policy/duration/tick-interval values. Collision contains radius:u32, hit filter, piercing:bool, despawn_on_hit:Option<bool>, max_hits:Option<u8>.

Thus UnitCombatProfile/BasicAttackDef/WeaponCombatProfile/DeliveryDef are safe borrowed-Serialize work leaves: no Arc/DB recursion, hash table or skipped deep-owned state was found. Their heap owners are tile-row Vec/String values, optional SkillId and target_traits. Cover every DeliveryDef shape even where current basic-attack validation rejects TileArea; meter shape without changing gameplay support. Option::None serialization skips in scalar collision fields hide no heap owner.

**LEAF05-G01: GrowthStack requires separate provenance.** growth.rs11–12 still has public stacks:std::HashMap<GrowthId,i32>. GrowthId currently has only KillStack; add() uses entry and never removes. But public raw HashMap storage permits externally reserved/churned tables, and serde reports live entries only. Neither one possible key nor current len proves retained buckets. OwnedHashMap::clone_bucket_bound is unavailable here. Root must use a source-proven fresh/insert-only origin, or introduce complete native owner bookkeeping within accepted API scope. Do not silently seed an arbitrary imported table from capacity, assume one slot, or claim a whole Employee Serialize call solved it. Changing this public field to a private wrapper is an API choice requiring actual consumer impact review, not a recommendation to change policy.

## UuidManager and PlayerProfileState

UuidManager owns private run_seed:u64 and counters:BTreeMap<u64,u64> (uuid_manager13–16). Its whole value is safe for borrowed serialization; owner-local BTree len admits native entries first. Numeric keys are serde JSON-compatible. next(namespace:u64) is public and accepts more than the five named constants, so do not hardcode five counters. Never call next or restore to measure; serialization/borrowed fields do not execute RNG.

PlayerProfileState (encounter_compendium23–29) owns encounter_compendium:{registered_ids:BTreeSet<String>}, standard_tutorial_completed:bool, highest_floor_reached:u32, longest_deathless_floor_streak:u32, all_live_final_bosses_defeated:bool. This is a closed safe serde leaf plus native set cardinality/string visits. Methods accepting GameDataBase do not make it an owned field; do not invoke current-catalog projection/validation for cloning.

EquipmentLoadoutEquipPlan is a separate short-lived Clone type, not an Inventory/Employee field: item_uuid/target_unit, optional source/replaced UUIDs, incoming_ref, allow_duplicate and projected_target:ItemSlot. It lacks Serialize; if metered separately, visit only projected_target plus fixed fields. EquipOutcome owns updated_item_uuids:Vec<Uuid>. EquipmentLoadoutTransition borrows &mut Inventory/&mut EmployeeRoster and is not a cloned owner.

## Meter integration rules and source limits

OwnedHashMap Clone copies its native table plus MapCloneHistory; custom Serialize delegates only to inner entries. The history survives deletes/clear and is not serializable game state (owned_collections14–73,148–170). Admit clone_bucket_bound before map traversal; None must fail explicitly rather than fall back to len/capacity. Then count actual live key/value descendants. Keep original Clone/storage semantics and read-only state.

“Safe Serialize leaf” means its serialized data covers actual deeply copied children without hidden owned skip fields or shared-definition expansion. A byte-only writer such as charge_serialized does not independently meter Vec/BTree slots. CloneMeter must admit collection cardinalities separately, preferably before traversal, or its serializer must explicitly admit sequence/map lengths. Vec/String Clone follows live content, not old spare capacity; BTree owners need native entries, not HashMap history. None and empty containers still have fixed parent/owner cost according to root's common primitive.

Do not globally deduplicate equal IDs/strings. Do not use to_value, DTO constructors or getters that allocate a clone before admission. All preflight access remains borrowed; it must not advance UUIDs, consume inventory, normalize loadouts or resolve shared database definitions. These are logical ownership/work units, not measured allocator/RSS or wall-time bounds.

At 2026-09-10T09:00:49.758090+00:00, all nineteen final rehashes matched the captured input snapshots. Individual reads are not an atomic repository snapshot; root may subsequently add visitors. No proposed CloneMeter implementation or test result is audited here.

| Source under repository root | SHA256 | Read spans |
| --- | --- | --- |
| core_enhanced/src/game/resources/inventory.rs | 5c429345a210ffc303d7274c3f6bfb5bdb8d053091638d5a7af3f946497fbcc6 | 219–226, 457–460, 508–571, 587–590, 702–713 |
| core_enhanced/src/game/employee.rs | 7e755c27aeef108f2c328ed2e3e25043054a41468664066e0c6d0963c3edb53d | 21–29, 69–135, 207–214, 305–373, 636–685 |
| core_enhanced/src/game/resources/item_slot.rs | c20b7a68a8f5c88f2f506445a81989b8d27a764fd8890e97e15d45466294a381 | 5–49, 235–248 |
| core_enhanced/src/game/resources/equipment_loadout.rs | e76eff46bdbddbfa4f6cc1d4dc686fba8f2d51cfbcba0e11a9575a9ff90ce51e | 1–39, 61–76 |
| core_enhanced/src/game/skill_fragment.rs | b4bb2acf2bda08b3ebcb1d7f740fb7b51a12d4cb3fb588b3ae1d1ad50a13a38d | 18–40, 74–108, 576–608 |
| core_enhanced/src/game/data/skill_fragment_data.rs | 5fd2772af7d0dc8ff5c0abee5cb7b05bdcb0996ce55a67e1a717d34dff5bc9ad | 20–34 |
| core_enhanced/src/game/employee_psychology.rs | fc130cf2cb5bb3182962f6feae485078b8d63cd7dbbb4c73971d364a01ef9f69 | 1–53 |
| core_enhanced/src/game/growth.rs | 9005a4751ab9a7bcf1c12253028a752828495d6166caee4b1cdcc56e30c50bb4 | 1–28 |
| core_enhanced/src/game/battle/types.rs | e388e9eb7d882dbcbd81eb9291a3cf77f1b8841fc060201da7735c750666075f | 273–291, 392–409 |
| core_enhanced/src/game/data/abnormality_data.rs | bbda968e58c0bcfdeae09f7796ae4ed1fa92893fd89f8c250e4014a6e3275828 | 72–115, 241–254 |
| core_enhanced/src/game/data/equipment_data.rs | a9a6b06fccd5c7bf78879b010531533a56f6c1264bd02989ae6d3d9de8173e1e | 34–114, 168–219 |
| core_enhanced/src/game/data/consumable_data.rs | 9211a070c85911c1650b999b0e5970aaed4aee6792cc3f4e2552e00017155114 | 10–17, 34–65, 97–115 |
| core_enhanced/src/game/ability.rs | c7cec94493e6de6d2ebb005941f3f80a276396e3803afb07f5d5a884d65ab195 | 9–23, 85–92, 165–192, 230–242, 472–533 |
| core_enhanced/src/game/battle/tile_range.rs | 73930102fc59b03f028189681934de2859b2dad19e76d3622f4967b5f594e8aa | 5–12, 53–60 |
| core_enhanced/src/game/battle/damage.rs | 5bfa34c21ad87ad672a4c249b38641f620358a7f34d57f4aa59df47246b5e46a | 143–173 |
| core_enhanced/src/game/stats.rs | 9c3d49b44c773288da84e35bc0411292e423583a4b481a1e044622c138d10269 | 183–225 |
| core_enhanced/src/game/managers/uuid_manager.rs | b78fca9caefe3ebda1203686e3efc6f6dcb1f02e7b9cb2276280aea9532809a9 | 12–41 |
| core_enhanced/src/game/encounter_compendium.rs | 4b8d897d8c683b4f7ce87ba0a168d1545b495c784928deb0dfaaa2fd041b1083 | 13–29 |
| core_enhanced/src/owned_collections.rs | ec828c1c582f283b7dd946c9e08ba3a12843bd81fac10b7299a0f6c3856dcb46 | 14–73, 148–170 |
