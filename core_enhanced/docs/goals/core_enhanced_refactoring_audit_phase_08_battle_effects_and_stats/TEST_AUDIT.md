# Core Enhanced Refactoring Audit Phase 08 — CRA-14 Test Audit

Audit Date: 2026-08-07
Status: complete; phase-exit audit `Complete / High`

Runner status is supporting evidence only. Dispositions come from setup/action/assertion comparison with production,
published data and canonical.

## Family Dispositions

| Family | Unique invariant/assertion | Reachability and overlap | Disposition |
| --- | --- | --- | --- |
| UnitStats arithmetic | saturation, interval floor and zero movement | production primitive; omits extreme interval percent | `retain_and_strengthen` C03 |
| Battle-entry stat pipeline | fragments/consumable/growth/equipment/scale/condition order | direct build policy; five tests protect distinct ordering | `retain_and_strengthen` C03 |
| Damage formula | typed mitigation, modifier phase, crit, minimum, breakdown and tags | one production formula; assertions are exact, not success-only | `retain` |
| Damage snapshot/log | stable roll identity, launch source/live target and rich HpChanged | production instant/projectile/skill carriers | `retain`; merge executor fixtures with C05 |
| Buff metadata/live load | deterministic ID, capability, stack/tick validation and current RON | actual bundle boundary | `retain` |
| Buff/control lifecycle | overlapping instances, expiry, action cancellation and released effects | direct production state; periodic reapply/cause gaps remain | `retain_and_strengthen` |
| Trigger collection/target | item UUID order and OnAttack counterpart buff | production item effects; only one occurrence/ordinary effect | `retain_and_expand` C01 |
| Proc activation limits | immediate invocation, absolute cooldown and max count | production ability activation; exact cast/damage counts | `retain` |
| Resonance | clamp, auto/manual intent, Hard CC and cooldown coexistence | production state and canonical timing | `retain` |
| Runtime stat commands | ordinary attack/move modifier and resonance delta | production command; no MaxHealth/HP/death case | `retain_and_strengthen` C02 |
| Production death | HP zero, movement position, graveyard and caster/target buff expiry | actual Core lifecycle; no full death-trigger matrix | `retain_and_expand` C01/C02 |
| Top-level DeathHandler | queue dedupe and sorted allies on disconnected partial model | no production consumer; asserts false authority | `remove_after_transfer` C06 |
| Equipment validation | weapon/profile/recipe/event metadata | production publication but no effect references/matrix | `retain_and_strengthen` C04 |
| Live item activation | two skill IDs, pendant battle-start activation and forced OnAttack proc | actual RON, but skips parasite-seed effect and most references | `retain_and_strengthen` C01/C04 |
| Skill runtime composition | damage-gated heal, buff stacks/repeat, multi-step event causes | production skill effects; exact state/events, delivery details Phase 9 | `retain` effect assertions |
| Authored skill scenarios | exact heal/stat/buff/damage target sets and deltas | live skill definitions through production runtime | `retain`; target geometry owned Phase 9 |
| World incapacitation | per-instance attempt penalty and final one-time commit | production world handoff; separate from Core death details | `retain` |

## Exact Gaps And Required Tests

### C01 trigger occurrence dispatcher

- Equip live `parasite_seed` on one ally, start with at least two allies and assert one +6% StatChanged/final attack per
  active ally, deterministic order and no application to enemies.
- Combine an OnBattleStart effect and activation on one item; assert effects-before-activation event causes.
- Table-test OnAttack, OnHit, OnKill, OnDeath and OnAllyDeath ordinary effects plus activations, counterpart/all-side
  targets, dead-caster permission and sorted chained deaths.

### C02 runtime max-health lifecycle

- Apply negative flat and percent MaxHealth modifiers to active units above and at the lethal boundary. Assert exact
  StatChanged/HpChanged order, source, current/max values, lifecycle, graveyard, buff expiry and one UnitDied.
- Trigger nested OnDeath/OnKill damage or heal and prove no duplicate death or post-death mutation.
- Freeze the chosen modifier-death credit rule before changing event/result semantics.

### C03 arithmetic

- Add AttackIntervalMs percent cases around `i64::MAX`, `u64::MAX`, `i32::{MIN,MAX}` modifier values and the 1ms
  floor; no debug panic or release wrap.
- Scale high Attack/MaxHealth at 207%, assert saturation/cap and exact current-health derivation. Golden current live
  values must remain unchanged.

### C04 publication validation

- Reject missing `ApplyBuff` ID and activation ability ID with exact equipment/path/cause.
- Reject unexecutable trigger/effect combinations only after the supported matrix is frozen; load the full current
  live bundle as the positive case.
- Replace the hand-written two-ability resolution loop with complete database-edge validation.

### C05/C06 consolidation and cleanup

- Characterize the same instant/projectile hit under source death, target defense drift, mixed bonus/modifier, crit,
  kill and event-cause cases; compare behavior, not private context structs.
- Transfer DeathHandler's unique dedupe/sorted-ally intent to production death tests, then delete its test/module.
- Compile-search public carrier removals and inventory external Rust consumers before pruning fields/variants.

## Merge And Removal Decisions

- The 22 damage formula tests protect distinct arithmetic phases; do not collapse them into broad battle smoke.
- Instant/projectile tests that differ only by duplicated executor construction should share fixtures after C05, but
  launch/live-target assertions remain separate.
- Control tests are numerous but cover different cancellation, overlap, release and engagement boundaries; no weak
  duplicate was proved. Add focused cadence/reapply assertions rather than broadening all of them.
- Skill scenario tests observe exact published effects and event lineage, so happy-path status is meaningful. Their
  geometry assertions are Phase 9 carriers, not Phase 8 removal evidence.
- The sole `DeathHandler` test is removed only after its sorted/dedupe invariant is expressed against Core death.
- No scoped ignored test exists.

## Focused Runner Evidence

| Group | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| damage formula | 22 | 0 | 0 |
| buff metadata/live load | 5 | 0 | 0 |
| UnitStats/stat pipeline | 8 | 0 | 0 |
| equipment validation | 6 | 0 | 0 |
| trigger target/proc | 5 | 0 | 0 |
| control/resonance/runtime commands | 16 | 0 | 0 |
| death production/legacy | 3 | 0 | 0 |
| damage snapshot/log | 9 | 0 | 0 |
| live item/skill integrations | 20 | 0 | 0 |
| world incapacitation | 3 | 0 | 0 |

Thirty-one valid invocations executed 97 tests. Green results preserve current behavior but do not close the static
gaps: live parasite-seed omission, reference validation, max-HP death, extreme stat arithmetic or executor ownership.

## CRA-14 Verdict

Complete. Every assigned effect/stat/death family has a unique-invariant, reachability, overlap and disposition.
Only the disconnected death-model test is a removal candidate, after transferring its useful assertion. Current
production tests are retained and strengthened exactly at the six candidate boundaries.
