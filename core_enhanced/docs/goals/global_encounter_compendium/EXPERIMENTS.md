# Global Encounter Compendium Experiments

Goal: G29

Updated: 2026-07-14

## 2026-07-14 — Dirty baseline

- Core: branch `main`, HEAD `c2c5dc5ec54bd26530e46f0d7ee7909de3ed7a1f`; status fingerprint
  `8a7384a765bbe4b2dd409d7b81d43d0dde5a3a1e483bd47dcf2008f6450850e0`, tracked diff fingerprint
  `c0596f311bf61368d564573081e79be2226b9e82acd3920ded8cd198f2649997`.
- game_server: branch `main`, same HEAD; status fingerprint
  `1540fda89fbf57f99302f68890c93cd9fd063e419c476d3f9a938a6aa0ed836a`, tracked diff fingerprint
  `55cac557ad6dc24bffda70b355acc3f074990034baba39a14b9cd71e276861ba`.
- Unity ArkClient: branch `master`, HEAD `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`; status fingerprint
  `377bb589e11d102dcdbf823da9e161d8eaa13f98510f188e30273407487fe1d3`, tracked diff fingerprint
  `8bfa1f69f6bfde6f3ed02b7c8c402997b8055bc97ed5dd53e8e06a268a657f8c`.
- All worktrees are heavily dirty and predecessor changes overlap likely Core world/state/snapshot and
  battle files. Preserve unrelated/user-owned hunks; no stash, reset, restore, clean, or baseline
  commit.

## Entry gate

- G16 completion review: Complete / High; gameplay record schema is gone and must not be reused.
- G15 completion review: Complete / High; committed victory/final defeat and retry/abandonment seams
  are the authoritative consumption boundary.
- Start with read-only runtime/data/DTO/server/Unity tracing before choosing storage/projection shape.

## Initial runtime and contract trace

- There is no existing compendium/profile state. `GameCoreState` is the smallest owner outside
  `RunState`, `RunCheckpointPayload`, and `BattleEntrySnapshot`; `StartNewGame` resets run/battle
  state but does not reconstruct the whole `GameCoreState`.
- `BattleLogEvent::UnitSpawned` is the exact actual-spawn ledger. It is emitted for initial and
  delayed-wave units, carries side plus canonical `BattleUnitSourceIdentity`, and excludes previewed
  or authored-but-never-spawned entries by construction.
- Relevant source variants are opponent-side `Abnormality { abnormality_id, .. }` and
  `CorrodedEmployee { profile_id, .. }`. Employee, facility, defense-object, and test-fixture
  identities do not qualify.
- The current result owns the completed event log until `CompleteCombatResult`. Retryable defeat
  returns to NodeConfirm without `plan_complete_current_node`; exhausted/fatal defeat and victory
  use the G15 consuming result paths. Retreat restores the pre-entry snapshot before any result.
- Core has no disk/profile backend. The durable domain boundary should therefore be an explicit
  serializable ID-set profile value with validate-and-union import, stored in `GameCoreState` and
  projected in the root snapshot. Ordinary run/checkpoint APIs must never replace it.
- game_server flattens the generic Core `RunSnapshotDto` without a field mapper. Unity stores the
  root `JObject` generically but has no compendium reader/presenter. Adding the accepted player
  surface requires an exact Core DTO plus a strict Unity read model; server mapping remains generic.
- Current live abnormality and corrosion profile catalogs already provide canonical id/name; the
  abnormality catalog also provides risk level. Persist IDs only and resolve this current metadata
  during snapshot projection rather than copying it into profile state.

## Implementation

- Added strict `EncounterCompendiumProfileState` with sorted abnormality/corrosion ID sets,
  actual opponent `UnitSpawned` extraction, current-catalog validation, and monotonic union.
- Stored the profile in `GameCoreState` only. `RunState`, `RunCheckpointPayload`, and
  `BattleEntrySnapshot` deliberately contain no copy, so checkpoint load, new-run construction, and
  retreat rollback cannot replace newer global knowledge.
- Victory and consuming/fatal defeat validate discoveries before authoritative mutation and merge
  them only after the node-completion or RunFailed transition succeeds. Retryable defeat and retreat
  do not call the merge path.
- Added a required root `encounter_compendium` DTO. Snapshot projection resolves current catalog
  name/risk metadata; `BTreeSet` iteration fixes canonical ID ordering.
- game_server passes the typed root snapshot through unchanged and keeps a player-keyed monotonic
  profile while recreating actors. Core import validation remains the catalog authority.
- Added a strict Unity `EncounterCompendiumModel` plus a Rust-serialized fixture. The reader rejects
  missing or unknown fields, missing metadata, and duplicate identities; it contains no battle-log,
  preview, record, or local-resource discovery path.
- Canonical Core, server transport, Unity contract, and rulebook text now distinguish permanent
  compendium knowledge from run-local Endless research and non-authoritative combat traces.

## Focused validation

- Core `encounter_compendium`: 3 passed (spawn filtering, strict profile schema, profile
  merge/catalog projection).
- Consuming victory/checkpoint/new-run/preview-only wave: 1 passed after the final explicit
  unspawned-wave assertion.
- Retryable defeat/retreat/exhausted defeat: 1 passed.
- Final fatal boss defeat retention: 1 passed.
- game_server actor recreation profile restore: 1 passed; generic root snapshot pass-through: 1
  passed.
- Rust generator output matched
  `Assets/Tests/Fixtures/encounter_compendium_contract.json` byte-for-byte through `diff`.
- Unity exact compendium methods: 4 passed; full EditMode: 206 passed, 0 failed, 0 skipped.
- Unity PlayMode was not applicable: G29 adds no screen route, presenter, command, or gameplay
  execution path; the affected client blast radius is the root read model covered in EditMode.

## Broad validation and hygiene

- Core `cargo test --all-targets -q`: 743 passed (701 library + 3 + 3 + 19 + 13 + 4), 0 failed,
  0 ignored.
- game_server `cargo test --all-targets -q`: 29 passed, 0 failed, 0 ignored.
- Core and game_server `cargo fmt --all -- --check`: passed.
- Core, corrected game_server paths, and Unity G29-scoped `git diff --check`: passed. New untracked
  files also had no trailing whitespace.
- Production/current-contract search found no battle record, trace linkage, alias/default on the new
  required profile/root shape, Unity-local discovery path, or ignored G29 test. The sole
  `battle_records` match is an intentional negative strict-deserialization test.

## Failed or excluded attempts

- An early general workspace format invocation could not write the external game_server view under
  the managed sandbox; the approved external rerun passed. Final format checks are read-only and
  passed in both workdirs.
- The first Unity class-filter attempt returned no reliable nonzero result and is excluded. Exact
  method filters produced the recorded focused results.
- A resumed Unity full run had no recoverable job ID/count in editor state and is excluded; a fresh
  full run passed 205/205 before the strict-reader correction, then 206/206 after it.
- The first final Core focused command combined `--exact` with an unqualified function name and ran
  0 tests; it is excluded. The corrected name filter ran the intended test 1/1.
- The first fixture comparison was blocked by the sandbox's read-only shared Cargo lock; the
  approved rerun exited 0 with no diff.
- Completion review found that the first Unity reader accepted unknown fields. The reader and tests
  were tightened. One focused run then failed because its old error-message assertion expected
  `incomplete`; after updating that expectation, Unity initially reran a stale assembly. A forced
  script refresh/compile produced the final 4/4 and 206/206 passes.
- Two audit searches initially used a duplicated `game_server/game_server` path and were excluded;
  corrected paths produced the recorded results.
- Whole-Unity-worktree `git diff --check` still reports unrelated pre-existing CRLF/trailing-space
  noise in files outside G29. The G29-scoped check is clean; no unrelated user-owned file was edited.
