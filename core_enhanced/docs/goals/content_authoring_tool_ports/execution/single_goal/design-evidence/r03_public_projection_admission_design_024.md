# R03 first public JSON admission — bounded source design 024

This is read-only source/design assistance for PT020-F01 under AUTORUN §8, not an implementation, full heap proof, performance result or completion audit. Root alone writes production artifacts and runs validation. The source facts below use the inspected versions; application.rs, behavior.rs and snapshot.rs changed before close and those later bytes were only rehashed.

## 1. Minimal shared primitive: root's selected count-first native conversion

The initially considered bounded encode → charged parse path is not a generally exact replacement for serde_json::to_value. Root's subsequently selected **count-only Serialize writer → precharged native conversion → existing to_value → post-poll** is the smaller compatibility-preserving design.

Proposed crate-private helper signature: to_public_value<T: Serialize + ?Sized>(value: &T) -> serde_json::Result<Value>. No active scope: call the existing serde_json::to_value directly. Active scope:

1. Enter the intended projection phase, normally Receipt as in the current trial public_value adapter; preserve the enclosing preparation/execution phase on guard drop.
2. Serialize the borrowed source once with serde_json::to_writer into a count-only io::Write. Each write performs checked cumulative byte addition, ResultBytes admission and scan-work charge **before** accepting the chunk. Store only the count, not a byte buffer or JSON tree. Reuse the actual Writer/control error/admission conventions.
3. After the entire source representation passes, charge a separately named amount of native-conversion work based on the completed count, and poll before conversion. Use checked accounting; an overflow must fail, not silently become an unlimited allowance.
4. Invoke the unchanged serde_json::to_value(value), then perform a post-conversion cancellation/sticky-failure poll before returning its Value. Preserve a genuine serde conversion error; if control failed, its sticky outer failure remains authoritative.

Existing output::Writer is private and buffers Vec<u8>; encode_result is authoring-module-private (output.rs:7–59). A count-only writer can sit beside it and share the same checked admission/charge conventions without exposing arbitrary writers publicly or routing world errors through AuthoringError. Return serde_json::Error (including Error::io for a control bridge), matching the existing projection functions; current GameError/ApplicationError mapping can remain at their owners. control::charge_serialized currently counts only work and lacks ResultBytes admission (control.rs:451–470), so it alone does not close first-Value admission.

This counts source serialization bytes/work, **not native Value heap bytes, allocation peak, or CPU nanoseconds**. It deliberately adds a second Serialize traversal. Native to_value has no per-node cancellation poll; after admission, that one conversion is a finite precharged interval followed by a rejection poll. Do not describe it as interruption inside every native allocation. Earlier native DTO creation is a separate boundary (§5).

The source values must be the same immutable, pure Serialize inputs for both passes. Do not expose this helper as a generic public admission proof for arbitrary user-written serializers with interior mutation, I/O, hidden allocations or changing sequence lengths. Actual domain search found mostly derive serialization; the inspected custom OwnedHashMap serializer delegates to its inner map, and BattleEventEntries delegates to its Arc-backed entries (owned_collections.rs:150–153; event_log.rs:123–129). Those two implementations do not themselves clone their storage. This is supporting evidence for these leaves, not an exhaustive transitive Serialize audit.

## 2. The first-Value producer map

Wrapping an already created Value at ApplicationResult::from_behavior or TrialState::public_value is too late. The necessary caller families are:

| Current first allocation | Minimal connection | Existing result/error shape to retain |
|---|---|---|
| behavior.rs application_result_projection's object branches | serialize a borrowed object projection directly through shared helper, before json! or field to_value | exact selected keys/wrappers/Option nulls; CommandPayload result_type unchanged |
| behavior.rs five bare to_value sites | helper on the same borrowed map/result | bare RunComplete map; UnEquipItem and EquipItem result; two read-only previews |
| nested_command_result_projection | child projection stays fallible/bounded, then bounded borrowed {result_type,payload} wrapper | its special nested envelope; do not replace it with raw externally tagged BehaviorResult |
| application.rs behavior_variant_payload:422 | helper(&BehaviorResult) before existing variant extraction | exact one-key externally tagged object, expected_meaning removal, unit→Null and existing mismatch errors |
| world/snapshot.rs four JSON adapters | build the same native DTO, helper(&dto) at current to_value | current GameError mapping and selected-event Option behavior |
| CoreApplication::snapshot's three direct adapters | helper on the same action-capability/profile/compendium value | exactly current ActionCapabilities, PlayerProfile and EncounterCompendium read models |

application_result_projection (behavior.rs:3840–4259) has **37 command branches**, comprising 33 object branches, 3 bare conversions and the constant Ok→Null. There are also 2 bare read-only preview conversions and 10 BattleUpdate-class variants delegated to application.rs. The object branches are:

`RunStartFlowStarted`, `StarterEmployeesSelected`, `MapState`, `NodeEntered`, `NodePreview`, `NodeCompleted`, `SavePointState`, `MaintenanceState`, `HeadquartersContactState`, `FloorAdvanced`, `RunFailed`, `RunFailureDecisionRequired`, `RunAbandoned`, `ConsumableItemUsed`, `SkillFragmentLoadoutUpdated`, `SkillFragmentUpgraded`, `SkillFragmentAwakened`, `SkillFragmentDismantled`, `EquipmentDismantled`, `EquipmentEnhanced`, `ShopState`, `RefreshShop`, `SellItem`, `PurchaseItem`, `EmployeeRecruited`, `EmergencySuppliesGranted`, `EventState`, `VisualNovelState`, `RewardGranted`, `CombatRewardsGranted`, `MasteredEliteSweepResultStaged`, `MasteredEliteSweepCompleted`, `RewardState`.

The five bare sites are RunComplete (3976), UnEquipItem (3999), EquipItem (4002), DeploymentRangePreview (4242) and SkillTargetPreview (4245). Constant Value::Null for Ok owns no tree; retain its mapping. RunAbandoned must remain an empty object rather than null.

The ten battle variants are BattleAdvanced, BattleState, BattlePlaybackChanged, BattleUnitDeployed, BattleUnitWithdrawn, BattleEngagementStanceChanged, BattleUnitTacticallyRelocated, BattleUnitTacticallySwapped, BattleSkillTargetingFocusChanged and BattleSkillActivated (behavior.rs:4221–4239). A single helper at behavior_variant_payload covers their actual serde serialization, including fields not manually selected by the command projection. Do not serialize every BehaviorResult indiscriminately to replace the command mapper: RunComplete and nested completion have deliberately different public shapes.

Direct public consumers are execute, readonly active_battle_reconnect, elapsed advance, optional host tick and next-bucket advance via from_behavior/from_read_only_behavior (application.rs:355–395,899–974; execute call discovered at 896). HostTick None creates no public result Value and should remain None.

The four snapshot adapters are get_run_snapshot_json (snapshot.rs:418–420), get_inventory_snapshot_json (751–753), get_employee_roster_snapshot_json (942–944), and get_selected_event_snapshot_json (1155–1164). The last must retain None→None locally, with CoreApplication converting absence to read_model=null (application.rs:909–912); it also preserves the existing selected-event serialization error prefix. The other three direct snapshot to_value calls are application.rs:914,918,922. All seven use the same ordinary JSON representation, not a replacement DTO schema.

## 3. Heterogeneous borrowed object is sufficient; json! wrapping is not

Root's proposed private Field<T,Tail>/Object + SerializeMap pattern fits the actual fixed keys and avoids a new derived type declaration for every branch:

- A field owns a &'static str key and a borrowed expression value, followed by the next statically typed field; () terminates the chain.
- Its private field-list trait supplies an exact field count and serializes entries in the written order. Object calls serializer.serialize_map(Some(field_count)), visits each field with serialize_entry(key,value), and ends the map.
- A small macro constructs that heterogeneous borrowed object and **immediately invokes the shared helper**. Do not build json!(...) and then pass it in; json! has already invoked native Value conversion.
- Evaluate every field expression once. A borrowed temporary from nested_command_result_projection(completion)? must live through both helper traversals, not escape the macro. Avoid a macro returning an object that borrows temporaries after the creating statement.
- Keep literal keys and the complete list at each exhaustive BehaviorResult match arm. Both count and native conversion consume exactly the same Object instance; there is no second separately maintained field inventory.

serialize_map preserves an object representation. Existing json! None values become null; do not add skip_serializing_if or omission behavior. The empty Field chain serializes {}. Do not substitute serialize_tuple/sequence or serialize the whole BehaviorResult enum for these object payloads.

There are four nested-completion callers: EmployeeRecruited (4139), EmergencySuppliesGranted (4151), CombatRewardsGranted (4196) and MasteredEliteSweepCompleted (4218). Their child is first produced by its own bounded projection; the {result_type,payload} wrapper and outer object must also pass the shared helper. This may charge/convert the same nested Value again, as existing JSON projection already does. It remains bounded cumulative work and per-projection admission, not a claim of one allocation or one traversal. Keep the existing fallible recursive mapping rather than flattening away completion meaning. The preexisting panic for an impossible nested BattleUpdate is not redesigned in this slice.

## 4. Number semantics, failure propagation and infallible bridges

Pinned Cargo.lock selects serde_json 1.0.140 (2010–2018); game_core requests ordinary serde_json without special features (Cargo.toml:48–57). The local pinned source gives a concrete reason to retain native conversion:

- value::Serializer serializes f32 through Value::from(float); Number::from_f32 stores f as f64 without arbitrary_precision (value/ser.rs:150–157; number.rs:320–335).
- The text serializer formats f32 directly (ser.rs:153–162). Parsing that text can produce a different f64 Number from direct native to_value.
- Native value conversion rejects i128/u128 outside supported integer ranges without arbitrary_precision (value/ser.rs:96–109,134–145), while text serialization writes those integer tokens (ser.rs:112–115,146–149).

These are generic conversion differences, not a claim that a particular currently exposed DTO contains every such number. Count-first preserves the **native** conversion result/error. It also introduces no new JSON parser depth policy. Counted source-text length need not equal a later serialized Value's numeric text length; final mandatory encoding must still enforce its own actual byte ceiling. Do not claim the preflight count is exact final-wire bytes or allocator usage.

The active writer can fail through existing Result-returning projection functions without changing public method signatures. Existing application_result_projection and behavior_variant_payload already return serde_json::Result. Snapshot adapters already return Result<_,GameError> and map serde errors. No .expect(), unwrap_or_default or fake successful empty payload should be introduced to accommodate a new helper error.

**Infallible diagnostics are a distinct real boundary.** ApplicationResult::from_behavior currently returns Self and converts projection Err into AcceptedRecoveryPending via projection_recovery_pending; that routine executes json!({projection_error:error.to_string()}) (application.rs:355–405). Snapshot/error paths call ApplicationError::from_domain, which also constructs json! details from owned reason/ID/vector fields (594–674), and invalid_input builds a message object (582–591).

Control sticky failure ensures an authoring attempt cannot finally commit such a recovery response: the outer control poll returns the original execution failure even if inner serde/domain errors were caught (control.rs:310–356). But this **does not pre-admit the diagnostic JSON allocation itself**. A claim covering every first public/error JSON must explicitly handle those infallible builders too. Keep ordinary AcceptedRecoveryPending and existing error kind/code/details semantics. Options for the root owner include an active-control interruption bridge before recovery formatting/Value construction, or a separately fallible bounded diagnostic-details builder with a truthful control-error route. Do not recursively call the same bounded error builder after it fails, replace failure with success, or emit a false Err after an ordinary authoritative commit.

For this bounded change, success projection admission can be implemented first and the error-details limitation recorded explicitly; it must not be called complete first-public-JSON coverage while that branch is outside the claimed boundary. The current two-pass count design does not itself solve this infallible return-type issue.

Small output::usage_prefix to_value operates on fixed-width unsigned ExecutionUsage fields (output.rs:128–152); it is not a content-sized DTO producer. Command constructors' fixed enum kind serialization, admin/debug producers, and active-run checkpoint serialization are separate paths discovered by search; do not silently claim they are covered by the seven snapshot and command-result seams. The private generic TrialStep grammar uses command/snapshot/clock rather than an admin extension, but broad public API coverage needs an explicitly wider inventory if claimed.

## 5. Native DTO work before JSON admission remains separate

Concrete earlier owned work is visible in the source:

- get_run_snapshot_dto calls current_map_view, gathers/sorts Monolith entries/residuals and builds a native HashSet before returning its DTO (snapshot.rs:233–317).
- get_inventory_snapshot_dto clones IDs/names/compatibility and collects inventories before its JSON adapter (663–701,730–748).
- get_employee_roster_snapshot_dto computes effective profiles and gathers/sorts employee records/equipment before conversion (756–770,922–939).
- SelectedEvent clones the staged sweep result, shop fields/offers and combat result stats before JSON (1035–1072,1132–1148).
- player_profile() and encounter_compendium_profile() clone their full native profile owners before the direct snapshot call (world.rs:133–138).
- get_action_capabilities builds/sorts a native Vec before snapshot conversion (action_capability.rs:70–94).
- The native BehaviorResult/battle update already exists before from_behavior is called.

The new count helper should borrow those already-created DTOs so it adds no extra DTO clone. A future owner-specific visitor/borrowed projection can admit or remove the earlier copies. This report neither proves those allocations fully bounded nor recommends hiding them inside the JSON byte count. It also does not certify arbitrary custom Serialize methods against hidden native allocations.

## 6. Meaningful checks proposed, not executed

Compare active-scope and ordinary Value equality for every existing result mapping, the special bare/nested shapes, absent selected event, Option nulls, Unicode/escaping, read-only disposition and all seven snapshots. Reuse the existing exhaustive characterization oracle where its source/input claim is applicable, without claiming this helper executed it.

A Serialize probe whose second traversal is observable can establish that over-budget/cancelled counting never reaches native conversion; exact-cap and cap+1 should exercise large fields and cumulative nested wrapper work. Verify pure counting leaves state/RNG unchanged, a post-conversion cancellation rejects the candidate, and a projection failure cannot become an accepted session through RecoveryPending. Focused integer/f32/deep-value tests should prove the **helper preserves native to_value semantics**, not arbitrary encode/parse equality. Measure the added scan/conversion cost separately; no numerical multiplier is a measured heap or time bound merely because it is used as work units.

## 7. Read/hash provenance and drift

Reads used bounded rg discovery, then Python Path.read_bytes and numbered UTF-8 slices; full SHA-256 was calculated from the same bytes furnishing each slice. Registry source reads were local, pinned 1.0.140 files; no network, build, test, runtime or Git command was run. Some exploratory missing paths were corrected by file discovery and are not evidence of their existence. Only the two new /tmp reports were written.

15 read snapshots across 14 distinct paths. Close rehash: 2026-09-10T11:50:29.584194+00:00. application.rs changed c37e28bf…→aeeb0b18…, behavior.rs c38657dd…→adfc9b54…, snapshot.rs 35759cd8…→4d214bdb…; all three close versions were **rehashed only**, not reviewed as implemented code. The remaining sources were unchanged through close. JSON stores full hashes/read spans/timestamps and close versions.

1. `core_enhanced/src/application.rs`: spans 323–442, 561–681, 899–975; SHA-256 `c37e28bfccbed675dc268a59b477ddc3041759d02ba9e6e94019c8a9506a84e8`.
2. `core_enhanced/src/game/behavior.rs`: spans 3538–3572, 3812–4270; SHA-256 `c38657dd64aeae18fbb6a4998cfeb65cc45b015a897f379360236b2b4c3e2552`.
3. `core_enhanced/src/game/world/snapshot.rs`: spans 1–115, 399–431, 730–770, 922–960, 1117–1174; 232–318, 663–701, 1033–1077; SHA-256 `35759cd8335ef6565b0dabded301d69a7513be125ddbb0014eda29e37294c10e`.
4. `core_enhanced/src/authoring/output.rs`: spans 1–175; SHA-256 `2951433ffb08b52568028b70485cc12992c3c172a064e44df4795b1a22c71ddb`.
5. `core_enhanced/src/authoring/control.rs`: spans 310–364, 417–470, 502–565; SHA-256 `1a15e8bdd7618572950ddc69ad25b5cda674eebfecd0f544010abb3477b2fed8`.
6. `core_enhanced/Cargo.toml`: spans 1–67; SHA-256 `f489f2392299ff61ecda75e5f2e2e83acb3e72419b4128cfd9cec064eb0d419d`.
7. `core_enhanced/src/game/world.rs`: spans 128–140; SHA-256 `6dc90b38c0d4aa09a692b757665d45ff47fce274aa47df955eb23e8d8fff2de7`.
8. `core_enhanced/src/game/world/action_capability.rs`: spans 68–94; SHA-256 `10dab9e64a4e5d93a0a0e7d79b9f5421678bdc1985df812ed3f6f8f6981bf2dd`.
9. `core_enhanced/src/owned_collections.rs`: spans 146–160; SHA-256 `9591a49e57393a61bd525b5d3220e204af8d9f8c0d5a784089f6df631ad428f7`.
10. `core_enhanced/src/game/battle/event_log.rs`: spans 119–134; SHA-256 `49ddbf7545b8939ab4ae2b3cd42a87e1b42fbe97c7dbdfec67201daf33b63619`.
11. `Cargo.lock`: spans 2009–2021; SHA-256 `80eed115bdcf0aa881da5b6c1525e6f19a0d6d8e1101ee4b285840ab7b3a8f19`.
12. `/home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.140/src/value/ser.rs`: spans 94–166, 284–303; SHA-256 `b5cd3f4b4a58b2f53c4334c6d20705fc7c7b2a2e8988ef93b265a679c952cdfe`.
13. `/home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.140/src/number.rs`: spans 317–340; SHA-256 `186c2da2243afda82848a81fa401142ae8cc58ed820b3c3155b1dd5ab58f52fa`.
14. `/home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.140/src/ser.rs`: spans 108–170, 408–443; SHA-256 `5352ffeafb6397b4a837ca4d40b0353510788d445938f1bff96cde66d0467c1d`.
