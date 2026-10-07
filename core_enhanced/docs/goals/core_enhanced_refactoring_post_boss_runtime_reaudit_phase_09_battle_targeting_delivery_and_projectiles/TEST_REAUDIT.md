# Phase 09 Test Reaudit

## Current Results

| Runner/family | Passed | Failed | Ignored/skipped |
| --- | ---: | ---: | ---: |
| tile range | 2 | 0 | 0 |
| target selector | 11 | 0 | 0 |
| range preview | 7 | 0 | 0 |
| skill publication/negative schema | 13 | 0 | 0 |
| basic definition validation | 4 | 0 | 0 |
| basic projectile/command runtime | 28 | 0 | 0 |
| skill projectile math | 1 | 0 | 0 |
| skill runtime contract integration | 13 | 0 | 0 |
| live RON integration | 25 | 0 | 0 |
| focused area/usefulness/target/range cases | 12 | 0 | 0 |
| released-work lifecycle | 7 | 0 | 0 |
| world deployment projection | 1 | 0 | 0 |
| Boss private runtime | 22 | 0 | 0 |
| **Total physical** | **146** | **0** | **0** |

One Boss released-projectile case intentionally ran in both the lifecycle filter and the Boss module; unique cases
are 145. No selected test was ignored, and package-filter nonmatches are not counted.

## Assertion Dispositions

| Family | Current meaning | Disposition |
| --- | --- | --- |
| tile range/selector | protects exact current geometry, ties and policy | `retain`; consolidate adapters under C06 |
| basic projectile | protects target-locked continuous motion and launch/live-target snapshots | `retain`; share motion table under C05 |
| skill projectile/runtime contract | protects directional/homing/pierce/hit/kill behavior | `retain + strengthen` with max-kills cast cleanup/exactly-once terminal |
| publication/live RON | protects current policy combinations and bundle | `retain + strengthen` with zero values and basic compatibility matrix |
| area/usefulness | protects current anchor-inclusive live behavior | `retain + strengthen` with anchor-excluding EnemySingle footprint and multi-step rule |
| preview/world projection | protects cast cells/facings/no mutation | `retain + strengthen` with preview/runtime footprint parity after P10 decision |
| released lifecycle/Boss | protects cancellation boundary and delayed leaf survival | `retain`; not evidence that C05 is cleanup or closed |
| negative schema | visibly accepts an unrelated missing-field panic | `correct` under C08; otherwise-valid fixture and exact typed cause required |

## Required Safety-Net Additions

- C01: RON and programmatic zero speed/limits; complete basic/weapon compatibility table with exact errors.
- C02: active projectile and cast absence, one terminal delivery/cast completion under max-kills, same-time combined
  stop policies.
- C03: EnemySingle footprint excluding anchor across candidate selection, cast gate and execution; freeze multi-step
  usefulness rule.
- C04: runtime/preview footprint parity at edges/facings; defer multi-step representation to P10.
- C05/C06: common adapter behavior tables and cross-product membership/materialization parity.
- C07: server/external symbol and serialized migration inventory before cleanup.
- C08: prove repaired fixture parses, then assert exact object/path/cause for the single restored defect.
