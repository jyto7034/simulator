# Phase 10 CRA-P08-C03 Experiments

Status: complete
Implementation Complete: true

## 2026-08-10 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 09 CRA-P07-C02 has a current `Complete / High` audit with improvement class `none`.
- Phase 10 is next in the exact registry; Phases 11~44 remain pending and unauthorized.
- No candidate-specific policy ID applies. CRA-POL-006 freezes current public/data/save/checkpoint/DTO/wire shape.

### Evidence Carried Into Initialization

- CRA-P08-C03 remains a correctness correction with `1/2/1/2/2/2/unknown = 10/18`.
- Historical defects are `u64→i64` AttackIntervalMs narrowing plus unchecked signed multiplication, an unbounded
  percentage-ceil quotient narrowed to `u32`, and draft Attack/MaxHealth assignment without destination cap.
- Required first proof is a 0/1/type-max/`u64::MAX`/extreme-signed-percent table plus high Attack/MaxHealth at the
  authored 207% stage, executed with debug/release semantics.
- Preservation boundary is exact ordinary/live values, modifier/stat-pipeline order, interval floor, rounding,
  defense sign, MaxHealth/current-HP behavior, certified-power policy and all public/serialized shapes.
- Exact-order evidence places this Phase before CRA-P08-C02 so numeric cap semantics are stable before HP/death-aware
  runtime transitions.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 10 coordination changes from `ready` to `active`; no other Child is active.
- Phase 10 production status/source/data investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents completely, capture expanded dirty status and prospective target hashes, then trace
the live arithmetic owners/callers, declared stat ranges, rounding and current-health derivation before baselines.

## 2026-08-10 — E01 Dirty Baseline, Live Authority And Nonzero Baselines

### Dirty Preservation Baseline

- HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded status after Phase initialization: 416 entries, SHA-256
  `64c4fa91cc7a572969cd73441d41d8bb91955682f4792ac49110a0a58bbeae8d`.
- Prospective target hashes:
  - `stats.rs`: `e2790dd9e5d7ca888b9d7328a903d199e9592396878f79d5d2c6e0d628cd29d5` (clean)
  - `employee.rs`: `b6adeef17585d008e4cd34dd8915dfbbe6ce4069a417c4fa57a864a869017984` (modified)
  - `battle/stat_pipeline.rs`: `6c76286f3234e266a4455822770edc5e4607c8ad508fbc91cff9ab147c892afb` (clean)
  - `core_runtime_contract.md`: `efe701dc50a5cd12d611819af173bb7cdf68ba480233bfe46fca5155ccad3efb`
    (modified)
- Existing `employee.rs` changes add deployment eligibility and checkpoint identity helpers in non-overlapping
  regions. The canonical contains prior campaign additions; both remain user-owned and must be preserved.

### Complete Scoped Authority And Caller Graph

- `UnitStats::apply_modifier` uses wide/saturating `i64` arithmetic for other percentage modifiers, but
  AttackIntervalMs alone casts its `u64` base to `i64`, multiplies ordinarily, then feeds an `i64` delta to the 1ms-
  floor `add_attack_interval_ms` boundary.
- `employee::percent_amount_ceil(u32, u32) -> u32` has seven production/test call sites. Its `u64` product is wide
  enough for two `u32`s, but a quotient greater than `u32::MAX` is narrowed with `as u32`.
- The helper feeds deploy-cost and trauma reductions, employee HP bonus, draft MaxHealth/Attack scale, signed defense
  magnitude scale, consumable Attack bonus and initial resonance. Saturating callers already define their result
  range; a saturated helper result preserves ordinary behavior and closes their extreme domain.
- `apply_draft_stat_scale` directly assigns scaled MaxHealth/Attack. Attack's declared destination is `u32`; MaxHealth
  is capped by `UnitStats` at `i32::MAX as u32`, but this assignment bypasses that authority.
- `effective_stats_for_draft` recalculates current HP after scale. Employee-condition math is already `u64`; the
  non-employee ratio uses `u32::saturating_mul`, which loses the exact ratio for high capped MaxHealth. This is the
  historical safety net's required exact current-health derivation, not a separate candidate.
- `BattleUnitStatScale` is internal, defaults to 100/100/100 and receives live manifested difficulty multipliers.
  Canonical live Monolith stage three is HP 207%, Attack/Defense 182%; the 207% HP boundary is an authored producer.

### Nonzero Baselines

| Exact family | Result |
| --- | --- |
| `game::stats::tests` | 3 passed, 0 failed |
| `game::battle::stat_pipeline::tests` | 5 passed, 0 failed |
| `game::employee::tests` | 16 passed, 0 failed |

### Next Experiment

Add table-driven AttackIntervalMs, percentage-ceil and direct 207% draft cases. Run exact debug and release filters
against the pre-fix tree and record panic/wrap/range failures separately before production mutation.

## 2026-08-10 — E02 Test-First Debug And Release Failure Proof

### Boundary Tests Added

- AttackIntervalMs table: 0, 1, ordinary -25%, `i64::MAX`, first value above `i64::MAX`, `u64::MAX` and
  `i32::{MIN,MAX}` percentage values. Expected values use mathematical `i128` delta plus the existing 1ms/u64 range.
- Shared percentage-ceil table: zero, one, 1%/100%, `u32::MAX` value and `u32::MAX` percentage, with the quotient
  saturated to the declared `u32` result domain.
- Authored stage-three scale: internal `207/182/182` over capped-high MaxHealth/full current HP, `u32::MAX` Attack
  and negative Defense, requiring exact caps, sign/ceil and full-health ratio.

### Pre-Fix Results

| Profile/proof | Result | Exact observed defect |
| --- | --- | --- |
| debug AttackIntervalMs | 0/1 | ordinary multiplication panicked on overflow |
| debug percentage ceil | 0/1 | max/max narrowed to `4,037,269,259` instead of `u32::MAX` |
| debug authored stage three | 0/1 | MaxHealth narrowed to `150,323,854` instead of `i32::MAX` |
| release AttackIntervalMs | 0/1 | extreme positive case wrapped to `9,315,605,757,201,848,728` instead of `u64::MAX` |
| release percentage ceil | 0/1 | same max/max narrowing defect |
| release authored stage three | 0/1 | same MaxHealth narrowing defect |

Three initial invocations used short filter names with `--exact` and selected zero tests. They are explicitly
discarded and not counted. Release proof used isolated `/tmp/simulator_phase10_release_red` because abandoned
parallel release invocations temporarily held the shared target lock; no lock or repository file was removed.

### Next Experiment

Use an `i128` signed interval calculation inside `UnitStats`, saturate percentage ceil through `u128`, route scaled
MaxHealth through its destination-owned cap and compute the final current-HP ratio with a wide product. Then rerun all
six exact profile/proof combinations before preservation suites.

## 2026-08-10 — E03 Range-Preserving Arithmetic Implementation And Focused Green

### Production Change

- `UnitStats::apply_attack_interval_percent` now computes the signed percent delta from the full `u64` base in
  `i128`, preserves truncation toward zero and clamps the adjusted value to `1..=u64::MAX`.
- `percent_amount_ceil` now multiplies/divides in `u128` and saturates the mathematical ceil result to `u32::MAX`.
  Its seven existing consumers and rounding rule are unchanged.
- Crate-private `UnitStats::set_max_health` applies the existing `i32::MAX as u32` destination cap and clamps current
  HP. Draft scale routes only MaxHealth through it; Attack continues to use its `u32` domain and Defense retains its
  sign/symmetric magnitude rule.
- Non-employee current-HP ratio after draft scale now uses a `u128` product instead of `u32::saturating_mul`, so exact
  full/partial ratios survive high capped values.
- No authored scale, stat order, public type, data/save/wire shape, certified-power or Phase 12 death behavior changed.

### Exact Green Proof

| Profile/proof | Result |
| --- | --- |
| debug AttackIntervalMs / percentage ceil / authored stage three | 3/3 |
| release AttackIntervalMs / percentage ceil / authored stage three | 3/3 |
| `game::stats::tests` | 4/4 |
| `game::employee::tests` | 17/17 |
| `game::battle::stat_pipeline::tests` | 6/6 |
| certified combat power | 4/4 |
| combat preview, including live Monolith scale production | 27/27 |
| live RON integration | 25/25 |

### Canonical And Static Result

- `core_runtime_contract.md` now records one unsigned ceil scale, destination caps, wide post-scale HP ratio and
  full-u64 signed interval behavior inside the existing Combat Preview boundary.
- The three production narrowing/unchecked patterns are absent. Three similar `div_ceil(... ) as u32` expressions
  remain only in world tests with bounded expected values and are not runtime authorities.
- `cargo fmt --all -- --check` passes after only the three scoped formatting hunks were adjusted.

### Next Experiment

Run complete Core and game_server all-target validation, inspect scoped diff/status/hashes and finish static/public/
data compatibility review before the independent completion audit.

## 2026-08-10 — E04 Broad Validation, Dirty Closure And Audit Readiness

### Broad Validation

- Final `cargo test -p game_core`:
  - library: 958 passed, 0 failed, 0 ignored;
  - live item activation: 3/3; live skill catalog: 4/4; live RON: 25/25;
  - skill runtime: 13/13; skill suite: 4/4;
  - main/generator/doc targets selected zero and are not counted as behavior proof.
- `cargo check -p game_server --all-targets`: pass.
- `cargo fmt --all -- --check`: pass.
- scoped `git diff --check`: pass.

### Dirty Closure

- HEAD remained `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Final pre-audit expanded status: 418 entries, SHA-256
  `adcb874b0d4ddd7b629b9a840ed10eba8839afe7387d493727cc7725cd0c365f`.
- Exactly the two initially clean implementation targets changed classification to modified; pre-modified
  `employee.rs` and canonical stayed modified, and no unrelated path classification changed.
- Baseline → final hashes:
  - `stats.rs`: `e2790dd9e5d7ca888b9d7328a903d199e9592396878f79d5d2c6e0d628cd29d5` →
    `9c4d2c5645a5a701c041eae8ba14edbdfca232182eff0eca1da152c9b535fc1f`
  - `employee.rs`: `b6adeef17585d008e4cd34dd8915dfbbe6ce4069a417c4fa57a864a869017984` →
    `8d3940dce94a7e8d8858b39c2c48c22f367d76d3b31d23319e76859cc53eaa54`
  - `battle/stat_pipeline.rs`: `6c76286f3234e266a4455822770edc5e4607c8ad508fbc91cff9ab147c892afb` →
    `16b769a0d6dfd30236bdcc996f639f404e0759704d6bad7dc6436f4d163e38f2`
  - `core_runtime_contract.md`: `efe701dc50a5cd12d611819af173bb7cdf68ba480233bfe46fca5155ccad3efb` →
    `6e20d6a6c0609e6217b3a00d98371b7e13af12ef7d649f86855238c74c1ec00d`

### Compatibility And Audit Readiness

- No RON/live data, serialized/public type, save/checkpoint, command/result, server or Unity wire shape changed.
- Existing seven ceil consumers, authored scale production, stat order, ordinary/live outputs and certified-power
  composition are unchanged; only extreme valid values now saturate to their declared domains.
- Implementation is complete. Phase remains `active` until the completion guide is fully reread and an independent
  `Complete / High` audit is written.

### Next Experiment

Read all 567 lines of `goal_completion_review_guide.md`, perform the independent completion review and advance only
if the final verdict is `Complete / High` with no required immediate correction.

## 2026-08-10 — E05 Independent Phase-Exit Audit

- All 567 lines of `goal_completion_review_guide.md` were reread in four complete ranges before review.
- The audit independently rechecked runtime owners, seven ceil consumers, live scale production, public/data/save/
  wire absence, canonical alignment, dirty preservation and current validation evidence.
- Completion evidence C-01~C-10 all pass; no material legacy, compatibility, consumer, migration, scope or policy
  finding remains.
- Long-term fit: `High`; improvement class: `none`; primary verdict: `Complete`.
- Final phase-exit verdict: `Complete / High`. Phase 11 activation is allowed only after Master registry completion,
  relevant historical review, Child initialization and complete Child-document read.
