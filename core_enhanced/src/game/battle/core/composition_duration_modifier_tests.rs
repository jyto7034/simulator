mod composition_duration_modifiers {
    use super::*;
    use crate::authoring::composition::{compile_json, parse_yaml, Artifact};
    use serde_json::{json, Value};

    fn fixture(
        edit: impl FnOnce(&mut Value),
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let mut doc = parse_yaml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/duration_modifiers.skill.yaml"
        )))
        .unwrap();
        doc["skills"][0]["id"] = json!("r11_aura");
        edit(&mut doc);
        let artifact = compile_json(&doc.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        fixture_from_native(artifact.load_native().unwrap(), false, false)
    }

    fn equip(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        owners: &[Option<UnitInstanceId>],
        single: bool,
    ) {
        equip_named(core, actor, owners, single, "long-breath");
    }

    pub(super) fn equip_named(core: &mut BattleCore, actor: UnitInstanceId, owners: &[Option<UnitInstanceId>], single: bool, relic: &str) {
        let data: ValidatedGameData = core.game_data.clone().into();
        let employee = core.units[&actor].source_owned_uuid;
        let mut inventory = RelicInventory::default();
        let mut ids = UuidManager::new(0xd240);
        for (slot, owner) in owners.iter().enumerate() {
            let copy = inventory
                .grant(data.relic_data.get_by_id(relic).unwrap(), &mut ids)
                .unwrap();
            if let Some(owner) = owner {
                inventory
                    .equip(
                        copy,
                        core.units[owner].source_owned_uuid,
                        slot,
                        &data.relic_data,
                    )
                    .unwrap();
            }
        }
        if single {
            let copy = inventory
                .grant(
                    data.relic_data
                        .get_by_id("single-target-conversion")
                        .unwrap(),
                    &mut ids,
                )
                .unwrap();
            inventory
                .equip(copy, employee, owners.len(), &data.relic_data)
                .unwrap();
        }
        let providers = [ProspectiveProvider::Fragment(
            core.units[&actor]
                .fragment_source
                .as_ref()
                .unwrap()
                .as_ref()
                .clone(),
        )];
        let plan = compile_effective_plan(EffectivePlanInput {
            data: &data,
            employee_uuid: employee,
            inventory: &inventory,
            providers: &providers,
        })
        .unwrap();
        assert_eq!(
            crate::game::relic::structural::employee_has_structural_rules(
                &inventory,
                employee,
                data.database(),
            )
            .unwrap(),
            plan.is_some(),
            "World admission must agree with the effective compiler for action-only relics",
        );
        if let Some(plan) = plan {
            let owner = crate::game::battle::core::effective_skill::EffectiveBattleOwner::new(
                std::collections::BTreeMap::from([(employee, plan)]),
            )
            .unwrap();
            core.set_effective_owner(owner).unwrap();
        }
    }

    #[test]
    fn composition_duration_modifiers_equipped_owner_once_ceil_and_stop() {
        for mode in ["none", "owned", "other", "one", "two", "single", "stop"] {
            let (mut c, mut s, a, b) = fixture(|_| {});
            let owners = match mode {
                "owned" => vec![None],
                "other" => vec![Some(b)],
                "two" => vec![Some(a), Some(a)],
                "one" | "single" | "stop" => vec![Some(a)],
                _ => vec![],
            };
            equip(&mut c, a, &owners, mode == "single");
            cast(&mut c, a, (mode == "single").then_some(b), 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            let view = c
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .into_iter()
                .find(|r| r.is_channel)
                .unwrap();
            let extended = matches!(mode, "one" | "two" | "single" | "stop");
            assert_eq!(
                view.expires_at_ms,
                1 + if extended { 252 } else { 201 },
                "{mode}"
            );
            assert_eq!(
                c.aura_basic_bonus_for_occurrence(b, true).unwrap(),
                if mode == "single" { (40, 20) } else { (20, 15) }
            );
            if mode == "stop" {
                cast(&mut c, a, None, 202);
                c.step_battle_execution_until(&mut s, 253).unwrap();
            } else {
                c.step_battle_execution_until(&mut s, view.expires_at_ms - 1)
                    .unwrap();
                assert!(c.channel_active(a), "{mode}");
                c.step_battle_execution_until(&mut s, view.expires_at_ms)
                    .unwrap();
            }
            assert!(!c.channel_active(a), "{mode}");
            assert_eq!(c.aura_basic_bonus_for_occurrence(b, true).unwrap(), (0, 0));
        }
    }

    #[test]
    fn composition_duration_modifiers_hit_status_has_independent_lifetime_and_same_equipped_rule() {
        for equipped in [false, true] {
            let (mut c, mut s, a, _) = fixture(|doc| {
                doc["skills"][0]["kind"] = json!("Targeted");
                doc["skills"][0]["cast_targeting"]["Explicit"] = json!({"selection":"automatic_unit","target":{"EnemySingle":{"rule":"Nearest"}},"reach":"global","air_capable":true});
                doc["values"][0]["value"]["Effects"] = json!([{"id":"support","attached_to":"impact","op":{"Sustained":{"definition":{"HitStatus":{"duration_ms":201,"profile":{"definition":"authored-slow","kind":{"Slow":{"percent":30}}}}}}}}]);
                let action = &mut doc["definitions"][0]["body"]["Action"];
                action["target"] = json!({"Literal":{"Target":{"EnemySingle":{"rule":"Nearest"}}}});
                action["effects"].as_array_mut().unwrap().insert(0,json!({"Damage":{"id":"impact","tags":[],"amount":{"Literal":{"Amount":{"moment":"Snapshot","root":{"Const":{"value":1,"unit":"Magnitude"}}}}},"damage_type":"True"}}));
            });
            if equipped {
                equip(&mut c, a, &[Some(a)], false);
            }
            let enemy = c
                .units
                .values()
                .find(|u| u.owner == Side::Opponent)
                .unwrap()
                .instance_id;
            place(&mut c, enemy, 3, 1);
            cast(&mut c, a, None, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            let view = c
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .into_iter()
                .find(|r| {
                    r.hit_status
                        .as_ref()
                        .is_some_and(|h| h.definition == "authored-slow")
                })
                .unwrap();
            assert_eq!(view.target, enemy);
            assert!(!view.is_channel);
            assert_eq!(view.expires_at_ms, 1 + if equipped { 252 } else { 201 });
            assert!(!c.resonance_use_has_obligations(a));
            c.step_battle_execution_until(&mut s, view.expires_at_ms - 1)
                .unwrap();
            assert!(c
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .iter()
                .any(|r| r.activation == view.activation));
            c.step_battle_execution_until(&mut s, view.expires_at_ms)
                .unwrap();
            assert!(!c
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .iter()
                .any(|r| r.activation == view.activation));
        }
    }

    #[test]
    fn composition_duration_modifiers_cancelled_extended_expiry_restores_and_retries() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (mut c, mut s, a, _) = fixture(|_| {});
        equip(&mut c, a, &[Some(a)], false);
        cast(&mut c, a, None, 0);
        c.step_battle_execution_until(&mut s, 252).unwrap();
        let snapshot = |c: &BattleCore, s: &BattleExecutionState| {
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                serde_json::to_value(c.runtime_fact_view()).unwrap(),
                format!("{s:?}"),
            )
        };
        let before = snapshot(&c, &s);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("sustained_ended", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || c.step_battle_execution_until(&mut s, 253),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(snapshot(&c, &s), before);
        c.step_battle_execution_until(&mut s, 253).unwrap();
        assert!(!c.channel_active(a));
    }
}
