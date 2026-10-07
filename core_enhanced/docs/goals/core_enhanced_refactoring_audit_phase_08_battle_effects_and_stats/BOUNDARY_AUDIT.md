# Core Enhanced Refactoring Audit Phase 08 — Boundary Audit

Audit Date: 2026-08-07
Status: complete; phase-exit audit `Complete / High`

## Exact Boundary

Phase 8 owns persistent inputs at the battle-stat conversion seam, current-hit damage/effect composition, active
buff/control/proc/resonance state, HP/death mutation and the employee-attempt incapacitation handoff. Phase 3 owns
persistent transactions, Phase 6 owns battle clock/result commit, Phase 7 owns tactical-effect transition timing,
Phase 9 owns target acquisition/delivery geometry and Phase 10 owns public/server projection refactor discovery.

Primary code is `battle/{buffs,cooldown,damage,death,stat_pipeline}.rs`,
`battle/core/{damage_runtime,death_runtime,resonance_runtime,triggers}.rs` and effect slices of
`battle/core/{build,commands,mod,sim,types}.rs`. Direct carriers are `stats.rs`, employee/growth/equipment/fragment
inputs, event/result types, world combat/state, published buff/equipment/skill/unit/run-policy RON, focused tests and
`docs/{core_runtime_contract,game_rulebook,data_loading_contract}.md`.

## Authority Matrix

| Domain | Current authority | Consumers | Finding |
| --- | --- | --- | --- |
| Entry stats | `stat_pipeline.rs` ordered draft transformation | runtime unit build, certified power seam | one order; extreme arithmetic is not total |
| Runtime stats | `UnitStats` plus `BattleCommand::ApplyModifier` | attack/move/damage/checkpoint/log | modifier can change HP without HP/death lifecycle |
| Damage formula | `calculate_damage` | basic, skill, area, buff and preview/usefulness | one formula; orchestration is duplicated |
| HP/death | `apply_*_and_record` → `finalize_unit_death` | log, graveyard, triggers, result/world | coherent production authority |
| Trigger lookup | runtime items sorted by instance ID | occurrence-specific dispatchers | one lookup, split and incomplete dispatch |
| Buff/control | `(caster,target,buff)` map and metadata capability | actions, block, ticks, checkpoint | derived, battle-time and stale-event safe |
| Proc limits | `AbilityProcKey` state with absolute ready time/count | triggered abilities | deterministic identity and coherent limit state |
| Resonance | runtime unit current/max plus action lock | casts, Rest/basic attack and events | one clamped battle-time authority |
| Persistent death | world instance ledger and staged result state | employee attempt/redeploy/final roster | exactly-once boundary retained |
| Legacy death | public `battle/death.rs` queue | only its own unit test | disconnected competing model |
| Effect publication | equipment/skill/buff data validators | stat/trigger/ability runtime | effect references and compatibility are not closed |

## Persistent To Runtime Stat Trace

Employee battle profile applies base, equipped fragments and the active consumable. The draft then applies weapon
profile, `GrowthStack`, equipment `Permanent` effects, enhancement modifiers, encounter stat scale and finally exact
condition-derived HP. Certified power deliberately stops before consumable and condition state. The two stages are
an explicit scenario/build boundary rather than duplicate policy. Canonical records `KillStack` as producer-less but
serialized state; Phase 3 already rejected deletion without save/external authority.

`UnitStats` generally saturates signed modifier application. Two holes remain. Attack-interval percent math casts a
`u64` base to `i64` and multiplies without saturation. Draft scaling calls `percent_amount_ceil`, whose unbounded
`u64` quotient is narrowed with `as u32`; at the current authored stage-three `207%`, a valid high stat can wrap and
max health can escape the type's own `i32::MAX` cap. Current live unit values are modest, so this is a valid-input
hardening correction rather than evidence of a present balance incident.

At runtime, `ApplyModifier` mutates all `UnitStats` and logs `StatChanged`. `add_max_health` clamps current HP to the
new maximum, but the command does not emit `HpChanged` or finalize death. A sufficiently negative max-health effect
can therefore leave an Active zero-HP unit, contradicting the canonical statement that `HpChanged` and final HP own
damage/death. Source provenance is also discarded when an effect becomes the command, so a death-credit rule must
be explicit before implementation.

## Damage, HP And Death Trace

All actual damage reaches `calculate_damage`: request and incoming/trigger modifiers merge, typed raw components are
crit-scaled, independently mitigated and post-modified, then minimum damage and feedback are derived. Core applies
the result, records one metadata-rich `HpChanged`, grants basic-attack resonance from actual HP delta and finalizes
zero HP. Death sets lifecycle/action state, captures the graveyard snapshot, clears target/caster buffs, logs
`UnitDied`, dispatches death/kill/ally triggers and reconciles field work. Nested damage sees inactive units and does
not repeat the death.

Instant basic attack uses `calculate_basic_attack_damage_snapshot`; projectile impact repeats nearly the same
OnAttack/OnHit effect and activation assembly, damage, resonance and command order. General `ApplyDamage` constructs
another context for skill/area/buff hits. The formula is not duplicated, but the lifecycle around it is. A typed
common resolution plan must preserve launch source/live target semantics and explicit basic-versus-skill trigger
policy rather than making every damage source an OnHit occurrence.

`calculate_damage` also computes `BattleCommand::UnitDied`; every consumer applies HP/death first and command
processing explicitly ignores that variant. `DamageRequest.time_ms`, attacker/target-side and attack/max-HP context
fields similarly have no formula reader. These authority-shaped leftovers enlarge the public Rust API and test
fixtures without governing behavior.

## Trigger, Buff, Cooldown And Resonance Trace

Items are collected deterministically by runtime item UUID. Damage occurrences process current-hit effects and then
ordinary effect/ability commands; death occurrences process OnDeath, OnKill and sorted OnAllyDeath effects plus
activations. Battle start, however, collects only ability activations. The live `parasite_seed` OnBattleStart
AllAllies attack modifier is therefore never executed, while the live pendant activation test passes. One occurrence
dispatcher should assemble effects and activations in the established effects-before-activations order.

Equipment validation closes weapon/profile/recipe metadata but not `ApplyBuff` IDs, activation skill IDs or the
supported trigger/effect matrix. Missing references silently no-op after publication. `BonusDamage` and
`ModifyDamage` consume current-hit context and ignore `TriggeredEffect.target`; other occurrences drop them. Live
`thorned_lantern_guard` authors OnHit Counterpart bonus damage, but current code adds it to the defender's incoming
hit. The uniform target documentation and current-attack effect description conflict, so retaliation versus
targetless current-hit semantics is a policy migration signal, not a Phase 8 invented verdict.

Buff identity is `(caster,target,buff)`. Stack refresh keeps the first cadence, expiry events reject older deadlines,
ticks require exact `next_tick_ms`, and death/withdraw remove both caster- and target-owned instances. Control is
derived from active metadata; the latest Hard CC expiry and any Silence drive central gates. Proc roll identity is
independent of log sequence and its cooldown/count state only gates successful activations. Resonance uses battle
time, clamps changes, respects Hard CC/gain locks and schedules auto intent only on a permitted fill. These
authorities are retained, with stronger reapply/stale-cause characterization recommended.

## Persistent Incapacitation Handoff

World scans deployed employee units whose runtime lifecycle is Dead and whose instance ID is absent from
`processed_incapacitations`. It applies attempt-local physical damage/trauma once, records the instance ledger and
reconciles deployment. Final result completion commits the already-adjusted attempt roster through one staged state
without recomputing the penalty. Tests cover multiple redeploy instances and defeat completion. This is Phase 6's
result transaction consumer, not a second Phase 8 death authority.

## Data, Canonical And Consumer Classification

- Published buffs explicitly define periodic/control kinds, positive tick interval where needed, control max stack
  one and reapply policy. Published effect and activation IDs currently resolve, but no complete validator proves it.
- Canonical agrees on battle-time Pause, derived control, released-effect survival, deterministic RNG, Core-owned
  `HpChanged`, runtime-instance incapacitation and noninheritance on redeploy.
- Confirmed consumers are BattleCore simulation, active battle/world reconciliation, event/result stats and current
  live data. Phase 9 target/delivery consumers were read only as damage carriers.
- Public `battle::{damage,death,cooldown}` types may have unknown external Rust consumers. Removal or field changes
  need inventory/compatibility approval. Unity was not read; server/DTO semantics remain Phase 10.

## Boundary Verdict

Three correctness candidates restore battle-start effect dispatch, HP-aware runtime stat mutation and total stat
arithmetic. One publication candidate closes silent effect references after its compatibility matrix is settled. A
behavior-preserving damage/trigger execution consolidation removes the split that caused drift, and one cleanup
candidate deletes the disconnected death model after public-consumer review. Buff/control/proc/resonance and world
incapacitation authorities are otherwise retained. No production file was changed.
