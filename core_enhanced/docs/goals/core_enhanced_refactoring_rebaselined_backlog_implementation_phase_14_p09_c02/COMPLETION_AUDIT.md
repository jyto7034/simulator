# Phase 14 CRA-P09-C02 Completion Audit

Audit Mode: phase_exit
Audit Target: Master Phase 14, `CRA-P09-C02 — Resolve Max-Kills Projectile Delivery Exactly Once`
Audit Date: 2026-08-10 (`Asia/Seoul`)
Current Status At Audit Start: active
Implementation Complete Claim At Audit Start: false
Policy Source: no candidate-specific gameplay decision; Master compatibility policy `CRA-POL-006`
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Repository: `/mnt/f/work/simulator`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`
Dirty State At Audit Start: 439 expanded entries; normalized SHA-256
`53ed00c3f2db0e5b0b65a5926004032a7ebe7c84a02441b6f9931db71097f8d8`
Review Guide: all 567 lines of `core_enhanced/docs/goal_completion_review_guide.md` reread before this audit

## Audit Scope And Authority

The approved correction restores one existing lifecycle. A released directional projectile increments its owning
step's pending delivery count once. An accepted impact that reaches `max_kills` must remove the projectile and feed a
terminal fact to the existing cast delivery resolver so that pending count, deferred steps and cast cleanup remain
one authority. Collision geometry, damage/effect meaning, authored kill semantics, event ordering, RON/public/save/
wire shape and future publication/damage/motion work are preserved.

Master Phase 13 is the direct linear predecessor and is reviewed `Complete / High`. Master Phase 15
`CRA-P09-C01` remains pending and cannot activate until this audit passes. Phase 29 `CRA-P09-C05` may later consume
the frozen terminal-delivery seam only after its other publication and committed-hit prerequisites.

## Completion Evidence Matrix

| ID | 완료 조건 또는 불변식 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P14-C01 | cast→projectile→impact→stop→delivery graph identified | Candidate C02 target authority | `sim.rs` launch count; `projectile.rs` advance/impact/typed outcome; `cast.rs` pending/cleanup | E02 current-tree inventory and audit reread | pass |
| P14-C02 | existing dirty work and scoped ownership preserved | preserved invariant 10 | 439-entry status/hash exact; scoped fingerprints; no reset/cleanup | fmt/diff checks and E01/E04 hashes | pass |
| P14-C03 | pending-cast defect reproduced before correction | test-first requirement | former side-effect-only helper removed projectile but returned no terminal fact | exact red: 0/1, cast cleanup assertion failed | pass |
| P14-C04 | same-time stop reasons compose deterministically | preserved invariant 4 | queued motion bit OR private post-effect stop outcome at one resolver | combined two-candidate max-kills/max-hits/expiry 1/1 | pass |
| P14-C05 | projectile and cast retire exactly once | objective and invariants 3/5 | one kill-stop producer/caller and one impact resolver call | cumulative max-kills 1/1; integration 14/14 | pass |
| P14-C06 | collision/effect/repeat/released-work behavior remains exact | invariants 1/2/6/7 | hit ordering and command path unchanged; existing released runtime remains detached | projectile 6/6; released 10/10; skill suites green | pass |
| P14-C07 | focused, broad, release and server validation pass | Child validation plan | current dirty Core/server state | Core 974; integration debug/release 14 each; server 59 | pass |
| P14-C08 | canonical, compatibility and downstream seams are current | CRA-POL-006 | nearest lifecycle canonical updated; public/data/save/wire types unchanged | RON 25; catalog 4; all-target checks; static inventory | pass |
| P14-C09 | independent exit review returns Complete / High | completion review guide | this current-tree audit | all required sections and no material finding | pass |

## Runtime And State Evidence

### Current Producer And Mutation Path

1. Skill-step execution dispatches every projectile repeat and passes `launched_count` to
   `begin_skill_step_delivery`, which increments the owning `SkillStepProgress.pending_delivery_count`.
2. Fixed projectile advancement removes the runtime during evaluation, orders collision candidates by hit time,
   distance and stable unit identity, queues accepted impacts, and reinserts only work that remains nonterminal under
   motion/collision policies.
3. Impact handling records the existing `SkillProjectileImpacted` event before effects, builds commands from the
   launch `DamageSourceSnapshot`, and consumes the authoritative `CommandExecutionSummary.killed_target_count`.
4. Private `SkillProjectileStopOutcome::{Continue, Terminal}` is returned from cumulative kill-stop evaluation.
   `Terminal` is produced only when the impact raises `killed_unit_count` to the authored limit and removes that
   delivery's active projectile.
5. The handler has one `resolve_skill_step_delivery` call. It combines the queued motion/collision bit with the typed
   post-effect outcome. The existing resolver alone records step results, decrements pending count, releases deferred
   steps and removes a finished active cast.

The former defect was real: the helper removed a `max_kills=1` runtime but returned nothing, and the original queued
`terminal=false` reached the cast resolver. The strengthened integration first failed only the active-cast cleanup
assertion while active projectile absence already passed. This separates the defect from collision exclusion.

### Exactly-Once And Nonterminal Boundaries

- The cumulative test authors `max_kills=2`. Its first lethal accepted impact remains nonterminal; its second lethal
  impact produces the terminal outcome. Two ordered targets receive damage, then the projectile count is zero and
  the first cast debug state is absent.
- The combined test presents two collision candidates at the same world position and authors `max_kills=1`,
  `max_hits=1` and one-millisecond expiry. Stable identity selects one target, one impact occurs exactly at the
  expected end time, one target receives damage, and both runtime owners are absent.
- Non-kill/no-command outcomes return `Continue`; pre-existing hit-limit, non-piercing, expiry and no-hit terminal
  paths retain their queued terminal bit. OR composition cannot create multiple resolver calls.
- The impact event and effect ordering, event cause/identity, source snapshot, repeat/retarget behavior and later
  target exclusion were not moved or reimplemented.

### Released Work, Cancellation And Boss Boundary

The correction is after accepted impact damage and does not change release/cancellation ownership. Core filters ran
withdrawal and Hard-CC released skill projectile preservation plus Boss cancellation of unreleased work while
preserving a released projectile: 10/10 passed. No Boss-specific max-kills branch or fallback exists.

## Data Evidence

`DeliveryDef::Projectile.max_kills` remains the existing optional programmatic/RON field and skill validation remains
unchanged. Production `game_resources/data/skills/base.ron` has no authored `max_kills`; it retained SHA-256
`4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`. This does not make the correction
inapplicable: the field is an already published constructible runtime contract and the integration exercises the
actual loader/database/runtime types. No live content rewrite or migration was approved or performed.

Complete RON loading (25/25), live skill catalog (4/4), live skill behavior suite (4/4), live item activation (4/4)
and the Core embedded/live-data unit coverage within 974/974 all passed. No invalid-data normalization or fixture-only
production special case was added.

## DTO, Transport And Persistence Evidence

- `BattleEvent::SkillProjectileImpact`, `BattleLogEvent::SkillProjectileImpacted`, behavior DTO conversion and server
  transport shapes are unchanged. Current reference hashes are event log
  `397e2380a21c79dc3addf10177c8ab0a7d85c25f80eba808da9894b09ef36e02` and behavior conversion
  `65349de8daf5230cd00041b9f3da4b66e76239245e608b98a2623312b2276139`; neither has a Phase 14 hunk.
- Append-only `SkillProjectileImpacted.terminal` remains the pre-effect motion/collision fact determined when the
  event is queued. The new private post-effect kill-stop result is consumed by cast lifecycle only; no past log event
  is rewritten and impact-before-effect order remains exact.
- `DeliveryDef`, public projectile/cast debug types, V3 checkpoint/save schema, event-log version, request/revision
  topology and server mappings did not change. Therefore no version bump, dual reader/writer or rollout interval is
  applicable.
- Debug and release integration suites plus Core/server all-target checks compiled the actual producer and transport
  types from one repository state.

## Consumer Evidence

- Confirmed workspace consumers: Core skill simulation, behavior DTO conversion, `skill_runtime_contract`, live skill
  suites, live RON loader and `game_server`; all current validations pass.
- Canonical external consumer: Unity consumes the existing skill projectile impact event/no-hit representation. The
  event type, fields, casing, cause and ordering were not changed by this correction.
- Inaccessible/unknown consumer: no separate Unity source repository was available in this workspace. Because no
  public/serialized value contract or shape was migrated, external source execution is not a Phase completion gate;
  this is recorded as an access boundary, not as verified consumer behavior.
- Explicitly excluded consumers: live WebSocket probe and Unity build/runtime, unless a serialized/message-order
  change had been found. None was found.

## Legacy And Compatibility Audit

Current search finds one private stop-outcome enum, one kill-stop producer and one producer caller. The impact handler
contains one cast delivery resolution. The old side-effect-only/stale-handoff shape no longer exists. No alternate
pending counter, second cleanup authority, hidden side table, debug bypass or compatibility alias was introduced.

No public/RON/save/wire legacy type was removed, so deprecation and version migration are not applicable. Existing
serde defaults and validation remain untouched under CRA-POL-006. No ignored test, duplicate schema, dual-read/
dual-write path, future Phase placeholder or silent fallback was added.

## Scoped Diff And Dirty Preservation

- E01 captured 439 expanded entries at normalized SHA-256
  `53ed00c3f2db0e5b0b65a5926004032a7ebe7c84a02441b6f9931db71097f8d8`. Immediately before this audit file,
  the count and hash are exact.
- Projectile runtime changed from E01
  `6de51ab750f571e8c9fecb921d770bb9a39cd31b98e9fe2e57dd149795f463a3` to
  `a100b805f01991aa392dddc3c1d4135f32b64d3bf414bfeae88118e1e834df5b` through the private outcome and one
  resolver composition. Its pre-existing Phase 12 modifier-source hunk remains intact.
- Integration changed from
  `773d5fdcdfdb90f58820c3b21ee441bb87bb16e8499a3bf2c2ad1b11466d4cfd` to
  `afd09b3f6392e8c0175a10d5be3781638e6221c956a55e8c4d129f80c1b49c58` through lifecycle assertions and the
  combined boundary. Its pre-existing Boss/selection/database-builder hunks remain intact.
- Target canonical changed from
  `81fa6a1e687a2eef98a0b6843fd00eae0a678cc77b93a97a9b9ef9af07abc135` to
  `631a36443745301231af6e93899b1e34a07846d7156dacf54fc3148b633d6e43` only for the current lifecycle text;
  existing campaign additions remain intact.
- Cast runtime, ability shape, core runtime canonical, rulebook and live Skill RON retain their E01 hashes.
- `cargo fmt --all -- --check` and `git diff --check` pass. No reset, restore, clean, stash, unrelated deletion,
  broad formatting or workspace cleanup was used.
- Audit-artifact-inclusive status is 440 expanded entries with normalized SHA-256
  `9332e4f77d62af8d674b215787afa7313c39101c0603e1887d0b38d610e627f5`; the sole added path-level entry is this
  required `COMPLETION_AUDIT.md`.

## Canonical Alignment

`docs/skill_target_contract.md` is the nearest targeting/delivery authority. It now states that accepted directional
impact effects precede kill-stop evaluation; reaching cumulative max-kills removes the active projectile and
terminally resolves the owning delivery once; same-time hit-limit/expiry compose into one lifecycle outcome; and
later candidates receive no impact/effect. It also explicitly distinguishes append-only event terminal metadata
from the private post-effect cast cleanup outcome.

The canonical describes current implemented behavior only. It does not claim live max-kills authoring, a public
event/save/wire migration, Phase 15 publication validation, Phase 28 committed-damage consolidation or Phase 29
motion extraction. No conflicting current canonical was found.

## Validation Results

All commands ran against the current dirty workspace. Audit-current reruns occurred after the evidence refinement;
production remained unchanged after broad validation.

- Pre-correction existing exact test: 1 passed, 0 failed, 12 filtered out.
- Pre-correction strengthened exact red: 0 passed, 1 failed, 12 filtered out; active cast remained.
- Pre-correction combined-stop characterization: 1 passed, 0 failed, 13 filtered out.
- Audit-current cumulative max-kills exact: 1 passed, 0 failed, 13 filtered out.
- Audit-current combined-stop exact: 1 passed, 0 failed, 13 filtered out.
- Audit-current `cargo test -p game_core --test skill_runtime_contract`: 14 passed, 0 failed, 0 ignored.
- Audit-current `cargo test -p game_core --release --test skill_runtime_contract`: 14 passed, 0 failed, 0 ignored.
- Focused Core lib `skill_projectile`: 6 passed, 0 failed, 968 filtered out.
- Focused Core lib `released`: 10 passed, 0 failed, 964 filtered out.
- `cargo test -p game_core --lib`: 974 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --test live_skill_catalog_audit`: 4 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --test skill_test_suite`: 4 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --test ron_loading`: 25 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --test live_item_skill_activation`: 4 passed, 0 failed, 0 ignored.
- `cargo test -p game_server --all-targets --quiet`: main target 59 passed, 0 failed, 0 ignored; four binary targets
  ran zero tests and passed, and are not counted as behavioral evidence.
- `cargo check -p game_core --all-targets` and `cargo check -p game_server --all-targets`: passed.
- `cargo check -p game_core --release` and `cargo check -p game_server --release`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.

No required validation was omitted. Live WebSocket and Unity runtime/build were correctly not run because public
serialization/message ordering did not change.

## Baseline Failures

The intended pre-correction red is the only product-behavior failure and is resolved. During audit evidence
strengthening, an initial three-opponent cumulative fixture failed one final global projectile-count assertion: a
third surviving opponent allowed a later autocast projectile unrelated to the first delivery. The two first-cast
impact assertions already passed. The fixture was narrowed to two opponents so battle completion makes the global
count refer to the audited delivery; the separate same-time test retains later-candidate exclusion. Both exact and
complete debug/release suites then passed. This was a test-observation scope correction, not an ambient baseline or
production failure.

## Cross-Component Findings

No material cross-component finding remains. Core and server compile and execute against the same unchanged public
types. Live RON still omits max-kills without contradicting the published optional field. Phase 15 may validate
projectile/basic publication rules without reopening terminal cleanup. Phase 29 receives one private, explicit
terminal-delivery seam and no temporary shim or motion placeholder.

## Long-Term Direction Review

| Candidate ID | 작업 유형 | 현재→목표 authority 결과 | 보존 불변식 결과 | 제거·통합 결과 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P09-C02 | correctness correction | side-effect-only projectile retirement → typed outcome consumed by existing cast owner | ordering, snapshots, exclusion, repeat/released behavior and shapes preserved | stale handoff removed; one resolver retained | exact red/greens, runtime 14 debug/release, Core 974, server 59 | pass |

- Fit: High
- Improvement class: none
- Reason: the correction reduces split lifecycle ownership without adding state, schema, compatibility or a parallel
  completion path. The private type sits at the domain seam that produces the fact, and the existing cast resolver
  remains the single mutation owner. Real integrations protect cumulative and combined behavior rather than helper
  layout. No future-Phase work was preimplemented.

## Primary Verdict

- Complete
- Long-term fit: High
- Material findings: none
- Policy questions: none
- Immediate correction required: none

## Status Transition

- from: `active`, `Implementation Complete: false`
- to: `complete`, `Implementation Complete: true`
- Master Phase 14 registry: may transition to `complete — Complete / High`
- next Phase activation: allowed only after the registry transition; Phase 15 may then become the sole active Child

## Remaining Risk And Required Action

No risk blocks completion. Max-kills is not presently authored by live RON, so production content does not yet
exercise it outside programmatic contracts; this is expected and does not weaken the published runtime correctness
proof. External Unity source was unavailable, but no external shape/order migration occurred. Preserve this terminal
delivery boundary when Phase 29 later extracts motion. Required action is only the linear Master transition to Phase
15; no follow-up correction or policy decision is required.
