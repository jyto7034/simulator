# R03 event-log COW — bounded source/design assistance 004

This helper performed read-only source/design work under root's AUTORUN §8 delegation. Root alone writes implementation/artifacts. No production/test/doc edits, builds, tests, runtime or Git mutations were performed. Only this new /tmp report and JSON companion are written. This is not a fresh completion audit.

The proposed eager preparation is technically sound for the identified log-COW paths **if the forthcoming opaque Session is fresh-only and every new candidate clone prepares the actual write side before operations**. It does not establish generalized imported-core coverage or all clone/copy/retention safety. Two issues observed during root's work were communicated and source-corrected: wrong ActiveBattleSession field names, and the spawn helper's need for an explicit interruption error.

## What actually shares and mutates

BattleEventEntries wraps Arc<Vec<BattleEventLogEntry>>, derives Clone, and exposes public DerefMut that calls Arc::make_mut (event_log.rs 76–103). BattleEventLog derives Clone and owns this wrapper alongside scalar accounting fields. The mutable IntoIterator also reaches DerefMut. Therefore any Vec-mutating operation can trigger the same hidden detach: reserve_exact, push, remove(0), clear, iter_mut or direct public index/Vec mutation.

Production append paths call ensure_accounting, reserve/push and enforce_caps; malformed/oversized entries can close and clear the window; cap enforcement can remove entries (reviewed event_log spans536–806, plus the preceding append entry). This is why preparing only before the usual push would be incomplete. Making the write-side entries unique before the candidate operation covers every later Vec mutation in that ownership interval.

record_event_log remains infallible and returns the actual sequence identity (recording.rs 13–35). It clones only the entry's source Arc handle and source-command string, observes result statistics, appends, then advances sequence or closes on actual sequence exhaustion. The existing u64::MAX sequence-exhaustion behavior is pre-existing; no new fake ID, skipped append, panic or authoring fallback is required for COW. Failure belongs before this recorder at the fallible candidate owner.

## Minimum actual clone seams

| Owner | Actual clone | Write-side preparation and failure ownership |
| --- | --- | --- |
| GameCore::execute_application | world/application183: draft = self.clone() | Prepare draft.state.active_battle's battle.event_log before operation at185. Error drops draft; live root untouched. Existing operation/commit polls remain. |
| with_staged_tactical_world_mutation | world/combat1833–1838: ActiveBattleSession clone | Prepare staged.battle.event_log1839 before revision checks/operation. Existing temporary replacement is restored if VN attachment fails. |
| BattleCore::execute_spawn_request | spawn_kernel511: before = self.clone() | Prepare **self.event_log**, not before. before is rollback storage. New error at512–515 restores before and returns ExecutionInterrupted before mutation callbacks. |
| commit_summon_batch | summon_runtime282: before = self.clone() | Prepare **self.event_log**283 before UUID/route/spawn work; failure restores before and propagates GameError. Nested spawn preparation is still necessary because its own snapshot shares the log again. |
| run_boss_runtime_boundary | boss_runtime1771: staged = self.clone() | Prepare staged.event_log1773 before plans/commits/summons. Existing loop commits by replacing self only on success. |
| Future opaque application/session fork | Not yet implemented | Immediately after that actual whole clone, prepare its active battle log before any candidate runtime/command path. Repeat at nested clones; one earlier prepare is insufficient. |

BossEncounterRuntime::plan_next at boss_runtime298 also says next = self.clone(), but that clones the boss runtime value, not a BattleCore/event log, and needs no log COW seam. Source searches distinguished this from the actual BattleCore clone at1771. Existing derived Clone is preserved.

The initial live edits used active.core/staged.core in the two world seams, while ActiveBattleSession actually defines battle: BattleCore (state507–527). Root corrected both to .battle; corrected snapshots are recorded. No build was run by this helper.

## Spawn interruption and rollback detail

execute_spawn_request returns Result<SpawnRequestSettlement, SpawnKernelError>; the prior enum had only request/trigger identity errors. Converting preparation/control failure into the existing MutationFailed → Ok(Rejected) path would produce a domain settlement and could cache it. Root added ExecutionInterrupted (spawn_kernel190–209), entry polling before cached-return paths, preparation failure restoration, and a post-placement poll before interpreting the mutation outcome (478–546). Thus a sticky authoring interruption wins over both an otherwise successful placement and a normal mutation error.

reserve_request(normalized.clone()) still occurs before before = self.clone() (509–511). Restoring before therefore preserves a Pending request reservation inside that private candidate, rather than reconstructing the entire pre-call state. This is not a fresh-session publication problem if the outer controlled candidate is discarded on failure. It is a reason not to claim standalone imported BattleCore atomicity from this seam.

Under the fresh-only ownership invariant, the source of each rollback clone was already unique before that clone. Detaching self/staged leaves the rollback log unique; restoring it after failure does not introduce an unprepared shared write path. This reasoning fails if arbitrary external log owners or fixture callbacks can create additional aliases. Those callers are excluded from the opaque Session contract.

## Current preparation shape and its limits

The task initially described borrowed serialization as a copy-work estimator. Root instead factored the existing exhaustive per-event payload shape match into owned_payload_shape and reused it for both deferred retention estimation and COW pricing (event_log140–307). The current helper at427 onward:

1. Returns immediately outside active authoring control, preserving ordinary lazy Arc COW semantics.
2. Polls before preparation; if strong_count is one, returns without detach.
3. Admits/charges live entry count + one vector owner before scanning.
4. Scans each entry with a poll and O(1) native field-length/cardinality inspection. It adds nested collection element counts, one slot for a present skill_effect_source Arc handle, and owned string/source-command lengths.
5. Admits total CloneSlots and charges native slot/string copy work before Arc::make_mut, then polls after detach.

Vec cloning copies live entries, not the original spare capacity, so live len is the relevant native count. Current variable-sized event owners are unit-source definition strings, object/work-cell IDs, skill/step/presentation/VFX strings, area affected tiles, damage-breakdown elements and feedback tags. Current nested DamageBreakdown/feedback tags/StatModifier/UnitStats are scalar/Copy shapes; source identity and presentation string fields were cross-read. The core-only skill_effect_source is serde(skip), but its Arc handle is counted separately. Arc<SkillEffectExecutionSource> retains Arc<EffectSource> and Arc<SkillCastExecutionSource>; detaching entries increments the handle, without deep-cloning those pointees (skill_source.rs 8–29).

Native shape avoids an extra full encoded scan just to discover actual copy work. It remains a logical slot/byte price, not allocator/RSS accounting. The match forces consideration of a new enum variant; adding an owned field inside an existing variant matched with .. still requires explicit shape maintenance. Current-source sufficiency must not be projected onto future unimplemented gameplay fields.

The final charge is itself a cancellation poll before Arc::make_mut. The deep clone has no internal poll, so cancellation during it is caught afterward and the candidate is discarded. This is bounded admission, not a measured wall-clock interruption deadline. Actual allocation/latency validation belongs to root's evidence owner.

Outside an authoring scope there is a constant helper/TLS check at the newly connected call sites, but no eager log detach or changed log/sequence/RNG behavior. Exact performance equivalence is not inferred from source; ordinary lazy allocation timing remains in the existing mutator.

## Constructor, time-zero, reset and terminal ownership

CombatExecutor::resolve_live_combat_setup_plan_with_authored_policy creates a fresh BattleCore and calls start_battle_execution before returning the plan (events/combat219–229). BattleCore::new_from_database constructs BattleEventLog::new with a fresh Arc (core/mod307–373). Startup resets runtime state, replacing the log with another fresh new log (sim1607–1641), then records BattleStart and enqueues events (1796–1835). No constructor COW detach is needed for these fresh unique logs. Time-zero work after planned live-battle installation is real execution: world/combat commit_planned_live_battle_start stores the active session then advances by zero (633–694). Any internal boss/summon/spawn clone during startup/time-zero still requires the identified internal seam.

finish_battle records BattleEnd before the debug/test/fixture-only event_log clone into BattleResult (sim1276–1300). The world then makes another cfg-gated log clone into CombatBattleState (world/combat1350–1392). In the supported flow, finalize is marked once, step methods reject finished state, and world transitions by removing active_battle after view/VN/result preparation (sim1682–1695; world/combat1443–1568 and1394–1408). No subsequent supported runtime event write was found between these terminal log copies and active removal. The debug exporter borrows the result log; it does not introduce another Arc clone (combat_trace38,84).

This describes the fresh public application flow, not arbitrary debug/fixture calls that mutate a BattleCore after finalization. A future observer inserted before final recording must not create an unprepared log alias.

## Smallest additional obligations and explicit exclusions

**Opaque fork and trace ownership.** The future Session must not import an externally created CoreApplication/GameCore/BattleCore or a supplied/deserialized log as an already-admitted state. Its only fresh basis is validated generation plus checked initial/player/seed/scenario inputs under control. Every later fork prepares immediately. Optional trace must never retain BattleEventLog/BattleEventEntries clones: conditional COW work depends on Arc sharing, so observer-owned aliases could change fuel/admission across trace on/off. Independent observation DTOs and source Arc handles do not share the entries Vec. No optional trace bytes/work belong in mandatory control accounting.

**Public log mutation is not closed by this design.** BattleEventLog.entries and BattleEventEntries::DerefMut are public; From<Vec> and Deserialize create arbitrary Vec contents; from_entries appends over an already-allocated input Vec. These do not pass through whole-core clone seams and an imported unique Arc may skip preparation entirely. CoreApplication's log-replacement fixture is feature-gated and constructs a log from a caller count before replacing it (application795–821; world73–86); from_core is test-only. Preserve these existing fixture APIs but do not route fresh production authoring through them or claim generalized imported-state safety.

**DTO cloning is separate.** entries_after_seq builds a new Vec by cloning each selected entry (event_log492–497); BattleCore exposes that to ActiveBattleSession DTO construction (sim1674–1676; state1149–1228). This does not share the entries Arc and therefore is not a missing COW alias seam. It is a separate deep-copy/result-projection work and retention boundary. COW preparation alone cannot certify all readonly/receipt projection cost.

**Retention and other clones are separate.** Unique-owner appends, encoded-length/accounting scans, eviction shifts, ordinary history byte/count caps and all non-log fields copied by GameCore/BattleCore are outside this narrow COW guarantee. The new helper must not replace/disable retention or mutate scalar log accounting to make admission appear cheaper. Full source, queue/entity, clone-memory and broad R23 claims require their own actual evidence.

**Acceptance.** If prepare or later poll fails, outer controlled failure must defeat inner domain results and no mandatory receipt/session commit can publish the candidate. Optional trace overflow remains gap-only. Eager preparation may price a candidate whose later command is rejected or never writes, which is an explicit controlled candidate cost; do not reset/share that attempt work via rollback clones.

## Meaningful validation seams, not executions here

- A populated log clone under a tight work/CloneSlots budget must fail before detach; original values/source handles/sequence/RNG remain unchanged.
- Exact admitted capacity succeeds; the first real append after preparation causes no further detach of the old shared entries; source handle identity and sequence continuity survive.
- Nested application → tactical or boss → summon → spawn clones each prepare the actual write side. Late control failure returns interruption, never a cached spawn rejection/success.
- Constructor/startup/time-zero fresh logs remain usable with the actual internal clone paths controlled.
- Ordinary no-scope Clone still shares until real mutation; record_event_log behavior and terminal source handling are unchanged.
- A trace-on observer must not alter entries Arc sharing, resulting work/admission or mandatory continuation. Debug terminal copies and private DTO copies need their correct separate expectations.

Root said tests were being frozen/run. This helper did not execute or adopt any test result as its own and does not certify completion.

## Input identity, read method and drift

Read-only rg/sed discovery was followed by full-file hashing plus selected source-span capture in a single read per file. Selected spans were inspected from the in-memory snapshots; no source copy was written. Root was implementing concurrently: the field-name correction, native shape preparation and spawn interruption handling were observed as work-in-progress changes.

At 2026-09-10T08:29:57.037896+00:00 a 22-file rehash matched all captured inputs except spawn_kernel, whose full hash continued to drift. A subsequent snapshot read returned de76e4e9cd653b3badf4a4bd971d6903f7e6b754b1f9ec37c72c5c4fc6ea2b1f; its reviewed production spans184–218 and463–552 were byte-identical to earlier f024851fdaa87054077f6cf2cc645d16afa506a11898e3811958e2a450553110. Changes outside those spans are not reviewed/test-certified. These are individually captured files, not an atomic frozen repository revision.

| Source | Bytes | SHA256 | Reviewed snapshot spans |
| --- | ---: | --- | --- |
| core_enhanced/src/game/battle/event_log.rs | 56048 | ff18749cccde8dfa01fe8a34fbbc0ae98da09ade32c277da2efc43c9f8686908 | 42–140, 140–307, 383–510, 536–837 |
| core_enhanced/src/game/world/application.rs | 9615 | abbc0534548428ab35286059f03b9b83c91f5f282e596c0eeffe8432e3777a9d | 170–216 |
| core_enhanced/src/game/world/combat.rs | 129670 | 19c5a48bdfe3e1a7acae673e55db5fbd51dcce933027289c59b63550f177641b | 465–529, 633–694, 1350–1408, 1443–1568, 1820–1870 |
| core_enhanced/src/game/world/state.rs | 70625 | 129a4063f7e448b5795fd370a22a997f66eb09cd3029fb87d41fee892cf34bb0 | 137–160, 507–546, 1149–1228 |
| core_enhanced/src/game/battle/core/spawn_kernel.rs | 37114 | de76e4e9cd653b3badf4a4bd971d6903f7e6b754b1f9ec37c72c5c4fc6ea2b1f | 184–218, 463–552 |
| core_enhanced/src/game/battle/core/summon_runtime.rs | 65702 | f6ed6b5869490f1bb2ec29aaaff93c5bda2639f6d3ba5f2380bd43e57bbbc5a6 | 266–291, 350–407 |
| core_enhanced/src/game/battle/core/boss_runtime.rs | 138620 | cf67be542a70a8ae71e5507264fe90dd96b60e9ff1a755103e0bfec17490e757 | 286–316, 1758–1814 |
| core_enhanced/src/game/battle/core/sim.rs | 130316 | e624854ac13e8172b104c54e4626a761c4d017d67b8ffdb4adf3f15e50a31207 | 1276–1300, 1607–1698, 1796–1860, 1938–1960 |
| core_enhanced/src/game/battle/core/mod.rs | 349296 | 026b85b1b7d47cfc3f08d3e88467dfe7de9558926aa1783e76b3cb56bf5bf67e | 79–130, 299–373 |
| core_enhanced/src/game/events/combat.rs | 84303 | d88719417886eea018ae08a66c750884f163051f160d0b9b21ece3c1eddffc66 | 185–230 |
| core_enhanced/src/game/world.rs | 34199 | fc72d1029c33c560f918dc2487071c5720ad231d6983b9aa66dc15e4d5ee75ed | 60–98 |
| core_enhanced/src/application.rs | 44770 | 636b3c9cc15ae2adfa39a15392e0dee22ee7c6683885087f9cc77cefda66081a | 780–826 |
| core_enhanced/src/authoring/control.rs | 16946 | 1c465fc9e00ef53d1b93cab11f4cf97fdffa341b02715d3227af8d0f8803edcc | 29–59, 87–155, 159–289, 320–443 |
| core_enhanced/src/game/battle/mod.rs | 429 | c740b990aef0013af17d7261f7d06ebf7950a1588338457904902c3a8e09b76e | 1–18 |
| core_enhanced/src/game/battle/recording.rs | 3082 | e3c1afedaf9b5ec61c76d415e2123dbea8a9c6d9213da5393dddfaf0e6beaf2d | 1–92 |
| core_enhanced/src/game/world/combat_trace.rs | 3153 | ab8879db23ea68c343465da07765a5dc5e7f69da0867a2c4b7c057e72abab092 | 1–93 |
| core_enhanced/src/game/battle/skill_source.rs | 870 | 4fff389728a0dcb34fa50b56108e8790a750205fa0d391c21e65fd4553ca50d1 | 1–29 |
| core_enhanced/src/game/battle/damage.rs | 45184 | 5bfa34c21ad87ad672a4c249b38641f620358a7f34d57f4aa59df47246b5e46a | 225–276 |
| core_enhanced/src/game/stats.rs | 18446 | 9c3d49b44c773288da84e35bc0411292e423583a4b481a1e044622c138d10269 | 184–224 |
| core_enhanced/src/game/ability.rs | 38098 | c7cec94493e6de6d2ebb005941f3f80a276396e3803afb07f5d5a884d65ab195 | 615–631 |
| core_enhanced/src/game/battle/types.rs | 36600 | e388e9eb7d882dbcbd81eb9291a3cf77f1b8841fc060201da7735c750666075f | 186–218 |
| core_enhanced/src/game/battle/cooldown.rs | 711 | fa401d4db5a2c6572a8ef404979303fe4c5b957e1aa3160f62befef12a3d55f4 | 1–14 |

The JSON companion retains rehash/drift details. The report provides bounded design evidence only, with no full R03 or imported-core completion verdict.
