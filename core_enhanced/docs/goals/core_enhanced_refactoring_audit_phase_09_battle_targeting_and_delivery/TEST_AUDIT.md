# Core Enhanced Refactoring Audit Phase 09 — CRA-14 Test Audit

Audit Date: 2026-08-07
Status: complete; phase-exit audit `Complete / High`

Runner status is supporting evidence only. Dispositions come from setup/action/assertion comparison with production,
published data and canonical. A zero-test filter is not a valid invocation and is excluded below.

## Family Dispositions

| Family | Unique invariant/assertion | Reachability and overlap | Disposition |
| --- | --- | --- | --- |
| Tile range policy/pattern | exact membership, facing and valid-board cells | production primitive shared by attack/cast/preview | `retain`; consolidate with C06 |
| Basic target choice | hostile eligibility, deterministic ordering and tie behavior | direct production acquisition | `retain` |
| Basic windup/release | pending state, release recheck/reacquisition and no invalid hit | direct production lifecycle | `retain` |
| Basic projectile | moving-target homing, continuous target-only collision and fractional timing | direct production delivery | `retain`; share motion fixtures with C05 |
| Delivery math | ceiling flight time including zero-speed fallback | production primitive; zero case protects invalid behavior | `strengthen_then_replace` C01 |
| Skill publication | selection/range/delivery validation and current bundle | direct publication; programmatic zeros/basic compatibility omitted | `retain_and_strengthen` C01 |
| Negative target RON | conflict/legacy authority rejection | production schema, but several fixtures fail earlier | `correct_or_merge` C08 |
| Manual/automatic cast | typed acceptance, no-mutation rejection and target capture | direct player/autocast paths | `retain` |
| EnemySingle/usefulness | direct target usefulness and selection independence | production path; actual TileArea target set omitted | `retain_and_strengthen` C03 |
| Skill projectile collision | order, pierce/hit/kill stop and directional geometry | direct runtime; max-kills test omits cast cleanup | `retain_and_strengthen` C02/C05 |
| TileArea runtime | footprint/filter, fixed/follow tracking and event causes | direct production area state | `retain`; share query with C06 |
| Range/effect preview | cast/effect cells, all facings and pending deployment | direct checkpoint projection; area footprint parity omitted | `retain_and_strengthen` C04 |
| Released-effect lifecycle | withdrawal, relocation, Hard CC and instant cleanup | direct production; exposes stale target canonical | `retain` |
| Skill runtime contract | step/repeat/retarget, pending count and terminal cleanup | direct runtime, broad but stateful assertions | `retain_and_strengthen` C02/C03 |
| Authored skill scenarios | exact live targets, positions, effects and event causes | live definitions through production runtime | `retain` |
| Live catalog/reference | explicit target authority, delivery kind and resolved references | actual embedded bundle | `retain` |

## Exact Gaps And Required Tests

### C01 publication compatibility

- Reject speed zero and every optional projectile limit zero through both RON and programmatic constructors, asserting
  exact object/field/cause. Retain current positive ceiling cases and full embedded-source load.
- Table valid/invalid basic and weapon delivery: TargetLocked defaults accepted; targetless, collision, lifetime,
  kill/pierce and DirectionalCollision policies rejected rather than ignored.

### C02 terminal projectile lifecycle

- Add `debug_skill_cast_state(ability_seq).is_none()` and active-projectile absence to the current max-kills test.
- Add multiple same-time collision candidates and combined max-hits/max-kills/expiry cases; assert one terminal
  delivery, one cast completion and no later event.

### C03/C04 usefulness and projection

- Build an EnemySingle TileArea whose footprint excludes its anchor. Cover useful anchor/empty affected set and
  useless anchor/useful affected neighbor through automatic candidate, cast gate and execution.
- Assert range-preview effect cells equal actual runtime footprint at center/edge/all facings for 3x3 and 5x5.
  Multi-step differing footprints wait for the Phase 10 projection decision.

### C05/C06 consolidation

- Run common behavior tables for moving target, multiple bodies, exact/sub-ms impact, board exit and expiry through
  both basic and skill adapters. Assert external events/effects, not private record fields.
- Cross-product every tile policy/pattern/facing so materialized-cell containment equals membership after clipping.

### C07/C08 cleanup and test correction

- Inventory workspace, server and generated fixture consumers before removing public/serialized range or delivery
  surfaces; retain target-set goldens through any data migration.
- Make every negative RON fixture otherwise valid. Prove the repaired fixture parses, then restore exactly one invalid
  field and assert its typed failure path. Do not use catch-any-panic or bare `is_err` as contract proof.

## Merge And Removal Decisions

- Range pattern tests protect distinct geometry; share tables/builders under C06 but keep all pattern/facing cases.
- Basic instant/projectile and skill homing/directional cases protect different delivery semantics. Only duplicated
  motion setup/result comparison should merge under C05.
- Broad skill scenarios observe authored target/effect contracts and are not removable happy paths.
- The `lowest_health_enemy_skill_rule` filter repeated a case already selected by `skill_rule_`; the runner
  invocation is redundant, not the test itself.
- Negative RON cases may merge only after each unique rejected field/path is preserved. Their current false-positive
  shape is corrected, not promoted as validation evidence.
- No scoped `#[ignore]` marker exists.

## Focused Runner Evidence

| Group | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| selector/range/projectile primitive modules | 29 | 0 | 0 |
| skill runtime/catalog/scenario integration suites | 21 | 0 | 0 |
| focused selector/cast/schema filters | 49 | 0 | 0 |
| basic/released-effect withdrawal, relocation and control lifecycle | 13 | 0 | 0 |
| live bundle, typed error, cast interruption and deployment projection | 13 | 0 | 0 |

Forty-eight valid invocations executed 125 passing test instances with no failure or ignore. One focused filter
re-executed a test already selected by a broader filter, leaving 124 unique test cases. During result recovery,
unqualified guessed names combined with `--exact` produced zero-test results; those attempts were discarded, names
were source-verified and the actual tests were rerun. Green execution does not close the static max-kills leak,
zero-value/basic compatibility gaps, anchor-only usefulness, preview drift or false-positive negative fixtures.

## CRA-14 Verdict

Complete. Every assigned targeting/delivery family has an assertion meaning, production reachability, overlap and
disposition. No production test is removed on green count alone. C08 repairs false contract evidence, while C01–C07
receive the exact missing behavior and compatibility safety nets required before implementation.
