# Test Rebaseline Catalog

## Execution Evidence

P01~P10 executed 970 selected physical test instances in total, all passing with zero failures and zero selected
ignores/skips. This is not a repository-wide unique count: phases intentionally overlap consumer tests, and P09
itself reports 146 physical/145 unique cases. Final does not rerun a broad suite because all current facts needed for
reconciliation are closed by phase source inspection and focused tests; another green aggregate would not prove
identity, work type or gate correctness.

| Phase | Selected physical executions | Result |
| --- | ---: | --- |
| P01 | 28 | 28 pass |
| P02 | 12 | 12 pass |
| P03 | 159 | 159 pass |
| P04 | 58 | 58 pass |
| P05 | 75 | 75 pass |
| P06 | 137 | 137 pass |
| P07 | 101 | 101 pass |
| P08 | 123 | 123 pass |
| P09 | 146 | 146 pass; 145 unique |
| P10 | 131 | 131 unique pass |

## Exact Candidate Test Dispositions

`retain + strengthen` means existing tests protect a distinct current invariant but do not close the candidate's
named gap. `transfer then remove` applies only when the surviving production-path test receives the unique assertion.

| Final ID | Current test disposition | Required safety net before implementation/closure |
| --- | --- | --- |
| CRA-F-C01 | `retain + strengthen` | one-edge-at-a-time current-V3 seed/progression/roster/equipment/fragment/economy negatives; unchanged Core/export for Continue and explicit load |
| CRA-F-C02 | `retain + strengthen` | full-state success and late-failure equality for recruit, emergency, open-shop and common completion; preserve monotonic compendium exclusion |
| CRA-P01-C01 | `retain` | exact physical/source inventory, parse-all and empty-source provenance remain closure guards |
| CRA-P01-C02 | `retain` | manifested-only and Boss setup publication negatives plus official live bundle remain closure guards |
| CRA-P01-C03 | `retain + strengthen` | exact parsed/validated/compiled→published field/Arc/registry parity and failure ordering |
| CRA-P02-C01 | `retain + strengthen` | real-file higher-current→lower-incoming loads/peak/both while preserving incoming Core payload |
| CRA-P02-C03 | `retain + strengthen` | golden exact V3 field set/exclusions and bidirectional parity after F-C01 |
| CRA-P02-C04 | `retain + strengthen` | transition table for normal, BossOmen and admin paths plus post-restore invariant |
| CRA-P03-C02 | `retain + strengthen` | duplicate-row insufficient aggregate no-mutation/diff and exact aggregate success |
| CRA-P03-C03 | `retain + strengthen` | both-ledger transition table for move/replace/unequip/starter/death/dismantle |
| CRA-P03-C04 | `retain + consolidate after parity` | exact five-ledger state, diff order, UUID/seed and late failure across every caller |
| CRA-P04-C01 | `retain + strengthen` | typed and official-bundle negatives for each runtime-rejected repeat/source pair with exact provenance |
| CRA-P04-C03 | `retain + strengthen` | persisted-active equality for every Event/Shop/HQ entry/update and explicit no-run behavior |
| CRA-P04-C04 | `retain + strengthen` | Monolith queue-front/Tail placement, UUID order, active non-interruption and accept/decline behavior |
| CRA-P05-C01 | `retain + strengthen` | exact preview→cancel→SavePoint→durable Continue→start sequence with mutated serialized cache |
| CRA-P05-C02 | `retain` | every manifested case/five seeds plus setup/Boss feasibility negatives remain closure guards |
| CRA-P05-C03 | `retain + strengthen` | ordinary/Boss/Event/Monolith resolver parity and public caller inventory after C01 |
| CRA-P05-C04 | `retain` | setup-plan scenario/snapshot/first-checkpoint/reward parity and ordinary no-binding remain closure guards |
| CRA-P05-C05 | `transfer then remove redundant shapes` | retain category, SplitRoom and mission/reward matrices; prove wrapper callers absent before cleanup |
| CRA-P06-C01 | `retain` | cap entry-point parity, exact-cap/late-queue/single-BattleEnd and world terminal guards |
| CRA-P06-C02 | `retain` | every live variant equality/antisymmetry/permutation and Boss-wake order guards |
| CRA-P06-C03 | `retain` | future cursor/revision/trigger/reconcile/terminal rollback and elapsed-bucket parity guards |
| CRA-P06-C04 | `transfer then remove` | run useful validator predicates against real produced logs; preserve malformed-log assertions there |
| CRA-P06-C05 | `retain + strengthen` | extreme positive/negative weights, overflow and comparator/DTO equality in debug/release semantics |
| CRA-P06-C06 | `retain compile/behavior guards` | constructor inventory plus delayed-spawn/forced-end/scenario validation before field removal |
| CRA-P06-C07 | `retain standard/debug guards` | reference inventory, standard serde round trip, compression and retained debug-writer path |
| CRA-P07-C01 | `retain + strengthen` | Any-profile worker Ground↔Platform relocation/swap final aggregate and stale rest-event cases |
| CRA-P07-C02 | `retain + strengthen` | future cursor and injected reconciliation failure for every tactical command with exact whole-world equality |
| CRA-P07-C03 | `transfer then remove Direct-only tests` | inventory 17 Direct-dependent tests; port unique collision/clamp/static-obstacle/determinism assertions to Rapier/default |
| CRA-P07-C04 | `retain + targeted removal` | compile/field-use inventory and world position/range/event publication parity after C03 |
| CRA-P08-C01 | `retain + strengthen` | live parasite-seed two-ally exact occurrence/event/final-stat case plus instant/projectile/death occurrence matrix |
| CRA-P08-C02 | `retain + strengthen after policy` | nonlethal/lethal MaxHealth modifier, exact HpChanged/death/provenance/kill-credit and no active zero-HP state |
| CRA-P08-C03 | `retain + strengthen` | 0/1/type maxima/extreme signed-percent table and capped draft scale |
| CRA-P08-C04 | `retain + strengthen` | missing buff/skill, invalid Permanent/occurrence rows and exact official-bundle provenance; gated OnHit counterpart cases |
| CRA-P08-C05 | `retain + characterize` | identical instant/projectile/general damage under stat/source/target drift, lineage and command ordering |
| CRA-P08-C06 | `transfer then remove` | move the sole sorted/deduped ally occurrence invariant into production death tests; inventory public consumers |
| CRA-P09-C01 | `retain + strengthen after compatibility` | RON/programmatic zero speed/limits and full basic/weapon ignored-field matrix with exact errors |
| CRA-P09-C02 | `retain + strengthen` | projectile/cast absence, exactly-one terminal completion and same-time combined stop policies |
| CRA-P09-C03 | `retain + strengthen after policy` | anchor-excluding EnemySingle footprint across selection/cast/execution and frozen multi-step rule |
| CRA-P09-C04 | `retain + strengthen after projection policy` | runtime/preview footprint parity at edges/facings and chosen multi-step representation |
| CRA-P09-C05 | `retain + common parity table` | basic/skill pure motion parity for timing, moving targets, collision/expiry while preserving distinct payload/lifecycle |
| CRA-P09-C06 | `retain + cross-product table` | all pattern/policy/facing/clipping membership/materialization parity across callers |
| CRA-P09-C07 | `retain until inventory, then remove/migrate` | workspace/external Rust/server/serialized/fixture inventory and live RON compatibility decision |
| CRA-P09-C08 | `correct now; retain repaired negatives` | prove each repaired fixture parses, then assert exact object/path/cause rather than catch-any-panic/bare error |
| CRA-P10-C01 | `retain + strengthen` | exhaustive ActionKind owner/scope/authority descriptor and projected-versus-executable state parity |
| CRA-P10-C02 | `retain + strengthen` | every BehaviorResult type/payload/transport class, nested completion and exact RunComplete; API inventory |
| CRA-P10-C03 | `retain + exact parity` | old/new setup/update/resync JSON plus actor/reconnect order and consumer-aware fixture comparison |
| CRA-P10-C04 | `retain + strengthen` | game state × active-run × compendium overlay table, exact action order, no duplicates and fatal replacement |
| CRA-P10-C05 | `retain + strengthen` | exhaustive parameterized GameError/disabled-reason code equality across both Core snapshot paths and server |
| CRA-P10-C06 | `retain tool behavior; remove zero-test noise` | before/after metadata/dependency graph, Core/server all-target/feature checks, every retained fixture to `/tmp`, standalone inventory |
| CRA-P10-C07 | `remove` | metadata/test list before and after plus Core runner; no behavioral replacement for a disconnected empty file |

## Cross-Family Retention Rules

- Serialization tests, Core result mapping and actor-order tests are distinct layers; none replaces the other.
- Unit/compiler, official live-RON and world integration tests remain distinct where each observes a separate failure
  lifetime. Broad world success does not replace publication negatives.
- Private Boss kernel tests remain implementation-coupling sensitive but retain unique stale-wake, target identity,
  cancellation and movement-leaf invariants. Public tests must assert absence/ordinary compatibility rather than
  private struct layout.
- Cleanup is not authorized by zero runtime hits alone. Public/external inventory and assertion transfer precede every
  removal except the disconnected P10-C07 file.
