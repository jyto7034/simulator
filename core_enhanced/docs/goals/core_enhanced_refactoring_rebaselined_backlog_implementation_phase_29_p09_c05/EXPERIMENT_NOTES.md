# Phase 29 CRA-P09-C05 Experiment Notes

Status: complete
Implementation Complete: true
Candidate: CRA-P09-C05
Master Phase: 29 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Locked Decisions

- This is a behavior-preserving motion-authority consolidation, not targeting, collision, projectile-policy or
  effect change.
- The common target is a private pure typed motion query. Basic and skill runtime records/adapters remain separate.
- Phase 14 owns skill terminal-delivery exactly once; Phase 15/CRA-POL-003 owns publication validity; Phase 28 owns
  committed damage/effects/HP/death.
- Basic remains target-only TargetLocked homing. Skill retains TargetLocked and DirectionalCollision, ordered
  candidates, hits/pierces/kills/expiry and cast-pending lifecycle.
- Exact reevaluation timing, hit-time ceiling, interpolation, quantization, board exit, Pause and event identity are
  preserved.
- Current public Rust, RON, save/checkpoint, DTO/wire/server/Unity shapes remain compatible under CRA-POL-006.
- No runtime-record merge, fallback, normalization, alias, dual motion path, test-only branch or Phase 30 prebuild.

## Open Evidence Slots

1. Activation status, HEAD and normalized dirty hash.
2. Prospective source/test/data/canonical hashes before content inspection.
3. Both runtime record, launch, event queue, advance, impact, expiry and consumer graphs.
4. Exact numeric type/conversion, sweep, hit fraction, ceiling and interpolation mechanics.
5. Common inputs/outputs versus adapter-specific filtering/lifecycle ownership.
6. Moving/relocated/dead/withdrawn target and source snapshot lifetimes.
7. Same-time board/expiry/collision and stable multi-candidate ordering.
8. Phase 14 terminal, Phase 28 committed-hit and Boss release/cancel seams.
9. Public Rust, serde, V3, RON, server/Unity and unknown deployment consumers.
10. Old-production baseline and new characterization counts.
11. Minimal extraction and duplicate/fallback/public-narrowing absence.
12. Focused/broad debug/release/static evidence and independent verdict.

## Historical Risk Register

- A shared query can accidentally make basic collide with non-target bodies or erase skill multi-body ordering.
- Unifying homing and fixed-direction input incorrectly can retarget DirectionalCollision or freeze a moving locked
  target.
- Moving ceiling/interpolation between integer and floating domains can shift impact event time or position by one
  millisecond/quantization unit.
- A query that owns event scheduling or terminal flags can duplicate or skip Phase 14 cast delivery resolution.
- Returning accepted hits rather than pure collision facts can absorb max-hit/kill/pierce and effect policy.
- Rebuilding a source snapshot in the kernel can violate Phase 28 launch-source/live-target and roll identity.
- Treating zero speed as runtime fallback can reopen CRA-POL-003 strict publication.
- Merging runtime records or narrowing visibility can break public/unknown Rust consumers under CRA-POL-006.
- Boss cancellation can lose already-released work if the shared seam is placed above the leaf runtime.

## Historical Evidence Summary

- Original P09 found duplicated 1ms reevaluation, homing movement, continuous sweep, fractional hit placement and
  expiry scheduling in two adapters, while affirming distinct identities, filters, stop policies and payloads.
- The post-Boss reaudit found the candidate unchanged. Boss has no third projectile implementation; it adds one
  released-leaf preservation consumer.
- Final order requires P09-C01 publication, P09-C02 terminal delivery and P08-C05 committed damage first. Phases 15,
  14 and 28 are now `Complete / High` and provide those exact seams.
- Required first proof is one common behavior table across zero/sub/exact-ms, moving target, multiple bodies, board
  exit, expiry and relocation while comparing observable events/effects rather than private records.
- Priority is `3 / 1 / 3 / 3 / 2 / 2 / unknown`, subtotal 14, Tier B.

## Activation Boundary

No new Phase 29 production source, tests, live data or canonical content was inspected during Child initialization.
After Master sole activation, reread this entire Child plus final Phase 28/14/15 and Boss completion audits, capture
prospective hashes using path discovery only, and only then begin current-source investigation.

## N01 — Activation Gate Closed

- The post-activation bounded reread is complete. HEAD and the 520-row normalized dirty baseline are recorded in
  E02 with source/test/data/canonical prospective fingerprints collected before current Phase 29 content inspection.
- Existing modifications on every prospective target are ownership boundaries. Before editing, inspect current
  source, the HEAD-to-worktree diff and both complete caller/consumer graphs; preserve unrelated hunks exactly.
- The 57-byte Phase 27 approval marker is byte- and metadata-identical. It is neither a Phase 29 artifact nor a
  general cleanup waiver.
- Phase 29 current-source investigation may now start. Production mutation remains gated on complete authority
  inventory, nonzero old-production baseline and characterization-first parity tests; Phase 30 remains forbidden.

## N02 — One Motion Fact Owner, Two Lifecycle Adapters

- Place the common implementation in the existing private Core `projectile_math` module. It may expose only
  `pub(in crate::game::battle::core)` typed facts needed by sibling adapters; it must not alter either public runtime
  carrier.
- The kernel owns path-window construction, collision fraction/time/positions, deterministic ordering and the next
  reevaluation timestamp. It must be pure: no `BattleCore`, unit lookup, event queue, log, damage, effect, cast,
  hit/kill/pierce or runtime-map mutation.
- Basic supplies exactly one already-validated locked-target candidate. Homing skill supplies exactly one
  skill-validated candidate and its projectile radius. Directional skill supplies its already-filtered candidate
  set, then applies seen-hit, max-hit, piercing, max-kill and terminal rules after receiving ordered collision facts.
- Preserve floating-point operation order where it is observable: speed conversion, maximum step, sweep inputs,
  ceiling, clamping, hit-fraction interpolation, squared-distance tie-break and UUID-byte order.
- Preserve the existing first-at-launch and subsequent `+1ms` scheduling semantics, deadline clamp and stale-event
  guards in the adapters. A returned next-time fact does not authorize the kernel to queue an event.
- Characterization will compare basic and homing-skill observable impact time/quantized position for zero,
  sub-millisecond, exact-millisecond and moving-target rows before production extraction. Existing directional
  multi-body, board/expiry, terminal, released-work and strict-publication suites supply the remaining matrix rows.
- Only the contradictory withdrawal sentence in the skill target contract may be aligned. No policy text, public
  event, RON/data value, server bridge or Unity contract shape is in implementation scope.

## N03 — Implemented Ownership Boundary

- Common authority is `projectile_math::ProjectileMotionWindow::resolve`; both adapters call it and neither contains
  a direct sweep, ceiling or hit-fraction interpolation implementation.
- Fixed skill candidate acceptance intentionally occurs after motion resolution. This preserves the distinction
  between a geometric collision fact and an authored accepted hit, including prior hits, piercing and max-hit state.
- Absolute kernel hit times are equivalent to the former fixed adapter's `spawn + elapsed` times; all candidates in
  a window share the same spawn origin, so time/distance/ID ordering is unchanged.
- The records, launch/impact events, quantization point, source snapshots, damage/effect handoffs and exactly-once
  terminal owner were not moved. Compatibility work is documentation alignment only.

## N04 — Validation Disposition Before Independent Audit

- Implementation evidence satisfies P29-C01 through P29-C10 provisionally: one pure private fact owner, two
  lifecycle adapters, test-first parity, nonzero focused/broad debug/release Core/server results, unchanged public
  and serialized consumers, and no fallback or duplicate motion path.
- The test-only fractional-position assertion now checks the externally meaningful distinction from integer event
  time instead of relying on exact floating equality from a synthetic zero-radius quadratic. This changes no product
  tolerance or runtime calculation.
- All Clippy findings are advisory established debt outside this behavior-preserving phase; changing them would
  expand scope and risk unrelated dirty work.
- P29-C11 remains open until a fresh completion-guide audit independently rereads the final implementation,
  evidence, diffs and downstream boundary and returns `Complete / High`. Phase 30 remains pending and untouched.

## N05 — Completion Disposition

- Fresh independent verdict: `Complete / High`; improvement class `none`; immediate correction `none`.
- P29-C01~C11 are closed. The private pure motion authority is the durable downstream seam, while all public and
  lifecycle compatibility constraints remain unchanged.
- Phase 30 may be marked `ready` but is not initialized or active. Its own activation protocol must run before any
  source/test/data/canonical detail inspection.

## Dirty Worktree Note

The repository is extensively dirty. Pre-initialization expanded state is 517 rows at normalized SHA-256
`9b5cf1cf...`. Every existing path remains user-owned. Preserve the current 57-byte one-path exception approval
marker (`614d2fbd...`) unchanged; Phase 27's approval is not a general deletion/cleanup waiver. Never use reset,
restore, clean, stash or unrelated formatting to manufacture a clean baseline.

## External And Compatibility Note

Workspace Core/server/data consumers and accessible Unity canonicals are available. Standalone/deployed Rust and
deployed Unity consumers of any public projectile runtime/helper surface remain unknown. Private consolidation is
allowed, but public removal/narrowing requires separate evidence and migration policy.
