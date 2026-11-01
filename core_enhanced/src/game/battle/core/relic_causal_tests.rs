//! Independent native probes complement normal World and external inputs.
//! Fixtures supply equipment and actors; deaths still originate in the normal
//! committed damage consumer and dispatch their real equipment occurrences.
use super::*;
use crate::game::{
    ability::{AbilityActivationBinding, AbilityActivationDef, SkillId},
    battle::{
        damage::{BattleCommand, DamageSource, DamageType},
        event_log::BattleLogEvent,
        scenario::BattleScenario,
    },
    data::{
        equipment_data::{EquipmentMetadata, EquipmentType},
        skill_data::SkillDatabase,
        GameDataBuilder,
    },
    enums::{RiskLevel, Side},
    stats::{
        Effect, StatId, StatModifier, StatModifierKind, TriggerEffectTarget, TriggerType,
        TriggeredEffect,
    },
};

/// Native item placement for a published equipment definition. The World
/// fixtures still publish, grant, equip and deploy their relics normally.
pub(crate) fn joint_install_published_equipment(
    core: &mut BattleCore,
    owner: UnitInstanceId,
    definition: Uuid,
    instance: Uuid,
) {
    assert!(core
        .game_data
        .equipment_data
        .get_by_uuid(&definition)
        .is_some());
    core.items.insert(
        instance,
        RuntimeItem {
            weapon_stage: Default::default(),
            persistent_copy_uuid: None,
            instance_id: instance,
            owner: core.units[&owner].owner,
            owner_unit_instance: owner,
            base_uuid: definition,
        },
    );
}

pub(crate) fn joint_causal_owner_present(core: &BattleCore) -> bool {
    core.runtime_facts.causal.is_some()
}

pub(crate) fn joint_committed_damage(
    core: &mut BattleCore,
    actor: UnitInstanceId,
    target: UnitInstanceId,
    amount: u32,
    now: u64,
) {
    core.runtime_transaction(|candidate| {
        let damage = candidate
            .damage_source_snapshot_for_unit(
                actor,
                target,
                DamageSource::Ability,
                DamageType::True,
                amount,
                Default::default(),
                0,
                now,
                false,
            )
            .unwrap();
        candidate.process_commands(
            vec![BattleCommand::ApplyDamage {
                target_id: target,
                source_snapshot: damage,
            }],
            now,
        );
        crate::game::battle::runtime_control::poll()
    })
    .unwrap();
}

/// Observe actual queued carriers; do not synthesize a context or invocation.
pub(crate) fn joint_pending_causal_rules(core: &BattleCore) -> Vec<String> {
    let mut rules = core
        .event_queue
        .iter_queued()
        .filter_map(|queued| {
            queued
                .fact_source
                .causal()
                .and_then(|source| source.cause.as_ref())
                .map(|cause| cause.rule_id.clone())
        })
        .collect::<Vec<_>>();
    rules.sort();
    rules
}

/// The native comparison drains the original queued envelopes, including their
/// actual source contexts. Movement ticks are outside this finite reaction
/// consumer probe; the separate World case advances normal host buckets/waves.
pub(crate) fn joint_drain_finite_reactions(core: &mut BattleCore, until: u64) {
    let mut consumed = 0;
    while core
        .event_queue
        .peek_time_ms()
        .is_some_and(|time| time <= until)
    {
        let queued = core.event_queue.pop().unwrap();
        consumed += 1;
        assert!(consumed < 1000, "finite reaction queue must terminate");
        if matches!(
            queued.event.view().as_ref(),
            crate::game::battle::enums::BattleEvent::ContinuousMovementTick { .. }
        ) {
            continue;
        }
        let time = queued.time_ms();
        core.process_event(queued, time).unwrap();
    }
}

pub(crate) fn evict_all_existing_log_rows(core: &mut BattleCore, now: u64) {
    use crate::game::battle::event_log::BATTLE_HISTORY_MAX_EVENTS;
    let old_last = core.event_log.latest_seq();
    for _ in 0..=BATTLE_HISTORY_MAX_EVENTS {
        core.record_event_log(
            now,
            BattleLogEvent::BattleStart {
                width: 8,
                height: 8,
            },
        );
    }
    assert_eq!(core.event_log.entries.len(), BATTLE_HISTORY_MAX_EVENTS);
    assert!(
        core.event_log.floor_seq() > old_last,
        "the bounded log actually evicted every earlier causal correlation row"
    );
}

pub(crate) fn actual_damage_transaction(
    core: &mut BattleCore,
    actor: UnitInstanceId,
    target: UnitInstanceId,
) -> Result<(), crate::game::behavior::GameError> {
    core.runtime_transaction(|candidate| {
        let damage = candidate
            .damage_source_snapshot_for_unit(
                actor,
                target,
                DamageSource::Ability,
                DamageType::True,
                1,
                Default::default(),
                0,
                0,
                false,
            )
            .unwrap();
        candidate.process_commands(
            vec![BattleCommand::ApplyDamage {
                target_id: target,
                source_snapshot: damage,
            }],
            0,
        );
        crate::game::battle::runtime_control::poll()
    })
}

pub(crate) fn actual_basic_attack_transaction(
    core: &mut BattleCore,
    actor: UnitInstanceId,
    target: UnitInstanceId,
    now: u64,
) -> Result<(), crate::game::behavior::GameError> {
    // Explicit range preparation for this committed attack only. Leaving it in
    // place would let later World AI ticks schedule unrelated basic actions.
    let previous_range = core.units[&actor].basic_attack.range_policy;
    core.units
        .get_mut(&actor)
        .unwrap()
        .basic_attack
        .range_policy = crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
    let outcome = core.runtime_transaction(|candidate| {
        if !candidate.resolve_basic_attack(actor, target, now) {
            return Err(crate::game::behavior::GameError::InvalidAction);
        }
        crate::game::battle::runtime_control::poll()
    });
    core.units
        .get_mut(&actor)
        .unwrap()
        .basic_attack
        .range_policy = previous_range;
    outcome
}

pub(crate) fn prepare_existing_invulnerability(
    core: &mut BattleCore,
    source: UnitInstanceId,
    target: UnitInstanceId,
) {
    core.begin_sustained(
        target,
        target,
        crate::game::ability::sustained::SustainedEffectDef::Parry {
            duration_ms: 1000,
            window_ms: 100,
            invulnerable_ms: 100,
            miss_recovery_ms: 10,
            reduction_percent: 0,
            counter_skill: "u10_strike".into(),
            end_on_resolution: false,
            guard_charge: None, fragment_guard: None,
        },
        0,
    )
    .unwrap();
    let incoming = core
        .damage_source_snapshot_for_unit(
            source,
            target,
            DamageSource::Ability,
            DamageType::Physical,
            1,
            Default::default(),
            0,
            0,
            false,
        )
        .unwrap();
    let deflection = core
        .deflect_direct_strike(target, &incoming, 0)
        .unwrap()
        .unwrap();
    assert_eq!(
        deflection.kind,
        crate::game::battle::runtime_facts::DeflectionKind::Parry
    );
    let snapshot = core.sustained_effect_snapshot().unwrap();
    let parry = snapshot
        .active
        .iter()
        .find(|entry| entry.target == target && entry.parry.is_some())
        .unwrap()
        .parry
        .as_ref()
        .unwrap();
    assert_eq!(parry.invulnerable_until_ms, Some(100));
}

#[test]
fn u10_f_dead_owner_damage_creates_new_victim_death_and_preserves_equipment_eligibility() {
    let skills = SkillDatabase::from_ron("u10/equipment_death.ron", r#"SkillDatabase(skills:[
        SkillDef(id:"u10_death_burst",name:"dead owner burst",kind:Targeted,
            focus_time_ms:0,use_lifetime:Action,
            cast_targeting:Explicit(selection:automatic_unit,target:EnemySingle(rule:Nearest),reach:global,air_capable:true),
            steps:[(id:"burst",delay_ms:0,target:CastTarget,delivery:Instant,
                effects:[(id:"burst",op:Damage(amount:100,damage_type:True))])]),
        SkillDef(id:"u10_death_marker",name:"dead owner occurrence",kind:Untargeted,
            focus_time_ms:0,use_lifetime:Action,
            cast_targeting:Explicit(selection:self,target:SelfUnit,reach:pattern,air_capable:false),
            steps:[(id:"marker",delay_ms:0,target:SelfUnit,delivery:Instant,
                effects:[(id:"marker",op:Heal(amount:1))])]),
        SkillDef(id:"u10_ally_marker",name:"living ally occurrence",kind:Untargeted,
            focus_time_ms:0,use_lifetime:Action,
            cast_targeting:Explicit(selection:self,target:SelfUnit,reach:pattern,air_capable:false),
            steps:[(id:"marker",delay_ms:0,target:SelfUnit,delivery:Instant,
                effects:[(id:"marker",op:Heal(amount:1))])]),
        SkillDef(id:"u10_kill_marker",name:"live killer ability only",kind:Untargeted,
            focus_time_ms:0,use_lifetime:Action,
            cast_targeting:Explicit(selection:self,target:SelfUnit,reach:pattern,air_capable:false),
            steps:[(id:"marker",delay_ms:0,target:SelfUnit,delivery:Instant,
                effects:[(id:"marker",op:Heal(amount:1))])])
    ])"#).unwrap();
    let (seed, _, killer, first_victim, player_ally) =
        tests::native_action_fixture(skills.skills[0].clone(), Default::default());
    let activation = |trigger, id: &str| AbilityActivationBinding {
        ability_id: SkillId::from(id),
        activation: AbilityActivationDef::TriggerProc {
            trigger,
            proc_chance_percent: 100,
            internal_cooldown_ms: 0,
            max_triggers_per_battle: None,
        },
    };
    let modifier = |target, amount| {
        TriggeredEffect::targeted(
            target,
            Effect::Modifier(StatModifier {
                stat: StatId::Attack,
                kind: StatModifierKind::Flat,
                value: amount,
            }),
        )
    };
    let burst_id = Uuid::from_u128(0xd201);
    let observer_id = Uuid::from_u128(0xd202);
    let equipment = |id: &str, uuid, death_skill: &str| EquipmentMetadata {
        content_family: None,
        id: id.into(),
        uuid,
        name: id.into(),
        equipment_type: EquipmentType::Weapon,
        rarity: RiskLevel::ZAYIN,
        price: 0,
        allow_duplicate_equip: true,
        bound: false,
        cannot_unequip_reason: "bound".into(),
        weapon_profile: Some(Default::default()),
        triggered_effects: std::collections::HashMap::from([
            (
                TriggerType::OnKill,
                vec![modifier(TriggerEffectTarget::AllAllies, 3)],
            ),
            (
                TriggerType::OnAllyDeath,
                vec![modifier(TriggerEffectTarget::SelfUnit, 2)],
            ),
        ]),
        ability_activations: vec![
            activation(TriggerType::OnDeath, death_skill),
            activation(TriggerType::OnKill, "u10_kill_marker"),
            activation(TriggerType::OnAllyDeath, "u10_ally_marker"),
        ],
        event_check_modifiers: vec![],
        event_choice_keys: vec![],
    };
    let data = GameDataBuilder::from_validated(&crate::game::data::ValidatedGameData::from(
        seed.game_data.clone(),
    ))
    .with_skills(skills)
    .with_equipment(vec![
        equipment("u10_burst_item", burst_id, "u10_death_burst"),
        equipment("u10_observer_item", observer_id, "u10_death_marker"),
    ])
    .build_with_runtime_safety(crate::runtime_safety::RuntimeSafetySettings::embedded_shared())
    .unwrap();
    let mut core =
        BattleCore::new_from_scenario(BattleScenario::empty((8, 8)), Arc::new(data), 0xd203);
    core.units = seed.units.clone();
    let second_player_ally: UnitInstanceId = Uuid::from_u128(0xfa04).into();
    let opponent_ally: UnitInstanceId = Uuid::from_u128(0xfa05).into();
    for (new_id, owner) in [
        (second_player_ally, Side::Player),
        (opponent_ally, Side::Opponent),
    ] {
        let mut unit = core.units[&player_ally].clone();
        unit.instance_id = new_id;
        unit.source_owned_uuid = new_id.as_uuid();
        unit.owner = owner;
        core.units.insert(new_id, unit);
    }
    for (index, id) in [
        killer,
        first_victim,
        player_ally,
        second_player_ally,
        opponent_ally,
    ]
    .into_iter()
    .enumerate()
    {
        let x = if index < 2 {
            index as i32
        } else {
            index as i32 + 2
        };
        relic_structural_tests::flow_cases::place_stationary(&mut core, id, x, 0);
        let instance_id = Uuid::from_u128(0xd210 + index as u128);
        core.items.insert(
            instance_id,
            RuntimeItem {
            weapon_stage: Default::default(),
                persistent_copy_uuid: None,
                instance_id,
                owner: core.units[&id].owner,
                owner_unit_instance: id,
                base_uuid: if id == first_victim {
                    burst_id
                } else {
                    observer_id
                },
            },
        );
    }
    let damage = core
        .damage_source_snapshot_for_unit(
            killer,
            first_victim,
            DamageSource::Ability,
            DamageType::True,
            100,
            Default::default(),
            0,
            10,
            false,
        )
        .unwrap();
    core.process_commands(
        vec![
            BattleCommand::ApplyDamage {
                target_id: first_victim,
                source_snapshot: damage.clone(),
            },
            BattleCommand::ApplyDamage {
                target_id: first_victim,
                source_snapshot: damage,
            },
        ],
        10,
    );
    while let Some(queued) = core.event_queue.pop() {
        if matches!(
            queued.event.view().as_ref(),
            crate::game::battle::enums::BattleEvent::ContinuousMovementTick { .. }
        ) {
            continue;
        }
        let time = queued.time_ms();
        core.process_event(queued, time).unwrap();
    }
    assert_eq!(core.units[&first_victim].stats.current_health, 0);
    assert_eq!(core.units[&killer].stats.current_health, 0);
    let deaths: Vec<_> = core
        .event_log
        .entries
        .iter()
        .filter_map(|entry| match entry.event {
            BattleLogEvent::UnitDied {
                unit_instance_id,
                killer_instance_id,
                ..
            } => Some((unit_instance_id, killer_instance_id)),
            _ => None,
        })
        .collect();
    assert_eq!(
        deaths,
        vec![(first_victim, Some(killer)), (killer, Some(first_victim))]
    );
    for ally in [player_ally, second_player_ally, opponent_ally] {
        assert!(core.units[&ally].is_active());
        assert_eq!(
            core.units[&ally].stats.attack, 5,
            "one actual ally death +2 and the owner's collected OnKill all-allies +3"
        );
    }
    let casts: Vec<_> = core
        .event_log
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            BattleLogEvent::AbilityCast {
                skill_id,
                caster_instance_id,
                ..
            } => Some((skill_id.as_str().to_owned(), *caster_instance_id)),
            _ => None,
        })
        .collect();
    assert_eq!(
        casts
            .iter()
            .filter(|(id, actor)| id == "u10_death_burst" && *actor == first_victim)
            .count(),
        1
    );
    assert_eq!(
        casts
            .iter()
            .filter(|(id, actor)| id == "u10_death_marker" && *actor == killer)
            .count(),
        1
    );
    let kill_marker_casters: Vec<_> = casts
        .iter()
        .filter_map(|(id, actor)| (id == "u10_kill_marker").then_some(*actor))
        .collect();
    assert_eq!(
        kill_marker_casters,
        vec![killer],
        "the first killer is alive at its kill; the already dead first victim cannot execute OnKill for the subsequent kill"
    );
    let player_ally_casts: Vec<_> = casts
        .iter()
        .filter_map(|(id, actor)| {
            (id == "u10_ally_marker" && core.units[actor].owner == Side::Player).then_some(*actor)
        })
        .collect();
    assert_eq!(player_ally_casts, vec![player_ally, second_player_ally]);
}
