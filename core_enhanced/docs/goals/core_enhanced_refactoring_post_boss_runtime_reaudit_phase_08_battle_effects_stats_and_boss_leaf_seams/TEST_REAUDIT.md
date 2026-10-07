# Phase 08 Test Reaudit

## Current Results

| Runner/family | Passed | Failed | Ignored/skipped |
| --- | ---: | ---: | ---: |
| UnitStats | 3 | 0 | 0 |
| battle-entry stat pipeline | 5 | 0 | 0 |
| damage formula | 22 | 0 | 0 |
| disconnected DeathHandler | 1 | 0 | 0 |
| trigger collection | 1 | 0 | 0 |
| committed damage/projectile/trigger commands | 28 | 0 | 0 |
| equipment validation | 6 | 0 | 0 |
| exact runtime stat/death command tests | 3 | 0 | 0 |
| Boss private runtime | 22 | 0 | 0 |
| live item activation | 3 | 0 | 0 |
| live skill catalog audit | 4 | 0 | 0 |
| full live RON integration | 25 | 0 | 0 |
| **Total physical** | **123** | **0** | **0** |

No selected test was ignored. Package-filter nonmatches are not counted.

## Assertion Dispositions

| Family | Current meaning | Disposition |
| --- | --- | --- |
| UnitStats/stat pipeline | protects ordinary saturation/order/current live scale | `retain + strengthen` with C03 extreme interval and 207% cap cases |
| damage formula | protects the one pure arithmetic authority | `retain`; do not merge into C05 runtime consolidation |
| trigger/live item | protects item ordering, activation and one OnAttack counterpart case | `retain + strengthen` with live parasite and all occurrence classes |
| runtime stat commands | protects ordinary attack/movement changes only | `retain + strengthen` with nonlethal/lethal MaxHealth lifecycle |
| equipment/live bundle | protects local metadata and current loadability | `retain + strengthen` with missing buff/skill and matrix errors |
| committed instant/projectile | protects snapshot, target drift, roll identity and event detail | `retain`; share fixtures after C05 but preserve unique launch/impact assertions |
| production death | protects graveyard, movement and buff cleanup | `retain + expand` for complete occurrence ordering/dedupe |
| top-level DeathHandler | protects dedupe/sorted allies on a disconnected false model | `transfer then remove` under C06 |
| Boss cancellation | protects unreleased cancellation and released projectile survival | `retain under Boss owner`; leaf behavior remains P08-owned |

## Required Safety-Net Additions

- C01: live two-ally parasite-seed stat/effect order plus effect+activation same occurrence; table all attack/hit/death
  occurrences and dead-caster/target rules.
- C02: negative flat/percent MaxHealth, nonlethal and lethal, with exact StatChanged/HpChanged/death/source ordering
  and complete cleanup.
- C03: `u64::MAX` AttackInterval and extreme signed modifiers; high Attack/MaxHealth scale including 207%, with
  declared cap and live-value goldens.
- C04: exact missing buff/skill object/field/cause; positive full bundle; matrix cases only after target policy.
- C05: instant/projectile equivalence under source death, target defense drift, bonus/modifier, crit, kill and lineage.
- C06: express sorted/dedupe behavior through production death before deleting the legacy test.
