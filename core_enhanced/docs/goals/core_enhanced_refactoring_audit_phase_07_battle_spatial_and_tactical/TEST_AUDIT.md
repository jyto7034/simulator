# Core Enhanced Refactoring Audit Phase 07 — CRA-14 Test Audit

Audit Date: 2026-08-07
Status: complete; phase-exit audit `Complete / High`

Runner results are supporting evidence only. Each disposition comes from reading setup/action/assertions and
comparing them with current production paths, published policy and canonical.

## Family Dispositions

| Family | Production invariant and assertion meaning | Reachability/overlap assessment | Disposition |
| --- | --- | --- | --- |
| Battlefield layout | valid/non-rectangular tiles and static obstacles gate placement and enumerate voids deterministically | direct production topology primitive; two tests protect distinct invalid-cell classes | `retain` |
| Tile-range rotation | authored Up pattern rotates by facing and anchor inclusion is explicit | production deployment/range carrier; targeting meaning remains Phase 9 | `retain` |
| Movement world/data types | tile centers, data-unit conversion, quantization and body-radius reach are exact | position/serialization scale is production; body-shape construction will change in C04 | `retain_behavior`, decouple from ghost fields |
| Direct-only movement engine | custom movement, overlap, clamp and obstacle behavior | test-only backend; behavior largely duplicated by Rapier and makes alternate physics look authoritative | `remove_after_transfer` with C03 |
| Backend parity | Direct/Rapier order, obstacle, airborne and clamp agreement | mock-to-production equality is not an independent contract; the underlying behavior is valid | `merge_into_default_rapier` with C03 |
| Core Direct segment tests | explicit Direct tick plus immutable/no-motion segment logging | two segment assertions are unique, but selector is not production | `migrate_to_default_core`; remove selector-only test |
| Rapier synchronization | unit handle create/update/remove and static-obstacle lifecycle | actual production backend lifecycle; some assertions inspect private sets but guard stale physics state | `retain_and_privatize` with C03 |
| Rapier test-only board walls | wall creation and correction exclusion | `sync_board_bounds` is `#[cfg(test)]`; production bounds use pure clamp and never construct walls | `remove` with C03 |
| Rapier collision behavior | static stop/no-slide, unit overlap, airborne bypass and board clamp | direct production physics and meaningful position assertions | `retain` |
| Planner routes and blocking | range-before-route, no fallback chase, waypoint progress, sticky capacity and deterministic ties | production DefenseRoute logic; 25 tests separate route, air and block lifetimes | `retain` |
| Spatial distance/sweep | radii and moving-circle crossing math | body distance is production; sweep consumers are primarily Phase 9 delivery | `retain` as Phase 9 safety carrier |
| Stance/Focus/Rest | cooldown, Ground Focus, Platform Rest cadence, damage-before-Rest and action gates | production tactical state; assertions observe state/events/timing | `retain`; C01 adds activity-cross-tile cases |
| Ordinary relocation/swap | final position/facing, one batch/cost, profile/occupancy preflight, identity and engagement reacquisition | production commands and current canonical; no FieldWorking cross-kind case | `retain_and_strengthen` for C01/C02 |
| Field-work runtime | progress, multi-worker contribution, Hard CC suspension, completion, Platform suppression, final occupancy and death | production live object plus generic Core; relocation test is Ground-only and checks snapshot, not modifier identity | `retain_and_strengthen` for C01 |
| Tactical policy validation | required positive stance/Rest/relocation values and bounded resource prices | exact published typed-data boundary | `retain` |
| Tactical resource arithmetic | redeploy ceiling, recovery/payment/signed clamp and large elapsed saturation | direct active-session production state; three tests protect distinct overflow/payment cases | `retain` |
| Authored route integration | generated route progresses beyond first boundary and emits a segment | setup-to-Core integration is unique, but it explicitly switches to Direct | `retain_behavior_migrate_to_rapier` with C03 |
| World movement segment projection | adjacent immutable segments have authoritative times/positions in battle update | production Core projection; DTO shape itself remains Phase 10 | `retain` |
| World field-work projection/recovery | live setup/checkpoint progress, Pause and setup-loss recovery agree | current published Ground-only object; does not exercise C01's Any/cross-kind state | `retain` |
| World deploy/withdraw and Pause | cost, locks/current HP, mapping, frozen time and revision commit once | production commands; domain failures are checked before mutation | `retain`; add late cursor/reconcile cases for C02 |
| World stance/relocation/swap atomicity | cooldown/resource/profile/occupancy rejections preserve state and success commits once under Pause | production commands; tests stop at early domain validation and pass normal cursor | `retain_and_strengthen` for C02 |
| World future cursor | read-only state request rejects `since_seq` greater than latest | production resync behavior, but no mutating command is covered | `retain_and_strengthen` for C02 |

## Exact Gaps And Required New Tests

### C01 full-state tactical effect transition

- Construct an Any profile with a distinct unrelated incoming modifier and move a Ground FieldWorking unit to
  Platform, both leaving work and remaining assigned to a Platform work cell. Assert no extra subtraction, no effect
  while working, exact Ready+Defense Rest only after exit and one full interval.
- Mirror Platform→Ground and assert Focus is absent while working, then added exactly once on exit with exact
  base+bonus block and unrelated modifier preserved.
- Repeat both directions through two-unit swap, verify both final states, Rest epochs/deadlines and stale Rest no-op.

### C02 atomic tactical world mutation

- Submit an otherwise-valid deploy, withdraw, stance, relocation and swap with future `after_seq`; compare exact
  units/scenario/log/queue/signals, deployment mapping/salt/locks/facing, resource, employee attempt/focus and revision.
- Exercise a prevalidated employee-reconciliation failure and the direct Core invalid-draft/reused-salt seam to
  decide whether staging or stronger preflight is required; no error may leave partial state in the confirmed API.
- Retain stale UUID/revision and revision-overflow cases so early cursor validation does not reorder typed authority
  errors incorrectly.

### C03 production-only Rapier backend

- Run the authored-route integration and immutable/no-motion segment tests without any backend selector.
- Transfer every unique Direct collision/order assertion to default Core or Rapier, then delete assertions whose only
  meaning is parity with the removed backend.
- Preserve repeated-run exact movement log/result equality and current ground/air/unit-overlap policy.

### C04 minimal movement model

- Compile-search every proposed field/variant removal and characterize moving, blocked, dead, no-goal and reached-
  target output as observed by BattleCore rather than by unused engine shape.
- Golden compare default-backend body/checkpoint/segment events before and after reduction.
- Inventory public Rust/serde consumers before changing exported structs; no Unity DTO or event-log shape may move.

## Merge And Removal Decisions

- Seventeen test bodies depend on Direct: nine Direct-only engine tests, four backend-parity tests, three Core Direct
  tests and one authored-route integration test. Unique behavior transfers first; tests that protect only the removed
  selector/backend are then removed.
- The two Rapier board-wall tests protect a `#[cfg(test)]` subsystem that production never creates. They can be
  deleted with the subsystem while production manual-clamp tests remain.
- Rapier sync tests are not removed wholesale: stale unit/static collider lifecycle is production behavior. Public
  inspection helpers can be privatized or replaced with internal assertions after external-consumer review.
- Broad world tests remain because they observe distinct resource, Pause, mapping, revision, setup/recovery or
  final-occupancy contracts. Green happy-path status is not a removal reason.
- No `#[ignore]` marker exists in the Phase 7 scoped source/test paths.

## Focused Runner Evidence

| Command/filter group | Passed | Failed | Ignored | Meaning |
| --- | ---: | ---: | ---: | --- |
| battlefield field + tile range + movement types | 8 | 0 | 0 | topology, rotation and coordinate scale |
| movement engine + planner + Rapier backend | 59 | 0 | 0 | current alternate/default physics, path and block behavior |
| spatial + tactical stance + field work | 19 | 0 | 0 | geometry and ordinary tactical/activity behavior |
| tactical run-policy + resource state | 7 | 0 | 0 | strict positive/bounded policy and arithmetic |
| authored-route integration exact test | 1 | 0 | 0 | setup-to-route progression through Direct |
| nine exact world tests | 9 | 0 | 0 | movement projection, field work, cursor, deploy/Pause and tactical commands |

Twenty-three valid invocations executed 103 passing tests with no failure or ignore. Two initial parallel exact world
invocations did not return a terminal runner summary while build access was serialized; they were discarded and the
same tests were rerun serially, each executing once. Green execution does not close any static gap above.

## CRA-14 Verdict

Complete. Every assigned spatial/tactical family has a meaning, reachability,
overlap and disposition. Removal is limited to non-production backend/test-only paths after assertion transfer;
retained tests protect actual topology, physics, block, resource, Pause, tactical and field-work behavior; required
new tests directly observe the two confirmed failure boundaries.
