# Phase 09 Candidate Reaudit

## Current Dispositions

| Candidate | Current evidence and Boss impact | Disposition |
| --- | --- | --- |
| CRA-P09-C01 | skill validation still omits projectile speed and positive optional-limit checks; basic/weapon validation rejects TileArea/DirectionalCollision but still accepts skill-only target-locked fields it ignores | `still_valid` |
| CRA-P09-C02 | max-kills impact removes the active projectile after damage while the queued impact retains `terminal=false`, so cast delivery count/cleanup can remain pending | `still_valid` |
| CRA-P09-C03 | EnemySingle TileArea acquisition/usefulness still evaluates the anchor unit rather than the actual clipped/filter-applied delivery set; multi-step policy remains open | `still_valid` |
| CRA-P09-C04 | self-target preview still publishes cast-range cells as `effect_preview_cells` without inspecting delivery footprints | `still_valid` |
| CRA-P09-C05 | basic and skill projectile adapters still duplicate 1ms reevaluation, homing/sweep/fractional impact and expiry scheduling | `still_valid` |
| CRA-P09-C06 | targeting, cast validation, preview, usefulness and area runtime still repeat tile policy/pattern/facing/materialization adapters | `still_valid` |
| CRA-P09-C07 | serialized/copied `range_units`, public `bodies_in_range` and public base-UUID `basic_attack_delivery` remain outside production eligibility authority | `still_valid` |
| CRA-P09-C08 | inline-range/preset conflict fixture still omits required `delay_ms`; current run panics on that missing field and catch-any-panic reports success | `still_valid` |

## Candidate Evidence And Readiness

### C01/C02 — publication and terminal lifecycle corrections remain open

`validate_skill_contracts` validates collision and policy combinations but does not inspect
`speed_units_per_ms`, and optional numeric stop fields remain plain positive-unconstrained options for programmatic
construction. Basic definitions reject only TileArea and DirectionalCollision, not ignored targetless/lifetime/
kill/pierce fields. C01 remains a compatibility-significant `correctness_correction`, test first.

For C02, fixed projectile advancement queues an impact based on collision/expiry terminal state. The impact handler
then calls `apply_skill_projectile_kill_stop_policy`, which can remove the active projectile, but passes the original
`terminal` flag to `resolve_skill_step_delivery`. The current max-kills integration proves no further hit, not active
cast cleanup. C02 remains policy-free and test-first.

### C03/C04 — actual delivery set and projection are not one authority

Current automatic usefulness does materialize the actual footprint for SelfUnit/CastTarget TileArea steps, but the
candidate-defining EnemySingle path still tests `is_skill_step_useful_hostile_target` on the candidate anchor. Cast
selection also passes the step into anchor-oriented usefulness. This sibling improvement does not close C03's exact
delivery-target requirement or its multi-step policy gate.

`self_target_skill_preview` materializes cast targeting policy/pattern and writes those cells to
`effect_preview_cells`; it never reads step delivery/footprint. C04 therefore remains dependent on C06 and a P10
per-step/union DTO projection decision.

### C05/C06 — both remain behavior-preserving refactors

Basic and skill projectile files each define a 1ms reevaluation constant and impact-fraction helper, sample moving
bodies, perform continuous sweep, ceil hit time, update position and schedule expiry/next events. Their lifecycle,
IDs, target/collision policies and payloads are legitimately separate; the shared target remains only a pure motion
query. C05's current work type is `behavior_preserving_refactor`, not cleanup. Boss uses released basic projectile
behavior in one cancellation test but does not add a projectile implementation.

`TileRangePattern` owns rotation/materialization, yet preview adds valid-board clipping, targeting calls membership,
skill sim and area/usefulness materialize/filter independently. C06 remains an independent
`behavior_preserving_refactor`; Boss navigation/path-cost is not this tile-reach query.

### C07/C08 — public cleanup and safety-net correction remain open

Workspace/server search found no production call to `basic_attack_delivery(base_uuid)` or `bodies_in_range`; current
eligibility tests explicitly prove tile range wins over `range_units`. Serialized/public external consumers remain
unverified, so C07 stays compatibility-gated cleanup rather than rejected.

The C08 source and test run directly confirm the false-positive: the conflict fixture reaches
`missing field delay_ms`, while `catch_unwind(...).is_err()` accepts that unrelated failure. Other bare `is_err`
fixtures also remain. This is an implementation-ready `correctness_correction (test)`.

## Boss Reconciliation

Boss target lock/path selection remains private Boss authority and ordinary skill usefulness does not select Boss
targets. Boss cancellation invalidates unreleased work but preserves released projectile/effect leaves. Accordingly
C03/C06 remain independent, C05 remains deferred/conditional until after the proven vertical slice, and the other
P09 rows retain their ordinary owners. No distinct `CRR-P09-*` candidate was found.
