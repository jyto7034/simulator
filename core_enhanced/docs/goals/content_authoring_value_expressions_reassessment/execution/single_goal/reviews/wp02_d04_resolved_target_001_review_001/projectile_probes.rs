use super::*;
use crate::game::value_expr::{AmountExpr, EvalMoment, TypedInteger, ValueExpr, ValueUnit};

fn lost_target_projectile(moment: EvalMoment, aggregate: bool) {
    let amount = if aggregate {
        use crate::game::value_expr::{FiniteSource, Grouping, Reduction, ValueRef, ValueSubject, StatSelector};
        AmountExpr::authored(moment, ValueExpr::Aggregate {
            source: FiniteSource::ResolvedTargets, grouping: Grouping::WholeSet,
            reduction: Reduction::Sum, outer_reduction: None,
            member: Box::new(ValueExpr::Read(ValueRef::Stat {subject: ValueSubject::AggregateMember, stat: StatSelector::Attack})),
        })
    } else {
        AmountExpr::authored(moment, ValueExpr::Const(TypedInteger::new(1, ValueUnit::Magnitude)))
    };
    let skill = single_step_skill(
        "review_lost_target", SkillTarget::EnemySingle { rule: UnitTargetRule::Nearest }, 3, 0,
        DeliveryDef::Projectile {
            speed_units_per_ms: 1_000_000,
            hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
            allow_targetless_cast: false, max_range_tiles: None, max_lifetime_ms: None,
            max_kills: None, max_pierces: None, collision: Default::default(),
        },
        vec![SkillEffectDef::DamageExpr { amount, damage_type: DamageType::True }],
    );
    let mut core = core_with_skill_data(vec![], vec![skill.clone()]);
    let caster_id: UnitInstanceId = Uuid::from_u128(0xCD65).into();
    let target_id: UnitInstanceId = Uuid::from_u128(0xCD66).into();
    core.units.insert(caster_id, runtime_unit(caster_id, Side::Player));
    core.units.insert(target_id, runtime_unit(target_id, Side::Opponent));
    place_unit(&mut core, caster_id, Position::new(1, 1));
    place_unit(&mut core, target_id, Position::new(2, 1));
    sync_unit_to_battlefield_tile_center(&mut core, caster_id);
    sync_unit_to_battlefield_tile_center(&mut core, target_id);
    core.invoke_ability(0, caster_id, &skill.id,
        Some(SkillCastTarget::Unit { unit_instance_id: target_id }), BattleEventCause::default(), false);
    let step = core.event_queue.pop().expect("queued projectile step");
    core.process_event(step, 0).unwrap();
    assert!(!core.active_projectiles.is_empty());
    // The runtime explicitly handles a lost homing target as a terminal empty impact.
    core.units.remove(&target_id);
    let mut processed = 0;
    for _ in 0..16 {
        let Some(event) = core.event_queue.pop() else { break; };
        let time_ms = event.time_ms();
        let result = core.process_event(event, time_ms);
        assert!(result.is_ok(), "lost target must resolve an empty impact; moment={moment:?}, aggregate={aggregate}, result={result:?}");
        processed += 1;
    }
    assert!(processed > 0);
    assert!(core.active_projectiles.is_empty());
}

#[test]
fn review_scalar_snapshot_projectile_target_disappears() { lost_target_projectile(EvalMoment::Snapshot, false); }
#[test]
fn review_scalar_live_projectile_target_disappears() { lost_target_projectile(EvalMoment::Live, false); }
#[test]
fn review_aggregate_snapshot_projectile_target_disappears() { lost_target_projectile(EvalMoment::Snapshot, true); }
#[test]
fn review_aggregate_live_projectile_target_disappears() { lost_target_projectile(EvalMoment::Live, true); }
