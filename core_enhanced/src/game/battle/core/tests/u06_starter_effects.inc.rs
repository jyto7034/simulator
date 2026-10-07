fn u06_combat_effect_ron(reduction: u32, lifesteal: u32, cap: u32) -> String {
    R11_AURA.replace(
        "ChannelAura(duration_ms: 1000, aura: (family: \"fetus\", priority: 0, radius_units: 2000000, basic_damage_percent: 20, basic_speed_points: 15))",
        &format!("CombatModifiers(duration_ms: 1000, modifiers: (damage_reduction_percent: {reduction}, basic_lifesteal_percent: {lifesteal}, basic_lifesteal_max_heal: {cap}))"),
    )
}

#[test]
fn u06_combat_modifiers_public_use_reduction_expiry_and_clone() {
    use crate::game::battle::{core::sim::BattleLiveCommand, damage::{BattleCommand, DamageSource, DamageType}};
    let (mut core, mut execution, caster, _) = r11_employee_effect_core(&u06_combat_effect_ron(40, 0, 0));
    let enemy = core.units.values().find(|u| u.owner == Side::Opponent).unwrap().instance_id;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    let view = core.sustained_effect_snapshot().unwrap();
    assert_eq!(view.active[0].combat_modifiers.unwrap().damage_reduction_percent, 40);
    assert!(core.resonance_use_has_obligations(caster));
    assert_eq!(core.sustained_damage_reduction(caster, 1000).unwrap(), 40);
    assert_eq!(core.sustained_damage_reduction(caster, 1001).unwrap(), 0);
    let mut clone = crate::authoring::clone_shape::controlled_clone(&core).unwrap();
    let hp = clone.units[&caster].stats.current_health;
    let snapshot = clone.damage_source_snapshot_for_unit(enemy, caster, DamageSource::Ability, DamageType::True, 50, Default::default(), 0, 1, false).unwrap();
    clone.process_commands_with_modifier_source(vec![BattleCommand::ApplyDamage { target_id: caster, source_snapshot: snapshot }], 1, Some(enemy));
    assert_eq!(clone.units[&caster].stats.current_health, hp - 30);
    assert_eq!(core.units[&caster].stats.current_health, hp);
    clone.reconcile_sustained(1001).unwrap();
    assert!(clone.sustained_effect_snapshot().unwrap().active.is_empty());
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 1);
    core.apply_live_command(BattleLiveCommand::WithdrawUnit { unit_id: caster, time_ms: 1 }).unwrap();
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
}

#[test]
fn u06_combat_modifiers_lifesteal_keeps_source_and_uses_actual_hp_loss() {
    use crate::game::battle::{core::sim::BattleLiveCommand, damage::{DamageSource, DamageType}};
    let (mut core, mut execution, caster, _) = r11_employee_effect_core(&u06_combat_effect_ron(0, 50, 20));
    let enemy = core.units.values().find(|u| u.owner == Side::Opponent).unwrap().instance_id;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    let snapshot = core.damage_source_snapshot_for_unit(caster, enemy, DamageSource::BasicAttack, DamageType::True, 100, Default::default(), 1, 1, false).unwrap();
    assert_eq!(snapshot.basic_lifesteal.len(), 1);
    let source = snapshot.basic_lifesteal[0].fact_source.as_ref().unwrap();
    assert_eq!(source.cast.as_ref().unwrap().skill_id.as_str(), "r11_aura");
    assert!(source.authored.is_some());
    assert!(source.delivery.is_none() && !source.allows_finite_children);
    let spell = core.damage_source_snapshot_for_unit(caster, enemy, DamageSource::Ability, DamageType::True, 100, Default::default(), 0, 1, false).unwrap();
    assert!(spell.basic_lifesteal.is_empty());
    core.units.get_mut(&caster).unwrap().stats.current_health = 10;
    core.units.get_mut(&enemy).unwrap().stats.current_health = 7;
    let outcome = core.execute_committed_damage(super::super::damage_runtime::CommittedDamageInput {
        target_id: enemy, source_snapshot: snapshot.clone(), committed_at_ms: 1,
        policy: super::super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
    }).unwrap();
    assert_eq!(outcome.actual_hp_loss, 7);
    core.reconcile_sustained(1001).unwrap();
    core.finish_basic_lifesteal(&snapshot, outcome.actual_hp_loss, 1001).unwrap();
    assert_eq!(core.units[&caster].stats.current_health, 13, "released snapshot survives expiry, floor(7 * 50%)");
    core.finish_basic_lifesteal(&snapshot, 0, 1001).unwrap();
    assert_eq!(core.units[&caster].stats.current_health, 13);
    core.finish_basic_lifesteal(&snapshot, 1000, 1001).unwrap();
    assert_eq!(core.units[&caster].stats.current_health, 33, "per-effect cap");
}

#[test]
fn u06_combat_modifiers_strict_rejection_and_transaction_rollback() {
    use crate::game::battle::core::sim::BattleLiveCommand;
    for (reduction, lifesteal, cap) in [(0, 0, 0), (101, 0, 0), (0, 101, 1), (1, 0, 1), (0, 1, 0), (0, 1, u32::MAX)] {
        assert!(SkillDatabase::from_ron("u06/invalid.ron", &u06_combat_effect_ron(reduction, lifesteal, cap)).is_err());
    }
    let (mut core, _, caster, _) = r11_employee_effect_core(&u06_combat_effect_ron(10, 20, 30));
    let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot());
    let result: Result<(), GameError> = core.runtime_transaction(|candidate| {
        candidate.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 })?;
        Err(GameError::InvalidAction)
    });
    assert!(result.is_err());
    assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), before);
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
}

#[test]
fn u06_basic_heal_overlay_public_preflight_preserves_cost_and_state() {
    use crate::game::battle::core::sim::BattleLiveCommand;
    let authored = r11_overlay_ron("Projectile(speed_units_per_ms: 1000)", 1000, false);
    let (mut core, _, caster, _) = r11_equipped_effect_core(&authored, |_| {}, |weapon| {
        weapon.weapon_profile.as_mut().unwrap().action = crate::game::data::abnormality_data::BasicAttackAction::Heal {
            flat: 10, spell_power_percent: 0, footprint: None,
        };
    });
    let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot());
    assert!(core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).is_err());
    assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), before);
}
