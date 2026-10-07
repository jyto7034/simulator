# Phase 10 Test Reaudit

## Current Results

| Runner/family | Passed | Failed | Ignored/skipped |
| --- | ---: | ---: | ---: |
| Core behavior/wire helpers | 8 | 0 | 0 |
| Core snapshot/start projections | 23 | 0 | 0 |
| Core admin separation | 16 | 0 | 0 |
| server library/mapping/message/actor/persistence | 56 | 0 | 0 |
| exact capability/revision/preview/rollback/non-Boss public-shape cases | 6 | 0 | 0 |
| private Boss runtime | 22 | 0 | 0 |
| **Total unique** | **131** | **0** | **0** |

All selected cases are unique. No scoped `#[ignore]` was found. Package-filter nonmatches and Cargo check targets are
not counted as tests. `cargo check -p game_core --all-targets` passed separately.

## Assertion Dispositions

| Family | Current meaning | Disposition |
| --- | --- | --- |
| behavior/result | protects representative wire vocabulary and research delivery | `retain + strengthen` with every result, nested completion and exact RunComplete |
| snapshot/capability | protects real state transitions, enabled/absent semantics and root shape | `retain + strengthen` with exhaustive descriptor and host-overlay matrix |
| revision/preview/VN | protects authority rejection and no-mutation/read-only behavior | `retain` as C01/C03 prerequisites |
| server mapping/serde | protects transport class, exact topology and ordered publication | `retain`; use exact old/new JSON parity for C02/C03 |
| server host recovery | protects active-run/compendium behavior through real actor paths | `retain + strengthen` with exact action order/duplicate/fatal replacement table |
| error mapping | protects two representative codes only | `retain + strengthen` with exhaustive Core/server code identity |
| admin | protects its distinct gated path and immutable failures | `retain`; transaction/publication migration remains policy-gated signal, not a current candidate |
| private Boss/non-Boss public shape | protects private scheduling and ordinary public compatibility | `retain`; not evidence to invent a Boss wire |
| empty test module/zero-test bins | carries no assertion or unique invariant | C07 `remove`; normalize retained generator bins under C06 |

## Required Safety-Net Additions

- C01: exhaustive ActionKind owner/scope/authority descriptor and projected-versus-executable parity.
- C02: all BehaviorResult variants, exact type/payload/transport class, nested completion and RunComplete goldens;
  external Rust API inventory before helper removal.
- C03: old/new exact setup/update/resync JSON plus actor/reconnect ordering; consumer-aware fixture comparison.
- C04: active-run × compendium × game-state overlay table, exact order, duplicate absence and fatal replacement.
- C05: exhaustive parameterized GameError/disabled-reason code table shared across both Core snapshot paths and server.
- C06: before/after metadata, all-target/feature checks, retained fixture runs to `/tmp`, dependency graph and
  standalone consumer inventory.
- C07: metadata/test list before and after; no new behavioral test is needed for a disconnected empty file.
