# R03 owned HashMap clone history — design 007

This is the requested bounded read-only design investigation, not a completion audit. It proposes source changes but performs none. No repository/dependency/Git/build/test mutation or execution occurred. Only this new report and its JSON companion are written. Root's INPUTS035 backend20pass and BC006 correction are reported context, not tests rerun by this helper.

## Recommendation

Use one crate-private tracked std HashMap wrapper for maps whose concrete type is private to the application implementation. Keep the actual std HashMap as the sole storage, retain its ordinary Clone, and copy a small history record alongside it. This is smaller and less error-prone than adding unrelated counter fields to every owner and manually threading those fields through checkpoint capture/restore.

Use a narrow exception for BattleCore.units if its current debug/test-fixtures public HashMap type must remain compatible: retain that field type, add an owner-local history counter, and account for every internal insertion through a small BattleCore helper. The guarantee for that exception must be tied to application-owned state that cannot receive arbitrary externally mutated BattleCore instances. An unrestricted pub HashMap field cannot simultaneously preserve arbitrary external mutation compatibility and provide interception-based history accounting. A private wrapper alone does not solve that conflict.

No global cache, object-address registry, allocator hook, new public gameplay authority, map rehash/reinsertion during Clone, or snapshot-based replacement is needed or recommended.

## Exact visibility and owner map

| Actual owner/source | Retained std maps in the inspected scope | Compatibility and proposed seam |
| --- | --- | --- |
| GameCore, world.rs:49–57; state.rs:138–161 | State is owned by a crate-private GameCore; world/state is not publicly exported | Internal field-type changes do not change CoreApplication's public API. CoreApplication.core is private (application.rs:782–784), with from_core available only to crate tests (821–825). |
| RunState, world/state.rs:1413–1426 | event_sessions, shop_sessions, combat_previews, abnormality_attempts | Change all four field types to the internal wrapper. Although fields are written pub, the containing world module is pub(crate) and state is private (game/mod.rs:31; world.rs:40–44). Existing internal direct insert/entry calls can retain their shape. |
| RunCheckpointPayload macro, state.rs:380–477 | The same four run maps copied by capture and moved by restore | Change the four types in the macro invocation together with RunState. A wrapper automatically carries its history when the macro clones fields at 410–421 and moves them back at 437–446. Do not serialize accounting history into the checkpoint. |
| ActiveBattleSession / BattleEntrySnapshot, state.rs:508–565 | Nested BattleCore, RunState, Inventory, EmployeeRoster; deployment has deployed_units and redeploy_locks at 619–625 | Put history at each actual map owner. Each separately owned snapshot copy carries a separate copied history. Deployment map fields are internal through world/state and may use the wrapper directly. |
| Inventory, resources/inventory.rs:220–226,456–460,508–510,587–589 | EquipmentMaterialInventory.stacks, EquipmentInventory.items, ConsumableInventory.items | All three concrete maps are private fields. Public add/get/get_mut/iter/remove APIs can stay unchanged. These owners derive Serialize/Deserialize, requiring the fresh-deserialization seam below. |
| EmployeeRoster, employee.rs:635–681 | employees | Private map; public new/clear/add/get/get_mut/iter/iter_mut signatures need no change. Preserve ordinary clear allocation history. Roster copies inside active employee attempts and entry/checkpoint snapshots carry the record automatically. |
| BattleCore, core/mod.rs:79–108 | units plus ten other top-level maps: tactical_states, items, graveyard, buffs, active_skill_casts, ability_proc_states, projectiles, active_projectiles, active_areas, active_movement_segments | The ten private fields can use the wrapper. units is pub HashMap and needs the explicit exception described below. |
| ScenarioRuntimeState and BlockRuntimeState, core/mod.rs:184–199 | unit_refs; blocker_to_enemies; enemy_to_blocker | Internal wrapper is suitable. BlockRuntimeState derives PartialEq/Eq: wrapper equality must ignore history. |
| Boss runtime, core/boss_runtime.rs:99–135 | Per actor fixed_sequence_cursors and cooldown_ready_at_ms; per pattern loop_counters and batch_handles | Private wrapper fields. Existing new/default/clear/insert/entry operations fit a small mutation surface. Clone naturally carries the record through nested boss transition copies. |
| BattleResultStatsAccumulator, battle/result_stats.rs:114–121 | unit_sources, employees, active_deployments | Private wrapper fields. Read helper direct_employee_uuid takes &HashMap and can use immutable deref coercion (405–412). close_deployment currently takes two &mut HashMaps (415–431); change this private helper's parameters to wrappers rather than exposing DerefMut. |

This is a bounded source inventory, not an assertion that every transitive collection in GameCore has been visited. In particular, HashSets use the same kind of retained native hash storage but are not covered merely by finishing a HashMap wrapper. Actual neighboring owned sets include visual-novel consumed keys (visual_novel.rs:58–60), battlefield tiles/obstacles (battlefield/mod.rs:15–16), scenario spawned_groups, boss latched/cleared handles, and processed incapacitations. They need the analogous owner-local treatment when the whole visitor includes them. SkillFragmentInventory and UuidManager use BTreeMaps in the inspected source and should not be converted as part of this change. Shared immutable GameData maps behind Arcs should not be deep-copied to acquire history.

## The public units exception is real

game/battle/mod.rs:5–8 exports core publicly under test, debug_assertions, or test-fixtures; BattleCore.units at core/mod.rs:89 is a concrete pub HashMap<UnitInstanceId, RuntimeUnit>. Changing that field to a crate-private wrapper affects:

- Direct assignment from HashMap, direct reserve/extend/drain, and mutable borrowing as &mut HashMap.
- Type annotations and helper parameters relying on the concrete map type.
- Existing source tests already replace it with HashMap::new and extend it after a drain (core/commands.rs:2989–2995). Those tests are evidence of the existing mutation shape, not a recommended Clone implementation.

Immutable Deref does not preserve these mutation contracts. Making a private wrapper publicly nameable is also an API change, not a transparent implementation detail.

Minimum compatible application seam: keep units as HashMap, keep a copied private units_map_history in BattleCore, and route controlled insertions through an internal helper that updates this record before calling the same raw insert once. The inspected production insertion sites are build.rs:83–131 and 209–257. Keep the history across sim.rs:1608 clear. Construction/replacement with a genuinely new map is a separate reset boundary; whole BattleCore Clone copies the raw map and record together.

This can support authoring's private CoreApplication/TrialSession owner path because GameCore is not public, CoreApplication does not expose a mutable inner core, and the inspected from_core is cfg(test) pub(crate). Do not accept an arbitrary external BattleCore or arbitrary raw units map into that path while claiming its sidecar counter has observed past writes. If such an import is required, it must provide verified fresh provenance or be treated as unknown for opted-in cloning. The helper does not propose changing the existing public fixture API or creating an unapproved mutation ban for ordinary callers.

If the requirement instead extends to every arbitrary standalone public BattleCore after external units.reserve/insert/remove/replacement, an exact field-compatible sidecar cannot solve it: no operation is intercepted. Root must explicitly choose an encapsulation/API change or constrain which owner family receives the new clone guarantee. This is a technical compatibility boundary, not an invitation to silently weaken the accounting claim.

## Minimal internal type and mutation surface

A suitable file is core_enhanced/src/owned_collections.rs, exposed only as a crate-private module. Keep it independent of authoring TLS; it records storage history in ordinary gameplay and offers a pure checked bound to the existing preflight visitor.

    OwnedHashMap<K, V, S = RandomState> {
        inner: std::collections::HashMap<K, V, S>,
        history: MapCloneHistory,
    }

    MapCloneHistory {
        allocation_credit: usize, // saturating or explicit unknown sentinel
    }

For empty new/default construction, credit is zero. The insertion-only case can use the same lifetime insertion upper bound already established by root's std_hashmap_bound_001. If reserve/import paths are supported, name the field allocation_credit rather than claiming that every credit represents a successful insertion.

Keep the underlying map's Clone intact: clone inner once and copy history. Do not replace cloning with collecting entries into a new HashMap, sorting/reinserting them, shrinking, or rebuilding a hasher. This preserves the original copied buckets/control state, random seed/hasher, entry order, and value Clone calls. Also avoid an implicit clone_from that overwrites the history with the source's history while retaining a larger destination allocation; either keep the trait's replace-with-Clone default or separately preserve a proven upper bound across native clone_from. No inspected production mutation requires a custom clone_from optimization.

The wrapper need only expose the mutations actually needed:

| Method family | Accounting rule and source effect |
| --- | --- |
| insert(k,v) | Credit one possible insertion before forwarding exactly once. Counting replacements too is conservative and avoids an extra lookup/hash invocation. Saturate on overflow; never introduce an ordinary gameplay error. |
| entry(k) | Call inner.entry(k) once. If its returned std Entry is Vacant, credit one possible insertion in the disjoint history field, then return the SAME std Entry. Occupied entry replacement cannot add a key. A vacant entry dropped unused merely overcounts. This preserves current or_default/or_insert/or_insert_with/and_modify chains, callback evaluation, borrows, and hash count without a separate Entry state machine. |
| get_mut / values_mut / iter_mut | Return only mutable values or immutable keys plus mutable values. These cannot grow this map's backing allocation. Nested owned containers inside V still require their own history. |
| remove / remove_entry / retain / clear / drain | Forward without resetting history. These can leave retained allocation or tombstones. Returned Drain cannot insert back into the same map. |
| new/default | Create the ordinary raw empty map once with its ordinary hasher, and zero history. A genuinely new wrapper replacement resets history because the old storage is dropped. |
| Read access | Implement immutable Deref<Target=HashMap<...>> or explicit as_map(). Do not expose DerefMut, AsMut<HashMap>, raw_mut(), or an untracked mutable-closure escape. |
| Iteration | Implement IntoIterator for &OwnedHashMap and &mut OwnedHashMap if current for-loop sites need it. For-loop trait selection does not acquire an IntoIterator implementation merely through Deref. Consuming iteration can forward to the owned raw iterator because that owner is being consumed. |

Direct &mut HashMap helper parameters such as result_stats::close_deployment should change to the internal wrapper type or to narrower operations, instead of adding a raw mutable escape for convenience. Immutable &HashMap helpers, ordinary lookup methods, and values()/iter() reads retain coercions or method calls.

Do not offer broad From<HashMap>, FromIterator, extend, reserve, or collection-conversion conveniences until their allocation histories are defined. Many temporary maps in movement planning/blocking are not retained owners and can remain ordinary HashMaps. A global import alias replacing every HashMap occurrence would affect public aliases (for example stats.rs:166 TriggeredEffects), ephemeral working maps, and public data contracts unnecessarily.

If future required owned mutation uses reserve(additional), a checked credit increment covering the requested spare growth (with current live length already bounded by prior credit) can be derived before delegating the same native reserve. with_capacity(n) needs at least n initial credit even when len is zero. Bulk extend/collect may reserve from iterator size_hint before yielding any items; counting yielded items alone is not a sound allocation history. Preserve existing iterator evaluation and native reserve behavior instead of collecting a second buffer just for accounting. None of these extra APIs is needed merely to convert the ordinary private-map mutation shapes identified above.

## Equality, Debug, and serialization are part of semantic preservation

Do not derive PartialEq/Eq over both fields. Implement them by comparing inner only. This is materially required: BlockRuntimeState equality is used by movement/blocking.rs:119–124 to return early before changing target preferences. Comparing history would turn two gameplay-equal maps into different states and change that branch.

Likewise, delegate Debug to inner if preserving the existing owners' derived Debug output matters; do not print internal history through previously map-shaped output. Delegate Serialize directly to inner. The accounting record must not appear as a new field, alter map enumeration, or influence fingerprints/profile/checkpoint payloads. These details keep private implementation state out of public documents and logical equality.

All history is carried through derived owner Clone because it is inside the wrapper; a separate per-owner counter approach would have to remember it in GameCoreState::clone_non_battle_transaction_state, BattleEntrySnapshot::capture/restore, and the checkpoint macro's field list as well as ordinary constructors. Current paths at world/state.rs:164–190,549–565,400–446 are concrete reasons to prefer containment.

## Fresh Deserialize without changing wire shape or rehashing an existing map

Inventory's three map owners and EmployeeRoster currently derive Serialize/Deserialize. Simply adding a serde-skipped history field defaulted to zero is unsound: a newly deserialized nonempty or pre-reserved map would have no recorded allocation credit.

A small custom Deserialize for OwnedHashMap can preserve current native deserialize behavior:

1. Delegate to std HashMap<K,V,S>::deserialize(deserializer) exactly once.
2. Immediately record a conservative initial allocation credit from this freshly produced raw map's capacity.
3. Move that SAME raw map into the wrapper, without enumerating/reinserting or changing its hasher.

This is a **fresh-only** use of capacity, not a reversal of BC006-F01. Cargo.lock:1980–1993 pins serde/serde_core 1.0.228. The inspected serde_core-1.0.228/src/de/impls.rs map visitor at 1536–1548 creates HashMap::with_capacity_and_hasher(cautious(size_hint), S::default()) (1565–1571), then only inserts entries. It has no remove/clear path and therefore no tombstones; duplicate input keys replace values without deleting control slots. Under root's pinned std/hashbrown model, this no-deletion state has usable capacity equal to full usable bucket capacity, which provides a valid conservative seed for the 2P bucket envelope. Large reserved size hints are covered even when few entries result.

This constructor must stay coupled to that inspected Deserialize implementation. Do not generalize it to an arbitrary caller-supplied existing HashMap merely because it currently has the same len/capacity. Do not let untrusted serialized history supply the credit. The raw map's actual construction supplies it. The default Deserialize::deserialize_in_place can replace the whole wrapper using this fresh path; do not add an in-place raw-map reuse shortcut that resets history while retaining old allocation.

This keeps the same serialized map shape and the same underlying HashMap deserialization call. V6 import already obtains typed checkpoints through serde_json::from_slice (world/active_run.rs:114–124). Capture/load histories need not be byte-equal: history is private storage bookkeeping, while gameplay and wire shape retain their current meaning. Byte/fingerprint preservation should be validated by root's existing checkpoint tests after implementation; it is not proven by this report alone.

## Comparison with adding one counter per map

| Approach | Advantages | Actual costs/risks in this source |
| --- | --- | --- |
| Separate counter for every owner field | Leaves every raw HashMap field type unchanged; useful for the public units exception and already implemented Rapier wrapper maps | Counters must be updated beside insert and Entry operations across world/combat/event/shop and battle build/sim/skill/boss/result-stats modules. History must be separately carried through manually written captures/restores. Public mutable raw map access can bypass counters. Native clear must preserve counters; true replacement must reset the correct one. |
| Private tracked map | Centralizes insertion/entry/clear/import rules; containing owner Clone/capture moves history automatically; most existing private mutator call syntax remains | Requires controlled mutation methods, transparent Serialize plus custom fresh Deserialize, inner-only equality/Debug, private mutable-helper signature updates, and explicit treatment of public field/type boundaries. |
| Recommended hybrid | Wrapper for implementation-private maps; sidecar only where a real concrete public HashMap contract must remain | Must clearly state and enforce the application-owned provenance of the sidecar exception; arbitrary external mutation cannot be certified by inference. |

## Connection to whole-clone preflight

The owner visitor can read each map's checked bucket/slot bound in O(1) before walking it. This bounds a capacity-scanning iterator without first cloning the map. It then measures the owned K/V payloads using the existing typed visitor; sharing an immutable Arc remains sharing. Count each separately owned map in live state, entry snapshot, checkpoint payload, and active employee roster at its actual occurrence. A global cache keyed by a repeated pointer or logical map ID would risk conflating independent copies and is unnecessary.

Bookkeeping must remain value-owned and operation-independent; authoring usage/failure stays in existing TLS. A saturated/unknown history is rejected before an opted-in clone by the existing CloneSlots/work control, while ordinary map operations retain their prior result. A Clone simply copies the record. Dropping a rejected candidate drops only its history copy and does not alter A. This is compatible with the existing GameCore draft and tactical staged-copy structure (world/application.rs:176–214; world/combat.rs:1822–1840).

Root still needs to attach the whole visitor at actual copy boundaries and handle all other owned collections/coexistence. This report intentionally does not count those unimplemented parts as new defects. It also does not claim exact allocator or RSS bytes from a structural map-slot bound.

## Focused verification to perform after implementation

Relevant checks are: clear/remove churn followed by clone admission; cloning preserves raw iteration order/hasher and copies history; Entry closures/hash operations are not duplicated; history differences do not affect map equality or blocking early-return behavior; fresh Deserialize with duplicate keys and reservation hints yields nonzero adequate credit while Serialize matches the existing map shape; checkpoint/entry capture and restore keep gameplay output unchanged; saturation affects opted-in cloning only. Public fixture/source compatibility should be checked separately from release API compatibility. No tests were run for this report.

## Source ledger

The JSON records the 37-file source capture at 2026-09-10T08:22:52.619375Z, exact SHA-256 values/spans, and close-time drift. Searches are labeled as searches and are not represented as whole-file inspection.


Close UTC: 2026-09-10T08:27:02.377624+00:00. Capture-to-close drift: /mnt/f/work/simulator/core_enhanced/src/game/world/application.rs, /mnt/f/work/simulator/core_enhanced/src/game/world/combat.rs, /mnt/f/work/simulator/core_enhanced/src/game/battle/core/boss_runtime.rs

| Source | SHA-256 at capture |
| --- | --- |
| /mnt/f/work/simulator/core_enhanced/src/lib.rs | 65e7de2e100cafb81cb41e76e171be9e720dbe11f112c701f5339bc7934c93af |
| /mnt/f/work/simulator/core_enhanced/src/game/mod.rs | 8bfe8d164a9b2ca9339293d3cb4b1162d43b8bb82aa18e23c87ac11e5887be5a |
| /mnt/f/work/simulator/core_enhanced/src/game/world.rs | fc72d1029c33c560f918dc2487071c5720ad231d6983b9aa66dc15e4d5ee75ed |
| /mnt/f/work/simulator/core_enhanced/src/application.rs | 636b3c9cc15ae2adfa39a15392e0dee22ee7c6683885087f9cc77cefda66081a |
| /mnt/f/work/simulator/core_enhanced/src/game/world/state.rs | 129a4063f7e448b5795fd370a22a997f66eb09cd3029fb87d41fee892cf34bb0 |
| /mnt/f/work/simulator/core_enhanced/src/game/world/application.rs | eda3e77e4509368aec58228b8070b5d7edfdab6fc6f98ae036cb45e97ea7eafd |
| /mnt/f/work/simulator/core_enhanced/src/game/world/combat.rs | 23d440b58516258e9e174d586c6d06a675f9ce8c9b21534cde102574b2feb35f |
| /mnt/f/work/simulator/core_enhanced/src/game/world/event_node.rs | 947ceb5f2c4735846ef9df3ed9982da27884301284a7feb8c236515f73eeb861 |
| /mnt/f/work/simulator/core_enhanced/src/game/world/shop.rs | 22c5403b0de7169d47d62f4372ae65752998c122fc0fd5a0181bce6282a45d81 |
| /mnt/f/work/simulator/core_enhanced/src/game/world/node_flow.rs | 0521f6860a8079ec8b6558dec2b46bacdaed410a079f91f609e05debb937d79b |
| /mnt/f/work/simulator/core_enhanced/src/game/world/active_run.rs | c344f0fff0aa28e3f8afb9b5a48c2c706855dbca13c5c53342aa14d2b7d52b20 |
| /mnt/f/work/simulator/core_enhanced/src/game/resources/inventory.rs | 2b32a864fdae8958a0d57610d5193b688fe1699c4ae4f46a5cc43bbb50371de3 |
| /mnt/f/work/simulator/core_enhanced/src/game/employee.rs | 8d3940dce94a7e8d8858b39c2c48c22f367d76d3b31d23319e76859cc53eaa54 |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/mod.rs | c740b990aef0013af17d7261f7d06ebf7950a1588338457904902c3a8e09b76e |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/mod.rs | 026b85b1b7d47cfc3f08d3e88467dfe7de9558926aa1783e76b3cb56bf5bf67e |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/types.rs | 385402e5e49ead95a505f2345211875542ed370aa5558e73472c33de03e42a2d |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/build.rs | 79d13be1f52c5ad0658986216f10a941248c99116f1018a9f049de2449f8f9f7 |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/sim.rs | e624854ac13e8172b104c54e4626a761c4d017d67b8ffdb4adf3f15e50a31207 |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/boss_runtime.rs | 88e994497261f339d696b705c2e222a8256f56002591e36ef347b023a3a95153 |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/movement/blocking.rs | 1fbc2ce8bd7f51e5dd2fd6edd2b30f41d3318672c133fc1e7169c026e1de0417 |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/result_stats.rs | 1b11b716a276ab3bfd0590836f44ec71ffe0f01f800a309f8020d15ff7e8556b |
| /mnt/f/work/simulator/core_enhanced/src/game/stats.rs | 9c3d49b44c773288da84e35bc0411292e423583a4b481a1e044622c138d10269 |
| /mnt/f/work/simulator/core_enhanced/src/game/skill_fragment.rs | b4bb2acf2bda08b3ebcb1d7f740fb7b51a12d4cb3fb588b3ae1d1ad50a13a38d |
| /mnt/f/work/simulator/core_enhanced/src/game/managers/uuid_manager.rs | b78fca9caefe3ebda1203686e3efc6f6dcb1f02e7b9cb2276280aea9532809a9 |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/battlefield/mod.rs | 54d0466d93ac176ed131c57ccdd4954966bf2224d1a41ee5e0943f9a9ae77144 |
| /mnt/f/work/simulator/core_enhanced/src/game/visual_novel.rs | bd9c83551d5547a5ca425859fd40db197d3b64fbfcab27683dc5ce5a89ed29c3 |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/commands.rs | b32f1b9a9f47c7124d3944cf6bd409a6e514fb489ee36b9cbba8ea251177a542 |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/skill_runtime/projectile.rs | 8780ed276ff988e6ba8cb1e15e3cdc8d645f73e9207a0b71854286b2130f580c |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/skill_runtime/area.rs | b7cad108fd507a19ee7893bffc763bd9be06f6ab2187306a19f21b48042ea89d |
| /mnt/f/work/simulator/Cargo.lock | 80eed115bdcf0aa881da5b6c1525e6f19a0d6d8e1101ee4b285840ab7b3a8f19 |
| /mnt/f/work/simulator/core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/design-evidence/std_hashmap_bound_001.md | 3b6494a2f62af700323aa449085ce9e8c8c5e599c62dc308b88d0660c949a774 |
| /mnt/f/work/simulator/core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/design-evidence/std_hashmap_bound_001.json | 300af969d1051bf32abc95a2035cc17930cf0edd94c00bbd6d0d0020269f7c62 |
| /tmp/r03-rust-1.91.1-std-hash-map.rs | 628357909f9fa43cfe47e03f315c868b8115483079377d964e7758b106e7c6b1 |
| /tmp/r03-rust-1.91.1-Cargo.lock | b89b588a745aa7b8e57a31dd3d674eeb2d385510da9710483cff80200313027e |
| /tmp/r03-rust-1.91.1-std-Cargo.toml | 8039e2509a082251502fd014b3157e68250b488f84657af3c6bee05e2f1286e5 |
| /tmp/r03-hashbrown-0.15.5-raw-mod.rs | a88cd291be71cd2f6119a1f8b4030a0429b6b1f8771b7f2a8b140c1824788bed |
| /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_core-1.0.228/src/de/impls.rs | 9fb7a0322af51971fe1fa8f5b56c7ca3716622a6e9127b86bddfc0742c11383c |
