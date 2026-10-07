# Phase 03 Experiments

Status: complete

Initialized after P02 `Complete / High`; P03 was ready, P04 pending and path absent. No tests before documents.

## 2026-08-09 — Current source, Boss-impact and test reaudit

### Hypothesis

The original P03 correction/refactor causes remain in the current dirty tree unless Boss work replaced their complete
authority rather than merely adding consumers.

### Source and data checks

- Re-read the original detailed P03 candidate catalog and traced enhancement publication/admission/consumption.
- Traced both equipment binding ledgers across equip/unequip/starter/death/dismantle and battle setup consumers.
- Enumerated `GrantExecutor` and reward/shop/event/combat/admin five-ledger staging/commit call sites.
- Compared checkpoint capture/validation/restore with roster/inventory/fragment graph invariants.
- Compared HQ's handwritten rollback helper with `with_state_transaction`.
- Searched the scoped source/data/tests for Boss intersections, new `CRR-P03-*` causes and ignored tests.

### Test commands and results

The following current runners were executed from repository root; package-filter nonmatches were excluded.

```text
cargo test -p game_core --lib game::world::tests::equipment -- --nocapture
cargo test -p game_core --lib game::reward::tests -- --nocapture
cargo test -p game_core --lib game::world::shop::tests -- --nocapture
cargo test -p game_core --lib game::world::tests::node_sessions -- --nocapture
cargo test -p game_core --lib game::world::tests::facility -- --nocapture
cargo test -p game_core --lib game::world::tests::map_flow -- --nocapture
cargo test -p game_core --lib game::world::tests::snapshots_and_start -- --nocapture
cargo test -p game_core --lib game::world::tests::combat::combat_result_completion_failure_does_not_partially_apply_rewards_on_retry -- --exact --nocapture
cargo test -p game_core --test ron_loading -- --nocapture
cargo test -p game_core --lib game::resources::inventory::tests -- --nocapture
```

Results: 10 runner invocations, 159 passed, 0 failed, 0 ignored. Current lib filters reported 930 total tests, so
filtered counts differ from the historical pre-Boss audit and are not treated as skips.

### Actual result and decision

- CRA-P03-C02/C03/C04 remain valid.
- CRA-F-C01 P03 economy-graph and CRA-F-C02 P03 HQ rollback contributions remain valid.
- Boss adds consumers/coverage but no P03 mutation authority or new root cause.
- No `CRR-P03-*` candidate was confirmed.
- P00 status/content hashes remained exact after tests and documentation.

P03 exits `Complete / High`; production source/data/test/canonical/server/Unity remained read-only.
