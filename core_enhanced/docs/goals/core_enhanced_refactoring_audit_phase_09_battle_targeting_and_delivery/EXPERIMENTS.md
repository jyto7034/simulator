# Core Enhanced Refactoring Audit Phase 09 — Experiments

Status: complete

## 2026-08-07 — Activation gate

### Evidence and result

- Master, Phase 0 boundary and Phase 8 completion audit were rechecked.
- Direct predecessor Phase 8 is `Complete / High`; the reserved Phase 9 path did not exist.
- Phase 7 spatial and Phase 8 effect/damage authorities are fixed handoffs; Phase 10 remains projection/server owner.
- No other active child exists. Phase 9 is investigation-only and production mutation remains prohibited.
- Exact start commit remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.

### Tests

No runner was executed during activation.

### Decision

Phase 9 may perform read-only production investigation and write only audit artifacts. Phase 10 remains pending.

## 2026-08-07 — Scoped production baseline

### Exact categories and initial hashes

| Category | Initial SHA-256 |
| --- | --- |
| basic target/delivery Core (5 files) | `a1fdbe66001fd88c32e3977e90b91870471f0021d3e1dac61e1c911e4d1f7800` |
| skill runtime (4 files) | `87e0f7b3a66e603aea2f35ebaa42e6589d6f3e90682cc74c5579e18f77dc5f5d` |
| shared Core carriers (4 files) | `7b121bebe71a3b2ea6746c72dabcd10b2615e05a082ee04915982b8cb6cc6d29` |
| typed model/range carriers (3 files) | `82acd1626a572e922ca4a4c2726e0233143790e732e7021d5f488e4dab6f5aa4` |
| published typed input/validation (6 files) | `00e6b550d9d243e11d7b51a47132161f60a4a6ba2357ad3179b4c89f14210ff8` |
| relevant live RON (5 files) | `13c070173a2ece6a911c7b762a436d68159d56a5928b417dbbca53ee53fba37d` |
| focused integration sources (4 files) | `f8c4baf9a041cfe165299daa445822371e0406d667329b66ccd8d08813444775` |
| authored skill scenario tree (5 files) | `cd51e3cb2a5b621f6d4bf6512ceb4dbf23e9cb4516f930b8d6cfb1eca9b0d713` |
| focused world test source | `87861813beff818f37e914cb3e25515e4b324508225711feb6c43fd03eae53ef` |
| nearest canonical (4 files) | `fe6de029f1620a76957221120293e6e795ebc19f4a1a0da554e502babb08673d` |
| scoped Git status text | `c174dd493211e19b545e17df15c44c02cecbf3888f991d8ef812ef94b8945fd2` |

Basic Core is `core/{basic_attack,basic_attack_projectile,projectile_math,target_usefulness,targeting}.rs`; skill
runtime is all four `core/skill_runtime/**` files. Shared Core is `core/{commands,mod,sim,types}.rs`; typed/range is
`ability.rs`, `range_preview.rs` and `battle/tile_range.rs`. Published typed input is
`data/{skill_data,validation,abnormality_data,corroded_employee_data,equipment_data,skill_fragment_data}.rs`.

### Dirty-state result

Twenty-two scoped production/data/test/canonical paths were already modified, including all primary selector files,
ability/skill schemas, live equipment/skills, range preview, world combat tests and canonicals. These are user-owned
baseline changes.

### Tests

No runner was executed for the baseline.

## 2026-08-07 — Static targeting and delivery gaps before focused validation

### Evidence and result

- Basic attack eligibility has one actual authority: tile range policy membership plus deterministic target scoring.
  `range_units` is authored/copied/validated but not read by production eligibility; `bodies_in_range` and public
  base-UUID delivery lookup have no workspace production caller.
- Manual and automatic skill acceptance capture explicit targets and reuse/retarget them deterministically. EnemySingle
  TileArea usefulness nevertheless evaluates the anchor before runtime footprint/filter materialization, so a valid
  excluded-anchor footprint can make acquisition and execution disagree. Current live footprints include anchors.
- Basic TargetLocked and skill TargetLocked/DirectionalCollision runtimes preserve their intended target/source
  snapshots. Area GroundFixed/Follow modes have explicit capture/tracking lifetimes and released-effect tests agree.
- Directional projectile `max_kills` removes the runtime after a nonterminal impact but never resolves terminal step
  delivery. The active cast remains with a pending delivery count. Its current focused test asserts damage exclusion,
  not cast cleanup.
- Basic and skill projectile runtimes duplicate millisecond motion, continuous sweep, fractional hit-time/position and
  next-event logic. Payload/identity/stop semantics remain legitimately separate, and Phase 8 owns committed effects.
- Projectile speed and programmatic optional limits are not uniformly nonzero. Basic and weapon delivery validation
  accepts different skill-only policy fields that basic runtime ignores. Current live values are positive.
- Self-target effect preview projects cast reach rather than TileArea footprint. Live non-player examples have front
  3x2 acquisition with 3x3/5x5 delivery; no current player-fragment incident was found. Multi-step projection needs a
  union/per-step decision in Phase 10.
- Membership, manual reach, preview materialization and area materialization repeat the same policy/pattern/facing/
  clipping interpretation. Inspected current behavior agrees; one typed query can consolidate it without adding
  obstacle semantics.
- `skill_target_contract.md` says withdrawal removes every released skill projectile/area, while production, focused
  tests, `game_rulebook.md` and `core_runtime_contract.md` preserve released projectiles and GroundFixed areas and end
  Follow modes. The target-contract sentence is stale.
- One inline conflict test catches any panic but currently reaches a missing `delay_ms` first. Several negative legacy-
  authority RON fixtures also assert only `is_err` with unrelated required fields missing. They can remain green
  without proving the named rejection.

### Focused validation reason

Run the smallest selector, cast, projectile/area, range-preview, publication/live-data and released-effect families
to establish assertion meaning and runner counts. Green results cannot close the terminal leak, validation gaps,
anchor-only usefulness, projection drift or false-positive negative fixtures recorded above.

## 2026-08-07 — Focused validation

| Command/filter group | Passed | Failed | Ignored | Meaning |
| --- | ---: | ---: | ---: | --- |
| selector/range/projectile primitive modules | 29 | 0 | 0 | policy geometry, target choice, continuous sweep and flight math |
| skill runtime/catalog/scenario integration suites | 21 | 0 | 0 | step/delivery lifecycle and authored live behavior |
| focused selector/cast/schema filters | 49 | 0 | 0 | acceptance, reuse, usefulness and typed publication boundaries |
| basic/released-effect withdrawal, relocation and control lifecycle | 13 | 0 | 0 | release snapshots, tracking and control survival |
| live bundle, typed error, cast interruption and deployment projection | 13 | 0 | 0 | embedded publication, failure carriers and range transport |

Forty-eight valid invocations executed 125 passing test instances with no failure or ignore. One test was selected by
both `skill_rule_` and the narrower `lowest_health_enemy_skill_rule` filter, so the set contains 124 unique tests.
Guessed unqualified names combined with `--exact` produced zero-test recovery attempts; they were discarded, source
names were verified and actual tests were rerun. No scoped ignored marker was found.

Green results preserve current selection/delivery behavior but do not assert max-kills cast cleanup, programmatic
zero limits, basic consumer compatibility, actual-area usefulness or preview/runtime footprint parity. Negative RON
greens also do not prove their named cause until fixtures and assertions are repaired.

## 2026-08-07 — Candidate and CRA-14 disposition

- Confirmed C01 projectile publication/basic compatibility and C02 max-kills terminal lifecycle as correctness
  candidates; C02 is a demonstrated state leak even though live RON does not author the option.
- Confirmed C03 actual-delivery usefulness and C04 delivery-footprint preview corrections with multi-step/transport
  policy gates and accurate `not_observed` live classifications.
- Confirmed C05 continuous projectile kernel and C06 typed tile query as behavior-preserving refactors; neither absorbs
  Phase 7 position or Phase 8 committed effects.
- Deferred C07 ghost range/public delivery cleanup behind Phase 10, external Rust and serialized compatibility
  inventory. Confirmed C08 as a test-correction candidate for false-positive negative RON fixtures.
- Retained deterministic target choice/capture, delivery mode distinctions, area tracking and released-effect
  semantics; classified the contradictory target-contract sentence as stale canonical.

## 2026-08-07 — Final no-mutation proof

The exact initial categories and final recomputation are identical:

| Category | Initial SHA-256 | Final SHA-256 | Result |
| --- | --- | --- | --- |
| basic target/delivery Core | `a1fdbe66001fd88c32e3977e90b91870471f0021d3e1dac61e1c911e4d1f7800` | `a1fdbe66001fd88c32e3977e90b91870471f0021d3e1dac61e1c911e4d1f7800` | identical |
| skill runtime | `87e0f7b3a66e603aea2f35ebaa42e6589d6f3e90682cc74c5579e18f77dc5f5d` | `87e0f7b3a66e603aea2f35ebaa42e6589d6f3e90682cc74c5579e18f77dc5f5d` | identical |
| shared Core carriers | `7b121bebe71a3b2ea6746c72dabcd10b2615e05a082ee04915982b8cb6cc6d29` | `7b121bebe71a3b2ea6746c72dabcd10b2615e05a082ee04915982b8cb6cc6d29` | identical |
| typed model/range carriers | `82acd1626a572e922ca4a4c2726e0233143790e732e7021d5f488e4dab6f5aa4` | `82acd1626a572e922ca4a4c2726e0233143790e732e7021d5f488e4dab6f5aa4` | identical |
| published typed input/validation | `00e6b550d9d243e11d7b51a47132161f60a4a6ba2357ad3179b4c89f14210ff8` | `00e6b550d9d243e11d7b51a47132161f60a4a6ba2357ad3179b4c89f14210ff8` | identical |
| relevant live RON | `13c070173a2ece6a911c7b762a436d68159d56a5928b417dbbca53ee53fba37d` | `13c070173a2ece6a911c7b762a436d68159d56a5928b417dbbca53ee53fba37d` | identical |
| focused integration sources | `f8c4baf9a041cfe165299daa445822371e0406d667329b66ccd8d08813444775` | `f8c4baf9a041cfe165299daa445822371e0406d667329b66ccd8d08813444775` | identical |
| authored skill scenario tree | `cd51e3cb2a5b621f6d4bf6512ceb4dbf23e9cb4516f930b8d6cfb1eca9b0d713` | `cd51e3cb2a5b621f6d4bf6512ceb4dbf23e9cb4516f930b8d6cfb1eca9b0d713` | identical |
| focused world test source | `87861813beff818f37e914cb3e25515e4b324508225711feb6c43fd03eae53ef` | `87861813beff818f37e914cb3e25515e4b324508225711feb6c43fd03eae53ef` | identical |
| nearest canonical | `fe6de029f1620a76957221120293e6e795ebc19f4a1a0da554e502babb08673d` | `fe6de029f1620a76957221120293e6e795ebc19f4a1a0da554e502babb08673d` | identical |
| scoped Git status text | `c174dd493211e19b545e17df15c44c02cecbf3888f991d8ef812ef94b8945fd2` | `c174dd493211e19b545e17df15c44c02cecbf3888f991d8ef812ef94b8945fd2` | identical |

Only Master/Phase audit artifacts changed. Cargo wrote ignored target/debug artifacts; no production/data/test/
canonical/server/Unity file was edited, formatted, moved or deleted.
