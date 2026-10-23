//! Independent U10 inputs use ordinary publication, relic copies and World
//! commands. Charge, enemy durability and placement are explicit preparation.
use super::*;
use crate::game::battle::{
    core::relic_structural_tests::flow_cases as existing_native_probe, ids::UnitInstanceId,
};
use crate::game::data::relic_data::RelicDatabase;
use crate::game::enums::Side;
use std::collections::BTreeSet;

mod admission_cases;
mod ancestry_cases;
mod audit_joint_cases;
mod join_cases;
mod periodic_lifetime_cases;
mod resonance_cases;
mod wedge_cases;

fn skill_input(strikes: &[(u32, &[u32])]) -> String {
    let steps = strikes.iter().enumerate().map(|(strike, (delay, amounts))| {
        let effects = amounts.iter().enumerate().map(|(segment, amount)| format!(
            r#"(id:"damage_{strike}_{segment}",tags:["u10_direct","u10_multi_hit"],op:Damage(amount:{amount},damage_type:True))"#
        )).collect::<Vec<_>>().join(",");
        format!(r#"SkillStepDef(id:"strike_{strike}",delay_ms:{delay},target:CastTarget,delivery:Instant,effects:[{effects}])"#)
    }).collect::<Vec<_>>().join(",");
    format!(
        r#"SkillDatabase(skills:[SkillDef(
        id:"u10_strike",name:"independent direct strikes",kind:Targeted,
        focus_time_ms:0,use_lifetime:Action,
        cast_targeting:Explicit(selection:automatic_unit,target:EnemySingle(rule:Nearest),reach:global,air_capable:true),
        steps:[{steps}]
    )])"#
    )
}

fn relic_input(rules: &str) -> String {
    format!(
        r#"(relics:[(
        id:"u10_relic",uuid:"00000000-0000-0000-0000-00000000d101",
        name:"independent causal relic",description:"test input",rarity:"probe",
        allow_duplicates:true,effects:[],dismantle_equipment_dust:1,rules:[{rules}]
    )])"#
    )
}

fn damage_subscriber(id: &str, amount: u32, delay: u32) -> String {
    format!(
        r#"(id:"{id}",apply:Subscribe((
        event:DamageHpLost(perspective:Dealt),
        payload:Program((steps:[(id:"child",delay_ms:{delay},target:EventTarget,
            effects:[(id:"damage",tags:["u10_child"],op:Damage(amount:{amount},damage_type:True))])]))
    )))"#
    )
}

struct CausalWorld {
    core: GameCore,
    player: Uuid,
    employee: Uuid,
    copies: Vec<Uuid>,
    actor: UnitInstanceId,
    targets: Vec<UnitInstanceId>,
}

impl CausalWorld {
    fn prepare(skills: &str, relics: &str, equipped: &[&str], with_fragment: bool) -> Self {
        Self::prepare_with_buffs(skills, relics, equipped, with_fragment, None)
    }

    fn prepare_with_buffs(
        skills: &str,
        relics: &str,
        equipped: &[&str],
        with_fragment: bool,
        buffs: Option<&str>,
    ) -> Self {
        Self::prepare_full(skills, relics, equipped, with_fragment, buffs, None)
    }

    fn prepare_full(
        skills: &str,
        relics: &str,
        equipped: &[&str],
        with_fragment: bool,
        buffs: Option<&str>,
        enemy_count: Option<u32>,
    ) -> Self {
        Self::prepare_with_unowned_fragment(
            skills,
            relics,
            equipped,
            with_fragment,
            buffs,
            enemy_count,
            false,
        )
    }

    fn prepare_with_unowned_fragment(
        skills: &str,
        relics: &str,
        equipped: &[&str],
        with_fragment: bool,
        buffs: Option<&str>,
        enemy_count: Option<u32>,
        publish_unowned_fragment: bool,
    ) -> Self {
        let skills = SkillDatabase::from_ron("u10/independent_skills.ron", skills)
            .expect("independent normal skill input");
        let fragments = if with_fragment || publish_unowned_fragment {
            vec![active_skill_fragment("u10_fragment", 0xd002, "u10_strike")]
        } else {
            vec![]
        };
        // The intermediate PvE/fragment catalog builder has no buff input.
        // A custom-buff fixture publishes its actual skills and buffs together
        // in the final builder below; this seed skill never enters its battle.
        let seed_skills = if buffs.is_some() {
            SkillDatabase::from_ron("u10/catalog_seed.ron", &skill_input(&[(0, &[1])]))
                .expect("buff-free intermediate catalog seed")
                .skills
        } else {
            skills.skills.clone()
        };
        let base = GameCore::new(
            game_data_with_pve_equipment_and_active_skill_fragments(vec![], seed_skills, fragments),
            0xd001,
        );
        let mut enemies = base.game_data.abnormality_data.items.clone();
        for enemy in &mut enemies {
            enemy.max_health = 10_000;
            enemy.attack = 0;
            enemy.defense = 0;
            enemy.magic_resist = 0;
        }
        let mut builder = GameDataBuilder::from_validated(&base.game_data)
            .with_skills(skills)
            .with_abnormalities(enemies)
            .with_relics(ron::de::from_str::<RelicDatabase>(relics).unwrap());
        if let Some(buffs) = buffs {
            builder = builder.with_buffs(ron::de::from_str(buffs).unwrap());
        }
        if let Some(enemy_count) = enemy_count {
            let mut pve = (*base.game_data.pve_data).clone();
            let encounter = pve
                .encounters
                .iter_mut()
                .find(|entry| entry.id == "defense_encounter")
                .unwrap();
            let PveWaveSource::Manual(entries) = &mut encounter.waves[0].source else {
                panic!("manual fixture wave");
            };
            let PveWaveEnemyData::Abnormality { count, .. } = &mut entries[0] else {
                panic!("abnormality fixture wave");
            };
            *count = enemy_count;
            builder = builder.with_pve(pve);
        }
        let data = builder
            .build_with_runtime_safety(
                crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
            )
            .expect("ordinary complete builder publication");
        let mut core = GameCore::new(Arc::new(data), 0xd001);
        let player = Uuid::from_u128(0xd001);
        start_new_game_with_default_starters(&mut core, player);
        let employee = core.roster().unwrap().combat_eligible_employee_ids()[0];
        if with_fragment {
            let serum = consumable_meta(
                0xd003,
                "u10_initial_charge",
                ConsumableTier::Critical,
                ConsumableEffect::InitialSkillCharge { percent: 100 },
            );
            core.roster_mut()
                .unwrap()
                .get_mut(&employee)
                .unwrap()
                .apply_consumable_modifier(Uuid::from_u128(0xd004), &serum);
            let copy_uuid = core
                .apply_grant_effects(&[RewardEffect::GrantSkillFragment {
                    fragment_id: "u10_fragment".into(),
                }])
                .unwrap()
                .skill_fragment_diffs[0]
                .copy_uuid;
            core.execute(
                player,
                PlayerBehavior::EquipSkillFragment {
                    employee_uuid: employee,
                    copy_uuid,
                },
            )
            .unwrap();
        }
        let mut copies = Vec::new();
        for (slot, definition_id) in equipped.iter().enumerate() {
            let before: BTreeSet<_> = core
                .state
                .relics
                .copies()
                .map(|copy| copy.copy_uuid)
                .collect();
            core.execute_admin_command(super::super::AdminCommand::AdminGrantRelic {
                definition_id: (*definition_id).into(),
                count: 1,
            })
            .unwrap();
            let added: Vec<_> = core
                .state
                .relics
                .copies()
                .map(|copy| copy.copy_uuid)
                .filter(|copy| !before.contains(copy))
                .collect();
            assert_eq!(added.len(), 1, "one actual new copy per grant");
            core.execute(
                player,
                PlayerBehavior::EquipRelic {
                    employee_uuid: employee,
                    copy_uuid: added[0],
                    slot: slot as u8,
                },
            )
            .unwrap();
            copies.push(added[0]);
        }
        let node = force_first_available_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_defense",
            MapNodePayload::Encounter {
                encounter_id: Some("defense_encounter".into()),
            },
        );
        let preview = core
            .execute(player, PlayerBehavior::SelectMapNode { node_id: node })
            .unwrap();
        let BehaviorResult::NodePreview {
            combat_preview: Some(preview),
            ..
        } = preview
        else {
            panic!("ordinary encounter preview");
        };
        let position = preview
            .deployment_zones
            .iter()
            .find(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
            .and_then(|zone| zone.cells.first().copied())
            .unwrap();
        core.execute(player, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::DeployUnit {
                employee_uuid: employee,
                position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
        let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
        let actor = battle
            .units
            .values()
            .find(|unit| unit.source_owned_uuid == employee)
            .unwrap()
            .instance_id;
        let mut targets: Vec<_> = battle
            .units
            .values()
            .filter(|unit| unit.owner == Side::Opponent)
            .map(|unit| unit.instance_id)
            .collect();
        targets.sort();
        assert!(!targets.is_empty());
        let actor_stats = &mut battle.units.get_mut(&actor).unwrap().stats;
        actor_stats.max_health = 10_000;
        actor_stats.current_health = 5_000;
        actor_stats.attack = 0;
        existing_native_probe::place_stationary(battle, actor, 0, 0);
        for (index, target) in targets.iter().enumerate() {
            existing_native_probe::place_stationary(battle, *target, index as i32 + 1, 0);
        }
        Self {
            core,
            player,
            employee,
            copies,
            actor,
            targets,
        }
    }

    fn deploy_other_employee(&mut self) -> (Uuid, UnitInstanceId) {
        let other = self
            .core
            .roster()
            .unwrap()
            .combat_eligible_employee_ids()
            .into_iter()
            .find(|employee| *employee != self.employee)
            .unwrap();
        let active = self.core.state.active_battle.as_mut().unwrap();
        active.tactical_resource.set_current_for_test(100);
        let position = active
            .combat_preview
            .deployment_zones
            .iter()
            .find(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
            .unwrap()
            .cells[1];
        execute_current_battle_mutation(
            &mut self.core,
            self.player,
            PlayerBehavior::DeployUnit {
                employee_uuid: other,
                position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
        let battle = &mut self.core.state.active_battle.as_mut().unwrap().battle;
        let actor = battle
            .units
            .values()
            .find(|unit| unit.source_owned_uuid == other)
            .unwrap()
            .instance_id;
        existing_native_probe::place_stationary(battle, actor, 0, 1);
        battle.units.get_mut(&actor).unwrap().stats.attack = 0;
        (other, actor)
    }

    fn cast(&mut self) {
        execute_current_battle_mutation(
            &mut self.core,
            self.player,
            PlayerBehavior::ActivateSkill {
                employee_uuid: self.employee,
                skill_id: "u10_strike".into(),
                target: None,
            },
        )
        .expect("normal World accepted cast");
    }

    fn advance(&mut self, delta_ms: u64) {
        self.core
            .advance_active_battle_for_server_tick(delta_ms)
            .unwrap();
    }

    fn hp(&self, unit: UnitInstanceId) -> u32 {
        self.core.state.active_battle.as_ref().unwrap().battle.units[&unit]
            .stats
            .current_health
    }

    fn target_hp(&self) -> u32 {
        self.targets.iter().map(|target| self.hp(*target)).sum()
    }
}

fn causal_roots(core: &crate::game::battle::core::BattleCore) -> serde_json::Value {
    serde_json::json!({
        "existing":existing_native_probe::flow_roots(core),
        "causal":core.causal_runtime_snapshot().unwrap(),
    })
}

fn assert_causal_reclaimed(core: &crate::game::battle::core::BattleCore) {
    let snapshot = core.causal_runtime_snapshot().unwrap();
    assert!(
        snapshot.contexts.is_empty(),
        "live ancestry remains: {snapshot:?}"
    );
    assert!(
        snapshot.joins.is_empty(),
        "join participant ownership remains: {snapshot:?}"
    );
    assert_eq!(snapshot.wedge_counter_count, 0);
    assert_eq!(snapshot.independent_periodic_count, 0);
    assert_eq!(existing_native_probe::active_cast_count(core), 0);
}
