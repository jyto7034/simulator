# Core Enhanced Refactoring Audit Phase 10 — Test And Fixture Audit

Audit Date: 2026-08-07
Status: complete
Policy: CRA-14 meaning/reachability/assertion audit; green count is supporting evidence only

## Runner Evidence

Static gaps were recorded in `EXPERIMENTS.md` before runner execution. Nine valid invocations executed 86 unique
tests: 8 Core behavior, 16 Core admin, all 56 server library tests and six exact Core capability/revision/preview/
rollback tests. All passed; zero failed or ignored. Filtered tests and generator/empty-bin zero-test targets are not
counted as executions.

## Family Dispositions

| ID | Family and reachability | Unique invariant/assertion quality | Disposition |
| --- | --- | --- | --- |
| P10-T01 | `behavior.rs` command/capability wire tests; production serde carrier | exact command vocabulary, typed reason omission, qualitative monolith and event shape | `retain` |
| P10-T02 | result helper/research delivery tests; result contract path is production, helpers are workspace-test-only | research test observes presence/absence across authoritative returns; helper test only mirrors public variants | `retain` research; `merge_remove` helper with C02 API cleanup after external inventory |
| P10-T03 | snapshot/context capability tests; production root snapshot | observes state transition, enabled/absent actions and revalidation, not just serialization success | `retain`; `strengthen` exhaustive command-owner parity |
| P10-T04 | battle capability/revision/VN/preview tests; production command entry | stale/missing authority, overflow, blocking VN and preview assert authoritative no-mutation and exact revision behavior | `retain` |
| P10-T05 | Core rollback test; production server persistence primitive | distinguishes active-run rollback from monotonic profile/peak preservation | `retain` |
| P10-T06 | server result mapping tests; direct production adapter | observes transport class, trailing snapshot choice, research payload order and battle legacy rejection | `retain`; `strengthen` all result variants and exact `RunComplete` |
| P10-T07 | server message serde tests; direct client wire | exact setup/update/resync topology, tags, typed control reason and request decoding | `retain`; use as exact parity safety net for C03 |
| P10-T08 | server actor/session/order/reconnect tests; production actor path | observes response/update/snapshot ordering, no mutation, no extra root snapshot, reconnect batch and persistence rollback separation | `retain` |
| P10-T09 | server error mapping tests; production error adapter | exact codes for obstacle and disabled reasons, but only a subset of `GameError` | `strengthen` with exhaustive stable-code table; keep message-specific cases |
| P10-T10 | Core admin tests plus server admin access tests; environment/token gated path | live fixture shape, invalid-input immutability, atomic grants, production denial and token checks | `retain`; transaction/publication semantics `unverified` pending policy |
| P10-T11 | Core snapshot explicit-label test | protects deliberately non-derived string labels consumed on wire | `retain`; small but unique |
| P10-T12 | fixture binaries/server examples | externally invoked tools, not test modules | `retain_unverified`; historical provenance exists, current Unity comparison out of scope; do not count compilation as coverage |
| P10-T13 | `tests/test_actor/mod.rs` | empty, disconnected, absent from Cargo metadata | `remove` via C07; no safety-net loss |
| P10-T14 | empty Core main and fixture auto test harnesses | execute zero tests in broad Cargo runs | main `remove` via C06; mark retained generators `test = false`; zero executions provide no evidence |

## Meaning And Overlap Findings

### Strong retained safety nets

- `game_core_projects_context_capabilities_and_revalidates_on_state_transition` proves projection changes with the
  authoritative state and that disabled/absent semantics are not interchangeable.
- `blocking_visual_novel_execution_preflight_rejects_every_omitted_battle_action_without_mutation` is a broad parity
  table for the battle action membership set and will detect C01 omissions.
- stale authority and revision-overflow tests fingerprint playback/revision before and after failure, so they observe
  atomicity rather than only error type.
- preview tests compare revision and state while also checking typed cells/reasons, proving the read path does not
  advance battle state.
- actor tests inspect ordered side-message batches and absence/presence of snapshots. Session tests establish that
  actor output is not sent a second time.
- admin failure tests fingerprint fixture/inventory state, while success tests resolve authored/live definitions and
  resulting typed snapshots. They are not artificial `Ok`-only smoke tests.

### Gaps requiring strengthening

- No all-variant golden binds `BehaviorResult` to result type/payload/transport class. `RunComplete` is specifically
  uncovered at the server seam.
- No exhaustive `GameError` stable-code table binds Core snapshot and server codes. Compile exhaustiveness prevents a
  forgotten match arm but does not prevent two valid arms from using different strings.
- Command metadata is tested in representative states and a battle action table, but no exhaustive descriptor table
  covers host/common/battle owner, scope and authority need for every `ActionKind`.
- Snapshot overlay tests cover core-root preservation and major active-run/compendium states. C04 should add one table
  asserting exact merged actions, order, duplicate absence and fatal replacement for every host state combination.
- Battle DTO tests pin representative rich values. C03 should compare old/new exact JSON for setup/update/resync and
  keep actor order tests; private Rust enum shape should not be asserted.

### Merge/remove conclusions

- `behavior_result_helpers_match_variants` exists only to test workspace-unused convenience helpers. Removing just the
  test would leave unverified public API; remove/merge it only as part of C02 after external API inventory.
- The empty `tests/test_actor/mod.rs` has no unique invariant, setup, target or assertion and may be removed alone.
- Similar server serialization and actor tests are not duplicates: serde tests protect shape, state mapping protects
  transport classification, and actor tests protect order/persistence. They should remain separate layers.
- Generator source and generated external fixtures are provenance, not equivalent test copies. Until the current
  consumer is checked, neither side is treated as redundant.

## Ignored, Conditional And Zero-Test Audit

- No `#[ignore]` occurs in the scoped Core behavior/world/admin tests or direct server actor/example sources.
- Scoped platform `cfg` branches belong to server persistence/environment support and were not used as evidence for
  command/projection correctness.
- Wider Core `#[should_panic]` data validation tests are outside this Phase and are not ignored tests.
- Cargo metadata does not expose `tests/test_actor/mod.rs`. The empty Core main and generator binaries can appear as
  zero-test harnesses in broad runs; those executions are excluded from all quality/count conclusions.

## CRA-14 Verdict

Every assigned test/fixture family has reachability, unique invariant, assertion quality, overlap and disposition.
The retained suite is strong on revision/no-mutation/order and representative wire shape. The confirmed cleanup is
one empty disconnected file; other removal needs result-API or external fixture inventory. Test-audit confidence is
High.
