# Core Enhanced Refactoring Audit Phase 06 — Experiments

Status: complete

## 2026-08-07 — Activation gate

### Evidence and result

- Parent Master, Phase 0 boundary and Phase 5 completion audit were rechecked.
- Direct predecessor Phase 5 is `Complete / High`; Phase 6 reserved path did not exist.
- Phase 2 persistence, Phase 3 economy, Phase 4 Event lifecycle and Phase 5 setup handoffs were identified as
  predecessor decisions.
- No other active child exists. Phase 6 is investigation-only and production mutation remains prohibited.
- Exact start commit remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.

### Tests

No runner was executed during activation.

### Decision

Phase 6 may perform read-only production investigation and write only audit artifacts. Phase 7 remains pending.

## 2026-08-07 — Scoped production baseline

### Exact categories and initial hashes

| Category | Initial SHA-256 |
| --- | --- |
| `battle/core` lifecycle (6 files) | `c4fa0f9a71d5282c548d29cd30eecab8343d912c777db097c2bca2f33754bf61` |
| top-level battle lifecycle (8 files) | `66b962cf7489a7e55e3330ee2a5760ff02751c15360efd0193564bd24e5ebf6e` |
| validation harness (14 files) | `19e4bdb3b598b291ac81acf56af12fe9b1b74d9fddc57d9ddd112b00436c6fe7` |
| world lifecycle (3 files) | `e56c01eef661b6867fe14ccc5d19ba629d273ca6a47293a38ea4a7f8a159f8c1` |
| playback/identity/policy/result support + live RON (6 files) | `175e1ea64dac94ac1c4a21a29af1ed5a99e1636e24816c4bcffcd4417b1f0075` |
| focused test source (1 file) | `87861813beff818f37e914cb3e25515e4b324508225711feb6c43fd03eae53ef` |
| nearest canonical (3 files) | `28f1dd4f97b22b53344326075089dfe019ca1f1e213d8bee3971ef88404b99e0` |
| scoped Git status text | `0785a5610eeeb8300c8d9b079a610fdb0160864a8fbcba0436758717f607d004` |

Core paths are `battle/core/{build,commands,ids,mod,sim,types}.rs`; top-level paths are
`battle/{enums,event_log,ids,mod,recording,result_stats,scenario,types}.rs`. World paths are
`world/{combat,combat_trace,state}.rs`. Support paths are `behavior.rs`, `determinism.rs`, `managers/uuid_manager.rs`,
`data/run_policy_data.rs`, `pve_bonus_objectives.rs` and live `game_resources/data/run/policy.ron`. The focused test
source is `world/tests/combat.rs`; inline tests remain covered by their production-file category.

### Dirty-state result

Twenty-two scoped production/data/test/canonical paths were already modified: all three canonicals, `behavior.rs`, five core
lifecycle files, five top-level/validation files, run-policy typed/live data, world combat/state and world combat
tests. These are user-owned baseline changes.

### Tests

No runner was executed for the baseline.

## 2026-08-07 — Static lifecycle gaps before focused validation

### Evidence and result

- The full-run step path and elapsed-time step path disagree at `max_battle_time_ms`. The elapsed-time path caps
  its target, returns `Running` when the next queued event is later and can repeat forever at the cap; the direct
  path instead returns a terminal Draw after first moving the execution clock beyond the cap. Existing max-time
  coverage exercises only the direct full-run path and does not assert the execution clock.
- `BattleEvent::Ord` returns `Ordering::Equal` for distinct same-time events, including production-reachable
  `ManualCastEnd` events and the `AutoCastEnd`/`ManualCastEnd` cross-variant pair. The heap/bucket path therefore
  lacks a total semantic order, while the only ordering unit test covers a small set of already-tied identifiers.
- World advance mutates the battle, may finalize it, evaluates fallible visual-novel triggers, then reconciles
  employee state, commits a projection revision and only afterward validates `after_seq` while building the DTO.
  Its next-event-bucket test helper duplicates the orchestration. Current future-seq coverage calls the read-only
  state request, not either mutating advance API.
- The public validation harness has no workspace consumer outside its own modules/tests. Its default contiguous-seq
  rule expects `entry.seq == index`, although runtime starts at seq 1, and its movement-path configuration field is
  never read. Six validator tests therefore validate a self-contained, default-incompatible harness rather than a
  produced battle log.
- MVP comparison sums already-saturated metric products with ordinary `sum`, while the published score uses a
  second loop with `saturating_add`. Accepted typed policy has no weight bound, so extreme valid policy can panic or
  wrap during selection and can disagree with the returned score. Current live weights are small and safe.
- Result completion remains protected by the outer non-battle state transaction; retreat restores the exact entry
  snapshot, while run-power peak and encounter-compendium monotonic lifetime are deliberately outside rollback.

### Focused validation reason

Run the smallest existing lifecycle families to establish current behavior and test counts. Green results cannot
close the missing elapsed-time max-cap, total-order, mutating future-seq, real-log-validator or score-overflow cases;
those gaps are recorded before execution as required by CRA-14.

## 2026-08-07 — Focused validation

| Command/filter group | Passed | Failed | Ignored | Meaning |
| --- | ---: | ---: | ---: | --- |
| `cargo test -p game_core --lib game::battle::enums::tests -- --nocapture` | 1 | 0 | 0 | current partial heap ordering |
| `cargo test -p game_core --lib game::battle::result_stats::tests -- --nocapture` | 4 | 0 | 0 | aggregation/redeploy/incapacitation |
| `cargo test -p game_core --lib game::battle::scenario::tests -- --nocapture` | 4 | 0 | 0 | scenario construction/validation |
| `cargo test -p game_core --lib game::battle::validation::validator::tests -- --nocapture` | 6 | 0 | 0 | disconnected validator self-tests |
| four fully-qualified exact core cap/step/order tests | 4 | 0 | 0 | current full-run and ordinary step behavior |
| five exact world cursor/recovery/playback/revision tests | 5 | 0 | 0 | read-only cursor and preflight behavior |
| five exact world trigger/result/retreat/retry/trace tests | 5 | 0 | 0 | retained result and rollback behavior |

Eighteen valid invocations executed 29 passing tests. Four initial `--exact` invocations used unqualified names and
selected zero tests; they were discarded and rerun with fully-qualified names, each executing one test. No ignored
marker exists in the scoped source/test paths. The green result does not cover any static gap recorded above.

## 2026-08-07 — Candidate and CRA-14 disposition

- Confirmed C01 max-time terminal ownership, C02 total same-time order and C03 staged world advance as priority
  correctness candidates.
- Confirmed C05 single saturating MVP score as a separate accepted-policy correctness candidate.
- Confirmed C04 disconnected validator, C06 ghost scenario `once` and C07 event-log file I/O as cleanup candidates
  with public-consumer gates.
- Retained playback, event log, result transaction, exact retreat/retry, debug trace and deterministic identity.
- Classified 16 test families; the validator self-tests are the only removal family and require stronger produced-
  log tests first.

## 2026-08-07 — Final no-mutation proof

All final hashes equal their initial values:

| Category | Initial SHA-256 | Final SHA-256 | Result |
| --- | --- | --- | --- |
| `battle/core` lifecycle | `c4fa0f9a71d5282c548d29cd30eecab8343d912c777db097c2bca2f33754bf61` | `c4fa0f9a71d5282c548d29cd30eecab8343d912c777db097c2bca2f33754bf61` | identical |
| top-level battle lifecycle | `66b962cf7489a7e55e3330ee2a5760ff02751c15360efd0193564bd24e5ebf6e` | `66b962cf7489a7e55e3330ee2a5760ff02751c15360efd0193564bd24e5ebf6e` | identical |
| validation harness | `19e4bdb3b598b291ac81acf56af12fe9b1b74d9fddc57d9ddd112b00436c6fe7` | `19e4bdb3b598b291ac81acf56af12fe9b1b74d9fddc57d9ddd112b00436c6fe7` | identical |
| world lifecycle | `e56c01eef661b6867fe14ccc5d19ba629d273ca6a47293a38ea4a7f8a159f8c1` | `e56c01eef661b6867fe14ccc5d19ba629d273ca6a47293a38ea4a7f8a159f8c1` | identical |
| playback/identity/policy/result support + live RON | `175e1ea64dac94ac1c4a21a29af1ed5a99e1636e24816c4bcffcd4417b1f0075` | `175e1ea64dac94ac1c4a21a29af1ed5a99e1636e24816c4bcffcd4417b1f0075` | identical |
| focused test source | `87861813beff818f37e914cb3e25515e4b324508225711feb6c43fd03eae53ef` | `87861813beff818f37e914cb3e25515e4b324508225711feb6c43fd03eae53ef` | identical |
| nearest canonical | `28f1dd4f97b22b53344326075089dfe019ca1f1e213d8bee3971ef88404b99e0` | `28f1dd4f97b22b53344326075089dfe019ca1f1e213d8bee3971ef88404b99e0` | identical |
| scoped Git status text | `0785a5610eeeb8300c8d9b079a610fdb0160864a8fbcba0436758717f607d004` | `0785a5610eeeb8300c8d9b079a610fdb0160864a8fbcba0436758717f607d004` | identical |

Only Master/Phase audit artifacts changed. Cargo produced ignored `target` artifacts; no production/data/test/
canonical/server/Unity file was edited, formatted, moved or deleted.
