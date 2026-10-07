# Determinism Input and Ordering Inventory

- 기준선: `B-001`
- 상태: 최종

| input/order surface | authority/mechanism | evidence | remaining risk |
| --- | --- | --- | --- |
| battle time | `BattleExecutionState.last_event_time_ms`, staged external elapsed/bucket advance | E-008/E-017 | giant public delta has no generic work budget (RUNTIME-AUDIT-009) |
| event queue | `BinaryHeap<BattleEvent>`; time → priority → variant → identity → Debug payload | E-008 | non-finite Eq/Ord RUNTIME-AUDIT-004; comparator allocation |
| same-time events | bucket drain/sort; newly queued same-time events drained until empty | E-008 | zero-time self-scheduling bound/performance follow-up |
| battle IDs | deterministic `uuid_v4_from_seed`, unit/item material + namespace | E-008 | counter/source identity coverage follow-up |
| run IDs | `UuidManager` seed + per-namespace `BTreeMap<u64,u64>` counter | E-008 | checkpoint monotonic/local stream semantics follow-up |
| collection iteration | BattleCore uses many `HashMap`; movement/target/output paths materialize stable ID order; RNG candidates inspected sort or preserve authored Vec order | E-008/E-017 | exhaustive proof for every future reader is not encoded |
| physics | `RapierMovementWorld` private backend; reversed unit insertion probe compares outputs/positions/exact log | E-008/E-017 | cross-process/platform portability unproven |
| floats | internal `WorldVec2(f32)`; event log quantizes to milli | E-008 | finite validation and platform stability |
| RNG | all production hits use seeded `StdRng`; candidates sort identity or preserve validated authored Vec order | E-003/E-008/E-017 | future call-site discipline is convention/test based |
| wall clock/thread/fs | no core wall clock/thread RNG/fs input; debug trace fs failure is non-authoritative warning | E-017 | adjacent scheduling not a cross-process determinism proof |

## Required probe result

- A/B complete generated map equality: PASS in E-002 (`same_seed_and_state_reproduce_template_assignment_and_ids`).
- reversed insertion battle movement: PASS in E-002; exact ordered outputs, positions, and event-log JSON are compared with
  no canonical sorting.
- event heap: PASS in E-002 across 720 insertion permutations.
- separate process/platform run: not performed; same-process Rapier coverage and static ordering evidence only.

## Event ordering contract observed

1. Minimum absolute `time_ms` wins (heap comparison reversed for min-time behavior).
2. Semantic priority band fixes spawn/projectile/effect/cast/attack/rest/expiry/movement/boss/wave order.
3. Variant discriminator splits shared priority variants.
4. Variant identity fields order same-kind events.
5. Full Debug payload is the final tie-breaker after exact `PartialEq` early return.

This is a total order only if the manually asserted `Eq` precondition (notably finite floats) holds.
