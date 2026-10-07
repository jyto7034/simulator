# Phase 29 CRA-P09-C05 Completion Audit

Audit Mode: phase_exit
Audit Target: Phase 29 — CRA-P09-C05 One Continuous Projectile Motion Kernel
Current Status: active
Implementation Complete Claim: false
Candidate / Work Type: CRA-P09-C05 / behavior-preserving refactor
Policy Source: CRA-POL-003 and CRA-POL-006 in the approved final policy record
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Master Registry: Phase 29 active; Phase 30 CRA-P10-C01 pending on Phase 29 `Complete / High`
Repository / Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Dirty State Before Audit Artifact: 521 expanded rows, normalized C-sort SHA-256
`7190957ea2deef378bc645c3394520938b5f4246d95ce1c66ede5e4cdbf1f59b`
Audit Date: 2026-08-11 (`Asia/Seoul`)

## Audit Procedure And Independence

- This is a distinct audit-only pass after the implementation and its evidence were stabilized. No production,
  live-data, public-contract or canonical correction was made during this pass.
- The complete 874-line `goal_authoring_and_execution_guide.md`, complete 567-line
  `goal_completion_review_guide.md` and complete 198-line resume attachment were reread before the verdict.
- The final 212-line PLAN, 239-line experiments and 137-line notes were reread from beginning to end. Master Phase
  29/30 registry state and the direct Phase 28/14/15/Boss completion dependencies were checked separately.
- Current common math, both adapters, runtime carriers, event dispatch, committed damage/effect handoffs, strict
  publication, live RON, Boss leaf, behavior projection, server/Unity canonicals and scoped dirty state were read
  directly. Goal claims were treated as pointers, not proof.
- Static duplicate/fallback/placeholder searches and fresh audit-time focused and broad debug tests were executed.
  Earlier release tests/checks remain current because source, tests, data, dependencies and environment did not
  change afterward; only Goal evidence text and this audit artifact changed.

## Approved Policy Restatement

This Phase may change only private implementation ownership. One private pure typed query must own the mechanics
that are genuinely identical between basic and skill projectiles: path-window endpoints, continuous moving-body
sweep, fractional collision position, ceiling collision time, stable collision-fact ordering and next 1ms time.
It must not acquire unit lookup, target selection, runtime mutation, event scheduling, damage/effects, accepted-hit
policy or cast terminal lifecycle.

The basic adapter must remain a target-only detached delivery with its own ID, launch snapshot, Homing/Fixed guidance
meaning, validity/miss behavior, event identity and Phase 28 committed-damage handoff. The skill adapter must retain
TargetLocked homing and DirectionalCollision fixed paths, skill collision radius/filter, prior-hit, pierce/hit/kill/
expiry state, impact/effect construction and Phase 14 exactly-once terminal ownership. Boss cancellation must still
drop unreleased work while preserving an already-released ordinary projectile leaf.

CRA-POL-003 requires strictly positive published projectile speed and optional stop limits; the refactor cannot add
a permissive zero-speed publication route or normalization. CRA-POL-006 requires current public Rust records,
events, RON, save/checkpoint, server, wire and accessible/unknown consumer compatibility to remain intact. The
approved target contains no gameplay, rounding, collision, data, persistence or rollout migration.

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy basis | Runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P29-C01 | Both complete motion/lifecycle/caller graphs identified | Phase objective | common math; basic spawn/advance/hit; skill dispatch/spawn/advance/impact; event and consumer graph reread | static callers plus projectile 33, commands 43, skill 14, Boss 22 | pass |
| P29-C02 | Dirty targets and concurrent work preserved | authoring dirty gate | activation 520 rows; final pre-audit 521 differs only by newly modified clean math path; protected marker exact | scoped status/hash, marker stat/hash, fmt and diff checks | pass |
| P29-C03 | Old motion behavior characterized first | behavior-preserving boundary | four-row observable basic/homing parity test passed before production extraction; existing directional matrix was green | old production: parity 1, projectile 32, skill runtime 14; 0 failed/ignored | pass |
| P29-C04 | One private pure typed motion authority exists | target authority | `ProjectileMotionWindow::resolve` and private typed facts; no `BattleCore` or mutation access | one-owner search and common ordering unit test in projectile 33 | pass |
| P29-C05 | Numeric time/position/sweep behavior is exact | no rounding/cadence change | preserved f32 operation order, ceil/clamp/interpolation, fixed sampling, +1ms result and expiry clamp | parity zero/sub/exact/moving rows, fractional-position test and directional expiry tests | pass |
| P29-C06 | Basic lifecycle remains distinct and exact | target-only/basic invariant | exactly one locked candidate; adapter owns validation, miss, queue, record and Phase 28 hit | commands 43 plus projectile 33: bystander, moving, death, withdrawal, relocation, snapshot cases | pass |
| P29-C07 | Skill lifecycle remains distinct and exact | skill policy invariant | adapter owns filter, prior hit, pierce/max-hit/max-kill/expiry, effects and terminal composition | skill runtime 14 plus projectile 33: later body, facing, pierce, hit, kill and same-time expiry | pass |
| P29-C08 | Phase 14/28 and Boss seams remain exact | predecessor contracts | basic calls one committed authority; skill calls one effect/terminal path; Boss has no motion fork | commands 43, skill runtime 14 and Boss 22 debug/release | pass |
| P29-C09 | Public/data/save/server/Unity compatibility is exact | CRA-POL-003/006; no migration | public carriers/events/behavior, live RON, data/canonicals/server/Unity hashes unchanged except one stale canonical sentence | RON 25, live catalog 4, live item 4, Core/server broad debug/release | pass |
| P29-C10 | Focused/broad debug/release/static validation is green | Master validation gate | final tested source hashes recorded; no post-test source drift | Core 1,072 and server 59 in both profiles; checks/fmt/Clippy exit 0 | pass |
| P29-C11 | Downstream boundary and independent audit are current | Master linear gate | Phase 30 directory/source/placeholder absent; private motion seam is complete | this independent audit: `Complete / High` | pass |

## Runtime And State Evidence

### Current Authority Path

| Runtime path | Adapter-owned input and lifecycle | Shared pure facts | Authoritative handoff |
| --- | --- | --- | --- |
| basic projectile | one locked target, live validity/body, guidance aim, deadline/stale guard, miss/log/queue/map state | speed-limited window, one moving sweep, ceiling time, impact/end position, next time | one `execute_committed_damage(BasicAttack)` call |
| homing skill projectile | skill-valid locked target, collision radius/filter, runtime/impact/terminal event | same speed-limited window and one-candidate collision facts | one queued skill impact, then existing effect/cast resolution |
| fixed skill projectile | authored fixed aim, eligible bodies, prior hits, pierce/max-hit/max-kill/expiry and terminal state | fixed trajectory window, ordered moving-body collision facts, end/next time | accepted impacts queued in order, then existing effects and terminal composition |

- `projectile_math.rs:4-188` contains the full common authority. All new types and methods are bounded with
  `pub(in crate::game::battle::core)` and contain only scalar/vector/time/candidate values.
- `ProjectileMotionWindow::speed_limited` preserves the exact former endpoint sequence: saturating duration,
  direction/length, data-to-world speed conversion, f32 maximum step, epsilon/zero-speed/reached-aim branch and
  proportional position.
- `fixed_trajectory` samples the established linear launch path at the adapter-provided elapsed endpoints. A zero
  total or elapsed-at/after-total still returns the aim exactly.
- `resolve` calls the existing `moving_circle_sweep_hit_fraction` once per candidate, multiplies the same f32
  duration and fraction, applies `ceil`, clamps to window duration and uses saturating absolute time.
- Projectile and target positions are both interpolated at the continuous fraction. Their squared separation is
  retained solely as the second deterministic ordering key.
- Ordered facts use absolute hit time, squared separation and generic ID `Ord`. `UnitInstanceId::Ord` is UUID-byte
  order, exactly matching the former skill adapter's explicit byte comparison.
- The result exposes end position and `end_time + 1ms`; it does not enqueue that time. The constant has one owner.
- The common module imports no `BattleCore`, unit map, event, log, damage, effect, skill, cast, runtime record or
  persistence type. Its only domain dependency beyond vectors is the preexisting pure sweep function.

### Basic Adapter Ownership

- Spawn retains deterministic projectile UUID derivation, `ProjectileRecord`, launch snapshot, launch log and the
  first advance at `fired_at_ms` (`basic_attack_projectile.rs:40-100`).
- Advance still removes the record, clamps to `fired_at + max_travel_ms`, preserves its special zero-travel/stale
  behavior and reinserts stale work (`:102-121`).
- It validates only the stored target for active/enemy/air capability and samples only that body's start/end
  positions (`:123-179`). A bystander cannot become a kernel candidate.
- Homing uses the current endpoint body position; Fixed uses the stored launch aim. The adapter, not the kernel,
  selects that meaning (`:181-202`).
- On no collision it writes end position/aim/last time, owns deadline expiry/miss and queues the returned next time
  with the existing event/cause before reinsertion (`:218-244`).
- Hit/miss quantization and public log variants remain in the adapter. A hit revalidates active target and calls the
  Phase 28 committed-damage authority exactly once with projectile UUID and impact time (`:247-323`).

### Skill Adapter Ownership

- Top-level advance retains runtime removal, deadline clamp, duplicate/stale guard and guidance dispatch
  (`skill_runtime/projectile.rs:135-155`).
- Fixed advance still uses the established `last + 1` elapsed-window start, then applies acceptance policy after
  pure motion resolution. It owns max-kill truncation, hit list mutation, terminal flag, no-hit expiry, removal and
  event queue/reinsertion (`:157-233`).
- Homing still requires the stored target, applies live active/skill-delivery validation, samples its body twice,
  adds the authored projectile radius and owns miss/terminal/expiry/reinsertion events (`:235-389`).
- Fixed candidate construction continues to filter by caster owner, caster identity, authored hit target set and
  prior hits before body sampling. Only the resulting geometry enters the kernel (`:400-453`).
- `accept_fixed_skill_projectile_impacts` remains an adapter function. It owns prior-hit dedupe, max-hit and
  piercing short-circuit after the kernel's deterministic fact ordering (`:455-477`).
- Launch retains distinct Homing/Fixed `ActiveProjectileRuntime` construction and event identity. Dispatch retains
  TargetLocked target acquisition versus facing/range/lifetime-based DirectionalCollision aim (`:543-820`).
- Impact still updates cast context, records `SkillProjectileImpacted`, builds and executes effects, applies
  post-effect kill stop and calls `resolve_skill_step_delivery(terminal || kill_terminal)` once (`:822-908`).

### Timing, Pause, Expiry And Determinism

- The kernel accepts authoritative battle-time endpoints; it does not read wall time or playback state. Existing
  simulation Pause therefore still prevents scheduled work from advancing and creates no catch-up path.
- First reevaluation remains at launch. Subsequent live work remains exactly one millisecond after the clamped
  reevaluation endpoint. No independent timer, loop or wall-clock cadence was introduced.
- Directional board/range termination remains represented by the authored fixed aim and min(range/lifetime) travel
  time. The adapter still decides expiry and terminal miss at the same deadline.
- Same-time event priority and payload tie-breaking in `BattleEvent` are untouched. Kernel ordering only replaces
  the former local skill candidate sort with identical time/distance/UUID semantics.
- Source snapshot and per-hit identity never enter common math. Motion output is handed to the established basic or
  skill occurrence/effect owner, preventing cross-adapter identity collapse.

## Data Evidence

- Live Skill data contains multiple positive TargetLocked projectile speeds and loads through the unchanged skill
  schema. Its final SHA-256 is the activation value
  `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`.
- Live Equipment contains positive basic projectile speeds and uses the unchanged TargetLocked consumer contract;
  SHA-256 remains `322d7455a256082f03804c0dcc7a77a4caa3a06919d5e34b667042b411bcd1a6`.
- `DeliveryDef::try_validate_projectile_scalar_contract` still rejects zero speed/range/lifetime/kills/pierces/hits
  before publication. Basic consumer validation still rejects targetless, directional, stop-policy and nondefault
  collision authoring it cannot execute.
- The common math's established defensive zero-speed result is not reachable from successfully published live
  definitions and was not made into normalization, retry or silent recovery.
- No RON value, schema, loader, catalog, encounter or equipment definition changed. `ron_loading` passed 25 current
  production cases; live catalog and live item activation each passed 4.

## DTO, Transport And Persistence Evidence

- `ProjectileRecord` and `ActiveProjectileRuntime` remain distinct public non-serde Rust records with the same
  fields and visibility (`core/types.rs:56-101`). Their activation hash is unchanged at `ba33c56b...`.
- Internal `BattleEvent` advance/impact payloads, event time accessor, same-time priority and deterministic tie keys
  are unchanged. Public `BattleLogEvent` launch/impact variants retain every field and serde behavior.
- `behavior.rs` still mirrors Core's basic and skill projectile log variants field-for-field and maps them directly;
  there is no Phase 29 DTO adapter or recomputation.
- Projectile runtime values are battle-local and are not written to the run save. No checkpoint/version, replay,
  restore, reconnect, rollback, dual-read/write, alias or migration path changed.
- `game_server` forwards the unchanged Core events/checkpoints and passed 59 tests plus all-target checks in both
  profiles. Server Unity bridge canonical remains hash-identical.
- The new common types are private, non-serde and absent from public API, setup, checkpoint, save, wire and Unity
  contracts. No producer-consumer rollout coordination is needed.

## Consumer Evidence

- Confirmed workspace producers/consumers: basic attack release/spawn/advance/commit; skill cast delivery, Homing
  and DirectionalCollision advance, impact/effects/cast lifecycle; simulation event dispatcher; behavior event
  projection; live skill/equipment loaders; result/event validation; Boss released-leaf cancellation test.
- Confirmed server consumer: the current `game_server` Core dependency compiles and executes its 59-test suite in
  debug and release without a projectile mapping or protocol change.
- Canonical/accessed external consumer: accessible Unity canonical continues to consume published skill catalog and
  battle event/checkpoint contracts. The external file remains SHA-256 `a3224e24...`; no runtime shape it describes
  was edited.
- Inaccessible/unknown consumer: separately deployed Rust code may read the public projectile runtime records, and
  independently deployed Unity state may exist. It is not claimed as directly executed. CRA-POL-006 risk is closed
  for this Phase by retaining all public records/events/serialized fields and by making only private additions.
- Explicitly excluded consumer verification: live WebSocket and Unity Edit/PlayMode execution were not required for
  a private implementation refactor with exact unchanged producer/server/external hashes. This omission does not
  leave a changed wire behavior unverified.

## Legacy, Compatibility And Placeholder Audit

- Direct `moving_circle_sweep_hit_fraction` use exists only in `projectile_math.rs`; neither adapter contains a
  sweep, hit-fraction time calculation or interpolation helper.
- `PROJECTILE_REEVALUATION_TICK_MS` and `sample_projectile_position_at` each have one definition in common math.
  The old adapter-specific tick constants and duplicated impact-position helper names are absent.
- Both callers use `ProjectileMotionWindow`; there is no dormant old motion branch, feature flag, debug branch or
  fixture-only production special case.
- Target selection, accepted-hit policy, damage/effects and terminal lifecycle remain explicit adapter code rather
  than compatibility shims around the kernel.
- Search found no dual motion path, alias, normalization, ignored relevant test, silent recovery or compatibility
  layer in the changed implementation. The one `fallback` word is a comment explicitly denying publication
  fallback.
- The `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_30_p10_c01` directory does not exist.
  No Phase 30 command route, DTO/state placeholder or production seam was prebuilt.

## Scoped Diff And Dirty Preservation

- Phase 29 attribution is limited to `projectile_math.rs`, the motion portions of already-dirty
  `basic_attack_projectile.rs` and `skill_runtime/projectile.rs`, one characterization in already-dirty
  `commands.rs`, one stale bullet in `skill_target_contract.md` and this Child evidence.
- Final source hashes are common math `94494dc82ccfbade743cd08eeb44ab446175346a921832c99db8eec1edd88c14`,
  basic `62ce89f2d0d8200c82531a82a8f271035bfef96c52f2fefba0afc17b619e8ab1`, skill
  `8c1a2a84776a8f54efc496bba9e2153ca51aedfe6bb4ad2d929ae0d505f8d96e` and commands
  `e97e30edb75069f4f594e6bd6b9d292175d6f2cac6948bb2b8ec9c746155cae7`.
- Prior Phase 28 committed-damage hunks in basic, Phase 14 terminal hunks in skill and all unrelated commands work
  remain present. Current HEAD diffs in public DTO/types/data are extensive user work, but their prospective hashes
  prove Phase 29 did not alter them.
- Activation expanded state was 520 rows. The pre-audit 521-row state adds exactly the formerly clean tracked common
  math path; all other Phase targets already had status rows or are Goal/canonical evidence.
- No reset, restore, checkout, clean, stash, deletion or broad unrelated formatting was used. Scoped
  `git diff --check` and repository `cargo fmt --all -- --check` are green.
- Protected `game_server/logs/app.log.2026-08-09` remains exactly 57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with unchanged mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`. The prior one-path approval was not generalized.

## Canonical Alignment

- `skill_target_contract.md` now distinguishes source-bound active/follow runtime from detached released
  projectile and `GroundFixed` work. This corrects one internally contradictory stale sentence and matches actual
  runtime, the adjacent interrupt/release policy, Core canonical and rulebook.
- The same canonical continues to require positive projectile scalars, TargetLocked basic behavior,
  TargetLocked/DirectionalCollision skill behavior, miss representation, same-time terminal composition and
  Homing versus Fixed relocation semantics (`skill_target_contract.md:330-353`).
- `core_runtime_contract.md:186-205` and `game_rulebook.md:371-397` independently state that released basic/skill
  projectiles survive source withdrawal while an inactive target causes miss. Their hashes are activation-exact.
- Data-loading, server bridge and accessible Unity Core canonicals are unchanged because no data/wire/persistence or
  client-facing contract changed.
- Current runtime, current canonical and approved target policy therefore agree: observable behavior is unchanged,
  detached-delivery wording is consistent and private common ownership is reduced.

## Validation Results

All commands ran in the current extensively dirty worktree. Every counted suite reports zero failed and zero
ignored; zero-test binaries are excluded from evidence totals.

### Pre-Change Baseline And Characterization

- `(Core) cargo test --lib projectile --quiet`: 31 passed, 0 failed, 0 ignored before new characterization.
- `(Core) cargo test --lib game::battle::core::commands::tests --quiet`: 42 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test skill_runtime_contract --quiet`: 14 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test skill_test_suite --quiet`: 4 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test live_skill_catalog_audit --quiet`: 4 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test ron_loading --quiet`: 25 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test live_item_skill_activation --quiet`: 4 passed, 0 failed, 0 ignored.
- `(Core) cargo test --lib game::battle::core::boss_runtime::tests --quiet`: 22 passed, 0 failed, 0 ignored.
- New cross-adapter characterization selected exactly 1 test and passed all four internal rows on duplicated old
  production, 0 failed/ignored. The projectile inventory was then 32.

### Post-Implementation Focused

- `(Core) cargo test --lib projectile --quiet`: 33 passed, 0 failed, 0 ignored.
- `(Core) cargo test --lib game::battle::core::commands::tests --quiet`: 43 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test skill_runtime_contract --quiet`: 14 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test skill_test_suite --quiet`: 4 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test live_skill_catalog_audit --quiet`: 4 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test ron_loading --quiet`: 25 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test live_item_skill_activation --quiet`: 4 passed, 0 failed, 0 ignored.
- `(Core) cargo test --lib game::battle::core::boss_runtime::tests --quiet`: 22 passed, 0 failed, 0 ignored.
- Release reruns: projectile 33, commands 43, skill runtime 14 and Boss 22 passed, all 0 failed/ignored.

### Broad, Compile And Static

- `(Core) cargo test --all-targets --quiet`: 1,072 passed, 0 failed, 0 ignored. Calculation is 1,021 library plus
  official integration groups 4 + 4 + 25 + 14 + 4; zero-test binaries are excluded.
- `(Core) cargo test --release --all-targets --quiet`: the same 1,072 passed, 0 failed, 0 ignored.
- `(server) cargo test --all-targets --quiet`: 59 passed, 0 failed, 0 ignored; zero-test binaries are excluded.
- `(server) cargo test --release --all-targets --quiet`: the same 59 passed, 0 failed, 0 ignored.
- `(Core) cargo check --all-targets --quiet` and `cargo check --release --all-targets --quiet`: both exit 0.
- `(server) cargo check --all-targets --quiet` and release equivalent: both exit 0.
- Core and server `cargo clippy --all-targets --quiet` and release equivalents: all exit 0 with the established
  non-strict warning inventory. No warning points to common math or the new table; strict denial is not claimed.
- `(Core) cargo fmt --all -- --check`: exit 0. Scoped `(root) git diff --check`: exit 0.

### Fresh Audit-Time Revalidation

- `(Core) cargo test --lib projectile --quiet`: 33 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test skill_runtime_contract --quiet`: 14 passed, 0 failed, 0 ignored.
- `(Core) cargo test --lib game::battle::core::boss_runtime::tests --quiet`: 22 passed, 0 failed, 0 ignored.
- `(Core) cargo test --all-targets --quiet`: 1,072 passed, 0 failed, 0 ignored by the same calculation above.
- `(server) cargo test --all-targets --quiet`: 59 passed, 0 failed, 0 ignored.
- Fresh format, scoped diff, marker and expanded status/hash checks all passed after the source/canonical reread.

### Omitted Validation

- A live WebSocket probe was omitted because no Core event/checkpoint shape, behavior DTO, server mapping or
  ordering contract changed. Exact activation hashes plus server debug/release test/check gates cover the boundary.
- Unity Edit/PlayMode runners were omitted because no Unity or wire source changed. The accessible external
  canonical was reread and hash-identical; the Core producer and server forwarder are green.
- No performance benchmark is registered for this private consolidation. The number of sweeps remains one per
  candidate, the cadence remains 1ms and functional/broad suites are the applicable behavior gate.

## Failure Classification And Baseline Failures

- Baseline product failures: none. Every required pre-change nonzero suite passed.
- One initial command used the nonexistent target name `skill_contract`; Cargo supplied `skill_test_suite`, which
  then passed 4/4. This is command discovery, not a product failure.
- One focused command combined substring selection with `--exact` and selected zero; it was explicitly discarded
  and rerun with 1 selected test.
- After extraction, the migrated internal fractional-position test made an over-exact assumption about a synthetic
  zero-radius f32 quadratic and failed through two tighter assertions. The test was corrected to its durable
  contract: continuous impact is near 2.1 and differs from sampling at the ceiled 3ms event time.
- The observable cross-adapter table passed before and after extraction, and no broad/product test failed. The
  fractional assertion correction did not introduce a runtime tolerance or production branch.
- Non-strict Clippy warnings are diagnostic repository debt, not baseline/test failures, and strict warning denial
  was not a Phase completion condition.

## Cross-Component Findings

- Common math and adapters do not define competing sources of truth: the kernel returns geometry/time facts while
  each adapter remains the sole lifecycle/event/effect owner.
- Basic cannot acquire skill bystander or DirectionalCollision acceptance because it constructs a one-element
  locked-target candidate set after its own validity checks.
- Skill fixed collision cannot lose authored stop policy because filtering and acceptance remain outside the
  kernel, and post-effect max-kill terminal composition remains after command execution.
- Phase 28 committed damage receives the same projectile UUID/time/source snapshot once; Phase 14 receives the same
  impact terminal or post-effect kill terminal once. Neither predecessor seam is duplicated.
- Boss runtime contains no third motion implementation and continues to preserve an already-released ordinary leaf.
- Public events, behavior projection, live data, server and Unity have no temporary schema, compatibility shim,
  version conflict or hidden recomputation.
- Phase 30 receives no placeholder from this Phase. No material cross-component finding remains.

## Long-Term Direction Review

| Candidate ID | Work type | Current to target authority result | Preserved invariant result | Removal/consolidation result | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P09-C05 | behavior-preserving refactor | duplicated basic/skill endpoint, sweep, ceiling, interpolation, ordering and next-time mechanics become one private pure typed authority | timing/position, target-only basic, skill policy, expiry, effects, terminal, snapshots, Boss and public compatibility remain exact | adapter tick/sweep/interpolation/travel duplicates removed; distinct lifecycle records/adapters intentionally retained | pre-change parity 1; focused 33/43/14/22; broad Core 1,072 and server 59 debug/release | pass |

- Fit: High.
- Improvement class: none.
- Reason: ownership is simpler at the true common boundary; policy-bearing lifecycle remains explicit; private types
  align with the projectile math domain; tests protect observable parity and durable stop/terminal behavior; no
  parallel schema, compatibility shim, future placeholder or unnecessary public abstraction was introduced.

## Detailed Findings

- Material findings: none.
- Nonmaterial observation: public `ProjectileRecord` and `ActiveProjectileRuntime` may have unknown independently
  deployed Rust readers. They remain unchanged under CRA-POL-006, so this is a compatibility constraint rather than
  deferred correction.
- Nonmaterial observation: the established defensive zero-speed arithmetic remains for programmatic callers while
  publication is strict. It is unchanged behavior and is not exposed as live-data fallback.
- Nonmaterial observation: repository-wide non-strict Clippy debt remains outside Phase 29. It does not name the
  new common authority and all required compile/test gates pass.
- No policy, scope, external dependency, migration or correction decision is required.

## Primary Verdict

- Primary verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- Combined phase-exit verdict: `Complete / High`.

Every Phase 29 completion condition has current runtime/data/contract and nonzero validation evidence. No material
legacy, unauthorized compatibility, unverified required consumer, scope violation or Phase 30 prebuild is hidden
behind this verdict.

## Status Transition

- from: `active`, `Implementation Complete: false`.
- to: `complete`, `Implementation Complete: true`.
- Master Phase 29 registry: may transition to `complete — Complete / High` after this audit is recorded.
- Phase 30 readiness: allowed. It may become `ready` in the same record-only transition, but production inspection
  remains forbidden until its own Child initialization, required dependency/historical reads, sole activation and
  prospective fingerprint protocol are complete.

## Remaining Risk And Required Action

- Preserve public runtime/event shapes until separate consumer and migration evidence authorizes a change.
- Preserve strict positive projectile publication; do not reinterpret the defensive arithmetic as authored zero
  speed support.
- Preserve the 57-byte Phase 27 approval marker exactly; it is not a Phase 29 artifact or general cleanup waiver.
- Keep Phase 30 limited to CRA-P10-C01. It must not reopen projectile cadence, collision, lifecycle or public shape
  without a new explicit stop-condition decision.
- Immediate correction: none.
