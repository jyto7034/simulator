//! Native observation/preparation seams for independently authored World Flow
//! cases. These helpers do not manufacture a plan, cast, result or authority.
use super::*;
use crate::game::battle::runtime_facts::RuntimeFactSource;

pub(crate) fn legacy_shield(core: &mut BattleCore, target: UnitInstanceId, amount: u32) {
    core.begin_sustained(
        target,
        target,
        crate::game::ability::sustained::SustainedEffectDef::Shield {
            duration_ms: 1000,
            amount,
            priority: 0,
            shared_radius_units: None,
        },
        0,
    )
    .unwrap();
}

pub(crate) fn prepare_existing_invulnerability(
    core: &mut BattleCore,
    source: UnitInstanceId,
    target: UnitInstanceId,
) {
    // Component preparation only: arm the native parry's invulnerability
    // window via its real deflection consumer. Counter invocation is outside
    // this setup seam. The test then runs actual authored damage and checks
    // its receipt without manufacturing HpLost or a structural Flow result.
    core.begin_sustained(
        target,
        target,
        crate::game::ability::sustained::SustainedEffectDef::Parry {
            duration_ms: 1000,
            window_ms: 100,
            invulnerable_ms: 100,
            miss_recovery_ms: 10,
            reduction_percent: 0,
            counter_skill: "tc_bite".into(),
            end_on_resolution: false,
            guard_charge: None, fragment_guard: None,
        },
        0,
    )
    .unwrap();
    let strike = core
        .damage_source_snapshot_for_unit(
            source,
            target,
            crate::game::battle::damage::DamageSource::Ability,
            crate::game::battle::damage::DamageType::Physical,
            1,
            Default::default(),
            0,
            0,
            false,
        )
        .unwrap();
    let deflection = core
        .deflect_direct_strike(target, &strike, 0)
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
        .find(|row| row.target == target && row.parry.is_some())
        .unwrap()
        .parry
        .as_ref()
        .unwrap();
    assert_eq!(parry.invulnerable_until_ms, Some(100));
}

pub(crate) fn active_cast_count(core: &BattleCore) -> usize {
    core.active_skill_casts.len()
}

pub(crate) fn observe_amounts(core: &mut BattleCore) {
    core.enable_authoring_observation_with_options(Default::default(), true);
    core.bind_authoring_observation(authoring_observation::ObservationKey {
        entry_ordinal: 91,
        battle_uuid: Uuid::from_u128(0xc090),
    })
    .unwrap();
}

pub(crate) fn latest_amount_source(core: &BattleCore) -> RuntimeFactSource {
    core.authoring_observation
        .as_ref()
        .unwrap()
        .latest_amount
        .as_ref()
        .unwrap()
        .source
        .as_ref()
        .clone()
}

pub(crate) fn source_is_authorized(core: &BattleCore, source: &RuntimeFactSource) -> bool {
    source.action.as_ref().is_some_and(|action| {
        core.runtime_facts
            .validate_action_source(action, source)
            .is_ok()
    })
}

pub(crate) fn place_stationary(core: &mut BattleCore, id: UnitInstanceId, x: i32, y: i32) {
    let position = Position::new(x, y);
    core.battlefield.ensure_walkable_tile(position).unwrap();
    core.active_movement_segments.remove(&id);
    let unit = core.units.get_mut(&id).unwrap();
    unit.set_world_position(movement::types::WorldVec2::from_tile_center(position));
    unit.body.move_speed = 0.0;
    unit.move_epoch = unit.move_epoch.wrapping_add(1);
    unit.action_state = movement::ActionState::Idle;
    unit.enemy_movement_plan = None;
    unit.facing_direction = Some(crate::game::battle::tile_range::FacingDirection::Right);
}

pub(crate) fn hard_cc_cancel(core: &mut BattleCore, actor: UnitInstanceId, now: u64) {
    // Exercise the same cancellation callback used by the Stun consumer.
    core.cancel_unreleased_actions_for_hard_cc(now, actor, actor);
}

pub(crate) fn place_stationary_x(core: &mut BattleCore, actor: UnitInstanceId, x: f32) {
    core.active_movement_segments.remove(&actor);
    let unit = core.units.get_mut(&actor).unwrap();
    let mut position = unit.body.position;
    position.x = x;
    unit.set_world_position(position);
    unit.move_epoch = unit.move_epoch.wrapping_add(1);
}

pub(crate) fn refill_skill(core: &mut BattleCore, actor: UnitInstanceId, now: u64) {
    core.add_resonance(actor, 100, now, false);
}

pub(crate) fn support_bonus(core: &BattleCore, actor: UnitInstanceId, regular: bool) -> (u32, u32) {
    core.aura_basic_bonus_for_occurrence(actor, regular)
        .unwrap()
}

pub(crate) fn channel_active(core: &BattleCore, actor: UnitInstanceId) -> bool {
    core.channel_active(actor)
}

pub(crate) fn flow_roots(core: &BattleCore) -> serde_json::Value {
    serde_json::json!({
        "transaction": core.transaction_debug_snapshot(),
        "queue": core.event_queue_debug_snapshot(),
        "casts": format!("{:?}", core.active_skill_casts),
        "areas": format!("{:?}", core.active_areas),
        "facts": core.runtime_fact_usage(),
        "log": core.event_log.to_json_string().unwrap(),
        "seed": core.seed,
    })
}

pub(crate) fn effective_owner_identity(core: &BattleCore) -> Option<usize> {
    core.effective_owner
        .as_ref()
        .map(|owner| std::sync::Arc::as_ptr(owner) as usize)
}

pub(crate) fn selected_flow_diagnostic(core: &BattleCore) -> serde_json::Value {
    use crate::game::battle::enums::BattleEvent;
    let casts: Vec<_> = core
        .active_skill_casts
        .iter()
        .map(|(id, cast)| {
            serde_json::json!({
                "cast": id, "skill": cast.skill_id, "cancelled": cast.control_cancelled,
                "total_steps": cast.total_steps, "progress": format!("{:?}", cast.step_progress),
                "flow": cast.flow.as_ref().map(|flow| serde_json::json!({
                    "complete": flow.complete, "use_control": flow.has_use_control(),
                    "pending_sides": flow.has_pending_sides(),
                    "accepted_target": format!("{:?}", flow.accepted_target()),
                    "nodes": format!("{:?}", flow.nodes),
                })),
            })
        })
        .collect();
    let uses: Vec<_> = core
        .units
        .values()
        .filter_map(|unit| {
            unit.resonance_use.as_ref().map(|state| {
                serde_json::json!({
                    "actor": unit.instance_id, "use": format!("{state:?}"),
                })
            })
        })
        .collect();
    let queue: Vec<_> = core
        .event_queue
        .clone()
        .into_vec()
        .into_iter()
        .filter_map(|queued| {
            let event = queued.event.view();
            matches!(
                event.as_ref(),
                BattleEvent::SkillStep { .. }
                    | BattleEvent::SkillFlowWake { .. }
                    | BattleEvent::SustainedExpiry { .. }
            )
            .then(|| format!("{event:?}"))
        })
        .collect();
    let events: Vec<_> = core
        .event_log
        .entries
        .iter()
        .map(|entry| format!("{entry:?}"))
        .collect();
    serde_json::json!({"casts": casts, "uses": uses, "queue": queue,
        "sustained": format!("{:?}", core.sustained_effect_snapshot().unwrap()), "events": events})
}

pub(crate) fn damage_probe(
    core: &mut BattleCore,
    source: UnitInstanceId,
    target: UnitInstanceId,
    amount: u32,
    now: u64,
) {
    use crate::game::battle::damage::{BattleCommand, DamageSource, DamageType};
    let damage = core
        .damage_source_snapshot_for_unit(
            source,
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
    core.process_commands_with_modifier_source(
        vec![BattleCommand::ApplyDamage {
            target_id: target,
            source_snapshot: damage,
        }],
        now,
        Some(source),
    );
}

pub(crate) fn effect_history_counts(core: &BattleCore) -> Vec<(usize, usize, usize)> {
    core.active_areas
        .values()
        .flat_map(|area| area.effective_history.iter())
        .flat_map(|rows| {
            rows.iter().map(|(index, row)| {
                (
                    *index,
                    row.hit_unit_ids.len(),
                    row.previous_tick_unit_ids.len(),
                )
            })
        })
        .collect()
}

pub(crate) fn observe_amounts_mode(core: &mut BattleCore, mode: usize) {
    use crate::authoring::trace::{TraceLimits, TraceOptions};
    let limits = match mode {
        0 => None,
        1 => Some(TraceLimits::default()),
        2 => Some(TraceLimits {
            max_bytes: 1,
            max_row_bytes: 1,
            ..Default::default()
        }),
        _ => unreachable!(),
    };
    core.enable_authoring_observation_with_options(TraceOptions { limits }, true);
    core.bind_authoring_observation(authoring_observation::ObservationKey {
        entry_ordinal: 91,
        battle_uuid: Uuid::from_u128(0xc090),
    })
    .unwrap();
}

pub(crate) fn amount_receipt(core: &BattleCore) -> serde_json::Value {
    let observation = core.authoring_observation.as_ref().unwrap();
    let amount = observation.latest_amount.as_ref().unwrap();
    serde_json::json!({
        "evaluations": observation.amount_evaluations,
        "amount": format!("{:?}", amount.amount),
        "usage": format!("{:?}", amount.runtime_usage),
    })
}

pub(crate) fn recharge_personal(
    core: &mut BattleCore,
    actor: UnitInstanceId,
    now: u64,
) -> Result<(), GameError> {
    core.runtime_transaction(|core| {
        core.recharge_sustained_shield(actor, &"tc_bite".into(), 100, now)
    })
}

pub(crate) fn start_basic_for_support(
    core: &mut BattleCore,
    attacker: UnitInstanceId,
    target: UnitInstanceId,
    regular: bool,
    now: u64,
) -> (
    UnitInstanceId,
    crate::game::battle::damage::DamageSourceSnapshot,
) {
    prepare_support_attacker(core, attacker);
    core.process_event_for_test(
        BattleEvent::AttackStart {
            time_ms: now,
            attacker_instance_id: attacker,
            target_instance_id: Some(target),
            schedule_next: regular,
            cause: Default::default(),
        },
        now,
    )
    .unwrap();
    core.event_queue
        .iter()
        .find_map(|event| match event.as_ref() {
            BattleEvent::AttackResolve {
                attacker_instance_id,
                target_instance_id,
                source_snapshot,
                ..
            } if *attacker_instance_id == attacker => {
                Some((*target_instance_id, source_snapshot.clone()))
            }
            _ => None,
        })
        .expect("normal AttackStart emitted released damage owner")
}

pub(crate) fn support_contributors(
    source: &crate::game::battle::damage::DamageSourceSnapshot,
) -> Vec<(u32, u32)> {
    source
        .fact_source
        .regular_support()
        .map_or_else(Vec::new, |support| {
            support
                .contributors
                .iter()
                .map(|contributor| {
                    assert!(contributor.source.action.is_none());
                    assert!(contributor.source.delivery.is_none());
                    assert!(!contributor.source.use_control);
                    assert!(!contributor.source.allows_finite_children);
                    assert!(contributor.source.resource_delivery().is_none());
                    (contributor.damage_percent, contributor.speed_points)
                })
                .collect()
        })
}

pub(crate) fn priority_shield(
    core: &mut BattleCore,
    caster: UnitInstanceId,
    target: UnitInstanceId,
    amount: u32,
    priority: i32,
    shared: bool,
) -> u64 {
    core.begin_sustained(
        caster,
        target,
        crate::game::ability::sustained::SustainedEffectDef::Shield {
            duration_ms: 1000,
            amount,
            priority,
            shared_radius_units: shared.then_some(2000000),
        },
        0,
    )
    .unwrap();
    core.sustained_effect_snapshot()
        .unwrap()
        .active
        .iter()
        .map(|row| row.activation)
        .max()
        .unwrap()
}

pub(crate) fn prepare_support_attacker(core: &mut BattleCore, attacker: UnitInstanceId) {
    let unit = core.units.get_mut(&attacker).unwrap();
    unit.stats.attack = 100;
    unit.stats.attack_interval_ms = 1000;
    unit.basic_attack_speed_percent = 100;
    unit.basic_attack.action = crate::game::data::abnormality_data::BasicAttackAction::Damage;
    unit.basic_attack.range_policy =
        crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
    unit.basic_attack.delivery = crate::game::ability::DeliveryDef::Instant;
    unit.basic_attack.damage_type = crate::game::battle::damage::DamageType::True;
    unit.basic_attack.windup_ms = 20;
    unit.next_basic_attack_ms = 0;
}

pub(crate) fn pending_support_attacks(
    core: &BattleCore,
    attacker: UnitInstanceId,
) -> Vec<(
    UnitInstanceId,
    crate::game::battle::damage::DamageSourceSnapshot,
)> {
    core.event_queue
        .iter()
        .filter_map(|event| match event.as_ref() {
            BattleEvent::AttackResolve {
                attacker_instance_id,
                target_instance_id,
                source_snapshot,
                ..
            } if *attacker_instance_id == attacker => {
                Some((*target_instance_id, source_snapshot.clone()))
            }
            _ => None,
        })
        .collect()
}

pub(crate) fn remove_current_owner_keep_pending(core: &mut BattleCore, actor: UnitInstanceId) {
    let pending = core.units[&actor]
        .pending_skill_cast
        .as_ref()
        .unwrap()
        .clone();
    let skill = crate::game::ability::SkillId::from("tc_bite");
    core.set_effective_owner(None).unwrap();
    assert_eq!(
        core.current_skill_view(&skill, actor, &Default::default())
            .unwrap()
            .cast_selection(),
        crate::game::ability::SkillCastSelection::SelfUnit
    );
    assert_eq!(
        core.pending_skill_view(&pending, actor)
            .unwrap()
            .cast_selection(),
        crate::game::ability::SkillCastSelection::PlayerUnit
    );
}
