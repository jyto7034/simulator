//! Independent U09 authoring/compiler tests. Private fixture generations in
//! this module are not evidence of normal publication or runtime support.
use super::authoring::*;
use super::capabilities::descriptor;
use super::*;

mod compiler_cases;
mod storage_cases;

#[test]
fn u09_typed_errors_and_consumer_boundaries_do_not_clamp() {
    let damage = descriptor(EffectOperation::Damage, RelicProperty::DamageAmount).unwrap();
    let shield = descriptor(
        EffectOperation::PersonalShieldGrant,
        RelicProperty::PersonalShieldInitialAmount,
    )
    .unwrap();
    let footprint = descriptor(EffectOperation::Damage, RelicProperty::DamageFootprint).unwrap();
    let mut p = NumericProgram::default();
    p.include(NumericEditDef::Replace(TypedRelicValue::Hp(12)), damage)
        .unwrap();
    p.include(NumericEditDef::Replace(TypedRelicValue::Hp(12)), damage)
        .unwrap();
    assert_eq!(
        p.apply(TypedRelicValue::Hp(99), damage).unwrap(),
        TypedRelicValue::Hp(12)
    );
    assert_eq!(
        p.include(NumericEditDef::Replace(TypedRelicValue::Hp(13)), damage)
            .unwrap_err()
            .code,
        FaultCode::ReplaceConflict
    );
    assert_eq!(
        NumericProgram::default()
            .include(NumericEditDef::Add(TypedRelicValue::SpeedPoints(1)), damage)
            .unwrap_err()
            .code,
        FaultCode::UnitMismatch
    );
    assert_eq!(
        NumericProgram::default()
            .include(
                NumericEditDef::Scale {
                    increment_percent: 10
                },
                footprint
            )
            .unwrap_err()
            .code,
        FaultCode::UnsupportedOperation
    );
    let identity = NumericProgram::default();
    for (base, desc, expected) in [
        (0, damage, true),
        (0, shield, false),
        (1, shield, true),
        (u32::MAX as i64, damage, true),
        (u32::MAX as i64 + 1, damage, false),
        (-1, damage, false),
    ] {
        let result = identity.apply(TypedRelicValue::Hp(base), desc);
        if expected {
            assert_eq!(result.unwrap(), TypedRelicValue::Hp(base));
        } else {
            assert_eq!(result.unwrap_err().code, FaultCode::OutOfRange);
        }
    }
    for (increment, expected) in [(-100, true), (-101, false)] {
        let mut p = NumericProgram::default();
        p.include(
            NumericEditDef::Scale {
                increment_percent: increment,
            },
            damage,
        )
        .unwrap();
        if expected {
            assert_eq!(
                p.apply(TypedRelicValue::Hp(17), damage).unwrap(),
                TypedRelicValue::Hp(0)
            );
        } else {
            assert_eq!(
                p.apply(TypedRelicValue::Hp(17), damage).unwrap_err().code,
                FaultCode::OutOfRange
            );
        }
    }
    // Two legal i64 edits fit the accumulator, but their checked product does not.
    let mut overflow = NumericProgram::default();
    for _ in 0..2 {
        overflow
            .include(NumericEditDef::Add(TypedRelicValue::Hp(i64::MAX)), damage)
            .unwrap();
        overflow
            .include(
                NumericEditDef::Scale {
                    increment_percent: i64::MAX,
                },
                damage,
            )
            .unwrap();
    }
    assert_eq!(
        overflow
            .apply(TypedRelicValue::Hp(0), damage)
            .unwrap_err()
            .code,
        FaultCode::ArithmeticOverflow
    );
}

#[test]
fn u09_typed_composition_397_is_order_independent() {
    let damage = descriptor(EffectOperation::Damage, RelicProperty::DamageAmount).unwrap();
    // Independent arithmetic: (200 + 30 + 35) * (100 + 20 + 30) / 100
    // = 397.5, narrowed once to 397. Neither sequential scaling nor rounding
    // intermediate terms gives the permitted result.
    let edits = [
        NumericEditDef::Replace(TypedRelicValue::Hp(200)),
        NumericEditDef::Add(TypedRelicValue::Hp(30)),
        NumericEditDef::Add(TypedRelicValue::Hp(35)),
        NumericEditDef::Scale {
            increment_percent: 20,
        },
        NumericEditDef::Scale {
            increment_percent: 30,
        },
    ];
    fn permutations(
        rest: &mut [NumericEditDef],
        prefix: usize,
        damage: &capabilities::RelicCapabilityDescriptor,
    ) {
        if prefix == rest.len() {
            let mut program = NumericProgram::default();
            for edit in rest.iter().copied() {
                program.include(edit, damage).unwrap();
            }
            assert_eq!(
                program.apply(TypedRelicValue::Hp(999), damage).unwrap(),
                TypedRelicValue::Hp(397)
            );
            return;
        }
        for i in prefix..rest.len() {
            rest.swap(prefix, i);
            permutations(rest, prefix + 1, damage);
            rest.swap(prefix, i);
        }
    }
    permutations(&mut edits.clone(), 0, damage);
}

#[test]
fn u09_strict_serde_and_condition_limits_recover_after_failure() {
    use crate::game::value_expr::{with_parse_limits, ExprParseLimits};
    let valid = r#"(id:"edit",apply:Edit(property:DamageAmount,operation:Add(Hp(7))))"#;
    let rule: RelicRuleDef = ron::de::from_str(valid).unwrap();
    assert_eq!(rule.when, RelicConditionDef::Always);
    assert_eq!(rule.selector, StructuralSelectorDef::default());
    for invalid in [
        r#"(id:"edit",unexpected:true,apply:Edit(property:DamageAmount,operation:Add(Hp(7))))"#,
        r#"(id:"edit",apply:Edit(property:UnknownAmount,operation:Add(Hp(7))))"#,
        r#"(id:"edit",apply:Grant(InlineEffects(action_id:"a",effects:[(id:"e",op:GrantRule())])))"#,
    ] {
        assert!(
            ron::de::from_str::<RelicRuleDef>(invalid).is_err(),
            "accepted {invalid}"
        );
    }
    for (limits, at_limit, too_large) in [
        (
            ExprParseLimits {
                max_depth: 3,
                ..ExprParseLimits::HARD_MAX
            },
            "Not(Not(Always))",
            "Not(Not(Not(Always)))",
        ),
        (
            ExprParseLimits {
                max_nodes: 3,
                ..ExprParseLimits::HARD_MAX
            },
            "All([Always,Always])",
            "All([Always,Always,Always])",
        ),
    ] {
        with_parse_limits(limits, || {
            assert!(ron::de::from_str::<RelicConditionDef>(at_limit).is_ok());
            let error = ron::de::from_str::<RelicConditionDef>(too_large).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("relic condition parse depth/node limit"),
                "{error}"
            );
            // Failed nested parses cannot retain a meter into the next root.
            assert!(ron::de::from_str::<RelicConditionDef>(at_limit).is_ok());
        });
    }
}
