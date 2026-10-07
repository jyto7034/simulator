# Phase 14 CRA-P09-C02 Experiments

Status: complete
Implementation Complete: true

## 2026-08-10 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 13 CRA-P08-C04 has a current `Complete / High` audit with improvement class `none`; its final 234-line audit
  was fully reread after the record-only correction.
- P09-C02 has no technical entry dependency or candidate-specific policy decision. CRA-POL-006 applies globally.
- Phase 14 is next in the exact registry; Phases 15~44 remain pending and unauthorized.

### Evidence Carried Into Initialization

- CRA-P09-C02 remains correctness correction with `1/3/3/2/3/2/unknown = 14/18`.
- Historical defect: a nonterminal directional impact can reach max-kills, remove its projectile side-effectfully and
  still pass `terminal = false` to cast delivery resolution, leaking pending count and active cast.
- Target authority: one typed stop outcome is consumed by the existing delivery resolver; projectile retirement and
  pending-count decrement occur exactly once for the same accepted impact.
- Required first proof: projectile/cast absence, exactly-one terminal completion and same-time combined max-kills,
  max-hits and expiry cases, while collision/effect order and later-target exclusion remain exact.
- Publication validation, committed damage, motion extraction, targeting/usefulness and preview remain later scope.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 14 coordination changes from `ready` to `active`; no other Child is active.
- Phase 14 production status/source/data investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents and the Phase 13 completion audit completely. Then capture expanded dirty status and
prospective target hashes before tracing current projectile/cast lifecycle or running nonzero baselines.

## 2026-08-10 — E01 Required Reads And Dirty Baseline

### Resume And Dependency Gate

- The user-provided 198-line resume instructions and current 421-line Master Plan were fully read.
- All 262 lines across the Phase 14 Plan, Experiments and Notes were fully read before production/status/test
  inspection. The final 234-line Phase 13 completion audit was then fully reread.
- Child `active`, Master Phase 14 `active`, Phase 13 `complete — Complete / High` and Phases 15~44 `pending` are
  consistent. No technical dependency or policy gate blocks current-tree discovery.

### Dirty Boundary

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded post-initialization status: 439 entries; normalized SHA-256
  `53ed00c3f2db0e5b0b65a5926004032a7ebe7c84a02441b6f9931db71097f8d8`.
- Prospective fingerprints before Phase 14 content mutation include:
  - skill projectile `6de51ab750f571e8c9fecb921d770bb9a39cd31b98e9fe2e57dd149795f463a3`
  - skill cast `e8356dddee42726f862c5efae8f779572b32cb5b13078c532bd80271769d8b0d`
  - skill runtime module `6bd8faff6ccd9447fdc925ff996b014d322a5a613eba772d7eea74069b6cdbea`
  - skill area `2a22998063d482cc37eeeb620f61c6fac20a63ff1e0c5085fad094858b6978ee`
  - battle sim `050dc74b736d97796909ab1468b003018d5b07a269d651be880cbe53b239bde5`
  - commands `08c773999dac25f4e5bea3c35971b2c5aa24017b6a69d98d335cb9fd865c465a`
  - core types `ba33c56bf84231377114562e659c8543303ef5bfe5d62edd0ec7c42dbc9a7eaf`
  - ability `6e3f14a91945fd7554521e1113811d1948c2350d6a0593b995c2542c035840e8`
  - skill data `b70a9c87c56058027af82daace21b7d2090ed90fa1f99b281512a2218063418b`
  - runtime integration `773d5fdcdfdb90f58820c3b21ee441bb87bb16e8499a3bf2c2ad1b11466d4cfd`
  - target canonical `81fa6a1e687a2eef98a0b6843fd00eae0a678cc77b93a97a9b9ef9af07abc135`
  - runtime canonical `6e20d6a6c0609e6217b3a00d98371b7e13af12ef7d649f86855238c74c1ec00d`
  - rulebook `6c53ecd5d0588aace1a2b043b31434d004327d60938ecf61066ac19acfd31951`
  - live Skill RON `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`.
- Skill cast and skill runtime module are currently clean. The other 12 prospective tracked paths are already
  modified by user/campaign work and remain user-owned until exact scoped diffs are reconciled.

### Next Experiment

Inspect existing scoped diffs first, then trace the actual cast/projectile/impact/stop/delivery path and run nonzero
baselines before adding any Phase 14 test.

## 2026-08-10 — E02 Current Graph, Baseline And Red Proof

### Current Producer-To-Consumer Graph

1. `sim.rs::execute_skill_step` calls `dispatch_skill_projectile_delivery`; every successful launch contributes to
   `launched_count`, then `begin_skill_step_delivery` adds that count to the active cast step.
2. Fixed runtime advancement temporarily removes the `ActiveProjectileRuntime`, deterministically orders accepted
   sweeps by hit time, distance and unit identity, queues `SkillProjectileImpact`, and reinserts only nonterminal
   runtime work.
3. The queued impact's `terminal` bit is decided from motion/collision state (`max_hits`, non-piercing or expiry)
   before damage. `apply_skill_projectile_impact` records the impact, executes commands and receives the authoritative
   `killed_target_count` summary.
4. The former `apply_skill_projectile_kill_stop_policy` updated kill count and removed the active projectile as a
   side effect but returned no outcome. `resolve_skill_step_delivery` therefore received the earlier motion flag;
   only that resolver decrements `pending_delivery_count`, releases deferred steps and cleans the active cast.
5. With `max_kills=1`, a lethal impact queued as nonterminal removed the projectile but left the step at pending one.
   The source graph therefore exactly matches the historical defect and needs no policy decision.

### Baseline And Test-First Evidence

- Before assertions were strengthened, the exact existing test passed: `1 passed`, `0 failed`, `12 filtered out`.
- The existing max-kills integration was changed to retain the final `BattleCore` and require zero active skill
  projectiles plus `debug_skill_cast_state(ability_seq).is_none()`.
- Exact red command:
  `cargo test -p game_core --test skill_runtime_contract directional_collision_projectile_respects_max_kills_stop_policy -- --exact --nocapture`
- Red result: exit 101, `0 passed`, `1 failed`, `12 filtered out`; the projectile-count assertion passed and the
  active-cast cleanup assertion failed at `skill_runtime_contract.rs:2009`.
- A separate real-path boundary places two deterministic collision candidates at one world position and combines
  `max_kills=1`, `max_hits=1` and one-millisecond expiry. Before production correction it passed `1/1` with exactly
  one terminal impact at expected expiry, one damaged/killed deterministic candidate, no active projectile and no
  active cast. This freezes the already-correct combined-stop path while the isolated kill-stop leak remains red.

### Disposition

- CRA-P09-C02 is reproduced through the public integration harness and remains `still_valid`.
- Public event/RON/save/wire shape, collision ordering, effect meaning and Phase 28/29 seams need no change.

## 2026-08-10 — E03 Typed Stop Outcome And Focused Green

### Correction

- Added private `SkillProjectileStopOutcome::{Continue, Terminal}` at the impact/kill-stop seam.
- `apply_skill_projectile_kill_stop_policy` still performs the same cumulative kill accounting and projectile
  removal, but now returns `Terminal` only when that accepted impact reaches the authored kill limit.
- `apply_skill_projectile_impact` now has one `resolve_skill_step_delivery` call. It composes the queued
  motion/collision terminal bit with the typed kill-stop outcome, so the same impact retires the runtime and
  decrements the owning pending delivery once.
- The impact record remains before command/effect records, and target ordering, event identity/cause, committed
  source snapshot and damage execution are unchanged.
- The nearest projectile lifecycle authority, `docs/skill_target_contract.md`, now records kill-stop exactly-once
  resolution and composition with same-time hit-limit/expiry.

### Focused Validation

- Former red max-kills exact test: `1 passed`, `0 failed`, `13 filtered out`.
- Same-time combined-stop exact test: `1 passed`, `0 failed`, `13 filtered out`.
- Complete `skill_runtime_contract`: `14 passed`, `0 failed`, `0 ignored`.
- Core lib `skill_projectile` filter: `6 passed`, `0 failed`, `968 filtered out`.
- Core lib `released` filter: `10 passed`, `0 failed`, `964 filtered out`, including Boss released-projectile
  cancellation and withdrawal/Hard-CC preservation.

### Next Experiment

Run full Core/live-skill/skill-suite and affected server validation, then complete static compatibility, dirty-diff,
canonical and independent phase-exit review gates.

## 2026-08-10 — E04 Broad, Release, Static And Dirty Reconciliation

### Broad And Release Validation

- `cargo test -p game_core --lib`: `974 passed`, `0 failed`, `0 ignored`.
- `cargo test -p game_core --test skill_runtime_contract`: `14 passed`, `0 failed`, `0 ignored`.
- `cargo test -p game_core --release --test skill_runtime_contract`: `14 passed`, `0 failed`, `0 ignored`.
- `cargo test -p game_core --test live_skill_catalog_audit`: `4 passed`, `0 failed`, `0 ignored`.
- `cargo test -p game_core --test skill_test_suite`: `4 passed`, `0 failed`, `0 ignored`.
- `cargo test -p game_core --test ron_loading`: `25 passed`, `0 failed`, `0 ignored`.
- `cargo test -p game_core --test live_item_skill_activation`: `4 passed`, `0 failed`, `0 ignored`.
- `cargo test -p game_server --all-targets --quiet`: main target `59 passed`, `0 failed`, `0 ignored`; four binary
  targets ran zero tests and passed but are not behavioral evidence.
- `cargo check -p game_core --all-targets` and `cargo check -p game_server --all-targets`: passed.
- `cargo check -p game_core --release` and `cargo check -p game_server --release`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.

### Static Authority And Compatibility

- Current production inventory has one private `SkillProjectileStopOutcome`, one kill-stop producer, one producer
  caller and one `resolve_skill_step_delivery` call in the projectile impact handler.
- The cast resolver, `SkillStepProgress`, event enum/log, behavior DTO conversion, public `DeliveryDef`, validation,
  checkpoint and server/Unity-facing shapes were not changed by this Phase.
- Live `game_resources/data/skills/base.ron` still authors no `max_kills`; its SHA-256 remains the E01 value
  `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`. The correction remains a published
  programmatic runtime contract without a data migration.
- No alias, fallback, compatibility shim, side table, duplicate resolver or Phase 15/28/29 placeholder was added.

### Dirty Reconciliation

- Expanded status remains exactly 439 entries with E01 normalized SHA-256
  `53ed00c3f2db0e5b0b65a5926004032a7ebe7c84a02441b6f9931db71097f8d8`; no path-level dirty entry was added,
  removed or normalized.
- Final scoped fingerprints:
  - projectile runtime `a100b805f01991aa392dddc3c1d4135f32b64d3bf414bfeae88118e1e834df5b`
  - unchanged cast runtime `e8356dddee42726f862c5efae8f779572b32cb5b13078c532bd80271769d8b0d`
  - strengthened runtime integration `afd09b3f6392e8c0175a10d5be3781638e6221c956a55e8c4d129f80c1b49c58`
  - updated target canonical `631a36443745301231af6e93899b1e34a07846d7156dacf54fc3148b633d6e43`
  - unchanged ability shape `6e3f14a91945fd7554521e1113811d1948c2350d6a0593b995c2542c035840e8`
  - unchanged runtime canonical `6e20d6a6c0609e6217b3a00d98371b7e13af12ef7d649f86855238c74c1ec00d`
  - unchanged rulebook `6c53ecd5d0588aace1a2b043b31434d004327d60938ecf61066ac19acfd31951`.
- Current public-event reference hashes were recorded as event log
  `397e2380a21c79dc3addf10177c8ab0a7d85c25f80eba808da9894b09ef36e02` and behavior conversion
  `65349de8daf5230cd00041b9f3da4b66e76239245e608b98a2623312b2276139`; neither received a Phase 14 hunk.
- Existing Phase 12 projectile modifier-source work and the three pre-existing integration hunks remain intact.
  No reset, restore, clean, stash, deletion or broad formatter rewrite was used.

### Exit Readiness

P14-C01 through P14-C08 have implementation, behavior, compatibility and dirty-preservation evidence. Perform the
required full completion-guide reread and independent audit; Phase 15 stays pending until `Complete / High`.

## 2026-08-10 — E05 Phase-Exit Evidence Strengthening

### Independent Review Refinement

- The full 567-line completion review guide was reread before the phase-exit audit.
- Review found that the final max-kills proof should directly freeze a cumulative nonterminal branch, not only
  `max_kills=1` terminal behavior. The production code was already correct; the real integration was strengthened
  to `max_kills=2` so the first lethal impact continues and the second lethal impact terminally cleans the delivery
  and cast.
- The first three-target draft correctly produced two first-cast impacts but its final global projectile-count
  assertion observed a later autocast projectile while a third opponent kept the battle running: exit 101, one
  assertion failure. This was a test-scope defect, not a production regression or a repeat of the cast leak.
- The corrected two-opponent fixture ends the battle on the second cumulative kill. It now proves two ordered
  impacts/damage targets, continuation after kill one, terminal cleanup after kill two, zero active projectile and
  no active cast. Later-candidate exclusion remains independently frozen by the same-time two-candidate
  max-kills/max-hits/expiry test.

### Audit-Current Revalidation

- Cumulative max-kills exact test: `1 passed`, `0 failed`, `13 filtered out`.
- Same-time combined-stop exact test: `1 passed`, `0 failed`, `13 filtered out`.
- Complete debug `skill_runtime_contract`: `14 passed`, `0 failed`, `0 ignored`.
- Complete release `skill_runtime_contract`: `14 passed`, `0 failed`, `0 ignored`.
- Formatter and `git diff --check`: passed.
- Expanded dirty status remains 439 entries at the exact E01 normalized hash.

### Canonical Clarification

The nearest canonical now distinguishes the append-only impact event's pre-effect motion/collision terminal fact
from the private post-effect kill-stop outcome consumed by cast delivery cleanup. This documents current behavior
without changing or retroactively mutating event/DTO/wire data.
