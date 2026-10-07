# R03 Rapier backend clone storage seam — design 005

This is the requested bounded read-only design investigation, not an implementation or completion audit. No repository/dependency source, tests, docs, Git state, build output, or toolchain was changed; no build/test/network/install command was run. Only this new report and its JSON companion are written.

## Result

The current wrapper can acquire a defensible **structural clone-work/storage-slot bound without modifying Rapier**. The smallest seam is a few private, copied, monotonic history counters in RapierMovementWorld, plus an owner-local preflight before its existing derived Clone. Current live body/collider counts alone are insufficient.

There is a separate limitation: these public APIs do not expose every private allocation's exact byte layout. In particular, Arena's private Entry<T> backing cannot honestly be converted to exact native bytes with a guessed per-body multiplier. Existing authoring control has a work budget and retained-entity count, not a general heap-byte budget; a source-backed structural admission implementation should state that distinction. The following proposal is concrete for those existing units and does not introduce a new public budget or dependency.

## Dependency identity and capture

Workspace Cargo.lock pins rapier2d 0.32.0, parry2d 0.26.0, hashbrown 0.16.0, and vec_map 0.8.2. core_enhanced/Cargo.toml:50 requests rapier2d = "0.32.0"; no workspace source patch was found in the inspected manifest. Local sources are under:

    /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/<package-version>/

The .crate archives are also local under the corresponding registry/cache directory. Their SHA-256 values match Cargo.lock; every inspected dependency source file was compared byte-for-byte by SHA-256 against its archive member and matched. No extraction or download was performed. These local source directories lack .cargo-checksum.json, so the archive/member verification, not a nonexistent checksum file, is the provenance evidence.

    rapier2d-0.32.0  73edcbebd2c2a9eb0041e40b046b5136da32990fcd2ac76b9414ebff45908e18
    parry2d-0.26.0   e7b05ce9599a62eec9f6b761cf32647ab1dee43477b5c00ff9221a4b7cb80c4b
    hashbrown-0.16.0 5419bdc4f6a9207fbeba6d11b604d481addf78ecd10c11ad51e76c2f6482748d
    vec_map-0.8.2    f1bddf1187be692e79c5ffeab891132dfb0f236ed36a43c7ed39f1165ee20191

Paths below are relative to the identified package unless prefixed with "wrapper" or "control". The JSON contains exact read spans and file hashes. The 43-file capture occurred at 2026-09-10T07:56:17.895885Z, after the source investigation, followed by a close rehash. Six principal Rapier files also have earlier first-read hashes. This is not a claim that all workspace files were frozen throughout root's concurrent work.

The inspected dependency manifests select the normal HashMap alias, hashbrown plus foldhash, unless enhanced-determinism is enabled. core_enhanced's manifest requests no Rapier feature overrides. The bucket derivation below is scoped to that normal configuration. Enabling enhanced-determinism changes Parry's map to IndexMap and requires another derivation; Cargo.lock by itself is not a universal feature-resolution proof.

## What the seven derived Clones duplicate

Wrapper = core_enhanced/src/game/battle/core/movement/rapier_backend.rs.

| Owner and exact source | Owned Clone content | Public read seam and current wrapper restriction |
| --- | --- | --- |
| IslandManager, rapier dynamics/island_manager/manager.rs:32–46 | VecMap<Island>, awake/free Vecs, traversal VecDeque, stack Vec even though serde-skipped; scalar/Copy optimizer | active_bodies iterator is not an allocation-capacity accessor. Here no physics pipeline or island-building call occurs. Removal exits when the default empty island map has no entry (58–67). Optimizer is Copy, optimizer.rs:24–53. This owner stays default-empty. |
| BroadPhaseBvh, geometry/broad_phase_bvh.rs:17–33 | Bvh, BvhWorkspace including serde-skipped scratch storage, private pair HashMap, frame and optimization fields | Public as_query_pipeline yields QueryPipeline.bvh: &Bvh (pipeline/query_pipeline.rs:43–55,135–150). This exposes Bvh leaf_count and approximate heap_memory_size, but no BroadPhase pair/workspace capacity. This is the substantial nonempty private owner here. |
| NarrowPhase, geometry/narrow_phase.rs:63–74 | Arc query dispatcher (shared), two InteractionGraphs, Coarena<ColliderGraphIndices> | contact_graph/intersection_graph, pair iterators; each graph.raw_graph().raw_nodes()/raw_edges() exposes live slices. Here narrow phase is only queried for its dispatcher; wrapper never feeds pair events or invokes contact computation. All graph/coarena lengths remain zero. |
| RigidBodySet, dynamics/rigid_body_set.rs:70–79 | Arena<RigidBody>, ModifiedRigidBodies handle Vec, default_fixed body even though serde-skipped | len/is_empty/iter expose occupied bodies, not backing slots. Arena::capacity exists but bodies field is private. take_modified is crate-private. Wrapper creates one-collider kinematic/fixed bodies and never drains modified bodies. |
| ColliderSet, geometry/collider_set.rs:48–52 | Arena<Collider>, modified handle Vec, removed handle Vec | len/iter expose occupied colliders. take_modified/take_removed are public but mutate the source; they are unsuitable for preflight. Existing refresh uses them legitimately after iter_mut repopulates the modified list (wrapper:291–305; collider_set:130–140). |
| ImpulseJointSet, dynamics/joint/impulse_joint/impulse_joint_set.rs:69–78 | Coarena rb graph IDs, Arena joint IDs, InteractionGraph with joint payloads, wake/join HashSets | len/is_empty/joint_graph getter; no whole-set capacity accessor. Wrapper never inserts joints. Removal only acts after a nonempty coarena lookup (464–477), so default-empty state remains. Arena still owns four initialized Free slots: empty joint count is not zero arena-clone work. |
| MultibodyJointSet, dynamics/joint/multibody_joint/multibody_joint_set.rs:90–100 | Arena<Multibody>, Coarena link mapping, connectivity graph, wake/join HashSets | iter exposes active links; no whole-set capacity accessor. Wrapper never inserts multibodies. remove_joints_attached_to_rigid_body does nothing after the empty rb2mb lookup (291–309). Its arena likewise retains the default four Free slots. |

InteractionGraph::new uses Graph::with_capacity(10,10) (geometry/interaction_graph.rs:23–28). Graph derives Clone over two Vecs (data/graph.rs:127–133). These initially empty graph Vecs have reserved source capacity, but no live element payload is cloned. Keep source retained capacity, initialized vector length, and clone work distinct. The spare capacity of every Vec must not be assumed to be copied wholesale. Hashbrown's RawTable Clone is different: it explicitly allocates the source bucket count.

The empty-state reasoning is specific to this private wrapper's constructors and methods, not a general property of Rapier. Wrapper:115–127/254–267 passes sets to removal, but RigidBodySet::remove removes the body first, passes false to attached ColliderSet::remove, and then invokes the empty joint removal paths (rigid_body_set.rs:201–239). The missing parent and false wake flag cannot create an island. clear (wrapper:167–178) replaces all seven sets with new owners. Query controller calls use immutable pipelines. Do not expose mutable Rapier owners or add PhysicsPipeline/joint insertion later without revisiting this invariant.

Nested body/collider payloads matter. RigidBody contains RigidBodyColliders(Vec<ColliderHandle>) and RigidBodyMassProps.additional_local_mprops: Option<Box<...>> (rigid_body.rs:33–66; rigid_body_components.rs:337–365,1094). The wrapper's default builders do not populate that optional box (rigid_body.rs:1943–1960); each successful insertion attaches exactly one collider. ColliderShape is SharedShape, a derived Clone of Arc<dyn Shape> (collider_components.rs:108; parry shape/shared_shape.rs:71–77), so even shape geometry is shared by Clone. ColliderMassProps can contain a Box in general, but wrapper builders retain the default Density variant (collider_components.rs:113–131). The fixed default body has no attached collider. Controller is Copy (control/character_controller.rs:170–205). The two wrapper HashMaps hold Copy records and have public capacity() available directly to the owner.

## Three concrete traps and source-backed bounds

### 1. Arena holes and the never-drained rigid-body queue

Arena derives Clone over Vec<Entry<T>>; Free slots are real initialized entries (data/arena.rs:16–33). new starts with four slots (83–133); insertion when full appends exactly its current slot length (309–325,633–650); removal frees entries without shortening that vector. With only this wrapper's new/insert/remove lifecycle, the exact logical slot count is:

    S(H) = max(4, checked_next_power_of_two(H))

H is peak simultaneously occupied bodies/colliders since the last replacement with new(), not current len. Update H after every successful body/collider insertion, including static obstacles and radius-change replacement. Both arenas must be accounted for. Existing objects carry the historical arena slots through Clone even after large shrinkage of gameplay populations.

RigidBodySet::insert always pushes a handle into modified_bodies (150–162). Removal does not delete that pending handle. Wrapper never invokes the crate-private take_modified or PhysicsPipeline reset. get_mut uses push_once, whose flag remains set in this lifecycle (295–299; data/modified_objects.rs:52–67). Therefore cumulative successful rigid-body insertions since new() is the queue-length count for the current methods; at minimum it is a valid explicit history counter. Ten thousand insert/remove cycles with one live body are not a one-body clone. This is a necessary whole-fork preflight input.

Collider pending lists can be bounded without destructive inspection by cumulative successful collider insertions/removals since new(). A tighter counter can reset at the existing take_modified/take_removed seam. iter_mut first clears/repopulates modified handles and existing refresh drains both lists before update; current mutators do not arbitrarily reset collider change flags. Ordinary wrapper logic and control activation must not alter the queue lifecycle to make accounting easier.

These are **backing-entry/handle counts**. The private enum Entry<T> prevents a direct size_of::<Entry<RigidBody>>() at the wrapper. A tuple or Option with a similar-looking layout is not an honest substitute. Exact native allocation bytes for these private entries need a dependency-owned accessor or another explicitly audited compiler/layout mechanism, outside this no-dependency-change proposal.

### 2. BVH source estimate and persistent workspace

Parry Bvh derives Clone over node Vec, parent Vec, and VecMap<BvhNodeIndex> (bvh_tree.rs:1743–1756). That VecMap stores Vec<Option<V>>, expands to highest inserted key + 1, and does not shorten on remove (utils/vec_map.rs:84–88,578–610,743–755). Track maximum issued collider raw index + 1; it bounds leaf-map initialized length even when the collider was removed. Alternatively S(H) bounds the possible raw index range for this arena lifecycle.

Bvh::heap_memory_size() (bvh_tree.rs:2184–2193) is explicitly approximate. It sums node capacity * size_of<BvhNodeWide>, parent capacity * size_of<BvhNodeIndex>, and leaf-map capacity * size_of<BvhNodeIndex>. The last term omits the Option wrapper used by VecMap; BvhNodeIndex is a public usize newtype (1240–1248). It also does not include BroadPhase workspace/pairs. Do not present the raw result as an exact or complete clone bound. If used as a conservative estimate for the tree alone, a checked multiplier computed from ceil(size_of<Option<BvhNodeIndex>> / size_of<BvhNodeIndex>) repairs this particular element-size omission while overcounting the other terms. It still describes allocation payload estimates, not allocator/RSS accounting.

BvhWorkspace's full fields are at bvh_tree.rs:135–145: refit_tmp, rebuild_leaves, two frame/index scalars, optimization_roots, BinaryHeap queue, VecDeque dequeue, and traversal_stack. refit swaps the old nodes into refit_tmp and does not empty it (bvh_refit.rs:170–179). rebuild_leaves is left populated after optimization; early return on an empty tree can preserve the previous list (bvh_optimize.rs:237–325). Thus serde-skipped workspace is material clone storage.

For normal returns at this wrapper's mutation boundaries, a source-derived initialized-element bound can use collider peak H:

- Current tree nodes and parents are at most H, with zero/one/two-leaf root special cases included. BroadPhase always calls refit before returning.
- refit_tmp is at most 2H BvhNodeWide entries: with more than two leaves refit truncates source and target to the reachable compact tree size (bvh_refit.rs:251–255). With at most two leaves the source is not truncated, but it starts from the previous compact tree and this update inserts at most H new leaves; optimization of at most two leaves appends no additional internal node. This factor comes from those two buffers/branches, not from a body-byte guess.
- rebuild_leaves is at most H BvhNode entries: collected true leaves or disjoint pseudo-leaves of the current tree; an empty-tree early return can preserve an older value bounded by the same historical H.
- optimization_roots is cleared before normal return; heap queue is cleared, dequeue drained, and traversal stack emptied (bvh_optimize.rs:325,379–382,434–439; bvh_traverse_bvtt.rs:148–203). Their vector spare capacity does not add initialized Clone elements. Local traversal SmallVec is not retained in the owner.

This is an implementation proposal relying on the cited successful-return invariants, not a proof for arbitrary externally mutated Bvh or unwinding recovery. Do not silently generalize to other optimization strategies or clone in the middle of update. A dependency-owned workspace storage accessor would remove the need to maintain this invariant proof, but no such public accessor exists in the inspected API.

### 3. Pair-table retained buckets after churn

BroadPhase pairs are hashbrown::HashMap<(ColliderHandle,ColliderHandle),u32> in this configuration. Clone delegates to RawTable::clone, allocates the **same bucket count**, copies control bytes, and clones occupied entries (hashbrown map.rs:190–196; raw/mod.rs:3155–3185,3292–3327). Current occupied pair count is therefore insufficient even though key/value payloads are Copy. BroadPhase retain never shrinks the table (broad_phase_bvh.rs:225–258).

There is an existing wrapper-level observation seam: every actual new pair-map entry emits AddPair immediately after insertion (broad_phase_bvh.rs:196–207). The wrapper already owns the output events Vec, then discards it (wrapper:295–305). A private monotonic P += events.len() after every successful update is an O(1), order-independent upper bound on total pair insertions and on every historical live pair count. Counting only AddPair is tighter but adds a scan unless already scanning. This is ordinary internal backend bookkeeping; it must not depend on optional authoring trace or consume trace allowance.

A concrete conservative bucket bound follows from the pinned hashbrown source:

    P == 0: B = 0
    P > 0: R = checked_mul(P, 2)
             B = max(16, checked_next_power_of_two(checked_mul(R, 8) / 7))

Reason: reserve_rehash_inner reallocates on the tombstone path only when new_items > full_capacity/2; hence its max(new_items, full_capacity+1) requested capacity is at most 2*new_items, which is at most 2P (raw/mod.rs:2627–2681). capacity_to_buckets uses a small-table maximum 16 and the 8/7 load adjustment then next_power_of_two (103–161). This bounds retained buckets across deletion/churn and avoids assuming current geometry yields every potential pair. The pair counter is updated on every existing refresh, including refreshes inside corrected movement queries.

For the pair table alone, a native requested allocation-payload bound is also expressible without private-layout guessing. The element type is public:

    PairEntry = ((ColliderHandle, ColliderHandle), u32)
    A = max(align_of<PairEntry>(), 16)
    L = checked_align_up(B * size_of<PairEntry>(), A) + B + 16

The pinned TableLayout::calculate_layout_for uses precisely element bytes aligned to max(element alignment, Group::WIDTH), plus B + Group::WIDTH control bytes (raw/mod.rs:181–219). All inspected group implementations have width at most 16 (control/group/{mod,sse2,neon,lsx,generic}.rs). Using 16 is a source-derived architecture envelope, not a guessed per-unit memory constant. When B=0 there is only the empty singleton and no pair-table allocation. L excludes allocator overhead. If feature/backend implementation changes, revalidate or reject the unsupported accounting model; do not silently apply it to IndexMap.

## Minimum implementation seam

1. Add one private Copy/Clone bookkeeping record to RapierMovementWorld: body/collider occupancy peak, cumulative successful body/collider insertion and collider removal counts (or exact pending-list summaries), max issued collider raw index + 1, and cumulative broad-phase event count P. Counters are maintained from construction in ordinary use as well as inside authoring. A scope introduced later cannot reconstruct a lost private capacity history from current live values.
2. Update only at successful insertion/removal and existing refresh completion seams; reset exactly when clear replaces the Rapier owners with new(). Copy this record with the derived Clone. Do not reset counters on ordinary fork. The two wrapper HashMaps use clear(), which retains their capacity; do not reset a claimed map-capacity bound there—read their own public capacity() when needed.
3. Use checked arithmetic or an explicit overflow/unknown-history sentinel. Ordinary non-authoring gameplay must not fail because a bookkeeping total saturated; the sentinel makes an opted-in clone preflight fail through the existing control path before cloning. Zero/wrapped counters are unsound.
4. Add a private owner-local preflight that polls cancellation, computes and charges the structural subtotals above, and passes the appropriate subtotal into the enclosing whole-fork retained meter. Include source and candidate coexistence according to that meter; do not replace its existing aggregate with the backend count or double-count the same owner. control.rs:332–424 already supplies checked work charging and resource admission; its TLS scope remains operation-local.
5. After successful admission, retain the ordinary derived Clone and its arena generations/free-list order, BVH/pair timestamp state, controller config, Arc sharing, and wrapper maps. Poll again before accepting the candidate. No borrowed pointer/raw inner owner escapes. A cancellation arriving during the bounded native Clone is observed after it returns; admission is not instruction-by-instruction interruptibility inside Rapier.
6. If a whole-fork promise explicitly demands exact native heap bytes for all private fields, this wrapper-only design does not close that stronger requirement. Minimum missing dependency seam is a read-only typed storage summary exposing Arena initialized slots/entry allocation bytes and BroadPhase workspace/pair backing stats (with checked arithmetic), or equivalent dependency-owned clone-cost visitor. Do not use serialized shape, public battle snapshot, replay/reconstruction, dropping backend state, changing optimizer, or rebuilding the BVH to claim the existing clone cost was covered.

## Remaining boundaries

The normal wrapper production lifecycle makes empty Island/NarrowPhase/joint payloads and one-collider/default-mass bodies usable invariants. They are not interchangeable with a generic arbitrary Rapier world. Added mutation sites, feature changes, deserialization/import of Rapier owners, or a new backend must update this accounting seam. No new public state machine, gameplay rule, content-name branch, or success stub is proposed.

This report closes the local dependency-source availability question and identifies implementable structure/history bounds. It does not claim exact allocator/RSS coverage, cooperative cancellation within native Clone, or an implemented and tested whole-CoreApplication fork. No test was run.

## Source ledger

Full exact SHA-256 values, inspected spans, capture/close drift, and archive-member matches follow in the companion JSON. The table below duplicates file digests for reviewability.


Capture/close rehash: no drift in the 43 recorded files; the six earlier first-read hashes also match. Close time: 2026-09-10T08:01:16.254449+00:00

| Package | File | SHA-256 at capture |
| --- | --- | --- |
| workspace | Cargo.lock | 80eed115bdcf0aa881da5b6c1525e6f19a0d6d8e1101ee4b285840ab7b3a8f19 |
| workspace | Cargo.toml | 98c9cf4dfefe3eda0ff7c692af94666a491a3460f073b3022c4c360b1a3e849c |
| workspace | core_enhanced/Cargo.toml | f489f2392299ff61ecda75e5f2e2e83acb3e72419b4128cfd9cec064eb0d419d |
| workspace | core_enhanced/src/game/battle/core/movement/rapier_backend.rs | 764124edb49ab1181352f1e9da83b0ef48d2fe517dc64dc32982402fcf153af9 |
| workspace | core_enhanced/src/authoring/control.rs | fe69b34d3303d4d465d94c999dc750fa8eaf33a6f34982957f3c5a93ff976bc5 |
| rapier2d-0.32.0 | Cargo.toml | 30b1273b7ab08ebd86d773db5c5520e071533ec666b2951772bdff21276b531c |
| rapier2d-0.32.0 | src/dynamics/rigid_body_set.rs | 225b63f2303bdb0e3afda235b16a4a5e459c7b40296ba4d85fd253652af7557e |
| rapier2d-0.32.0 | src/geometry/collider_set.rs | 4895f6a2dad10b2b9c51527dd91cb0ba61f14694be36da2fcca02f0a98198c3e |
| rapier2d-0.32.0 | src/geometry/broad_phase_bvh.rs | f5025dc0d5f78bafbf253f8540784169e233dfecb2f6332ce9c866c0d89a9a14 |
| rapier2d-0.32.0 | src/geometry/narrow_phase.rs | 4410abdb51eadff99ff5b118debb21aa0b3e86aa5018118795765908f581f5d9 |
| rapier2d-0.32.0 | src/dynamics/joint/impulse_joint/impulse_joint_set.rs | 21ec909e763bcac67830be68e083928ef7e87c7e52c715f553e9b57f0305f61f |
| rapier2d-0.32.0 | src/dynamics/joint/multibody_joint/multibody_joint_set.rs | d82850546396942dd3ad85f0a4b1e03b052e6595fec9205dd6440272324df663 |
| rapier2d-0.32.0 | src/dynamics/island_manager/manager.rs | bb6c7372b439accdf765a60860a7aaa4dc07cc7de62ea623857aebb344646e92 |
| rapier2d-0.32.0 | src/dynamics/island_manager/optimizer.rs | bb98bf9944375be8a1947e4be53cfaa96976a460cf829a60654a3686d1e8b5c0 |
| rapier2d-0.32.0 | src/dynamics/rigid_body.rs | ed60848c41698a2bee8957cc5bfb61911a897fabfc7310eae623979fd50a65b1 |
| rapier2d-0.32.0 | src/dynamics/rigid_body_components.rs | a15d5e8ff29707505fb042b8717634c6efb30e1b32e7e051945344736162f7a4 |
| rapier2d-0.32.0 | src/geometry/collider.rs | df3f29104a05f147b94bdd94c73828743a3d8b36282b531926e525bd4a1a8423 |
| rapier2d-0.32.0 | src/geometry/collider_components.rs | 82805e8316c8f218b2fc3587f69ff65b696782c4c63ff7b853f1f9ac2ee18608 |
| rapier2d-0.32.0 | src/data/arena.rs | 605ee1a2f7059cea0f983265954cd2e9e7b44564504cc8073e1783eeeda9931f |
| rapier2d-0.32.0 | src/data/modified_objects.rs | 45ad943a88509b87a98cda228aa40f3742d8621c2cd4de1dd21ce0dac0e9fd64 |
| rapier2d-0.32.0 | src/data/coarena.rs | 37893a0284ac8b9273ac0c416e5d7d701305dd0fe5e39fb39a6a5739def92175 |
| rapier2d-0.32.0 | src/geometry/interaction_graph.rs | 181da80d53ceb56cae7b77c700060b0f585c0f8aa5254f4ccc14d410e3683f4a |
| rapier2d-0.32.0 | src/data/graph.rs | c88dfc5ba49d1002b90e0ce39b9354cad0f91298beccddd1af06823a858993c9 |
| rapier2d-0.32.0 | src/pipeline/query_pipeline.rs | d0eed8987f9c17aa6182034ae0693ee37960c42991e4288ea5e7262c8730bea3 |
| rapier2d-0.32.0 | src/control/character_controller.rs | 5425deffce8471eef6dacd380eb0296697a9d9632bdbff14b9955450da9aa95a |
| parry2d-0.26.0 | Cargo.toml | 52c658d92a218be5767a1e3a9b4eaf1d8c545bb89e6f0d5374a85caee5ae1ee6 |
| parry2d-0.26.0 | src/partitioning/bvh/bvh_tree.rs | bc5494726bfa107b2bbaf86a2f825e2707fd80b22d1199b263a052fed89ebd15 |
| parry2d-0.26.0 | src/partitioning/bvh/bvh_optimize.rs | cb9ec6f079badc2d997b5445c854494e03769b72231e220df4012c39092a81ad |
| parry2d-0.26.0 | src/partitioning/bvh/bvh_refit.rs | e1b83b4b376fdd7de4939428479c044929069e58d2d3c04e0483d52447cd287d |
| parry2d-0.26.0 | src/partitioning/bvh/bvh_insert.rs | 696f3139b1767633784a6455a61fa4f0fcb47d63f76bddcb9306febf870f45ec |
| parry2d-0.26.0 | src/partitioning/bvh/bvh_binned_build.rs | 15b77bad7489212b9faf1e047805a8e1732e8d02f02a830336b5f76dff7ed27e |
| parry2d-0.26.0 | src/partitioning/bvh/bvh_traverse_bvtt.rs | 5368f24673ed57d0c64f9bbdd3a7fdf7b1a1cbb859134229df06bb71217ba71d |
| parry2d-0.26.0 | src/utils/hashmap.rs | 0166090169c7a69271af7371f4089a14675070076c0ebcef39ff089598217b66 |
| parry2d-0.26.0 | src/utils/vec_map.rs | bac667db9df3f5c84e258e58e5d861ce3ce80b93a3a0d014b75f5b76614b7f00 |
| parry2d-0.26.0 | src/shape/shared_shape.rs | 7cec2012b2193c9dc60738a2a1ce2b9e873a83f730e6d0d15bfe616eead271c5 |
| hashbrown-0.16.0 | src/map.rs | bf069e9f391abbbfca0918f5798229fd44380afc4f126ce64afd387b5401a8e3 |
| hashbrown-0.16.0 | src/raw/mod.rs | a88cd291be71cd2f6119a1f8b4030a0429b6b1f8771b7f2a8b140c1824788bed |
| hashbrown-0.16.0 | src/control/group/mod.rs | 03d5cacaf762fcb0c9878e9f56355c7c9d0935d688d01c5e0d12aa673e16795d |
| hashbrown-0.16.0 | src/control/group/sse2.rs | 833da90ab3fd72d306d5b999de96a4b6b52aa1a16ee7273c89bd12c9ead3bc45 |
| hashbrown-0.16.0 | src/control/group/neon.rs | bde7726d321bd036cb9001818f962784346b932087b9f655c85ab26667d55fbb |
| hashbrown-0.16.0 | src/control/group/lsx.rs | 99c308a2e07133dce18b88230fbb054d5d1edcf465b87ee9fb1f409eacc4f241 |
| hashbrown-0.16.0 | src/control/group/generic.rs | 939b2e35e6ea5430a606a054ffc8ea3e22b867d83f2afed15525cf59a617eb6e |
| vec_map-0.8.2 | src/lib.rs | 660ae0d68bb09c0281c75c526c0bfb2059821dee065e14f5cb31e695b4c81c2d |
