# Boss Runtime v1 Phase 04 — Completion Audit

Audit Date: 2026-08-09  
Audit Mode: `goal_exit`  
Execution Mode: `implement_and_verify`  
Review-start Status: `active`  
Production Implementation Claim: `implemented`

Audit Target: `boss_runtime_v1_implementation_master_phase_04_track_and_rejoin_vertical_slice`  
Policy Source: Master BP-01~BP-34 coverage, P04 Plan and P02/P03 audited handoffs  
Repository/Commit: `/mnt/f/work/simulator`, `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Primary verdict

`Complete / High`

P04 has connected strict raw Boss/PvE authoring, atomic candidate loading, typed setup and the P03 runtime through an
actual world `TrackAndRejoin` slice. Success, ordinary no-binding parity, failure/interrupt/Pause/reconnect behavior
and unchanged game_server transport are verified. This verdict permits the P05 content decision; it does not publish
an official Boss, activate P05 without user approval, require P06 or complete the independent refactor catalog.

## Completion evidence matrix

| ID | Requirement | Policy basis | Runtime/data/contract evidence | Verification evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P04-C01 | Strict authoring→atomic source candidate→typed setup identity | BP-04~BP-05, BP-28~BP-29 | Raw serialized strict Boss catalog and exact PvE actor binding traverse the shared source-set loader, whole-domain validation, compilation, `GameDataBase`, world setup and late spawn | exact raw-bundle test `1/0/0/923`; expanded P04 matrix 8 | pass |
| P04-C02 | Complete `TrackAndRejoin` success lifecycle | BP-02~BP-03, BP-06~BP-15 | Exact deployed target locks through approach; existing movement leaf performs approach and forward rejoin; resident `RouteCombat` resumes ordinary authority | bound success case in P04 8; at least two movement starts and monotonic cursor asserted | pass |
| P04-C03 | Paired no-binding ordinary parity | BP-01 | Same synthetic encounter geometry without binding creates no Boss actor runtime and opens the existing authored route planner on its normal movement tick | paired case in P04 8; reconnect and public shape unchanged | pass |
| P04-C04 | Explicit no-target/invalidation/path/timeout results | BP-18~BP-27 | Named authored outcomes consume each supported result once; invalidation never implicitly reacquires; path feasibility remains distinct from target lifecycle | four matrix cases in P04 8 | pass |
| P04-C05 | Lifecycle interruption and released-work ownership | BP-22~BP-27, BP-31~BP-34 | World terminal priority removes the active session once; P03 audited cancellation remains the direct released/unreleased leaf-work proof | actor-death case plus P03 Boss 22 regression | pass |
| P04-C06 | Pause/same-time/deadline determinism | BP-16~BP-17, BP-22~BP-27 | Paused 50,000 ms server tick is a full no-op; approach/rejoin change only at exact checked battle-time deadlines | timeout/Pause case in P04 8 | pass |
| P04-C07 | Reconnect does not reroll or replay | BP-30~BP-34 | Pre-spawn, active-Pattern and post-Pattern reads reproject the same authoritative session; typed runtime and full battle snapshots are unchanged | success and failure reconnect assertions in P04 8 | pass |
| P04-C08 | Existing public transport is sufficient | BP-30 | Core produces existing `LiveBattleUpdateDto`; server update/resync mapper and three transport owners are unchanged; no Boss graph field/variant exists | exact update/resync tests `1/0/0/55`; server 56; workspace exit 0 | pass |
| P04-C09 | Production content remains unselected | P04 scope, P05 approval gate | `game_resources/data/boss_runtime/base.ron` still has three empty collections; fixture RON exists only in tests | source/hash audit and raw fixture serialization | pass |
| P04-C10 | No fallback, shim or scope expansion | BP-01, BP-18~BP-21, BP-28~BP-30 | Shared crate-private loader has one validation/compile path; test snapshot is `#[cfg(test)]`; no public/save/server/Unity branch or independent CRA cleanup added | source search, owner hashes and production Core check | pass |
| P04-C11 | Canonical, broad gates and dirty preservation | P04 completion conditions 11 | Canonical already describes current P02/P03 behavior and P04 only verifies it; exact P04 document subtraction restores baseline status | fmt/diff pass; Core 924; server 56; workspace all-targets; exact hashes | pass |
| P04-C12 | Independent Phase completion disposition | P04 completion condition 12 | This audit compares plan, implementation, data, contract, consumers and tests and records remaining decisions | `Complete / High`, `ready_for_p05_decision` | pass |

## Implemented vertical slice

- The existing embedded live loader delegates to a crate-private source-set loader containing the same parse,
  whole-domain validation, Boss compilation and `GameDataBase` construction path. This permits a raw candidate proof
  without a second loader, fallback or public API.
- One raw fixture serializes the strict P02 catalog, attaches an exact actor-slot binding to cloned embedded PvE RON,
  and traverses normal run, map select/confirm, deploy and battle-time advance entry points into the typed P03
  runtime.
- The main synthetic success fixture then proves the full target-lock, approach, forward-rejoin and RouteCombat
  lifecycle with stable identity, monotonic cursor and existing movement events.
- A paired no-binding fixture plus no-target, invalidation, path-unavailable, timeout/Pause and actor-death cases close
  the required ordinary/failure/interruption matrix.
- The only direct runtime observability added by P04 is a typed `#[cfg(test)]` snapshot. It is absent from production
  compilation and public projection.

## Validation integrity

- Raw source-to-world identity:
  `cargo test -p game_core --lib game::world::tests::combat::track_and_rejoin_raw_live_bundle_reaches_typed_world_setup_identity -- --exact`.
  Result: `1 passed`, `0 failed`, `0 ignored`, `923 filtered out`.
- Final focused command:
  `cargo fmt --all -- --check && git diff --check && cargo test -p game_core --lib game::world::tests::combat::track_and_rejoin_`.
  Result: formatting/diff pass; `8 passed`, `0 failed`, `0 ignored`, `916 filtered out`.
- P03 runtime regression remained `22 passed`, zero failures, and the original bound setup, ordinary reconnect and
  late invalid-setup rollback anchors each passed with nonzero exact filters.
- Production compile and final Core gate:
  `cargo check -p game_core --lib && cargo test -p game_core --lib -- --quiet`.
  Result: check pass; `924 passed`, `0 failed`, `0 ignored`, `0 measured`, `0 filtered out`.
- Final server gate: library `56 passed`, zero failures/ignored/measured/filtered; binary/doc zero-test runners pass.
- Final `cargo test --workspace --all-targets -- --quiet` exited `0`: Core `924`, integration targets
  `3 + 4 + 24 + 13 + 4`, server `56`, and all zero-test binaries/generators/examples pass.

## Findings and corrections preserved during execution

- The first success compile used the nonexistent `Side::Enemy`; the test was corrected to the authoritative
  `Side::Opponent` spelling before any success was counted.
- The paired ordinary test initially required route movement in the spawn bucket. Existing authority legitimately
  starts it on the following 100 ms movement tick; the assertion was corrected without production mutation.
- No-target isolation first encountered the default protected Player objective, then strict tactical-objective
  validation, then correct early terminal behavior when all Player units were removed. The final fixture keeps a live
  Player side but uses a supported `Grounded` query and Airborne Player units, producing genuine nonterminal
  `NoEligibleTarget`.
- The path-unavailable test initially assumed rejoin completed in the same 100 ms boundary. It now observes the
  distinct authored `rejoin` State and then lets the existing movement leaf complete.
- Pre-exit review found that P02 raw-bundle tests plus P04 compiled-fixture tests supplied only compositional evidence
  for P04-C01. The shared loader extraction and raw source-to-world test closed that gap. No failed or superseded run
  is represented as first-pass success.

## Current behavior, canonical and approved target

- Current behavior is P02's optional strict compiled setup, P03's internal runtime and P04's verified world/transport
  integration. An encounter without `boss_runtime` remains entirely on the ordinary route/combat path.
- Current canonical is `docs/core_runtime_contract.md`. Its Boss section already describes the implemented private
  setup/runtime, Track/Rejoin, cancellation, battle-time and no-public-Wire boundaries. P04 changes verification and
  loader testability, not those semantics, so no canonical edit is required.
- The approved target remains complete Boss Runtime v1. P05 still owns the first user-approved official live Boss and
  full live feasibility. P06 remains conditional on demonstrated public lifecycle needs; P07 owns the final combined
  audit. These future targets are not claimed as current behavior.

## Data, DTO, persistence and consumer evidence

- Production data: `game_resources/data/boss_runtime/base.ron` is valid and empty. P04 raw Boss/PvE content is created
  in test memory and never changes live RON.
- DTO/transport: no new public Core event, field or game_server mapping exists. Existing spawn, movement, damage,
  checkpoint, update and resync projections carry the slice.
- Save/checkpoint: private Boss graph/activation/lock state is not serialized. Reconnect tests cover read projection
  of the same in-memory active session and do not claim durable cross-process restore.
- Confirmed workspace consumer: game_server consumes the unchanged Core projection; all 56 tests pass and its three
  player actor transport files are byte-identical to the P04 baseline.
- External consumer: no Unity `Assets` tree exists in this workspace. The known Unity project remains a stale,
  external read-only consumer and was not run or mutated because no public serialized contract changed.

## Legacy and compatibility audit

- The source-set loader is shared by the official embedded entry point and the raw fixture; there is no dual parser,
  validation bypass, compatibility alias, fixture discriminator or hot-reload path.
- Production runtime consumes compiled captured setup and does not reopen raw RON/catalog state. Raw construction and
  typed runtime snapshots occur only in tests.
- No implicit retarget, infinity path fallback, hidden retry, public placeholder DTO, dual schema, ignored test or
  temporary server/Unity shim was added.
- The no-binding ordinary path is the deliberate opt-in contract, not obsolete compatibility behavior scheduled for
  deletion.

## Dirty-tree preservation

The P04 precreation baseline contains 256 sorted status entries with SHA-256
`e0e7f5cbf7d39f41a2ad8e2a97c4070f3a726d268a87242df096c373c5ce30fc`. Before this audit artifact was added, the
tree contained 259 entries with SHA-256 `a8eafc1a25cb1c5721ace69cd10cbcd98fd6c6156c18355f9c56ddd5ae89879f`.

After this audit is present, exact status contains 260 entries with SHA-256
`8db0fca2a7d2df522768b5db2473ffe5ba72689b2da1236a42d6ae6fff62be5d`. Excluding exactly the four P04 Goal
artifacts restores the 256-entry precreation baseline and exact hash
`e0e7f5cbf7d39f41a2ad8e2a97c4070f3a726d268a87242df096c373c5ce30fc`.
No checkout, reset, clean, stash, deletion or normalization of user-owned work occurred.

Final scoped owner hashes are:

- `game/data/mod.rs`: `4f51fdc3bbccc0941b38d1d063f6b2515611538097dd725d03163a4cd8f7d895`
- `battle/core/boss_runtime.rs`: `a4b99993f0be867d940952febd2c31cc611993205db310d246e17fae1d0066fd`
- `game/world/tests/combat.rs`: `b42e742e7618a6e0182a676a5007b85906bb84bdebb68ef42442fe339383603c`

Untouched event/world/Boss-data/server owners reproduce their recorded P04 baseline hashes. HEAD remains
`f318414fb0156a67e04c5479000c2a2683fc0c09`.

## Open decisions and operational watch

- P05 cannot be activated until the user approves the first live reference Boss identity/content boundary. P04 has
  intentionally made no such creative, balance or asset decision.
- P05 evidence must decide whether existing public events remain sufficient for the official Boss. Only an actual
  public lifecycle requirement may activate P06; otherwise the Master must explicitly disposition it not applicable.
- Whole-Core cloning at Boss boundaries remains a correctness-first performance watch. The representative P04 slice
  showed no correctness or runner problem, but P05 may collect representative live-content cost before any separately
  authorized optimization.

## Long-term direction review

- Fit: `High`.
- Improvement class: `none` for P04 completion. The crate-private loader extraction removes an evidence-only bypass
  by making production and fixture source candidates share one implementation; it does not create another authority.
- Domain ownership remains clear: raw/compiled data and setup stay in data/combat setup, lifecycle and intent stay in
  Core, existing movement/combat/event modules remain leaf producers and server remains a transport mapper.

## Status decision

- P04 Status: `complete`
- Implementation Complete: `true`
- Confidence/Fit: `High`
- Handoff: `ready_for_p05_decision`
- Next authorized candidate: P05 only after explicit reference identity/content approval
- Status transition: `active` → `complete`
- Next Phase activation: `not yet allowed`; decision gate remains open
