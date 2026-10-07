# Exact Linear Implementation Order

Order status: `recommended_current`
Evidence date: 2026-08-09 (`Asia/Seoul`)
Scope: all 44 `still_valid` final candidate identities, exactly once
Execution model: one candidate-scoped `implement_and_verify` operation at a time

This is the strict sequential execution order derived from the P01~P10 candidate, test and boundary reaudits, the
Final catalog, approved CRA-POL-001~006 contracts, source-authority boundaries and consumer gates. It refines the
older Wave grouping: Wave numbers classify work/risk, but are not a complete temporal order.

No production implementation is performed by this document. Before changing a candidate, its implementation Goal
must freeze the listed entry evidence, add the named safety net, preserve unrelated dirty work and name every
applicable policy ID.

## Ordering Rules

1. Repair false-positive safety nets before relying on them.
2. Correct fail-open, partial-mutation and arithmetic behavior before consolidating the affected authority.
3. Establish one lower-level query/transition authority before changing its higher-level consumer.
4. Freeze current public behavior with goldens before removing duplicate projection/mapping code.
5. Transfer unique assertions before deleting a false or legacy authority.
6. Put public/serialized/package cleanup last. A negative workspace search is never deletion authority under
   CRA-POL-006.
7. Start item N+1 only after N receives a `Complete / High` Phase audit. If an external-gated cleanup item cannot
   complete, stop automatic progression and obtain an explicit user disposition before rebasing the remaining
   independent tail. A newly discovered dependency pauses the sequence and updates this document first.

## Required Dependency Edges

These edges are supported directly by the final/phase evidence and must not be inverted.

| Predecessor | Successor | Dependency reason |
| --- | --- | --- |
| P09-C08 | P09-C01 | exact-cause negative fixtures must be trustworthy before expanding publication rejection |
| F-C01 | P02-C03, P03-C03 | invalid imported graphs must fail before checkpoint/loadout mapping consolidation |
| P03-C04 | F-C02 | five-ledger caller parity must be frozen before replacing the HQ transaction wrapper |
| P05-C01 | P05-C03 | serialized preview cache must stop acting as setup authority before resolver consolidation |
| P08-C01 | P08-C04, P08-C05, P08-C06 | one occurrence contract precedes publication matrix, committed damage and false-death cleanup |
| P08-C02 | P08-C05 | HP/death/provenance behavior must be final before committed-damage consolidation |
| P09-C01 + P09-C02 + P08-C05 | P09-C05 | publication, terminal delivery and committed-hit boundaries precede a shared motion kernel |
| P09-C06 | P09-C03, P09-C04 | one typed footprint query precedes usefulness and preview consumers |
| P10-C02 + P09-C04 | P10-C03 | semantic transport classification and final preview content precede server DTO de-duplication |
| P07-C03 | P07-C04 | the retained Rapier/default contract determines which movement fields are truly ghost state |

## Conservative Rework-Avoidance Edges

These are stronger than the minimum compile dependency. They make the single-file/single-agent sequence safer by
stabilizing shared behavior before editing overlapping seams.

| Earlier work | Later work | Reason for the stricter order |
| --- | --- | --- |
| P08-C03 | P08-C02 | numeric range/cap behavior is frozen before HP-aware runtime stat transitions |
| P08-C04 | P08-C05 | the approved occurrence/effect matrix is fixed before committed damage consumes it |
| P02-C03 | P02-C04 | checkpoint mapping is stable before encapsulating map mutation and post-restore transitions |
| P04-C03 | F-C02 | HQ Shop persisted/active commit semantics are fixed before common HQ transaction adoption |
| P09-C03 | P09-C04 | actual delivery-set semantics are fixed before their union preview is published |
| P10-C01 | P10-C04 | command owner/scope descriptors are fixed before the host capability overlay is consolidated |
| P05-C03 | P05-C05 | real resolver/mission consumers are known before shape-only helper/test cleanup |
| P08-C05 | P08-C06 | production damage/death assertions are final before deleting the disconnected DeathHandler model |
| P09-C03/C04/C05/C06 | P09-C07 | production targeting/delivery owners are final before deleting public/serialized ghost surfaces |
| P10-C01~C05 + P06-C07 + P09-C07 | P10-C06 | broad package/target/dependency cleanup runs only after retained code/tool surfaces stabilize |

## Exact 1~44 Sequence

### Stage A — Safety Net And Correctness Foundations

| No. | Candidate | Target | Entry dependency / policy | Test-first proof and exit condition |
| ---: | --- | --- | --- | --- |
| 1 | CRA-P09-C08 | exact-cause negative target tests | none | repaired fixture parses; restored single defect fails with exact object/path/cause, never catch-any-panic |
| 2 | CRA-F-C01 | complete checkpoint invariant graph | CRA-POL-006 | one-edge V3 seed/progression/asset/economy negatives reject before mutation; valid round trip unchanged |
| 3 | CRA-P02-C01 | host monotonic metadata on every publish | none | real-file higher-current→lower-incoming loads/peak/both preserve incoming Core payload and installed maxima |
| 4 | CRA-P03-C02 | atomic enhancement material aggregation | none | duplicate insufficient aggregate leaves state/diff unchanged; exact aggregate succeeds once |
| 5 | CRA-P04-C01 | VN repeat/source publication matrix | CRA-POL-006 | every runtime-rejected pair fails typed/official publication with exact provenance; live bundle remains valid |
| 6 | CRA-P05-C01 | keep serialized preview cache out of setup authority | CRA-POL-006 | preview→cancel→SavePoint→durable Continue→start ignores mutated cache and recomputes authoritative setup |
| 7 | CRA-P06-C05 | one saturating MVP score | none | extreme signed weights/overflow produce identical comparator and DTO winner under debug/release semantics |
| 8 | CRA-P07-C01 | tactical effects from full final state | none | Any worker Ground↔Platform relocate/swap derives one final activity/stance/focus/block/rest aggregate |
| 9 | CRA-P07-C02 | atomic tactical world mutations | P07-C01 behavior stabilized | each tactical command preserves complete world state on future-cursor/reconciliation failure |
| 10 | CRA-P08-C03 | saturating stat percentage arithmetic | none | 0/1/max/extreme-percent table and 207% draft scale obey declared caps without narrowing/overflow |
| 11 | CRA-P08-C01 | one typed trigger occurrence dispatcher | CRA-POL-002 | live parasite plus all attack/hit/death occurrence classes prove effects-before-activations, retaliation handoff and dead-unit rules |
| 12 | CRA-P08-C02 | HP/death-aware runtime stat mutation | P08-C03; CRA-POL-001 | nonlethal/lethal MaxHealth cases prove exact events, provenance, kill credit and no active zero-HP state |
| 13 | CRA-P08-C04 | effect/activation publication validation | P08-C01; CRA-POL-002, CRA-POL-006 | exact missing reference/matrix negatives and positive live bundle match approved retaliation semantics |
| 14 | CRA-P09-C02 | max-kills terminal delivery exactly once | none | projectile and cast both disappear with exactly one terminal completion under combined stop policies |
| 15 | CRA-P09-C01 | projectile/basic compatibility publication | P09-C08; CRA-POL-003, CRA-POL-006 | RON/programmatic zero and ignored-field matrix fail atomically with exact errors; live data loads |

### Stage B — Spatial Authority Required By Remaining Corrections

| No. | Candidate | Target | Entry dependency / policy | Test-first proof and exit condition |
| ---: | --- | --- | --- | --- |
| 16 | CRA-P09-C06 | typed tile reach query/projection | P09 publication/lifecycle corrections characterized | cross-product pattern/policy/facing/clipping table proves membership/materialization parity across callers |
| 17 | CRA-P09-C03 | usefulness over actual delivery targets | P09-C06; CRA-POL-004 | anchor-excluding actual footprints and useful-later/useless-first steps match selection/cast/execution rules |
| 18 | CRA-P09-C04 | effect preview from footprints | P09-C06, P09-C03; CRA-POL-005, CRA-POL-006 | edge/facing runtime parity and deterministic union cells preserve the existing DTO/wire shape |

Stage B intentionally crosses the old Wave boundary: P09-C06 was classified as a Wave 3 refactor, but P09-C03/C04
cannot be safely corrected before its typed query exists.

### Stage C — Behavior-Preserving Authority Consolidation

| No. | Candidate | Target | Entry dependency / policy | Test-first proof and exit condition |
| ---: | --- | --- | --- | --- |
| 19 | CRA-P01-C03 | private parsed→published mapping | publication semantics from P04-C01/P08-C04/P09-C01 stabilized; CRA-POL-006 | exact field/Arc/registry/failure-order parity; compile Boss definitions once without public API narrowing |
| 20 | CRA-P02-C03 | RunState↔V3 checkpoint conversion | F-C01; CRA-POL-006 | golden exact V3 field set/exclusions and bidirectional parity, including active-battle/Boss exclusion |
| 21 | CRA-P02-C04 | MapProgression mutation/availability | F-C01, P02-C03; CRA-POL-006 | normal/BossOmen/admin transition table plus post-restore invariant preserves sorting and graph rules |
| 22 | CRA-P03-C03 | bidirectional equipment/employee loadout | F-C01; CRA-POL-006 | one transition matrix preserves both ledgers across move/replace/unequip/starter/death/dismantle |
| 23 | CRA-P03-C04 | five-ledger grant transaction state | none | every reward/shop/event/combat/admin caller matches exact state, diff order, UUID/seed and late failure |
| 24 | CRA-P04-C03 | Event/Shop persisted-active commits | none | every Event/Shop/HQ entry/update preserves paired state and explicit no-run semantics |
| 25 | CRA-F-C02 | common HQ transaction | P03-C04, P04-C03 | full non-battle state equality on success/late failure for recruit/emergency/open-shop/common completion |
| 26 | CRA-P04-C04 | VN assembly and explicit queue placement | HQ/session semantics stabilized | trigger Tail and Monolith queue-front share assembly while preserving UUID/order/non-interruption/blocking |
| 27 | CRA-P05-C03 | preview resolution/ghost Floor authority | P05-C01; CRA-POL-006 | ordinary/Boss/Event/Monolith resolver parity and caller inventory remove duplicate private mapping only |
| 28 | CRA-P08-C05 | committed damage authority | P08-C01, P08-C02, P08-C04; CRA-POL-001, CRA-POL-002 | instant/projectile/general damage parity covers drift, crit, retaliation, kill, lineage and command order |
| 29 | CRA-P09-C05 | continuous projectile motion kernel | P09-C01, P09-C02, P08-C05; CRA-POL-003, CRA-POL-006 | basic/skill pure motion parity preserves distinct IDs, policies, payloads, lifecycle and Boss released-work seam |
| 30 | CRA-P10-C01 | typed command route/owner metadata | none | exhaustive ActionKind owner/scope/authority and projected-versus-executable parity |
| 31 | CRA-P10-C04 | typed host snapshot overlay | P10-C01; CRA-POL-006 | game-state × active-run × compendium table proves order, no duplicates and fatal replacement |
| 32 | CRA-P10-C05 | stable GameError code authority | none; CRA-POL-006 | exhaustive Core/server code equality retains context-specific messages |
| 33 | CRA-P10-C02 | one command-result projection authority | CRA-POL-006; public helper inventory | every BehaviorResult has exact type/payload/transport/nesting and direct-map RunComplete parity |
| 34 | CRA-P10-C03 | reuse Core battle DTOs in server transport | P10-C02, P09-C04; CRA-POL-005, CRA-POL-006 | old/new setup/update/resync JSON and actor/reconnect order are byte/structure equivalent; no Boss wire added |

### Stage D — Assertion Transfer And Cleanup

| No. | Candidate | Target | Entry dependency / policy | Test-first proof and exit condition |
| ---: | --- | --- | --- | --- |
| 35 | CRA-P10-C07 | empty disconnected test module | none | metadata/test listing unchanged except the absent empty file; Core runner remains green |
| 36 | CRA-P05-C05 | no-op mission branches/shape-only tests | P05-C03; CRA-POL-006 | category/SplitRoom/mission/reward assertions survive; wrapper callers are proven absent before removal |
| 37 | CRA-P06-C04 | disconnected event-log validator | CRA-POL-006; public consumer inventory | useful malformed-log predicates execute against real produced logs before false authority/tests are removed |
| 38 | CRA-P08-C06 | disconnected DeathHandler | P08-C01, P08-C02, P08-C05; CRA-POL-006; public consumer inventory | sorted/deduped ally occurrence lives in production death tests before module deletion/depublication |
| 39 | CRA-P07-C03 | Rapier-only private production backend | CRA-POL-006; public/external inventory | all 17 Direct-dependent assertions are inventoried and unique collision/clamp/obstacle/determinism cases pass on retained backend |
| 40 | CRA-P07-C04 | ghost movement tick state | P07-C03; CRA-POL-006; public/external inventory | field-use inventory plus world position/range/event parity proves each removed member has no surviving contract |
| 41 | CRA-P06-C06 | ghost ScenarioEvent.once | CRA-POL-006; public/external inventory | constructor inventory and delayed-spawn/forced-end/scenario validation remain exact before field removal |
| 42 | CRA-P06-C07 | battle-log I/O at debug boundary | CRA-POL-006; public/external inventory | standard serde/compression/debug-writer behavior remains while unused public file I/O is retained, deprecated or removed per evidence |
| 43 | CRA-P09-C07 | ghost range/public delivery surfaces | P09-C03/C04/C05/C06; CRA-POL-006; serialized/external inventory | target-set and live-RON goldens plus consumer/fixture inventory authorize only proven private cleanup or a staged migration |
| 44 | CRA-P10-C06 | Core package targets/dependencies | P10-C01~C05, P06-C07, P09-C07; CRA-POL-006; standalone inventory | before/after metadata/dependency/all-target/feature checks and retained fixtures-to-`/tmp` prove no build/tool/API loss |

P10-C07 moves from the old Wave 0 grouping to sequence 35 because it enables no later correction. Its old placement
described low risk, not a dependency. P10-C06 remains last because it has the broadest ability to alter which code,
tests and tools Cargo builds.

## External-Gate Handling For Items 37~44

The cleanup stage begins with read-only consumer and assertion inventories. If a public/serialized/standalone or
deployed consumer cannot be disproved or migrated, CRA-POL-006 requires retaining the surface. Record the Phase as
`externally_blocked` with the exact missing evidence; do not add a permissive fallback and do not infer absence from
workspace search. A blocked tail item does not invalidate already completed correction/refactor stages, but the
Master does not automatically activate the next Phase until the user supplies evidence or explicitly revises the
blocked/remaining disposition.

## Per-Candidate Linear Exit Gate

Every item must satisfy all of the following before advancing:

1. candidate-specific red/characterization test exists and proves the named defect or exact old behavior;
2. the smallest authority-scoped implementation is complete with no unrelated API/schema change;
3. focused tests, directly affected integration/live-data tests and required parity goldens pass;
4. applicable CRA-POL IDs and external-consumer evidence are recorded;
5. candidate disposition/test catalog is updated without changing other candidate identities; and
6. worktree review confirms unrelated user changes remain untouched.

Broad workspace validation belongs at risk boundaries—after Stages A, B, C and D—not as a substitute for each
candidate's focused proof.
