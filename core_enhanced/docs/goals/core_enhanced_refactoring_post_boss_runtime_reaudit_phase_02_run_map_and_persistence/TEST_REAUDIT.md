# Phase 02 Test Reaudit

## Current Results

| Runner | Passed | Failed | Ignored/skipped | Meaning |
| --- | ---: | ---: | ---: | --- |
| `cargo test -p game_server persistence::active_run_store::tests -- --nocapture` | 9 | 0 | 0 | real-file envelope/current/backup/invalidation/checksum/version behavior |
| checkpoint round-trip named filter | 1 | 0 | 0 | valid V3/host monotonic round trip |
| map progression named filter | 1 | 0 | 0 | normal select/complete/unlock transition |
| non-exact recruitment rejection named filter | 1 | 0 | 0 | one F-C01 no-partial-mutation domain edge |
| Total meaningful | 12 | 0 | 0 | package-filter nonmatches/zero-test bins excluded |

## Dispositions

| Family | Disposition | Missing/retained invariant |
| --- | --- | --- |
| active-run store real-file tests | `retain + strengthen` | add lower valid incoming current for loads, peak and both while preserving incoming Core payload |
| valid V3 round trips/facility/node session | `retain + strengthen` | add exact field-set/golden exclusions for C03 |
| checkpoint negative domain tests | `retain + strengthen` | P02 seed/progression graph matrix plus P03 asset/economy matrix |
| map/progression/map-flow tests | `retain + strengthen` | direct BossOmen/admin transition parity and post-restore invariant |
| backup recovery tests | `retain` | approved explicit recovery, not duplicate fallback debt |
| Boss reconnect/checkpoint tests | assigned P06/P10; P02 retains no-save assertion | private graph exclusion is distinct from V3 mapping |

No current P02 test has proven removal/merge safety. Green valid-path tests do not close the named negative gaps.

