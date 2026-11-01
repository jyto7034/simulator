// The fixture's a/b aliases are sorted runtime IDs, not spawn vector order.
// Read expected equipment from authored scenario input, independent of runtime facts.
fn declared_weapon_copy(f: &Fixture, actor: UnitInstanceId) -> Uuid {
    let employee = f.core.units[&actor].source_owned_uuid;
    let spawn = f.core.scenario.groups[1]
        .spawns
        .iter()
        .find(|s| s.draft.owned_uuid == employee)
        .unwrap();
    let copy = spawn.draft.equipped_items[0].persistent_copy_uuid.unwrap();
    assert!([Uuid::from_u128(0xe290), Uuid::from_u128(0xe291)].contains(&copy));
    copy
}

// Included in the mountain combination fixture. Expected copy IDs come from the
// independently authored spawn records, never from the observed damage source.
fn correct_equipment_source(
    source: &crate::game::battle::runtime_facts::RuntimeFactSource,
    actor: UnitInstanceId,
    copy: Uuid,
) -> bool {
    source.actor == Some(actor)
        && source.equipment.as_ref().is_some_and(|e| {
            e.definition_uuid == Uuid::from_u128(0x1101) && e.persistent_copy_uuid == Some(copy)
        })
        && source.cast.is_none()
        && source.fragment.is_none()
}

fn correct_basic_source(
    source: &crate::game::battle::runtime_facts::RuntimeFactSource,
    actor: UnitInstanceId,
    copy: Uuid,
) -> bool {
    correct_equipment_source(source, actor, copy) && source.authored.is_none()
}

fn expected_bonus_file() -> String {
    let source = SOURCE.replace("authored-mountain", "r11_aura");
    let artifact = crate::authoring::composition::compile_yaml(&source).unwrap();
    format!("composition:mountain-combination-fixture:{}", artifact.source_digest())
}

fn correct_bonus_source(
    source: &crate::game::battle::runtime_facts::RuntimeFactSource,
    actor: UnitInstanceId,
    copy: Uuid,
    file: &str,
) -> bool {
    correct_equipment_source(source, actor, copy)
        && source.authored.as_ref().is_some_and(|authored| {
            authored.source_file == file
                && authored.field_path == "equipment_attacks[0].program.bonus.amount"
                && authored.site.action.definition_id.as_str() == "equipment:r11_weapon"
                && authored.site.action.action_id == "basic_attack"
                && authored.site.effect_id == "bonus.amount"
                && authored.origin_definition_id.as_str() == "equipment:r11_weapon"
                && authored.origin == crate::game::ability::authoring::EffectOrigin::Original
        })
}

#[test]
fn composition_basic_weapon_source_area_copies_oracle_and_observation_parity() {
    let file = expected_bonus_file();
    let mut results = Vec::new();
    for observe in [false, true] {
        for second_actor in [false, true] {
            let mut f = fixture(&[], true, 3);
            if second_actor {
                std::mem::swap(&mut f.a, &mut f.b);
                super::super::compiled_action_fixture::place(&mut f.core, f.a, 2, 2);
                super::super::compiled_action_fixture::place(&mut f.core, f.b, 7, 3);
            }
            if observe {
                f.core.enable_authoring_observation();
            }
            let copy = declared_weapon_copy(&f, f.a);
            let other_copy = declared_weapon_copy(&f, f.b);
            assert_ne!(copy, other_copy);
            attack(&mut f, 10, true);
            f.core
                .step_battle_execution_until(&mut f.state, 12)
                .unwrap();
            let facts = f.core.runtime_fact_view();
            for target in [f.enemy, f.secondary] {
                let hit = facts
                    .actors()
                    .find(|(id, _)| **id == target)
                    .unwrap()
                    .1
                    .received
                    .latest_nonzero_damage
                    .as_ref()
                    .unwrap();
                assert!(correct_basic_source(&hit.source, f.a, copy), "{hit:?}");
                assert_eq!(
                    hit.actual_hp_loss(),
                    if target == f.enemy {
                        if second_actor {
                            100
                        } else {
                            130
                        }
                    } else if second_actor {
                        40
                    } else {
                        52
                    }
                );
                let mut bad = hit.source.as_ref().clone();
                bad.equipment = None;
                assert!(!correct_basic_source(&bad, f.a, copy));
                assert_eq!(hit.modifier_sources.len(), usize::from(!second_actor));
                if !second_actor {
                    let bonus = &hit.modifier_sources[0];
                    assert!(correct_bonus_source(bonus, f.a, copy, &file), "{bonus:?}");
                    assert_eq!(bonus.action_ordinal(), hit.source.action_ordinal());
                    let mut mixed = hit.source.as_ref().clone();
                    mixed.authored = bonus.authored.clone();
                    assert!(!correct_basic_source(&mixed, f.a, copy));
                    for mutation in 0..6 {
                        let mut bad = bonus.clone();
                        match mutation {
                            0 => bad.authored = None,
                            1 => Arc::make_mut(bad.authored.as_mut().unwrap()).field_path = "equipment_attacks[1].program.bonus.amount".into(),
                            2 => Arc::make_mut(bad.authored.as_mut().unwrap()).source_file = "another-generation.skill.yaml".into(),
                            3 => bad.equipment.as_mut().unwrap().persistent_copy_uuid = Some(other_copy),
                            4 => bad.actor = Some(f.b),
                            _ => Arc::make_mut(bad.authored.as_mut().unwrap()).origin =
                                crate::game::ability::authoring::EffectOrigin::Added {
                                    source_definition_id: "unrelated-relic".into(),
                                },
                        }
                        assert!(!correct_bonus_source(&bad, f.a, copy, &file));
                    }
                }
                let mut bad = hit.source.as_ref().clone();
                bad.equipment.as_mut().unwrap().persistent_copy_uuid = Some(other_copy);
                assert!(!correct_basic_source(&bad, f.a, copy));
                let mut bad = hit.source.as_ref().clone();
                bad.actor = Some(f.b);
                assert!(!correct_basic_source(&bad, f.a, copy));
            }
            results.push((
                second_actor,
                serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
                f.core.units[&f.enemy].stats.current_health,
                f.core.units[&f.secondary].stats.current_health,
            ));
        }
    }
    assert_eq!(results[0], results[2]);
    assert_eq!(results[1], results[3]);
}

#[test]
fn composition_basic_weapon_source_projectile_retains_copy_after_withdrawal() {
    use crate::game::ability::{DeliveryDef, ProjectileHitPolicy};
    let mut f = fixture(&[], true, 3);
    let copy = declared_weapon_copy(&f, f.a);
    f.core.units.get_mut(&f.a).unwrap().basic_attack.delivery = DeliveryDef::Projectile {
        speed_units_per_ms: 1000,
        hit_policy: ProjectileHitPolicy::TargetLocked,
        allow_targetless_cast: false,
        max_range_tiles: None,
        max_lifetime_ms: None,
        max_kills: None,
        max_pierces: None,
        collision: Default::default(),
    };
    attack(&mut f, 10, true);
    f.core
        .step_battle_execution_until(&mut f.state, 12)
        .unwrap();
    let projectiles: Vec<_> = f
        .core
        .projectiles
        .values()
        .filter(|p| p.source_snapshot.source_id == f.a)
        .collect();
    assert!(!projectiles.is_empty());
    for p in projectiles {
        assert!(correct_basic_source(
            &p.source_snapshot.fact_source,
            f.a,
            copy
        ));
        let capture = p.source_snapshot.equipment_attack.as_ref().unwrap();
        assert_eq!(capture.equipment.persistent_copy_uuid, Some(copy));
        assert_eq!(capture.bonus, 30);
    }
    f.core
        .apply_live_command(BattleLiveCommand::WithdrawUnit {
            unit_id: f.a,
            time_ms: 12,
        })
        .unwrap();
    f.core
        .step_battle_execution_until(&mut f.state, 1200)
        .unwrap();
    let view = f.core.runtime_fact_view();
    for target in [f.enemy, f.secondary] {
        let hit = view
            .actors()
            .find(|(id, _)| **id == target)
            .unwrap()
            .1
            .received
            .latest_nonzero_damage
            .as_ref()
            .unwrap();
        assert!(correct_basic_source(&hit.source, f.a, copy));
        assert_eq!(hit.modifier_sources.len(), 1);
        assert!(correct_bonus_source(&hit.modifier_sources[0], f.a, copy, &expected_bonus_file()));
        assert_eq!(
            hit.actual_hp_loss(),
            if target == f.enemy { 130 } else { 52 }
        );
    }
}

#[test]
fn composition_basic_weapon_source_cancel_start_restores_then_retries() {
    use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
    for (site, until) in [("basic_attack_weapon_source", 10), ("equipment_attack_damage_source_ready", 12)] {
    let mut f = fixture(&[], true, 3);
    let copy = declared_weapon_copy(&f, f.a);
    f.core.units.get_mut(&f.a).unwrap().next_basic_attack_ms = 10;
    let snap = |core: &BattleCore| {
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
        )
    };
    let event = BattleEvent::AttackStart {
        time_ms: 10,
        attacker_instance_id: f.a,
        target_instance_id: Some(f.enemy),
        schedule_next: true,
        cause: Default::default(),
    };
    f.core.event_queue.push(event);
    f.core.step_battle_execution_until(&mut f.state, 9).unwrap();
    if until == 12 {
        f.core.step_battle_execution_until(&mut f.state, 10).unwrap();
    }
    let before = (snap(&f.core), format!("{:?}", f.state));
    let token = CancellationToken::default();
    let result = control::with_test_cancellation(site, 1, || {
        control::run(
            &ExecutionLimits::default(),
            &token,
            ExecutionPhase::TrialExecution,
            || f.core.step_battle_execution_until(&mut f.state, until),
        )
    });
    assert!(result.is_err() && token.is_cancelled());
    assert_eq!((snap(&f.core), format!("{:?}", f.state)), before);
    f.core
        .step_battle_execution_until(&mut f.state, 12)
        .unwrap();
    let view = f.core.runtime_fact_view();
    let hit = view
        .actors()
        .find(|(id, _)| **id == f.enemy)
        .unwrap()
        .1
        .received
        .latest_nonzero_damage
        .as_ref()
        .unwrap();
    assert!(correct_basic_source(&hit.source, f.a, copy));
    assert_eq!(hit.actual_hp_loss(), 130);
    assert_eq!(hit.modifier_sources.len(), 1);
    assert!(correct_bonus_source(&hit.modifier_sources[0], f.a, copy, &expected_bonus_file()));
    }
}

#[test]
fn composition_basic_weapon_source_two_weapons_reject_and_preserve_then_retry() {
    let mut f = fixture(&[], true, 3);
    let second = *f.core.items.iter().find(|(_, item)| item.owner_unit_instance == f.b).unwrap().0;
    // Deliberately unsupported corruption, never a positive loadout fixture.
    f.core.items.get_mut(&second).unwrap().owner_unit_instance = f.a;
    f.core.units.get_mut(&f.a).unwrap().next_basic_attack_ms = 10;
    f.core.event_queue.push(BattleEvent::AttackStart {
        time_ms: 10, attacker_instance_id: f.a, target_instance_id: Some(f.enemy),
        schedule_next: true, cause: Default::default(),
    });
    f.core.step_battle_execution_until(&mut f.state, 9).unwrap();
    let snap = |f: &Fixture| (f.core.transaction_debug_snapshot(), f.core.event_queue_debug_snapshot(),
        serde_json::to_value(f.core.runtime_fact_view()).unwrap(), format!("{:?}", f.state));
    let before = snap(&f);
    let error = f.core.step_battle_execution_until(&mut f.state, 10).unwrap_err();
    assert!(format!("{error:?}").contains("basic_attack_multiple_weapons"), "{error:?}");
    assert_eq!(snap(&f), before);
    f.core.items.get_mut(&second).unwrap().owner_unit_instance = f.b;
    f.core.step_battle_execution_until(&mut f.state, 12).unwrap();
    let view = f.core.runtime_fact_view();
    let hit = view.actors().find(|(id, _)| **id == f.enemy).unwrap().1.received.latest_nonzero_damage.as_ref().unwrap();
    assert!(correct_basic_source(&hit.source, f.a, declared_weapon_copy(&f, f.a)));
    assert_eq!(hit.actual_hp_loss(), 130);
}

#[test]
fn composition_basic_weapon_source_saturated_base_does_not_claim_bonus() {
    let mut f = fixture(&[], true, 3);
    f.core.units.get_mut(&f.a).unwrap().stats.attack = u32::MAX;
    attack(&mut f, 10, true);
    f.core.step_battle_execution_until(&mut f.state, 12).unwrap();
    let view = f.core.runtime_fact_view();
    let hit = view.actors().find(|(id, _)| **id == f.enemy).unwrap().1.received.latest_nonzero_damage.as_ref().unwrap();
    assert!(correct_basic_source(&hit.source, f.a, declared_weapon_copy(&f, f.a)));
    assert_eq!(hit.requested_damage, u32::MAX);
    assert!(hit.modifier_sources.is_empty(), "a saturated base receives no mass contribution");
}
