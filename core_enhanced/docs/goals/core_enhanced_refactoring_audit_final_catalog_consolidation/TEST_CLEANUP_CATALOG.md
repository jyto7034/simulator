# Core Enhanced Refactoring Audit — Final Test And Cleanup Catalog

Audit Date: 2026-08-07
Status: complete
Scope: CRA-14 across Phase 1~10; Phase runner counts are supporting evidence only

## Phase 1~3 CRA-14 Backfill

### Phase 1 — Data Loading And Publication

| Family | Meaning/reachability | Final disposition |
| --- | --- | --- |
| `game::data::*` parser/domain/cross-catalog tests | strict schemas, provenance, ID/reference and accepted bundle predicates; production publication primitives | `retain`; `strengthen` exact source-set and manifested difficulty matrix |
| `tests/ron_loading.rs` | official embedded loader and assembled live database; distinct integration layer | `retain`; do not merge with unit validators |
| `live_skill_catalog_audit` | actual published skill catalog completeness/reference behavior | `retain`; strengthen through P08/P09 publication matrices |
| `combat_preview::*` publication tests | generated preview determinism/topology and baseline feasibility | `retain`; strengthen P01-C02/P05-C02 stage/setup negatives |
| builder/builtin/cache and `should_panic` fixture tests | separate programmatic/test construction lifecycle, not server startup | `retain`; no production-authority claim |

No exact Phase 1 test deletion is proven. The tautological manual-count assertion is replaced, not simply removed,
by P01-C01's exact set reconciliation.

### Phase 2 — Run, Map And Persistence

| Family | Meaning/reachability | Final disposition |
| --- | --- | --- |
| map generation/progression modules | deterministic templates/IDs, topology, availability, transition boundaries | `retain`; strengthen semantic progression invariant for F-C01/P02-C04 |
| `world::map_flow` run/Floor group | production selection/confirm/gate/Floor transactions and no-mutation failures | `retain` |
| node/facility/checkpoint group | SavePoint capture/restore/load count/anchor/VN/session lifetimes | `retain`; strengthen full V3 field and invariant graph |
| snapshots/start group | pre-run→run construction and current flow projection | `retain`; distinct from persistence byte contract |
| UUID manager | deterministic namespace/counter behavior | `retain`; add checkpoint seed-equality negative |
| server active-run store/actor | real-file version/checksum/backup/tombstone/replacement/rollback | `retain`; add lower-current high-water and policy-gated actor conflict tests |
| transaction-filter overlap | same named checkpoint transaction selected twice by filters | retain the test once; duplicate runner execution is not extra coverage |

No exact Phase 2 test deletion is proven. Main-binary zero-test selection is excluded from coverage and addressed by
P10 package cleanup.

### Phase 3 — Roster, Inventory And Economy

| Family | Meaning/reachability | Final disposition |
| --- | --- | --- |
| inventory/item-slot | identity, quantity, reserve and slot transitions | `retain`; characterization input for P03-C03 |
| employee/recruitment/snapshot | roster identity, condition/profile, authored/runtime loadout and starter flow | `retain`; strengthen checkpoint graph and bidirectional transition cases |
| fragment/research | progress/pending/loadout/dust and authoritative delivery | `retain`; strengthen F-C01 invalid graph cases |
| equipment commands | equip/unequip/maintenance/enhancement state and exact failures | `retain`; add duplicate-material aggregate tests for P03-C02 |
| reward/shop/node/facility | staged multi-ledger grants and active session cleanup/rollback | `retain`; strengthen P03-C04 and F-C02 caller/full-state parity |
| live RON/item-skill | published asset/reference integration | `retain`; publication authority remains Phase 1/P08/P09 |
| selected combat reward/research/rollback | battle-to-persistent handoff atomicity | `retain`; distinct from combat mechanics |

No Phase 3 happy path is removed merely because a broader world test exists; the layers assert different authority
handoffs. The earlier uncaptured rerun batch is not pass evidence.

## Consolidated Phase 4~10 Cleanup Decisions

| Final test-cleanup ID | Source | Removal/merge | Surviving or replacement safety net | Readiness |
| --- | --- | --- | --- | --- |
| CRA-TC-01 | P04 test audit | merge/remove four-case Event natural-extreme example | exhaustive face/DC/modifier matrix retains labeled natural 1/20/threshold diagnostics | ready with assertion-label check |
| CRA-TC-02 | P05-C05 | remove two no-op mission helper tests | category/SplitRoom/live pair matrix | after helper/caller inventory |
| CRA-TC-03 | P05-C05 | merge strict-subset default defense scenario test | stronger black-box defense test retains objective/unit/route/position assertions | ready after exact assertion diff |
| CRA-TC-04 | P05-C05 | merge two currency-only Defense reward failures | one case retains empty acceptance and exact failure text | ready after assertion transfer |
| CRA-TC-05 | P06-C04 | remove six disconnected `EventLogValidator` self-tests with harness | transfer useful seq/reference/buff/death/cadence checks to real produced-log tests | blocked on transfer/external API inventory |
| CRA-TC-06 | P06-C07 | merge pretty/debug file-output helpers | one explicit debug artifact helper plus serde/compression/trace-failure tests | external API inventory first |
| CRA-TC-07 | P07-C03 | remove Direct-only/parity/selector tests after behavior transfer | default Rapier Core tests cover every unique movement invariant; authored route runs production backend | external API inventory and transfer first |
| CRA-TC-08 | P07-C03 | remove two test-only Rapier board-wall tests/subsystem | production pure clamp/manual-bound tests remain | after C03 transfer |
| CRA-TC-09 | P08-C06 | remove disconnected DeathHandler test/module | production death occurrence test receives sorted/dedupe assertion | after P08-C01 and external inventory |
| CRA-TC-10 | P09-C08 | correct or merge false-positive negative RON fixtures | each otherwise-valid case proves exact repaired-success then one typed failure path | test correction ready |
| CRA-TC-11 | P10-C02 | remove helper-only result test with public helpers | all-variant result golden and external API inventory | blocked on C02 inventory |
| CRA-TC-12 | P10-C07 | delete empty disconnected `tests/test_actor/mod.rs` | Cargo metadata/test list unchanged | implementation ready |
| CRA-TC-13 | P10-C06 | stop compiling empty main/generators as zero-test harnesses | remove empty main; explicit retained fixture bins use `test = false` | package/external inventory first |

P09's repeated filter invocation is runner redundancy, not a test-body deletion. Zero-test guessed filters and
filtered-out counts are excluded from all totals and disposition evidence.

## Strengthening Index

| Boundary | Required safety net before implementation |
| --- | --- |
| publication | exact 23-file set; authored difficulty/seed matrix; full setup feasibility; VN pair and equipment/projectile compatibility negatives |
| checkpoint/economy | full invariant graph, exact V3 field parity, lower-current host high-water, duplicate material aggregate |
| sessions | HQ late failure, Event/Shop paired equality, Monolith VN queue/world hook |
| battle lifecycle | max-cap step parity, total event comparator, advance/tactical post-preflight failure no-mutation |
| tactical/effects | final-state Focus/block/Rest, max-health HP/death, occurrence matrix, extreme arithmetic |
| targeting | terminal max-kills cleanup, actual affected-set usefulness, runtime/preview footprint parity, motion/query cross-products |
| transport | all-result JSON including `RunComplete`, all-error code table, command owner table, host overlay state matrix, exact battle DTO parity |

## Retained-Layer Principle

Unit math, typed validation, official live publication, world command/state, server persistence/actor order and
external fixture provenance are separate safety-net layers. Similar inputs do not make them duplicate. Broad
scenario tests are retained when they observe a complete published contract; private-shape-only tests are removed
only with the implementation they falsely elevate.

## CRA-14 Verdict

All Phase 1~10 test/fixture surfaces now have family dispositions. Phase 1~3 backfill identifies no unsupported
deletion and records the exact candidate gaps. Phase 4~10 cleanup is consolidated into thirteen actionable entries
with replacement safety nets and readiness gates. Confidence is High.
