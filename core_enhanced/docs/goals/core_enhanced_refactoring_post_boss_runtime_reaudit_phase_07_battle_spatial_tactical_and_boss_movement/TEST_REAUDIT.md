# Phase 07 Test Reaudit

## Current Results

| Runner/family | Passed | Failed | Ignored/skipped |
| --- | ---: | ---: | ---: |
| tactical stance/relocation/swap core tests | 10 | 0 | 0 |
| field-work tests | 5 | 0 | 0 |
| movement engine tests | 19 | 0 | 0 |
| Rapier backend tests | 15 | 0 | 0 |
| movement planner tests | 25 | 0 | 0 |
| Boss private runtime tests | 22 | 0 | 0 |
| live world tactical relocate/swap tests | 2 | 0 | 0 |
| live stance test | 1 | 0 | 0 |
| live deploy/withdraw tests | 2 | 0 | 0 |
| **Total physical** | **101** | **0** | **0** |

Package-filter nonmatches are not counted. No selected test was ignored.

## Assertion Audit And Backlog

| Family | Current meaning | Disposition |
| --- | --- | --- |
| tactical stance/relocation/swap | protects cooldown, cadence, identity, engagement and selected preflight no-mutation | `retain + strengthen` for C01 final aggregate |
| field work | protects assignment/progress/death and same-kind relocation/swap occupancy | `retain + strengthen` with `Any` profile Ground↔Platform cases |
| live world tactical commands | protects cost/revision/pause and selected validation failures | `retain + strengthen` with future-cursor and reconciliation failure complete-state equality |
| Direct/Rapier parity | contains unique ordering, obstacle, airborne and clamp policy checks | `transfer before cleanup`; do not remove with C03 mechanically |
| Rapier backend | protects retained body/collider/static-obstacle implementation | `retain` |
| planner | protects route/block/target selection and the ordinary movement leaf | `retain` |
| Boss runtime movement | protects locked identity, monotonic route cursor, leaf reuse, lock timeout and no-target behavior | `retain under Boss owner`; not evidence that C03/C04 are closed |

Required C01 safety net must observe activity, stance, focus/block/rest schedule and final tile together after both
relocation and swap, including an `Any`-profile worker that remains assigned across tile kinds. Required C02 safety
net must snapshot the complete affected world state and assert equality for every named failure after command
preflight, especially a future event cursor and employee reconciliation failure.
