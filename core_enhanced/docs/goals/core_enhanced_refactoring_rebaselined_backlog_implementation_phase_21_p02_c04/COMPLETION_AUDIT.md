# Phase 21 CRA-P02-C04 Completion Audit

Audit Mode: phase_exit
Audit Target: Phase 21 CRA-P02-C04 — MapProgression mutation and derived availability encapsulation
Current Status: review_pending
Implementation Complete Claim: false; P21-C01–C09 claim complete and P21-C10 awaits this audit
Policy Source: CRA-POL-006; exact linear implementation order row 21; F-C01 and Phase 20 dependencies
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Repository/Commit/Dirty State: `/mnt/f/work/simulator`, HEAD
`f318414fb0156a67e04c5479000c2a2683fc0c09`, pre-audit expanded status 475 entries at SHA-256
`30ebdf00d0be8dd3239bfc1a8f169ffe1be77b1e8d0c322a2ed13b2d24beaed3`
Audit Date: 2026-08-10 (`Asia/Seoul`)

## Audit Method And Policy Restatement

All 567/567 lines of `core_enhanced/docs/goal_completion_review_guide.md` were reread before this pass. The current
Child was then reread in full: 180-line PLAN, 153-line experiments and 157-line notes. The audit separately reread
CRA-POL-006, exact-order row 21, the Master Phase 21/22 registry, original/re-audited C04 evidence, current
progression/facility/Boss/admin/node-flow/checkpoint/snapshot paths, the behavior tests, server and external Unity
consumer boundaries, canonical wording, hashes and dirty status.

Implementation notes and earlier test output were treated as pointers. The audit independently reran the complete
11-test progression, 40-test map-flow, 16-test admin, 11-test BossOmen and 6-test checkpoint families on the current
tree. The complete Core/server debug/release results remain current because only Goal documents changed after those
runs, while every production/test hash below is unchanged.

CRA-POL-006 allows private encapsulation only while public Rust, strict V3, authored RON, DTO/wire and deployed
consumer behavior remain exact. Row 21 specifically requires normal/BossOmen/admin transition characterization
plus a post-restore invariant preserving sorting and graph rules. It does not authorize stored-field removal,
navigation changes, schema migration, admin publication changes or Phase 22 equipment work.

## Completion Evidence Matrix

| ID | 완료 조건 또는 불변식 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P21-A01 | complete mutation/consumer graph and transition table identified | Child P21-C01; row 21 | `progression.rs`, `boss_omen.rs`, admin fixtures, staged node flow, F-C01 and snapshot | caller/direct-write audit; 84 focused audit reruns | pass |
| P21-A02 | activation dirty tree and user changes preserved | Child P21-C02; Master dirty rule | activation/current hashes and scoped status; no cleanup/reset | 472→475 explained rows; diff/fmt/status audit | pass |
| P21-A03 | exact valid and invalid behavior characterized before refactor | row 21; behavior-preserving type | normal/Boss/admin/restore transition assertions | baseline nonzero counts; test-first green table plus missing-terminal Red | pass |
| P21-A04 | typed progression boundary owns every legitimate paired mutation | Child objective | three crate-private helpers plus existing normal methods | source/caller inventory; compile and focused tests | pass |
| P21-A05 | obsolete direct production field/node writes removed | original C04 removal scope | Boss/admin delegation; all ID writes in `progression.rs` | negative production write search | pass |
| P21-A06 | sorting, graph, node state, directed/concealed/obscured and DTO behavior exact | CRA-POL-006; rulebook | unchanged normal algorithms and graph-derived snapshot projection | progression 11; map flow 40; full Core twice | pass |
| P21-A07 | strict V3, F-C01 and Phase 20 restore ordering exact | F-C01/P02-C03 dependencies | unchanged pre-restore `validate_checkpoint_state` and payload conversion | checkpoint 6; post-Continue invariant and exact re-export | pass |
| P21-A08 | BossOmen/admin/public/server/Unity boundaries remain exact | CRA-POL-006 | staged Boss completion, exact fixtures, unchanged public fields and external projection | Boss 11; admin 16; server 59 twice; canonical comparison | pass |
| P21-A09 | focused/broad/release/static evidence is sufficient | completion guide | current Core/server packages and official integrations | Core 998+51 twice; server 59 twice; checks/fmt/diff pass | pass |
| P21-A10 | canonical is current and Phase 22 was not prebuilt | Child P21-C10; linear gate | existing runtime/rulebook/server/Unity wording; Phase 22 pending | hashes, Master/source search and cross-component audit | pass |

## Runtime And State Evidence

The normal authoritative path remains:

```text
SelectMapNode
  -> validate_select_map_node
  -> selectable_map_node_ids
  -> MapProgression::selectable_node_ids(authored FacilityTemplate graph)
ConfirmEnterNode
  -> clone RunMap + MapProgression
  -> MapProgression::enter_node
  -> commit paired map/progression
CompleteNode / combat result
  -> clone RunMap + MapProgression
  -> MapProgression::complete_current_node
  -> commit staged completion
```

`enter_node` and `complete_current_node` are unchanged. Entry still rejects anything outside graph-derived
selection before mutation, changes the entered node to `Unavailable + Revealed`, sets current and clears stored
availability. Completion changes current to `Completed + Revealed`, inserts and UUID-sorts completed IDs, derives
successors through authored directed connections and completed transit, reveals concealed successors when they
become available, then sorts/deduplicates stored availability. `Obscured` remains selectable when graph-derived
admission permits it; `Concealed` does not.

The new crate-private methods live on the same `MapProgression` owner:

- `reveal_terminal_nodes` resolves one typed terminal resolution, rejects an empty set as `Ok(false)`, preflights
  every referenced node, changes all targets to `Available + Revealed` and installs the same UUID-sorted IDs. The
  preflight prevents partial helper mutation when one referenced target is missing.
- `activate_detached_fixture_node` appends the admin fixture and owns its paired `Unavailable + Revealed`, current
  and sole stored-availability state while retaining completed IDs.
- `activate_authored_fixture_node` preflights the authored node and owns its state/visibility, current, availability
  clear and completed-ID removal.

These methods do not create a second invariant checker or navigation engine. The existing
`validate_checkpoint_state` remains the sole full restored graph checker, and `selectable_node_ids` remains the
command/snapshot authority.

## BossOmen And Admin Lifecycle Evidence

BossOmen generation still validates and atomically replaces the exploration map with the authored capstone, then
constructs progression with `from_map`. On forced victory, `plan_complete_current_node` operates on cloned
map/progression/Boss state. `complete_forced_boss_with_exit_gate` verifies the forced current Boss, performs ordinary
completion, delegates only typed exit reveal and clears BossOmen state only after a real exit was revealed. The
staged result is then committed. Gate confirmation alone performs trauma recovery and Floor advance.

The exact Boss test proves current remains the completed Boss, completed IDs contain that Boss, stored availability
contains exactly the single Gate, Boss state is completed, Gate is `Gate/gate_stairs/Available/Revealed`, the F-C01
invariant holds, Floor remains zero before Gate confirmation and advances to one afterward.

Generic admin fixtures still consume the same UUID, create the same node/session/result and intentionally preserve
their special detached state: current is the fixture while stored availability contains it even though the node is
`Unavailable`. The helper centralizes this exact debug lifecycle; it does not relabel it as checkpoint-valid.

Authored SavePoint fixture creation still resolves or generates an authored facility and rewrites only the selected
node's content before the progression helper pairs its state. Its test proves empty availability, non-completed
current, `Unavailable + Revealed`, authored `save_archive` slot and valid F-C01, followed by a valid checkpoint.
Admin authorization, server publication and result/session DTOs are unchanged.

## Validation And Failure-Order Evidence

Normal unavailable and missing selection compares complete cloned map/progression before and after and proves no
mutation. The new forced-terminal unit test uses one valid and one missing target and proves the helper returns
`MissingNode` with both full values unchanged. Production forced completion is additionally staged on clones by
`plan_complete_current_node`, so any later error cannot partially mutate authoritative run state.

F-C01 still resolves the live facility, validates the exact map/template contract and calls the existing
progression invariant before Phase 20's `restore_into_state`. The six checkpoint tests exercise strict semantic
field values, Continue/recapture equality, duplicate/wrong-state/stale progression defects, explicit-load
no-mutation, foreign seed and fragment ownership. The valid Continue path now directly proves the restored
progression invariant and exact equality of stored availability with graph-derived selectability.

No invalid state is silently repaired. Normal algorithms retain their existing deterministic sort/dedup rules;
facility validation rejects duplicate terminal node IDs and missing terminal nodes before valid production maps are
used.

## Data Evidence

No production RON, facility template, assignment profile, Boss definition or fixture data was changed. The current
facility loader/generator still provides the same semantic slots, typed terminal resolutions, connection direction
and concealed capstone exit. `RunMap::validate_facility_template_contract` rejects duplicate terminal identities,
missing terminal nodes and mismatched expected terminal sets.

Both complete Core runs included all 51 official integration tests, including live RON loading/publication and map
content contracts. There is no data migration, default, alias, fallback or fixture-only production schema.

## DTO, Transport And Persistence Evidence

`MapProgression` remains public, derives the same `Serialize`/`Deserialize`, and exposes exactly the same three
public fields with the same types. Phase 20's strict V3 payload still serializes/restores the whole value unchanged;
no version, field, reader, writer or validation ordering changed.

Public snapshots do not expose internal availability/completion vectors. `snapshot.rs` continues to project only
`map_navigation.current_node_id` and `selectable_map_node_ids()`, while `MapProgression::view` produces the same
sorted node DTO and masks graph-unselectable `Available` nodes in the projection. No Core DTO, command/result,
server mapping, persistence envelope or message order was changed.

The server still passes through Core snapshot state and does not infer map reachability. The complete server suite
passed in debug and release. No coordinated rollout is needed because observable bytes and fields are unchanged.

## Consumer Evidence

- Confirmed workspace Core consumers: node selection/confirmation, normal and combat completion, BossOmen capstone,
  admin fixtures, snapshot/capability/recovery evaluators and checkpoint validation all use the current boundary.
- Confirmed workspace server consumer: the player actor transports Core snapshots/results and reads node state in
  server tests; it has no internal progression field mutation or navigation reconstruction.
- Canonical external consumer: `/mnt/f/unity projects/ark/docs/unity_core_contract.md` consumes only
  `map_navigation.current_node_id` and `selectable_node_ids`, honors Core direction/transit, and explicitly forbids
  legacy public progression lists and local reachability inference.
- Unknown deployed Rust/save consumers remain bounded by exact public/serde/V3 compatibility. Negative search was
  not used to narrow or remove a public symbol.
- No inaccessible consumer migration is required for this private-only consolidation.

## Legacy And Compatibility Audit

Production search finds progression ID mutations only inside `progression.rs`. Outside references in facility,
node flow, combat, BossOmen and state are reads. BossOmen has one `reveal_terminal_nodes` caller; generic and authored
admin lifecycles have one helper caller each. Whole-value `MapProgression::from_map` replacement remains legitimate
map construction, not an alternate in-place mutation seam.

Search finds no field rename/removal, old schema alias, fallback, dual read/write, compatibility wrapper, ignored
test, debug-only gameplay bypass, test-only production branch or Phase 22 equipment placeholder. The three public
fields remain because strict V3 and possible Rust consumers are compatibility authorities.

The retained normal methods, `refresh_available_nodes` and invariant checker are distinct lifecycle operations,
not duplicate helper implementations. Boss state clearing, admin content/session creation and checkpoint restore
remain at their proper orchestration owners.

## Scoped Diff And Dirty Preservation

Activation expanded status was 472 entries at normalized SHA-256
`c6a90d5151d585c0e3ff4d1122679829eb439b6dfd7fcd926a0d5a25af04e0a0`; pre-audit status is 475 at
`30ebdf00d0be8dd3239bfc1a8f169ffe1be77b1e8d0c322a2ed13b2d24beaed3`. The three added rows are exactly the
previously clean BossOmen, admin fixture and admin test files now changed by this Phase. Existing modified
progression/combat and untracked checkpoint test rows remain rows rather than being replaced or cleaned.

Activation→pre-audit scoped hashes are:

| Path | Activation | Pre-audit | Attribution |
| --- | --- | --- | --- |
| `map/progression.rs` | `135335b9...` | `798c04a1...` | three paired helpers and normal/invalid assertions |
| `boss_omen.rs` | `e586065d...` | `eb34f230...` | forced-exit delegation; direct writes removed |
| `world/admin/fixtures.rs` | `7bad7784...` | `c319893c...` | two fixture delegations; direct writes removed |
| `world/admin/tests.rs` | `456ff896...` | `8a1078a2...` | exact detached/authored state assertions |
| `world/tests/combat.rs` | `2e0dbdc4...` | `933df538...` | exact Boss progression/Gate assertions |
| checkpoint validation test | `9d95703b...` | `6c269025...` | post-Continue invariant/selectability assertion |
| `world/node_flow.rs` | `a311f02b...` | `a311f02b...` | unchanged staged transaction owner |
| map-flow test | `1fb7e507...` | `1fb7e507...` | unchanged broad navigation suite |

Repository-wide `git diff --check` and formatter check pass. No reset, restore, clean, stash, delete, move, broad
format or unrelated cleanup was performed. HEAD-relative diffs in overlapping files contain older campaign/user
work; this audit attributes only activation-to-current hashes and recorded Phase hunks.

Creating this audit adds exactly one expected untracked Goal artifact. Audit-inclusive expanded status is 476
entries at normalized SHA-256 `29e4646f9eaee517d07ac96593880c5b13776ab378e48653ad4ba2cfb1901523`;
production/test/canonical classifications remain unchanged.

## Canonical Alignment

The nearest canonicals already describe the preserved durable behavior and required no edit:

- `core_runtime_contract.md` is unchanged at `6e20d6a6...` and states forced Boss completion reveals the
  precommitted Gate as the only selectable successor in a staged commit before Gate confirmation.
- `game_rulebook.md` is unchanged at `6c53ecd5...` and states authored directed completed transit, Core-computed
  selection, node state/visibility meanings and exact BossOmen exit timing.
- `game_server/docs/unity_bridge_protocol.md` is unchanged at `4d734af0...` and states the server passes through the
  Core snapshot rather than deriving progression.
- The external Unity canonical is unchanged at `a3224e24...` and explicitly consumes `map_navigation`, rejects
  legacy internal ID lists and does not recalculate traversal from local graph/state.

The durable authority and behavior did not change; only the private location of paired mutations changed. Adding
helper names to canonical prose would incorrectly freeze private implementation detail.

## Validation Results

All commands ran in the current dirty worktree.

- Baseline before test edits: progression 10/10, map flow 40/40, admin 16/16, BossOmen 11/11, checkpoint 6/6.
- Test-first behavior characterization passed on the old implementation; the additional missing-terminal atomicity
  test then produced expected `E0599` because the private helper did not yet exist. This compile Red supplements,
  but does not replace, the behavior table.
- Final and audit `cargo test -p game_core game::map::progression::tests --lib`: 11 passed, 0 failed, 0 ignored;
  987 filtered out.
- Final and audit `cargo test -p game_core game::world::tests::map_flow --lib`: 40 passed, 0 failed, 0 ignored; 958
  filtered out.
- Final and audit `cargo test -p game_core game::world::admin::tests --lib`: 16 passed, 0 failed, 0 ignored; 982
  filtered out.
- Final and audit `cargo test -p game_core boss_omen --lib`: 11 passed, 0 failed, 0 ignored; 987 filtered out.
- Final and audit `cargo test -p game_core checkpoint_validation --lib`: 6 passed, 0 failed, 0 ignored; 992 filtered
  out.
- Complete `cargo test -p game_core`: 998 unit plus 51 integration tests passed, 0 failed, 0 ignored.
- Complete `cargo test -p game_core --release`: the same 998 + 51 passed, 0 failed, 0 ignored.
- Complete `cargo test -p game_server`: 59 passed, 0 failed, 0 ignored; zero-test main/doc harnesses are not counted.
- Complete `cargo test -p game_server --release`: 59 passed, 0 failed, 0 ignored.
- Core/server debug/release `cargo check`: all four passed.
- `cargo fmt --all -- --check` and repository-wide `git diff --check`: passed.
- Static public-field, helper-caller, direct-write and Phase 22 searches returned the intended boundary and no
  obsolete production mutation.

## Baseline Failures And Diagnostic Limits

There is no focused, broad, release, check, format or diff failure.

Non-strict all-target Clippy completes with established broad warning debt: Core library 83 and library-test 136
diagnostics, plus server library 3 diagnostics. No diagnostic points at the Phase 21 production edits. Some warnings
point at older regions of the already modified combat test, but none at the new Boss characterization. Strict
global `-D warnings` therefore remains non-green for pre-existing repository debt and is not misreported as passed.
Cleaning those unrelated files belongs to later candidates, not this Child.

## Cross-Component Findings

- Phase 20→21: exact restored `MapProgression` enters the existing F-C01 checker before Phase 20 conversion; the new
  methods neither validate nor restore checkpoint payloads.
- Core→server→Unity: Core graph-derived selection remains the only authority. Server passes through the snapshot;
  Unity consumes `map_navigation` and local facility data only for presentation.
- Map→BossOmen: typed terminal/node state mutation is centralized, while authored capstone generation and Boss
  lifecycle state retain distinct owners. No new edge or terminal is synthesized on victory.
- Admin→gameplay: debug fixture mutation is centralized without merging admin durability/authorization or pretending
  detached fixtures are normal checkpoint states.
- Phase 21→22: Phase 22 may rely on the existing exact checkpoint/public compatibility posture, but no equipment,
  loadout ledger, transition helper or placeholder was introduced.

No material cross-component conflict, competing source of truth, migration requirement or future-Phase prebuild was
found.

## Long-Term Direction Review

| Candidate ID | 작업 유형 | 현재→목표 authority 결과 | 보존 불변식 결과 | 제거·통합 결과 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P02-C04 | behavior-preserving refactor | normal, forced-terminal and fixture paired mutations now share the progression domain boundary | graph admission, V3/public/DTO, sorting, direction, visibility, Boss/admin timing exact | all Boss/admin direct field writes removed; one existing invariant checker retained | transition table, restore invariant, 84 audit-focused, complete Core/server debug/release | pass |

- Fit: High
- Improvement class: none
- Reason: mutation ownership is narrower and easier to audit without introducing a second graph engine, checker,
  schema or compatibility layer. Helper names and visibility match domain responsibility; orchestration stays with
  Boss/admin/world owners. Tests primarily protect exact state transitions and external projection rather than a
  private layout. No future equipment abstraction or speculative cleanup was added.

## Primary Verdict

- Complete
- Long-term fit: High
- Improvement class: none

All ten audit rows pass. There is no unresolved material policy, runtime, data, persistence, public API, consumer,
canonical, dirty-scope or validation finding.

## Status Transition

- from: `review_pending`, `Implementation Complete: false`
- to: `complete`, `Implementation Complete: true`
- P21-C10: complete
- next Phase activation: allowed only after the Master records Phase 21 as `complete — Complete / High` and then
  activates Phase 22 as the sole Child

## Remaining Risk And Required Action

- Global strict-Clippy warning debt remains outside this private consolidation; all required runtime validation is
  green and scoped non-strict output has no new production diagnostic.
- Unknown deployed consumers remain bounded by exact public/serialized compatibility; no migration or follow-up is
  required.
- No immediate correction, follow-up refactor or policy question is required for CRA-P02-C04.
- Do not inspect or mutate Phase 22 production until this audit is fully read and the Master transition protocol is
  complete.
