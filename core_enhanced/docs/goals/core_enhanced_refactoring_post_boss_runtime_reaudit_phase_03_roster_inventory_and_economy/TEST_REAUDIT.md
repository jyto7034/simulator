# Phase 03 Test Reaudit

## Current Results

| Runner | Passed | Failed | Ignored/skipped | Meaning |
| --- | ---: | ---: | ---: | --- |
| equipment world tests | 28 | 0 | 0 | loadout, enhancement, fragments, death/dismantle and two-ledger success behavior |
| reward + shop tests | 8 | 0 | 0 | canonical grant vocabulary and staged shop mutation |
| node-session tests | 18 | 0 | 0 | reward atomicity, HQ flows and one checkpoint recruitment rejection |
| facility tests | 11 | 0 | 0 | valid checkpoint restore/reserve and maintenance capabilities |
| map-flow tests | 40 | 0 | 0 | normal/BossOmen progression, event grant and planning-return atomicity |
| snapshot/start tests | 23 | 0 | 0 | starter roster/equipment and projection/checkpoint behavior |
| combat-result late-failure exact test | 1 | 0 | 0 | outer combat/grant rollback |
| live RON loading | 25 | 0 | 0 | current published economy/Boss inputs |
| inventory resource tests | 5 | 0 | 0 | reserve, stack and duplicate-instance atomicity |
| **Total meaningful** | **159** | **0** | **0** | package-filter nonmatches excluded |

## Dispositions

| Family | Disposition | Missing/retained invariant |
| --- | --- | --- |
| equipment/loadout success tests | `retain + strengthen` | add one transition matrix asserting both ledgers for move/replace/unequip/death/dismantle/starter |
| enhancement tests | `retain + strengthen` | add duplicate-row insufficient-aggregate unchanged state/diff and exact-aggregate success before C02 |
| checkpoint valid round trips | `retain + strengthen` | add mutated-current-V3 economy graph negatives with unchanged load count/state/re-export |
| reward/shop/event/combat/admin grant tests | `retain + consolidate after parity proof` | characterize exact five-ledger state, diff order, seed/UUID and late failure for C04 |
| HQ/node-session tests | `retain + strengthen` | compare full non-battle state and compendium on late failures before F-C02 consolidation |
| Boss map/setup tests in selected families | `retain under owning phases` | they prove P03 has consumers but no new asset mutation owner |

No focused file contains `#[ignore]`. Current green paths do not prove duplicate-cost failure safety, imported economy
graph rejection, grant caller consolidation or full HQ rollback parity, so they do not close any still-valid row.
