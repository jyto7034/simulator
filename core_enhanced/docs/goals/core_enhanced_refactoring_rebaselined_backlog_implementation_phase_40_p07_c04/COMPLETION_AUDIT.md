# Phase 40 CRA-P07-C04 Completion Audit

Audit Mode: `phase_exit`
Audit Target: `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_40_p07_c04`
Audit Date: 2026-08-11 (`Asia/Seoul`)
Current Status: `active`
Implementation Complete Claim: `false`
Candidate / Work Type: CRA-P07-C04 / cleanup
Direct Policy: CRA-POL-006
Approval Source: user's 2026-08-11 approval for every remaining registered cleanup Phase
Finding Sources: original/post-Boss P07-C04, Phase-39 exit, final catalog/order/test evidence
Goal Documents: current Child `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`; parent Master `PLAN.md`
Repository / Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Audit Scope And Independence

This is the separate Phase-exit review required by the Child and Master. It treats implementation notes as pointers,
then independently reconstructs current movement input, body, output, visibility, event and consumer authority. It
checks every historical member rather than assuming the old report remains true after Phase 39, verifies transfer of
useful assertions before removal, and reviews Boss/world/range/event, public/serde, data/transport and dirty-tree
boundaries.

After the implementation record was written, the audit reran the complete Core debug all-target suite, repeated the
legacy/member and outside-module visibility searches, verified source fingerprints, format/diff syntax, expanded
status and the protected marker. Release all-target, workspace checks and normal Clippy remain current because no
production, test, manifest, data or dependency source changed after those successful commands; only Goal evidence
documents changed.

## Approved Condition Restatement

CRA-P07-C04 may remove current movement fields, variants, payloads and public shape only after a complete
producer/reader/serde/public-consumer inventory proves them ghost and retained production-path controls protect body
position, planner goals, terrain/locks/lifecycle, facing, active segments, event logs, Boss route behavior and
spatial/range consumers. The user's candidate-scoped approval authorizes a verified public/serde break without a
shim. It does not authorize a gameplay, event DTO, save, server/Unity, live-data, canonical or Phase-41+ change.

## Completion Evidence Matrix

| ID | Audit question | Independent evidence | Verdict |
| --- | --- | --- | --- |
| P40-C01 | Is the current member/public graph exact? | Current definitions, visibility, constructors, reads/writes/matches and repository consumers reconstructed | pass |
| P40-C02 | Is production authority singular? | planner/Boss goal → sorted input → private Rapier → three applied outputs → Core body/target/segment/event state | pass |
| P40-C03 | Is dirty work preserved? | activation/current hashes, 584-row pre-audit status and exact marker distinguish attributable changes | pass |
| P40-C04 | Is every candidate classified? | exhaustive member table below covers all historical fields plus exposed dependent payload/helpers | pass |
| P40-C05 | Was behavior characterized? | moving, locked/no-goal, target, obstacle, Boss, route, range, event and world controls are nonzero | pass |
| P40-C06 | Did useful assertions move first? | new default-Core retained-observable control passed with old members and after removal | pass |
| P40-C07 | Is public/serde removal authorized? | no accessible consumer plus explicit candidate-scoped user approval; package risk recorded | pass |
| P40-C08 | Was only ghost state removed? | removed-symbol search is zero; three real body facts, goals, segments/events and range remain | pass |
| P40-C09 | Is downstream behavior exact? | focused families and exact authored route green; both all-target profiles 1,057/1,057 | pass |
| P40-C10 | Is DTO/data/save/server/Unity boundary exact? | zero relevant consumer, unchanged manifests/live data/canonicals, workspace server compile green | pass |
| P40-C11 | Is broad/static validation green? | debug/release tests/checks, normal Clippy, format and diff checks accepted | pass |
| P40-C12 | Is the next boundary valid? | this separate audit returns `Complete / High`; Phase 41 remains absent/pending during review | pass |

## Current Authority Reconstruction

1. `BattleCore` derives current ordinary/Boss movement goals, sorts live unit IDs and projects a minimal input.
2. Each unit row contains only ID, copied `UnitBody { position, radius, move_speed }`, explicit current goal,
   terrain policy, current movement permission and lifecycle state.
3. The one private `RapierMovementWorld` consumes that input and returns only `BodyMoved`, `MovementStopped` or
   `TargetReached`. There is no alternate backend, selector or second stored goal/physics lifecycle.
4. Core applies those transitions to authoritative body position, opponent facing, current target and active
   movement segments. It continues to emit the retained serialized `BattleLogEvent::MovementSegmentStarted` from
   exact endpoints/timestamps.
5. Spatial, targeting, commands, range, world publication and Boss route logic consume retained body position,
   radius/speed or explicit goals. `range_units` and tile-range/`Nearest` policy were not removed or changed.

This is one authority path. The reduced tick vocabulary is implementation-private, while the actual public spatial
values and serialized battle-log contracts remain.

## Exhaustive Member Disposition Audit

| Historical/current member | Independent current finding | Result |
| --- | --- | --- |
| tick `now_ms` | had no backend reader; builder time still evaluates action/hard-CC locks | input field removed; semantic time retained |
| unit owner/current target | copied but never read by movement | removed |
| `previous_position` | writes only plus one obsolete shape assertion | field/writes/assertion removed |
| `velocity` | output calculation and writes fed no production reader | field/payload/helper/writes removed; endpoints retained |
| body `mode` / `MovementMode` | construction/relocation writes only | removed |
| body `goal` | overwritten on a copied body from the current goal map before its only read | transferred to explicit input goal; stored slot removed |
| physics handle/type | always absent with no consumer | removed |
| `GoalAssigned` | never produced; only a no-op apply arm | removed |
| internal `SegmentStarted`/reason | never produced; distinct from the live serialized event | removed; real event retained |
| stopped position/reason payload | produced but ignored by Core; distinct live log reason type survives | payload/type removed |
| engine/steering public modules and tick APIs | no consumer outside implementation/tests | made private/minimally scoped |
| public body apply clone surface | no external caller; existed to write ghost fields together | removed; Core writes position directly |
| `position`, `radius`, `move_speed` | live movement/spatial/range authority | retained public/serde |
| goal/vector/segment/event types | live planner, Boss, geometry and publication authority | retained |

Current Core/server source search selects zero files for every removed symbol/member/helper. A second search selects
zero files outside movement for internal tick/build/run/apply vocabulary. A server/world/data search selects zero
`UnitBody` files; actual battle-Core spatial/targeting/command/build users remain. There is no placeholder alias,
compatibility field, deprecated copy or dual schema.

## Behavior And Assertion Transfer

The new `battle_core_default_moving_locked_and_no_goal_observables_are_stable` control was added and passed while all
ghost members still existed. It asserts only retained semantics:

- exact `BodyMoved` unit/from/to and final world position for movement;
- exact serialized segment start/target/start/end timestamps;
- a stopped result, unchanged position and empty log for current movement-lock and no-goal cases.

The same exact full-path control passes 1/1 after cleanup. Rapier endpoint, collision, clamp, obstacle, void-tile,
overlap and determinism controls continue to assert actual production-default behavior. Tactical relocation asserts
the real new position, facing and retained state instead of deleted carrier shape. Boss and authored-route controls
continue through the ordinary movement leaf.

## Public, Serde And Compatibility Audit

`UnitBody` remains public and derives serde because its three spatial facts are real. Removing five false fields is
therefore a Rust/serde shape break in principle. No repository, server, tool, DTO, RON, JSON, save/checkpoint or
accessible Unity consumer serializes the body or internal tick model. The crate can be published, so search alone
would not authorize removal; the user's explicit remaining-phase approval supplies the required CRA-POL-006 breaking
authority after this inventory and transfer proof.

No migration alias, defaulted ghost field, dual read/write or deprecation copy is warranted. Actual event-log DTOs,
world/public range results and live content shapes did not change.

## Data, DTO, Transport And Persistence Evidence

- Root workspace manifest, lockfile, Core/server manifests and Core lib root retain activation SHA-256 values
  `98c9cf4d...`, `4223318b...`, `40d8a9ec...`, `19b41cd...` and `3fbfdca2...`.
- Live Boss RON remains `f696c9d054ed1e4397809f80c2707e06614f4324a1829bb7efa43ccb666c223a`.
- Gameplay, runtime and data canonicals remain `6c53ecd5...`, `ab7d2f98...` and `30ed5fd5...`.
- No live RON/data, event schema, DTO mapping, checkpoint, server route, Unity source, manifest or dependency file was
  a Phase-40 target. Workspace debug/release checks compile both `game_core` and `game_server`.

Canonical updates are correctly omitted: the removed members described false implementation shape, not gameplay or
serialized event meaning.

## Scoped Diff And Dirty Preservation

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Sole activation baseline was 581 expanded status rows at canonical C-sort SHA-256
  `71c6e602b0b6e1e1d55fcc22f693469015a2d4f7c0cb25e1bcf83535b433c258`.
- Before this audit artifact, the tree is 584 rows at
  `637c9c92e2d9091cc1b40ca573fdfd154b785a4df3b1d3fd4cf990d28ba7c83b`. The three added source status rows are
  formerly clean `movement/lifecycle.rs`, `movement/steering.rs` and `movement/types.rs`; each diff is exactly the
  inventoried ghost reset/helper/fields. Other carriers were already dirty/untracked and are bounded by activation
  and current hashes.
- Unchanged non-target blocking, planner, path and sim fingerprints remain `10323071...`, `cb4cbec3...`,
  `c75566b9...` and `af5130fc...`.
- Current central fingerprints are engine `36e6b8e0...`, movement types `d18acf36...` and Rapier `787378c3...`.
- Whole-worktree `git diff --check` and workspace format check pass.
- The protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`; mtime and ctime remain exactly
  `2026-08-10 22:55:07.178423600 +0900`.

## Validation Results

### Baseline And Transfer Proof

- Pre-cleanup movement 61, Boss 25, tactical 34, range 52, event 76 and world 293 all passed.
- With the new retained-observable control and old members still present: exact control 1/1, engine 7/7 and Rapier
  17/17 passed. This is accepted transfer-before-removal proof.

### Post-Cleanup Focused Validation

- Exact retained-observable control: 1/0/0 with 1,005 filtered.
- Movement 62/0/0; Boss 25/0/0; tactical 34/0/0; range 52/0/0; event 76/0/0; world 293/0/0.
- Exact authored defense route: 1/0/0 with 1,005 filtered. Every accepted focused filter is nonzero.

### Post-Cleanup Broad And Static Validation

- `cargo test -q -p game_core --all-targets`: 1,057 passed, 0 failed, 0 ignored.
- `cargo test -q -p game_core --release --all-targets`: 1,057 passed, 0 failed, 0 ignored.
- Each distribution is library 1,006 plus integrations 4 + 4 + 25 + 14 + 4. Six executable/fixture targets select
  zero tests and are not behavior evidence.
- Final sorted test list: 1,057 rows at SHA-256
  `361fb0181fd22cb7215116bca3579d970ef8022793c3ea04d8126748d3537658`.
- Workspace debug/release all-target checks compile Core and server with exit 0.
- Normal `cargo clippy -p game_core --all-targets` exits 0 with inherited warnings and no movement dead-code warning.
- `cargo fmt --all -- --check` and whole-worktree `git diff --check` exit 0.

### Independent Audit Refresh

- Current `cargo test -q -p game_core --all-targets` reruns the same 1,057/0/0 distribution.
- Removed-symbol and outside-module tick-surface searches each select zero files.
- Engine/types/Rapier fingerprints remain `36e6b8e0...`, `d18acf36...`, `787378c3...`.
- The pre-audit 584-row status/hash, marker and diff/format checks were independently reverified.
- Release, workspace and normal Clippy commands were not repeated during refresh because all audited source
  fingerprints equal their successful validation state; only Goal documents changed afterward.

## Failures And Diagnostics

No behavioral test failed.

- An initial shortened `--exact` filter for the new control selected zero tests. It is rejected as evidence; the full
  module path then passed 1/1.
- The first post-cleanup release run passed all 1,057 tests but exposed one test-only convenience builder warning.
  The wrapper was removed, its tests use the real builder, release check/full tests pass, and the test list is
  byte-identical. The warned state is not final evidence.
- A voluntary stricter `-D warnings` Clippy diagnostic stopped on 122 inherited warnings across unrelated current
  code/tests. It is not a required success gate and caused no source mutation. Required normal Clippy exits 0; the
  Phase-40 movement warning count is zero.

## Cross-Component And Long-Term Direction Review

| Candidate | Current → target authority | Preserved invariants | Cleanup result | Validation basis | Verdict |
| --- | --- | --- | --- | --- | --- |
| CRA-P07-C04 | stored/duplicated ghost tick/body lifecycle → explicit current goal plus minimal body/input/output | Rapier default, position/radius/speed, locks/terrain, facing, segments/events, Boss/route/range/world | false fields, writes, payloads, variants and public tick surface removed without replacement | transfer control, focused downstream, both profiles, workspace/static and consumer proof | pass |

- Fit: `High`.
- Improvement class: `none`.
- Reason: one current movement contract now names only facts it consumes and transitions Core applies. No new
  abstraction, compatibility debt, gameplay policy or Phase-41+ coupling was introduced.
- Material cross-component findings: none.

## Primary Verdict

- Primary verdict: `Complete`.
- Long-term fit: `High`.
- Combined verdict: `Complete / High`.
- Confidence: High. Exhaustive current member disposition, pre-removal assertion transfer, approved compatibility
  authority, zero legacy/public leakage, exact downstream profiles and dirty/marker evidence converge.

## Status Transition

- Child: `active` / `Implementation Complete: false` → `complete` / `Implementation Complete: true`.
- Master Phase 40: `active` → `complete — Complete / High`; verified public/serde break applied.
- Master Phase 41: `pending` → `ready` only after Child/Master exit records are updated.
- Phase 41 may then be initialized and activated under its own complete-document reread, prospective fingerprint,
  inventory and assertion-transfer gates. This audit did not create or inspect a Phase-41 Child.

## Remaining Risk And Required Action

The Rust/serde shape removal is intentionally breaking and explicitly approved. No Phase-40 correction remains.
Apply the status transitions above, preserve the marker and dirty work, then initialize Phase 41. Repository-wide
Stage-D validation and the Master final audit remain after Phase 44.
