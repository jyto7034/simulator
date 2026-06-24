# Experiments

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-22 | Design audit | Read current basic attack, projectile, skill command, skill projectile, damage calculation, and timeline event code. | Current system is mixed: some per-resolve snapshots exist, projectile launch stores partial source context, but most damage source data is still live/graveyard-read at apply/impact time. | Use split `DamageSourceSnapshot` + apply-time `DamageTargetContext` as the proposed long-term design. |
| 2026-06-22 | Goal setup | Created `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md` for the damage snapshot contract goal. | Success. | Implementation should start only after reviewing policy-sensitive timeline/DTO questions. |
| 2026-06-22 | Implementation preflight | Re-read `damage.rs`, `core/commands.rs`, `core/basic_attack.rs`, `core/sim.rs`, `core/skill_runtime/projectile.rs`, `core/types.rs`, and `battle/enums.rs` before editing. | Policy-sensitive decisions are unavoidable before implementation: crit roll timing, source-side on-attack trigger semantics after source death, skill multi-hit snapshot granularity, and delayed skill projectile snapshot timing. | 사용자와 정책 논의 필요. Do not edit runtime code until these decisions are confirmed. |
| 2026-06-22 | Policy resolution | Recorded confirmed user decisions: freeze crit from source snapshot inputs, keep committed source-side damage modifiers after source death, suppress source-side command/ability triggers after source death, use target-specific skill snapshots, use projectile launch snapshots, and keep serialized DTO/RON/timeline shapes stable. | Success. | Runtime implementation may proceed within these exact bounds. |
| 2026-06-22 | Source snapshot contract | Added `DamageSourceSnapshot` and `DamageBonusSnapshot`; changed `BattleCommand::ApplyDamage` to carry a snapshot. | Success. `process_commands()` now uses source snapshot data and target live context instead of live/graveyard source recomputation at apply time. | Keep `DamageContext` stable for calculation and adapt source snapshots into existing `SourcedEffect` damage inputs. |
| 2026-06-22 | Basic attack | Changed internal `AttackResolve` events to carry source snapshots and delivery kind. Instant committed attacks can resolve after same-timestamp source death. | Success. | Pin with same-timestamp source death test. |
| 2026-06-22 | Basic attack projectile | Stored source snapshots in `ProjectileRecord` and used them at impact. | Success. Projectile damage uses launch source attack after source stat drift/death; target defense remains live at impact. | Pin source drift and live target defense tests. |
| 2026-06-22 | Skill damage and skill projectile | Skill `ApplyDamage` commands now carry source snapshots; skill projectile runtime carries a launch source template and materializes target-specific snapshots on impact. | Success. Delayed skill projectile damage no longer depends on live caster state at impact. | Pin caster-death projectile impact test. |
| 2026-06-22 | Buff tick damage | Buff tick damage creates a source snapshot at each tick event and sends it through `ApplyDamage`. | Success. This follows the plan's tick commit point. | Broader buff/death policy remains owned by other policy goals. |
| 2026-06-22 | Dead code cleanup | The old direct `resolve_basic_attack(...)` wrapper is used only by unit tests, so it was restricted to `#[cfg(test)]`. | Success. Production code now routes committed basic attacks through explicit source snapshots without leaving an unused runtime wrapper. | Keep tests using it as a helper only. |

## Failed Approaches

| Date | Scope | Attempt | Failure | Resolution |
| --- | --- | --- | --- | --- |
| 2026-06-22 | Focused tests | Tried to pass four separate test names to one `cargo test --lib` command. | Cargo accepts only one positional test filter and rejected the extra arguments. | Re-ran focused validation with the module filter `game::battle::core::commands::tests`. |
| 2026-06-22 | Compile cleanup | First final `cargo check --lib` passed but warned that `resolve_basic_attack` was unused in production. | The wrapper was now test-only behavior and could be mistaken for a remaining runtime compatibility path. | Added `#[cfg(test)]` to the wrapper and re-ran `cargo fmt` and `cargo check --lib`. |

## Validation Commands

- `cargo fmt`
  - Result: passed.
- `cargo check --lib`
  - First final run: passed with an unused `resolve_basic_attack` warning.
  - Final run after cleanup: passed.
  - Remaining output: pre-existing workspace manifest warning from `auth_server/Cargo.toml` (`unused manifest key: env`).
- `cargo test --lib game::battle::core::commands::tests -- --test-threads=1`
  - Result: passed, 21 passed, 0 failed, 498 filtered out.
- `cargo test --lib -- --test-threads=1`
  - Result: passed, 519 passed, 0 failed.

## Code Reading Notes

- `src/game/battle/core/basic_attack.rs`
  - `handle_basic_attack_resolve_event()` drains same-timestamp attack resolves before winner finalization, but still checks the current live attacker state.
  - Comment confirms current behavior: each resolve uses current live attacker/target state.
- `src/game/battle/core/commands.rs`
  - `BasicAttackDamageSnapshot` exists but is built inside `resolve_basic_attack()`, so it snapshots resolve-time data rather than committed attack-time data.
  - `ProjectileLaunch` and `ProjectileRecord` keep partial launch context for basic attack projectiles.
  - `advance_basic_attack_projectile()` and `apply_basic_attack_projectile_hit_at()` still evaluate target defenses live at impact.
  - Dead projectile attackers can use `graveyard` attack, but this is an ad hoc fallback rather than a first-class source snapshot contract.
  - `process_commands()` handles `BattleCommand::ApplyDamage` by reading source attack/owner from live units or graveyard and target defenses live at command time.
- `src/game/battle/core/sim.rs`
  - skill step damage currently emits `BattleCommand::ApplyDamage` with source id, target id, amount, damage type, modifiers, and source, but no source snapshot.
- `src/game/battle/core/skill_runtime/projectile.rs`
  - skill projectile launch/runtime stores caster id/owner and projectile metadata, but not full damage source context.
- `src/game/battle/damage.rs`
  - `DamageContext` combines source side/attack, target defenses/HP/modifiers, and on-attack/on-hit effects.
  - This makes source/target snapshot boundaries hard to enforce unless the context is split.
- `src/game/battle/timeline.rs`
  - `HpChanged` records damage result metadata.
  - No timeline event currently serializes damage source snapshot data.
- `src/game/battle/core/basic_attack.rs`
  - `AttackResolve` events currently carry only attacker id, target id, kind, and cause.
  - A committed source snapshot for instant basic attacks would need to be created before resolve-time damage, but the event does not currently carry internal snapshot data.
- `src/game/battle/enums.rs`
  - `BattleEvent::AttackResolve` has no source snapshot field. Adding one would be an internal runtime event shape change, not a serialized timeline change, but it still changes the battle scheduling contract.
- `src/game/battle/core/types.rs`
  - `ProjectileRecord` stores partial basic attack source context but not attack value, source damage modifiers, crit roll, or frozen source effects.
  - `ActiveProjectileRuntime` stores skill projectile caster id/owner and delivery metadata but no source snapshot.
- `src/game/battle/core/skill_runtime/projectile.rs`
  - Skill projectile impact rebuilds damage commands at impact by calling `build_skill_step_commands(caster_instance_id, step, &targets)`.
  - This means delayed projectile skill damage currently reads source context through `BattleCommand::ApplyDamage` at impact, not at launch.
- Final audit:
  - `BattleCommand::ApplyDamage` has no remaining old `{ source_id, amount, damage_type, modifiers, source, minimum_damage }` runtime shape.
  - Internal `BattleEvent::AttackResolve` and `SkillProjectileImpact` carry snapshots, but serialized `TimelineEvent` does not.
  - Projectile source snapshots are carried by runtime projectile records instead of recovered from live/graveyard attacker state at impact.
  - Target defense and HP are still read at apply/impact time, matching the confirmed split-context policy.
