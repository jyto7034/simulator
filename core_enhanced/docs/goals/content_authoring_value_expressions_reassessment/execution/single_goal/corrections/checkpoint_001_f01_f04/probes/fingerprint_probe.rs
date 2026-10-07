use game_core::game::ability::{
    authoring::{ActionSite, EffectOrigin, EffectSite, EffectSource},
    SkillId,
};
use game_core::game::value_expr::*;

fn source() -> EffectSource {
    EffectSource {
        source_file: "fingerprint.ron".into(),
        site: EffectSite {
            action: ActionSite {
                definition_id: SkillId::new("fingerprint"),
                action_id: "strike".into(),
            },
            effect_id: "damage".into(),
        },
        field_path: "amount".into(),
        origin_definition_id: SkillId::new("fingerprint"),
        origin: EffectOrigin::Original,
    }
}

fn main() {
    let constant = AmountExpr::authored(
        EvalMoment::Live,
        ValueExpr::Const(TypedInteger::new(42, ValueUnit::Magnitude)),
    );
    let dynamic = AmountExpr::authored(
        EvalMoment::Live,
        ValueExpr::Add(vec![
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::Attack,
            }),
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::SpellPower,
            }),
        ]),
    );
    let aggregate = AmountExpr::authored(
        EvalMoment::Live,
        ValueExpr::Aggregate {
            source: FiniteSource::AuthoredMembers(vec![
                AuthoredMember::scalar(
                    "b",
                    TypedInteger::new(2, ValueUnit::Magnitude),
                ),
                AuthoredMember::scalar(
                    "a",
                    TypedInteger::new(1, ValueUnit::Magnitude),
                ),
            ]),
            grouping: Grouping::WholeSet,
            reduction: Reduction::Sum,
            outer_reduction: None,
            member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
                unit: ValueUnit::Magnitude,
            })),
        },
    );
    for (label, expression) in [
        ("constant", constant),
        ("dynamic_add", dynamic),
        ("authored_aggregate", aggregate),
    ] {
        let plan = compile(
            &expression,
            &source(),
            DeliveryCapabilities {
                snapshot: true,
                live: true,
                target_at_release: true,
                target_at_consume: true,
                resolved_targets_at_release: true,
                resolved_targets_at_consume: true,
                resources: true,
                result_prefix: true,
                deadline: true,
                owner_grouping: true,
                projected_tile_grouping: true,
            },
            &ExprLimits::default(),
        )
        .unwrap();
        println!("{label} {}", plan.semantic_fingerprint_hex());
    }
}
