# DefenseRoute Sticky Block Engagement Notes

## Policy Judgment

Follow Arknights-style blocking.

Blocking is not a momentary proximity check. It is a persistent engagement created when a valid blocker catches a valid blockable enemy. The blocked enemy remains held until a release condition occurs.

This is the long-term model because it matches the player's expectation for DefenseRoute:

- A deployed ground unit with block capacity holds enemies.
- Capacity determines how many enemies can be held.
- Extra enemies can pass.
- Held enemies do not gradually leak through because priority order changed on a later tick.

## Why Per-Tick Recalculation Is Wrong

The current `refresh_block_state()` implementation rebuilds `BlockRuntimeState` every movement tick. That makes block assignment sensitive to tiny position/progress changes.

In the abnormal record, two enemies enter one blocker's radius. Since priority is based on route progress and recomputed every tick, a capacity-1 blocker can effectively alternate which enemy is considered blocked. The enemy not selected for that tick can move forward, and later priority can change again. Over several ticks, both enemies advance through the blocker.

That is not acceptable DefenseRoute behavior.

## Current Data Judgment

No policy blocker was found for the two compared records:

- Both deployed player units are ground units.
- The default employee profile has block capacity.
- Corroded guard enemies are ground and blockable.
- Platform-only units already have runtime block capacity forced to zero.

Therefore the goal should not begin by changing RON data, deployment affinity, unit stats, or Unity rendering.

## Release Conditions

Initial implementation should release block engagements only for explicit state changes:

- blocker dead
- enemy dead
- blocker removed/withdrawn
- blocker no longer valid
- enemy no longer valid/blockable/ground
- forced displacement / teleport / knockback / special mechanic explicitly releases block
- battle reset/end

Normal movement distance must not release block. A blocked enemy should not drift out of engagement through ordinary route movement because the block itself movement-locks it. If a future forced movement effect places the pair outside the held engagement, the forced movement system must release the block explicitly rather than relying on a generic radius check.

## Timeline / DTO Considerations

No Unity-facing DTO change is required for the first implementation.

The existing observable contract should improve indirectly:

- Blocked enemies stop receiving route `MovementSegmentStarted` events.
- `MovementStopped` may appear when an active movement segment is stopped by block engagement if existing movement stop semantics support it.
- Checkpoint unit positions should reflect the held position.

If Unity later needs block VFX, targeting lines, or block counters, add an explicit presentation field in a separate goal.

## Test Philosophy

Do not test private implementation shape first. Pin user-visible battle behavior:

- Movement timeline does not pass a blocked enemy through the blocker.
- Block capacity overflow still passes.
- Death/withdraw releases capacity.
- Airborne bypass remains.
- Platform-only block prevention remains.

Internal helper tests are acceptable only to make the persistent engagement update rules easy to reason about.

## Follow-Up Candidates

These are not part of the first goal unless implementation proves they are required:

- Add `BlockStarted` / `BlockEnded` timeline events for better Unity VFX and debug visibility.
- Expose block engagement state in checkpoint for HUD/debug overlays.
- Add equipment/armor modifiers that increase block capacity.
- Add boss/elite mechanics that ignore or break block.
- Add block weight if some enemies should consume more than one capacity.
