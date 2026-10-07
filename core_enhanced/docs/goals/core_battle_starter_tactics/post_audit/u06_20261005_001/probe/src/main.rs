use game_core::game::{battle::{core::BattleCore,scenario::*,types::*,tile_range::TileRangePolicy,event_log::BattleLogEvent},data::{GameDataBuilder,abnormality_data::{BasicAttackAction,BasicAttackDef}},enums::{Side,Tier},growth::GrowthStack,resources::Position};
use uuid::Uuid;
fn run(flat:u32,spell_power_percent:u32)->serde_json::Value {
 let mut profile=UnitCombatProfile::employee_default();
 let encoded=serde_json::json!({"action":{"Heal":{"flat":flat,"spell_power_percent":spell_power_percent,"footprint":null}},"range_policy":"whole_field_valid_tiles","interval_ms":100,"windup_ms":1,"delivery":"Instant"});
 // Decode the actual strict BasicAttackDef; no invalid in-memory definition injection.
 let def:BasicAttackDef=serde_json::from_value(encoded).expect("strict basic action decode");
 assert!(matches!(def.action,BasicAttackAction::Heal{..}));
 profile.basic_attack=def;
 profile.basic_attack.range_policy=TileRangePolicy::WholeFieldValidTiles;
 profile.stats.attack_interval_ms=100;
 profile.stats.spell_power=0;
 profile.weapon_profile=None;
 let id=Uuid::from_u128(100);
 let draft=BattleUnitDraft{owned_uuid:id,source:BattleUnitSource::TestFixture{base_uuid:id,profile},threat_class:BattleUnitThreatClass::Normal,level:Tier::I,stat_scale:Default::default(),growth_stacks:GrowthStack::new(),equipment_layout:None,equipped_items:vec![],equipped_item_enhancements:vec![],employee_start_condition:None};
 let mut scenario=BattleScenario::empty((4,4));
 scenario.terminal_outcomes=vec![TerminalOutcome{id:BattleOutcomeId::new("end"),verdict:BattleVerdict::Victory,condition:TerminalCondition::TimeReached{time_ms:1000},priority:0}];
 scenario.groups.push(ScenarioSpawnGroup{id:ScenarioGroupId::new("healer"),side:Side::Player,required_for_victory:false,enemy_movement_plan:None,spawns:vec![ScenarioUnitSpawn{unit_ref:ScenarioUnitRef::new("healer"),side:Side::Player,draft,position:Position::new(1,1),instance_salt:0}]});
 scenario.events.push(ScenarioEvent{id:ScenarioEventId::new("start"),trigger:ScenarioTrigger::AtBattleStart,action:ScenarioAction::SpawnGroup{group_id:ScenarioGroupId::new("healer")}});
 let mut core=BattleCore::new_from_scenario(scenario,GameDataBuilder::empty().build_arc(),17);
 let mut execution=core.start_battle_execution().unwrap();
 core.step_battle_execution_until(&mut execution,0).unwrap();
 let actor=*core.units.keys().next().unwrap();
 // Explicit injury fixture; source/data/target files remain untouched.
 core.units.get_mut(&actor).unwrap().stats.current_health=50;
 core.step_battle_execution_until(&mut execution,400).unwrap();
 let entries=core.event_log_entries_after_seq(0);
 let starts=entries.iter().filter(|e|matches!(e.event,BattleLogEvent::AttackStart{attacker_instance_id,..} if attacker_instance_id==actor)).count();
 let heals=entries.iter().filter(|e|matches!(e.event,BattleLogEvent::HpChanged{delta,..} if delta>0)).count();
 serde_json::json!({"flat":flat,"spell_power_percent":spell_power_percent,"actor_spell_power":0,"strict_decode_accepted":true,"attack_starts":starts,"positive_hp_events":heals,"final_hp":core.units[&actor].stats.current_health,"events":entries})
}
fn main(){println!("{}",serde_json::json!({"zero":run(0,0),"coefficient_rounds_to_zero":run(0,100),"positive_control":run(10,0)}));}
