# R03 backend clone review 006

This is a bounded read-only source review of CloneHistory, RapierMovementWorld::preflight_authoring_clone, the max_clone_slots control primitive, and the new boss-boundary call site. It is not a completion audit. No source, tests, docs, Git state, build output, dependency, or toolchain was modified; no build/test was run. Root's INPUTS034 test execution was not performed or independently verified by this helper.

Reviewed workspace capture: 2026-09-10T08:07:50.203417Z. The JSON records exact source hashes and capture/close drift. Root is the sole writer and may produce a successor after this report. Conclusions below apply to the captured source, not an unreviewed successor.

The previous design source basis is /tmp/r03_backend_clone_design_005.md (SHA-256 029a63fd0bb6b830e25180a6feabf93aa06379438550d652218424613203f46f). Root separately reported an offline cargo tree confirming default/hashbrown and no enhanced-determinism; this helper did not run that command.

## Finding BC006-F01 — map usable capacity is not an established upper bound on retained backing slots

Status: required accounting correction in the captured code; exact std implementation envelope remains for root's follow-up. This is a structural clone-slot/work accounting issue, not a claim about full heap bytes.

Location: core_enhanced/src/game/battle/core/movement/rapier_backend.rs:121–125. The preflight adds unit_handles.capacity() and static_obstacles.capacity() directly to the Rapier history-derived slot subtotal. Those are std::collections::HashMap values (imports:1; fields:101–102), not the private hashbrown pair table whose retained buckets were bounded explicitly at 63–70.

A HashMap usable capacity value is not a raw bucket count or an upper bound on all cloned backing records. The locally pinned hashbrown 0.16 source illustrates the exact distinction: map.rs:604–619 documents capacity as a lower bound on elements that fit without reallocation; raw/mod.rs:1282–1284 returns items + growth_left. RawTable::clone explicitly allocates the source bucket count and copies its control region (raw/mod.rs:3155–3185,3292–3327). That source is verified for Rapier's internal pair map; it must not silently be claimed to be the Rust standard library's exact version.

The mismatch is observable as a lifecycle obligation, not merely a missing annotation. Both wrapper maps retain storage after clear(), while CloneHistory resets when the native Rapier owners are replaced (wrapper:248–258). A previously large map followed by clear therefore has almost no other history allowance to conceal a capacity-versus-backing discrepancy. The newly added test at 1049–1071 explicitly verifies capacity retention, but only checks that capacity is preserved; it does not establish that this value bounds cloned backing slots. Deletion/tombstone churn also makes a conversion based only on the current usable capacity suspect. The helper did not execute a runtime repro or inspect installed Rust std/hashbrown source: local rust-src was not found.

Minimal correction: establish the pinned standard-library HashMap growth/clone relationship, then preserve an owner-local map growth/capacity high-water summary at every insertion (unit map:305–313; static map:330–337). Preserve that summary across backend clear() because those map allocations survive; copy it across Clone. Use checked arithmetic and a proven bucket/work envelope, with overflow retaining the existing opt-in failure behavior. A map replacement with HashMap::new would have a different reset boundary. Merely multiplying the current capacity by an arbitrary constant does not close the current-source finding.

Root acknowledged this issue and is independently checking Rust 1.91.1 official source. This report does not mark a proposed factor or successor fix as verified.

## Checked connections with no additional concrete defect established

| Area | Inspected source and conclusion |
| --- | --- |
| Insertion history | wrapper:293–303 and 315–329 update history after the successful body-plus-collider insertion. Both production insertion paths are covered. Each current builder creates exactly one body and one collider, so inserted() incrementing each insertion counter by one matches the methods. Counters are maintained outside authoring as well as inside it. |
| Removal history | forget_unit:193–206 and remove_static_obstacle:340–354 record the actual decrease in ColliderSet::len. The pinned RigidBodySet removal path only removes attached colliders, so previous minus current cannot become negative through that operation. Counters saturate rather than wrapping. Radius changes call forget then insert (357–366); stale/dead-unit and static-obstacle replacements converge on these methods. |
| Arena and pending-list accounting | storage_slots:58–87 uses checked arena rounding with a four-slot minimum and keeps cumulative insertion/removal history. This preserves the 005 finding that a removed body's pending modified handle remains significant even when current live len is zero. It does not destructively drain queues during preflight. |
| Pair events | refresh_broad_phase:379–394 updates pair_events on every normal return from the existing update, using events.len(). The private map's new-entry branch emits AddPair after insertion; total event count is a conservative insertion-history bound. The ordinary query path also uses this same refresh (146). No optional trace path controls the counter. The 2P, 8/7, next-power-of-two envelope matches the pinned hashbrown derivation from 005. |
| Backend clear | wrapper:248–258 actually replaces all seven Rapier owners before resetting their history. That reset is valid for those owners. It is not a valid reset for a new wrapper-map retained-allocation summary, because the two map clear calls retain allocation; see F01. |
| Overflow | Saturating usize counters feed checked sums/multiplications. An overflow produces None then usize::MAX at preflight:123–126. Control validates max_clone_slots at a nonzero ceiling of 1<<20 (control:104–111), so the sentinel is rejected before native Clone rather than wrapping into a cheap request. Inactive preflight returns Ok before computing the estimate (wrapper:117–119), preserving ordinary gameplay behavior. |
| Control scope, cancellation, precedence | preflight polls entry before estimating/admitting, then admits CloneSlots before charging work (wrapper:116–134). control::inspect preserves an existing sticky failure and observes cancellation before the budget operation (control:230–269). No RefCell borrow or callback escapes these helpers. If both clone slots and work are exhausted at this site, clone admission wins after prior sticky failure/cancellation; this matches the code's explicit order. |
| Boss connection | run_boss_runtime_boundary calls the backend preflight before self.clone() and polls immediately afterward (boss_runtime.rs:1761–1772). A backend admission failure returns before this newly guarded copy. Cancellation seen after the native copy drops the local staged value before gameplay transition commit. The new control branch does not affect an inactive scope. |
| History cloning and ordinary continuation | RapierMovementWorld and CloneHistory retain derived Clone; no arena IDs, free-list generations, BVH/pair state, controller configuration, shapes, or ordinary gameplay branch was replaced by bookkeeping. Source tests:981–1046 check history equality, threshold rejection before copy, and equal subsequent movement results between source and its admitted clone. These tests were read, not executed by this helper. |

The tests at 1074–1097 also exercise a saturated body insertion count, inactive-scope success, and opted-in clone-limit failure without moving the original body. They are relevant checks of intended semantics, but no test outcome is claimed here.

## Primitive record meaning and limits of this review

The new field max_clone_slots and usage.clone_slots_peak are trusted host configuration and usage values, not deserializable scenario authority (control.rs:27–55,90–100). The code calls this structural admission and expressly excludes exact native allocation bytes (wrapper:114–115). A clone slot should remain an explicit bounded container/backing record work unit; it must not be described as one byte, one CPU instruction, or an exact allocator/RSS quantity. In particular, the source pair table's native bucket/control traversal cannot be justified by saying public usable capacity is the same record count.

005 identifies RigidBodyColliders as a one-handle Vec for every current wrapper body. The current formula does not label a separate attached-handle subtotal. This review did not establish an independent total underestimate from that fact: the BVH/history terms are intentionally conservative and may supply enough aggregate allowance. It should remain part of the source-to-record explanation, rather than silently suggesting that an arena entry automatically covers arbitrary nested unbounded payloads. The current wrapper's one-collider/default-mass invariant makes the payload bounded; adding multi-collider bodies or boxed custom mass properties would require another accounting review.

The boss-boundary preflight currently covers the backend part of a whole BattleCore clone. Its other fields, the nested BossRuntime clones, a whole GameCore/Application visitor, source/candidate coexistence, and complete native heap accounting remain separate obligations that root explicitly excluded from this narrow review. Their absence is not counted as a new backend defect here. Likewise, this report does not claim instruction-level cancellation during native Clone.

No new panic, missed production insertion/removal/refresh seam, or ordinary gameplay/ID change was independently established in this captured patch. F01 remains open for the root's implementation and verification.

## Exact source ledger

The companion JSON records all file digests, inspected spans, capture timing, and drift. Package/source facts reused from 005 are identified separately from fresh workspace reads.


Close UTC: 2026-09-10T08:12:44.332738+00:00. Workspace capture-to-close drift: none in the five initially captured files.

| File | SHA-256 at reviewed capture (or close-only, explicitly recorded in JSON) |
| --- | --- |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/movement/rapier_backend.rs | c8ce8325a28d9ac2685bc6c73ba6b63cf7e472163200743f203b89ba0b5993e9 |
| /mnt/f/work/simulator/core_enhanced/src/authoring/control.rs | 1c465fc9e00ef53d1b93cab11f4cf97fdffa341b02715d3227af8d0f8803edcc |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/boss_runtime.rs | 88e994497261f339d696b705c2e222a8256f56002591e36ef347b023a3a95153 |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/movement/mod.rs | 096f70bc180482ae54d3ef142989ca44b7cc677cb1a050c56df19286ff77ce32 |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/movement/engine.rs | 89e753f624927d7c4a057777f85d816f189c019001ea6c76227bed3047fa582f |
| /mnt/f/work/simulator/core_enhanced/src/game/battle/core/mod.rs | 026b85b1b7d47cfc3f08d3e88467dfe7de9558926aa1783e76b3cb56bf5bf67e |
| /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.16.0/src/map.rs | bf069e9f391abbbfca0918f5798229fd44380afc4f126ce64afd387b5401a8e3 |
| /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.16.0/src/raw/mod.rs | a88cd291be71cd2f6119a1f8b4030a0429b6b1f8771b7f2a8b140c1824788bed |
| /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rapier2d-0.32.0/src/geometry/collider_set.rs | 4895f6a2dad10b2b9c51527dd91cb0ba61f14694be36da2fcca02f0a98198c3e |
| /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rapier2d-0.32.0/src/data/arena.rs | 605ee1a2f7059cea0f983265954cd2e9e7b44564504cc8073e1783eeeda9931f |
| /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/parry2d-0.26.0/src/partitioning/bvh/bvh_tree.rs | bc5494726bfa107b2bbaf86a2f825e2707fd80b22d1199b263a052fed89ebd15 |
| /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/parry2d-0.26.0/src/partitioning/bvh/bvh_optimize.rs | cb9ec6f079badc2d997b5445c854494e03769b72231e220df4012c39092a81ad |
| /tmp/r03_backend_clone_design_005.md | 029a63fd0bb6b830e25180a6feabf93aa06379438550d652218424613203f46f |
| /tmp/r03_backend_clone_design_005.json | 9b7ae39a98f764ee4e7c715f64cdb3051cdc1570a0c6a489d1bc60df2d0d5673 |
