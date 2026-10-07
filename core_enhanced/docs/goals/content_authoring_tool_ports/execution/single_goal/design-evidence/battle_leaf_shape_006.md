# R03 battle leaf ownership design 006

Bounded AUTORUN §8 read-only source/design assistance for root. This is not a completion audit. No source, repository, tests, build, runtime, or Git mutation/execution was performed. Only the new report pair is written. Root remains the implementation/artifact owner.

The six root families below require native ownership visitors except **BattleUnitSourceIdentity**, which is a supported borrowed-serde leaf. UnitSnapshot, BattleScenario, queue BattleEvent, DamageSourceSnapshot, ActiveBuff, and BuffDef do not implement Serialize in the inspected source. Adding runtime Serialize implementations solely for metering is unnecessary. JSON contains the full field/variant map; this document identifies the admission boundaries.

## Observation versions

Paths below are relative to core_enhanced/src unless absolute or Cargo.lock. Line references in the ownership discussion use the first recorded file version. All full-file SHA256 values and exact inspected spans are in the source inventory below and the JSON; hashes came from the same byte read as each excerpt. These are sequential per-file observations, not an atomic workspace snapshot.

A 2026-09-10T09:22:58Z rehash found root's concurrent changes in seven files: scenario.rs, damage.rs, core/types.rs, core/mod.rs, battle/types.rs, core/movement/types.rs, and authoring/clone_shape.rs. Narrow follow-up at 09:23:35Z confirmed unchanged candidate field ownership and newly added UnitSnapshot/DamageSourceSnapshot/ActiveBuff visitors, Copy-bounded scalar visitors, and BattleUnitSourceIdentity in the whitelist. Those additions are recorded as observed implementation, not a new implementation audit. Original and follow-up hashes remain distinct. Root's reported test passes are not this helper's executions.

## Six root families

| Root | Direct serde leaf? | Required ownership distinction |
|---|---|---|
| UnitSnapshot | No Serialize | Fixed-size native record; all children Copy |
| BattleScenario | No Serialize | Nested owned Strings/Vec; draft GrowthStack native HashMap; boss definition Arc |
| BattleEvent queue enum | No Serialize | 21 variants; 8 carry dynamic ownership, including nested damage snapshot |
| DamageSourceSnapshot | No Serialize | Bonus Vec deep-copy; skill_cast_source Arc handle only |
| BattleUnitSourceIdentity | Yes | Seven closed struct variants, String/Uuid/u64 only |
| ActiveBuff / BuffDef | Neither implements Serialize | ActiveBuff fixed only; BuffDef owns name; BuffDatabase has hidden native index |

### 1. UnitSnapshot

types.rs:92–102 has exactly id:UnitInstanceId, owner:Side, role:BattleUnitRole, threat_class:BattleUnitThreatClass, mobility_kind:MobilityKind, position:Position, world_position:WorldVec2, stats:UnitStats. It has no String, Vec, Arc, map, or skipped ownership. UnitInstanceId transparently wraps Copy Uuid; Position owns two i32; WorldVec2 owns two f32; UnitStats owns seven integer values. Other children are Copy enums.

Use an exhaustive native struct visitor, then fixed child visitors. Do not call to_snapshot to measure an existing snapshot. BattleCore.graveyard owns the enclosing OwnedHashMap, so its bucket admission precedes entry visitation. The later root addition at types.rs:104–113 follows this direct-field form. Root's general Copy-bound visitor prevents a formerly fixed type silently acquiring owned fields.

### 2. BattleScenario

scenario.rs:243–251 owns battlefield, groups:Vec, events:Vec, terminal_outcomes:Vec, tactical_plan, and boss_runtime:Option. Most runtime scenario records have no Serialize. The complete native child map is in JSON.

BattleFieldSpec owns two Vec<Position> plus width/height. ScenarioSpawnGroup owns id:String-wrapper, optional EnemyMovementPlan::PathAlongCells{cells:Vec<Position>}, and Vec<ScenarioUnitSpawn>. Each spawn owns unit_ref:String-wrapper and BattleUnitDraft, plus fixed side/position/salt.

BattleUnitDraft (types.rs:104–182) owns UUID, BattleUnitSource, threat/level, fixed BattleUnitStatScale and optional exact override, GrowthStack, Vec<Uuid>, Vec<BattleEquipmentEnhancement>, and optional fixed EmployeeBattleStartCondition. BattleUnitSource::Employee owns UnitCombatProfile; TestFixture/DefenseObject/SummonedUnit also own profiles. CorrodedEmployee owns profile_id:String; SummonedUnit owns definition_id:String; all remaining IDs/sequence are fixed. Abnormality stores only base UUID, not a database entry.

Reuse the explicit UnitCombatProfile leaf. Its deeply owned parts are SkillId(String), nested TileRangePattern.rows:Vec<String>, and target_traits:Vec; delivery/weapon/stat records have only fixed values, Options, or tile patterns. SkillProjectileCollisionDef's skip_serializing_if(None) excludes only an absent scalar Option, not hidden allocated children. It contains no definition Arc/native table. Metadata database types in the same files are outside this proof.

GrowthStack.stacks is an actual std HashMap<GrowthId,i32>, so serde is insufficient. The current growth.rs:15–33 native visitor already uses MapCloneHistory::from_tombstone_free_capacity plus one record slot, based on fresh production ingress and no removals. Its keys/values are Copy. This bounded fresh-owner proof does not authorize arbitrary raw GrowthStack imports; this report does not re-audit all mutation provenance.

Scenario events own id:String, trigger, and action. Wave triggers additionally own trigger_id, group/actor/point string IDs and fixed thresholds/deadlines. TerminalOutcome owns BattleOutcomeId(String) and a condition which may own unit_ref or BossSignalId(String). TacticalPlan owns objective unit refs, enemy path cells, tactical point IDs/positions, and field-work objects. Each field-work object owns id, Vec<cell{id,position,tile_kind}>, and a completion unit_ref; timing, presentation kind and HP are fixed. These need native struct/enum/string-wrapper visitors, not validation or DTO construction.

ResolvedBossEncounterSetup (boss_runtime_data.rs:754–779) owns:
- definition:Arc<CompiledBossEncounterDefinition>, charged as one retained handle;
- actors:Vec<{actor_slot_index,unit_ref:String-wrapper}>;
- enemy_stat_scale:fixed record;
- summon_spawn_zones and summon_routes:Vec<{id:String,cells:Vec<Position>}>.

Stop at definition. Its compiled phases/behaviors/patterns are shared definitions, not copied by the scenario's derived Clone. Do not recurse into them or resolve them from the DB while measuring. Actual ownership admission is: scenario record → each Vec storage before iteration → each child/native table → Arc handle; finish cumulative copy work before the actual outer clone.

### 3. BattleEvent and EventQueue

enums.rs:19–164 declares 21 Clone variants and no Serialize. Use an exhaustive native match with every field explicitly bound, avoiding wildcard variants and `..` field patterns. Each variant record is admitted before recursively owned children.

| Variant(s) | Dynamically copied fields |
|---|---|
| SpawnGroup | group_id:String-wrapper |
| AttackResolve | source_snapshot:DamageSourceSnapshot |
| SkillProjectileImpact | skill_id:String-wrapper, step_id:String, source_snapshot, impact_vfx_id:Option<String> |
| SkillAreaTick, SkillAreaExpire | skill_id and step_id |
| FieldWorkCompletion | object_id:String-wrapper |
| SkillStep | skill_id and step_id |
| WaveTriggerWake | trigger_id:String |

The other thirteen variants are AttackStart, BasicAttackProjectileAdvance, SkillProjectileAdvance, AutoCastStart, AutoCastEnd, ManualCastEnd, RestTick, ApplyBuff, BuffTick, BuffExpire, ContinuousMovementTick, BossRuntimeWake, BossSummonWake. Their fields are fixed integers, bools, UUID/UnitInstanceId/BuffId, fixed coordinates or Copy enums/Options. BattleEventCause is Copy Parent{seq} or Root{kind}; SkillCastTarget is Copy Unit{id} or Tile{Position}. No event owns a native table directly. Only the two snapshot-bearing variants transitively retain a source Arc.

EventQueue (core/queue.rs:7–10) privately wraps BinaryHeap<BattleEvent>. Its owner must admit the inner heap's live length plus wrapper/storage record before borrowed iter traversal, then visit each event. The current borrowed iter method at :30–31 exposes exactly the necessary scan. Do not clone, pop, sort, convert into a Vec, or format Debug as a measuring step. BattleEvent::complete_payload_key (:261–265) allocates a formatted String and can traverse Arc pointees; it is an ordering implementation, not a clone shape.

**Pinned native proof:** installed rustdoc source headers identify Rust 1.91.1 (ed61e7d7e 2025-11-07). binary_heap/mod.rs:274–279 stores Vec<T,A>; :452–455 Clone only returns BinaryHeap{data:self.data.clone()}, with no Ord/reheap operation. :1037–1038 iter directly wraps data.iter(). vec/mod.rs:3539–3545 dereferences to its slice; :3561–3566 Clone invokes slice::to_vec_in with a cloned allocator. alloc/slice.rs:394–438 requests capacity s.len(), clones each live element in slice order, and sets result length s.len(). For this Global-allocator, non-Copy BattleEvent queue, spare heap capacity is not copied; no native table history is required. This proves logical live storage/copy shape, not allocator byte/RSS precision. clone_from is a separate implementation and is not this derived-Clone path. HTML originals, source anchors, version marker, and hashes are recorded.

### 4. DamageSourceSnapshot

damage.rs:38–54 owns fourteen fields. Thirteen are fixed values/records/Options or the bonus Vec; the remaining field is skill_cast_source:Option<Arc<SkillCastExecutionSource>>. Precisely, visit source_id, source_side, source_attack, source, damage_type, base_damage, modifiers, crit_roll_percent, crit_roll_identity, minimum_damage, committed_at_ms, on_attack_modifiers, and on_attack_bonus_damage in addition to the source handle.

DamageBonusSnapshot has flat:i32, percent:i32, damage_type:Copy enum. Admit bonus Vec length before iteration. CombatRollIdentity has Copy roll kind, optional source/target unit IDs, source_instance_id:Uuid and hit_index:u32; DamageModifiers has fourteen i32. Both are fixed.

Charge Option plus Arc handle without visiting SkillCastExecutionSource. That pointee owns skill_id:String and provenance records but derived Arc clone does not copy them. The later root visitor at damage.rs:56–71 lists every snapshot field.

Keep this distinct from BattleEventLogEntry.skill_effect_source (event_log.rs:803–815): that is a different Option<Arc<SkillEffectExecutionSource>> with serde(skip). It must remain covered by the event-log owner/COW proof, not by whole-entry borrowed Serialize. Optional trace settings must never change mandatory clone admission or accounting.

### 5. BattleUnitSourceIdentity

types.rs:184–216 derives Serialize with internal tag kind and seven struct variants. Employee owns two UUIDs; Abnormality owns abnormality_id:String plus UUID; CorrodedEmployee and FacilityEntity own profile_id:String plus UUID; DefenseObject and TestFixture own one UUID; SummonedUnit owns definition_id:String, base UUID, UnitInstanceId, u64 sequence, batch UUID.

No Arc, maps, custom formatting, flatten, or skipped field occurs. This concrete enum is safe for the whitelist. The later root whitelist includes it at clone_shape.rs:260.

Compatibility is source-backed: pinned serde_derive 1.0.228 uses SerializeStruct for these internally tagged struct variants (ser.rs:632–635,885–952), which LeafSerializer accepts. Pinned uuid 1.17.0 uses a stack encode buffer with serialize_str for human-readable serialization and serialize_bytes otherwise (:25–32); it never needs collect_str. UnitInstanceId's transparent derive delegates to Uuid, not its Display implementation. Counting fixed tag/UUID text conservatively may overcharge logical clone work but introduces no hidden native allocation.

### 6. ActiveBuff / BuffDef family

core/types.rs:496–508: BuffInstanceKey owns caster/target UnitInstanceId and BuffId, all Copy. ActiveBuff owns only stacks:u8, expires_at_ms:u64, next_tick_ms:Option<u64>. Neither owns BuffDef or an Arc. BattleCore.buffs is the surrounding OwnedHashMap; admit bucket storage before visiting fixed keys/values. Root's later ActiveBuff visitor is at :666–670.

BuffDef (buffs.rs:64–72) owns id:BuffId, name:String, kind:BuffKind, tick_interval_ms:u64, max_stacks:u8, reapply_policy:BuffReapplyPolicy. Kind and policy are Copy, Serialize enums; BuffDef itself lacks Serialize. If metering an actual BuffDef clone, native visitor visits name plus fixed values. BuffMetadata (:82–90) does derive Serialize and owns the same shape without id; it is independently a safe closed leaf.

BuffDatabase is not a safe leaf: buffs:Vec<BuffMetadata> plus serde(skip) by_id:OnceLock<HashMap<BuffId,BuffDef>> (:74–79). Derived Clone of an initialized DB value includes that native index. BattleCore's whole clone instead retains game_data:Arc<GameDataBase> (:117); do not cross that Arc to count the DB.

ApplyBuff and BuffTick perform buff_data.get(buff_id).cloned() in sim.rs:3173 and :3282. These transient BuffDef/name copies are separate operations, not transitive ActiveBuff storage. Their presence is concrete evidence for any later per-operation copy coverage; this report does not claim to audit all leaf-call budgets.

## Bounded implementation and verification obligations

Keep ordinary Clone behavior and fixed allocation/ID semantics unchanged. Reuse native owner visitors, explicit field/variant patterns and closed serde leaves; do not create serializer mirrors, a second DB, or clone owners for measurement. Arc accounting counts handles, irrespective of strong count or optional trace collection. Map proof and native ownership remain necessary even if Serialize exists.

Relevant validation seams for root, none executed here:
- exact work/slot success and one-less failure before native clone, with unchanged original state;
- heap with large reserved capacity and short live length versus the same events in a compact heap: equal native clone shape, preserved queue order;
- every dynamic queue variant, including both source-snapshot variants and nested long strings/bonus vectors;
- identical source Arc handles with very different pointee sizes: equal handle cost; actual owned strings/Vec grow copy cost;
- all seven source-identity variants pass the leaf path without map/collect_str fallback;
- large scenario nested vectors, both optional boss setup states, and the native GrowthStack boundary;
- ActiveBuff cost independent of BuffDatabase size; metering a direct BuffDef copy includes its name.

These observations do not establish total BattleCore/runtime coverage, imported-core eligibility, broad R23 safety, allocator/OOM bounds, or completion. Root owns remaining owner integration, actual tests and acceptance.

## Source inventory

Every hash covers the full original file, not only its excerpt. For rustdoc HTML, spans are decoded source anchors; hashes cover original HTML bytes. Multiple rows for a path preserve drift versions.

| Source and inspected spans | SHA256 |
|---|---|
| game/battle/scenario.rs 1–420 [version 1] | `1caed730779f276b0e83369d2f3bbd818272c557cf56c7775b37a883561aebc1` |
| game/battle/scenario.rs 49–69, 244–260 [version 2] | `6ed384c71cf9d9913f348ac168cdb0b3ed7900dcd1757bd2459c78984830c922` |
| game/battle/damage.rs 1–190, 220–255 [version 1] | `5bfa34c21ad87ad672a4c249b38641f620358a7f34d57f4aa59df47246b5e46a` |
| game/battle/damage.rs 31–75 [version 2] | `20b826044fa46bd7b10cb07bd0fec1a5b540a90228c95ead1bea36fefbe0c2da` |
| game/battle/buffs.rs 1–185 | `4b501650a51b5835814fa3a2e689ba19e8de1780144f1c29ddf07f4b28ec783b` |
| game/battle/core/types.rs 470–535 [version 1] | `385402e5e49ead95a505f2345211875542ed370aa5558e73472c33de03e42a2d` |
| game/battle/core/types.rs 496–517, 662–672 [version 2] | `65b6ab56b9e40eb35f195b34dac6472c35055f61234e7ab277a3d143d392870a` |
| game/battle/core/mod.rs 72–190 [version 1] | `46e0da5070f321ca2d3e3bf85f32ac7b2d8a49d763e405d030489122defd2008` |
| game/battle/core/mod.rs 79–135 [version 2] | `397f706e58168c78f435fa245153ccdc15cf2f8ec7365cd517f9273aea0c7b4e` |
| game/data/boss_runtime_data.rs 1–100, 510–550, 700–800 | `05b2fa00aa7aed65becf8dbc20da866b35d2dfe4dca4318f4b8cc8fc1013d4d3` |
| game/battle/skill_source.rs 1–29 | `4fff389728a0dcb34fa50b56108e8790a750205fa0d391c21e65fd4553ca50d1` |
| game/battle/event_log.rs 315–395, 800–850 | `82975ec82b87e469eaa206e46a2042a51a12ee4b6adf0375148ab64ace97d7b8` |
| game/battle/core/queue.rs 1–44 | `a2b3edc11f43590fc368cf3c9ae7be8919055bdd3d5c2aa448d533a9372714d5` |
| game/growth.rs 1–49 | `cc752dbc450f64735f6cb0bc6078a930cff934be86ca6e97f6776ac4a23f7694` |
| game/battle/types.rs 90–230, 230–300, 388–410 [version 1] | `e388e9eb7d882dbcbd81eb9291a3cf77f1b8841fc060201da7735c750666075f` |
| game/battle/types.rs 90–116, 192–228 [version 2] | `04142c509b0b006359b0cd5e113d52706cb4108b90bbafbd886b704f1a62d5ac` |
| game/battle/ids.rs 1–49 | `063075a0f4ac754132949232bd9db555277b4828c43aaecc9a57fd7561069ac9` |
| game/resources/board.rs 1–21 | `3318239bfa2142d812f561c7aff2010221767ebb36520b90b62a7e2f48ec213c` |
| game/battle/core/movement/types.rs 120–190 [version 1] | `4ca957c60c97be29f673abda8c73a7e21b0067cad3aba303ca00af84f950913c` |
| game/battle/core/movement/types.rs 125–137 [version 2] | `2ad7e06157b4fbf5cb6ab1d0ee9ccde20d371bae79693057e85cfb87a5dbe5be` |
| game/stats.rs 195–225 | `9c3d49b44c773288da84e35bc0411292e423583a4b481a1e044622c138d10269` |
| game/enums.rs 1–50 | `85255ba6c40ba34e3d8aa3c546d5d54e187d84146347f0de7db04c0bc0971b58` |
| uuid-1.17.0/src/external/serde_support.rs 1–65 | `cd5cf95fc393ac2f28f21b435f6b134ab6a9089d59c889359f1f0ee44385a8d7` |
| Cargo.lock 1978–2008, 2519–2532 | `80eed115bdcf0aa881da5b6c1525e6f19a0d6d8e1101ee4b285840ab7b3a8f19` |
| authoring/clone_shape.rs 1–393 [version 1] | `5ee66133b8abb0e5badd9b929613bdb2db620fd1c2b32b755cbf2dbf0c983ed1` |
| authoring/clone_shape.rs 21–29, 230–262 [version 2] | `76dbffb8484f641a948c49437f45ec100f224a493293080727550e2c1a5e5d00` |
| authoring/control.rs 280–355 | `72c81215a3b7794f3387925524ba75c0359fafa8c84e86122d9d2a8f048f1f23` |
| game/battle/enums.rs 1–195, 274–310, 256–273 | `cac118b9704c9c461d3e8c6146943d097ffc0e531a7fc6e54bf0b81473857b00` |
| game/battle/core/sim.rs 3165–3220, 3272–3295 | `e624854ac13e8172b104c54e4626a761c4d017d67b8ffdb4adf3f15e50a31207` |
| serde_derive-1.0.228/src/ser.rs 576–635, 885–965 | `d43be68a3c1d5d4617c30baebe06aceeaf012097ffa343102328ff94349e35e9` |
| game/data/abnormality_data.rs 70–116, 240–252 | `bbda968e58c0bcfdeae09f7796ae4ed1fa92893fd89f8c250e4014a6e3275828` |
| game/data/equipment_data.rs 1–111, 109–118 | `a9a6b06fccd5c7bf78879b010531533a56f6c1264bd02989ae6d3d9de8173e1e` |
| game/battle/tile_range.rs 1–67 | `73930102fc59b03f028189681934de2859b2dad19e76d3622f4967b5f594e8aa` |
| game/ability.rs 1–20, 77–94, 160–260, 450–535, 126–161 | `c7cec94493e6de6d2ebb005941f3f80a276396e3803afb07f5d5a884d65ab195` |
| Rust 1.91.1 HTML/alloc/collections/binary_heap/mod.rs.html 274–279, 451–465, 1034–1043 | `218846109b4b726501f11b0f49c44fa04c19ba6ff7b8d4c39f6e250f5580ffec` |
| Rust 1.91.1 HTML/alloc/vec/mod.rs.html 3559–3566, 3537–3547 | `51fb8604038b698379a9fd5e3f104ed3c06d9df0e79a5cc608964113da6dd753` |
| Rust 1.91.1 HTML/alloc/slice.rs.html 394–456 | `adaf7573c58b39ab78a6023084565dbda8dcebdaf17ddd8b04d994af03d2ba0b` |
