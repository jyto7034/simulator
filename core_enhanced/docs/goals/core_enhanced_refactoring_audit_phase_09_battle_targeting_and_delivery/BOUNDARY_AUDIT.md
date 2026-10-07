# Core Enhanced Refactoring Audit Phase 09 — Boundary Audit

Audit Date: 2026-08-07
Status: complete; phase-exit audit `Complete / High`

## Exact Boundary

Phase 9 owns target eligibility and deterministic choice, cast target capture/retarget, delivery geometry and released
projectile/area cleanup. Phase 7 supplies authoritative body/tile/facing state; Phase 8 receives committed hits and
owns damage/effect/death; Phase 10 owns public command, range/snapshot DTO and server projection. Published ability,
equipment, abnormality, corroded-employee and fragment data are inputs, not separate runtime authorities.

Primary code is `battle/core/{basic_attack,basic_attack_projectile,projectile_math,target_usefulness,targeting}.rs`,
all four `battle/core/skill_runtime/**` files and target/delivery slices of `core/{commands,mod,sim,types}.rs`.
`ability.rs`, `range_preview.rs`, `battle/tile_range.rs`, published validation, live RON, focused tests and
`docs/{skill_target_contract,core_runtime_contract,game_rulebook,data_loading_contract}.md` are direct carriers.

## Authority Matrix

| Domain | Current authority | Consumers | Finding |
| --- | --- | --- | --- |
| Basic eligibility | `TileRangePolicy` membership plus deterministic target score | windup, reacquisition, release | coherent; `range_units` is not read |
| Skill acceptance | typed `SkillTarget`/selection/range validation | manual and automatic cast | coherent capture; delivery-aware usefulness gap |
| Step targeting | captured target plus explicit retarget/repeat rules | instant/projectile/area steps | deterministic and one runtime authority |
| Basic projectile | target-locked continuous sweep | committed basic hit | coherent semantics; motion code duplicated |
| Skill projectile | target-locked homing or fixed directional collision | step delivery count/effects | `max_kills` stop leaks cast completion |
| Tile area | captured source/anchor and tracking mode | filtered effect application | coherent runtime; preview can project different cells |
| Usefulness | effect preview plus target-kind helpers | acquisition, autocast, step no-op gate | target-centered AoE checks anchor, not actual area |
| Range preview | materialized cast reach and self-target helper | checkpoint/UI transport | cast reach is reused as effect footprint |
| Publication | skill/basic/weapon validation | runtime constructors/live bundle | zero speed and skill-only basic policy fields escape |
| Spatial projection | four policy/pattern/facing adapters | membership, manual cast, preview, area | behavior agrees now; ownership duplicated |

## Basic Attack Trace

Basic target discovery filters active hostile bodies through `TileRangePolicy`, then resolves deterministic score and
identity ties. Windup stores the target, release revalidates eligibility, and failed revalidation clears or
reacquires through the same selector. Instant delivery commits immediately. Projectile delivery snapshots source
position and selected target identity; every battle millisecond it homes toward the live target, sweeps only that
target continuously, resolves fractional hit time with ceiling and does not reapply release range at impact. This
matches current tests and canonical launch/live-target behavior.

`BasicAttackDef.range_units` and `WeaponCombatProfile.range_units` are authored, copied and checked as positive but
have no production eligibility reader. `bodies_in_range` likewise has no production caller. The true authority is
tile range policy. Public `BattleCore::basic_attack_delivery(unit_base_uuid)` has no workspace caller and consults
base abnormality data, so it can disagree with an equipped runtime weapon profile. These are compatibility-gated
cleanup signals, not permission to alter serialized data in this audit.

## Skill Target And Step Trace

Manual validation records the selected unit/tile; automatic acquisition uses explicit skill selection and range
rules. Cast start creates one `ActiveSkillCast`, steps resolve in authored order, repeats preserve identity and
explicit retargeting asks the same deterministic selector again. Missing/invalid targets produce typed rejection or
step no-op without inventing a fallback target. Current EnemySingle, SelfUnit, global, reused-target and selection-
independence tests substantively observe identity, state or effect rather than only success.

Usefulness is not fully delivery-aware. The helper chooses the first target-bearing hostile step, and EnemySingle
TileArea acquisition evaluates the anchor unit before the runtime materializes footprint/filter tiles. An authored
footprint that excludes the anchor can therefore select a useful anchor and affect nobody, or reject an anchor even
when its actual area contains a useful enemy. Current live EnemySingle areas include their anchor, so this is a
schema-valid hardening correction rather than a demonstrated live balance incident.

## Projectile And Area Lifecycle

Basic and skill projectiles independently implement millisecond reevaluation, homing movement, continuous segment
sweep, fractional impact placement and next-event/expiry scheduling. Their IDs, payloads and hit policies correctly
differ; the shared geometry/time kernel does not. Phase 8 remains owner of the committed damage/effect result.

Directional skill projectiles queue a nonterminal impact before applying the kill-stop policy. When that impact
reaches `max_kills`, the policy removes the projectile but does not report terminal delivery or decrement the cast's
pending delivery count. The impact is then resolved with its stale `terminal = false`, leaving the active cast
permanently pending. The existing max-kills test proves only first-target damage and second-target exclusion; it never
asserts cast cleanup. No current live skill uses this option, but it is a published/tested runtime contract.

Tile areas capture geometry and source. `GroundFixed` preserves the captured anchor after release; `FollowCaster`
and `FollowTarget` re-anchor while their owner exists and end when that owner leaves. Target filter/order and
per-target deterministic identities are explicit. Withdrawal, relocation and Hard CC tests confirm released-effect
survival and tracking behavior.

## Validation And Projection Drift

Projectile speed is a plain `u32`. Neither skill validation nor basic/weapon validation rejects zero; the defensive
flight helper returns zero duration, effectively converting invalid projectile timing into immediate delivery.
Programmatic `SkillDatabase::new` can also accept `Some(0)` policy limits that RON's deserializer happens to reject.
Basic attack validation rejects TileArea and DirectionalCollision, but accepts and ignores target-locked fields that
are meaningful only for skills; weapon profile validation is weaker and allows DirectionalCollision to be copied
into a basic projectile that still runs as homing. One domain-specific publication matrix should close both RON and
programmatic paths.

`self_target_skill_preview` derives `effect_preview_cells` from cast acquisition reach rather than the step
`TileArea.footprint`. Published examples cast over a front 3x2 reach but apply 3x3 or 5x5 footprints. These skills are
currently assigned to non-player units and do not create a current player checkpoint incident, yet the projection
does not mean what its field name/canonical promises. Multi-step skills need an explicit per-step or union policy
before implementation.

Runtime membership, manual cast acceptance, range materialization and area materialization each adapt
`TileRangePolicy`/pattern/facing/valid-tile rules separately. Their inspected behavior currently agrees and obstacle
filtering is intentionally absent. A typed spatial query/projection primitive can reduce drift while preserving the
important distinction between membership and materialized cells.

## Live, Canonical And Consumer Classification

- All live projectile speeds and optional projectile limits are positive. Current EnemySingle TileArea footprints
  include their anchor; current self-target spatial skills use one consistent footprint within each skill.
- `skill_target_contract.md` says caster withdrawal removes every released skill projectile/area. Production,
  focused tests, `game_rulebook.md` and `core_runtime_contract.md` instead preserve released projectiles and
  GroundFixed areas while ending Follow modes. The target contract sentence is stale canonical, not a production
  refactor candidate.
- Pause is inherited from Phase 6 battle-time scheduling: target/delivery runtimes have no wall-clock catch-up path.
- Public Rust methods/records and serialized `range_units` may have external consumers. Phase 10 must inspect direct
  server/DTO usage; Unity remains unverified and out of scope.
- No production ignored marker was found in the assigned target/delivery tests.

## Boundary Verdict

One confirmed lifecycle correction closes the directional-projectile cast leak. Publication validation, actual-area
usefulness and actual-footprint preview are correctness candidates with live/non-live scope stated precisely. Two
behavior-preserving candidates consolidate projectile motion and tile projection without merging Phase 8 outcomes or
Phase 7 position. Ghost range/public surfaces are cleanup-gated, and false-positive negative RON tests require a
test-correction candidate. Coherent selection, target capture, area tracking and released-effect semantics are
retained. No production file was changed.
