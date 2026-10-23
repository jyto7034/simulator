//! Independent literal terminal arithmetic and genuine selected command inputs.
use game_core::mod_validation::CoverageReport;
use serde_json::{json,Value};
use std::{collections::BTreeMap,error::Error,path::Path};
use u11_independent_public_probe::{fixture_coverage as oracle,normal_preparation as normal};
type Result<T>=std::result::Result<T,Box<dyn Error>>;
fn bind(step:usize,pointer:&str,to:&str)->Value {
    json!({"from":{"step":step,"pointer":pointer},"to":to,"value_type":"string"})
}
pub fn manual_steps(prefix_len:usize)->Vec<Value> {
    let authority=|kind:&str,source:usize|json!({"kind":"command",
        "command":{"type":kind,"employee_uuid":null,"skill_id":"u11_recipe_strike"},
        "context":{"expected_battle_authority":{"battle_uuid":null,"battle_state_revision":null}},
        "bindings":[bind(source,"/payload/battle_uuid","/context/expected_battle_authority/battle_uuid"),
            {"from":{"step":source,"pointer":"/payload/battle_update/battle_state_revision"},
             "to":"/context/expected_battle_authority/battle_state_revision","value_type":"number"},
            bind(prefix_len,"/payload/battle_update/checkpoint/deployment/deployed_units/0/employee_uuid","/command/employee_uuid")]});
    let begin=authority("begin_skill_targeting",prefix_len);
    let mut activate=authority("activate_skill",prefix_len+1);
    activate["command"]["target"]=json!({"type":"Unit","unit_instance_id":null});
    activate["bindings"].as_array_mut().unwrap().push(bind(prefix_len,
        "/payload/battle_update/checkpoint/deployment/deployed_units/0/unit_instance_id","/command/target/unit_instance_id"));
    let mut steps=vec![json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":8000,"after_seq":0}}),begin,activate];
    steps.extend((steps.len()..254-prefix_len).map(|_|json!({"kind":"clock","clock":{"mode":"next_event_bucket"}})));
    steps
}
pub fn manual_ready(projection:&Value,out:&Path)->Result<()> {
    let units=projection.pointer("/payload/battle_update/checkpoint/deployment/deployed_units")
        .and_then(Value::as_array).ok_or("actual manual deployment missing")?;
    normal::save(&out.join("actual_manual_ready.json"),&units)?;
    assert_eq!(units.len(),1);let ready=&units[0]["skill_readiness"];
    assert_eq!(ready["skill_id"],"u11_recipe_strike");assert_eq!(ready["selection"],"player_unit");
    assert_eq!(ready["activation_mode"],"manual");assert_eq!(ready["manual_activation_allowed"],true);
    assert_eq!(ready["eligibility"]["enabled"],true);
    let maximum=ready["resonance_max"].as_u64().ok_or("actual resonance maximum absent")?;
    assert!(maximum>0);assert!(ready["resonance_current"].as_u64().unwrap()>=maximum);
    Ok(())
}
pub fn terminal(report:&CoverageReport,events:&BTreeMap<u64,Value>,projection:&Value,out:&Path)->Result<()> {
    for(effect,terminal_stage)in [("cost",4),("heal_positive",4),("heal_noop",3),("shield",4),("absorbed",3),("zero_expr",3)] {
        let count=|stage|oracle::count(report,"u11_terminal_chain","ordered",effect,"skill_effect",stage);
        assert_eq!(count(1),1);assert_eq!(count(2),0);
        assert_eq!(count(3),u64::from(terminal_stage==3),"one exact legal-noop terminal for {effect}");
        assert_eq!(count(4),u64::from(terminal_stage==4),"one exact positive terminal for {effect}");
    }
    let casts:Vec<_>=events.values().filter(|e|e["event"]["type"]=="AbilityCast"&&e["event"]["skill_id"]=="u11_terminal_chain").collect();
    let steps:Vec<_>=events.values().filter(|e|e["event"]["type"]=="AbilityStepTriggered"&&e["event"]["skill_id"]=="u11_terminal_chain").collect();
    assert_eq!(casts.len(),1);assert_eq!(steps.len(),1);let step=steps[0];let caster=&step["event"]["caster_instance_id"];
    assert_eq!(step["event"]["target_instance_id"],*caster);assert_eq!(step["cause"]["seq"],casts[0]["seq"]);
    let hp:Vec<_>=events.values().filter(|e|e["event"]["type"]=="HpChanged"&&e["event"]["target_instance_id"]==*caster
        &&e["time_ms"]==step["time_ms"]).collect();
    assert_eq!(hp.len(),4,"cost, positive heal, full-health heal, absorbed strike; zero expression emits no HP command");
    let costs:Vec<_>=hp.iter().copied().filter(|e|e["event"]["reason"]=="HealthCost").collect();assert_eq!(costs.len(),1);
    let cost=costs[0];let full=cost["event"]["hp_before"].as_u64().unwrap();assert!(full>7);
    assert_eq!(cost["event"]["hp_after"],full-7);
    let heals:Vec<_>=hp.iter().copied().filter(|e|e["event"]["reason"]=="Command"&&e["event"]["damage_source"].is_null()).collect();assert_eq!(heals.len(),2);
    assert_eq!(heals[0]["event"]["hp_before"],full-7);assert_eq!(heals[0]["event"]["hp_after"],full);
    assert_eq!(heals[1]["event"]["hp_before"],full);assert_eq!(heals[1]["event"]["hp_after"],full);
    let absorbed:Vec<_>=hp.iter().copied().filter(|e|e["event"]["damage_source"]=="Ability").collect();assert_eq!(absorbed.len(),1);
    assert_eq!(absorbed[0]["event"]["raw_damage"],5);assert_eq!(absorbed[0]["event"]["final_damage"],0);
    assert_eq!(absorbed[0]["event"]["hp_before"],full);assert_eq!(absorbed[0]["event"]["hp_after"],full);
    for event in &hp {assert_eq!(event["event"]["source_instance_id"],*caster);assert_eq!(event["cause"]["seq"],step["seq"]);}
    assert!(cost["seq"].as_u64().unwrap()<heals[0]["seq"].as_u64().unwrap());
    assert!(heals[0]["seq"].as_u64().unwrap()<heals[1]["seq"].as_u64().unwrap());
    assert!(heals[1]["seq"].as_u64().unwrap()<absorbed[0]["seq"].as_u64().unwrap());
    let checkpoint=projection.pointer("/payload/battle_update/checkpoint").ok_or("actual terminal checkpoint absent")?;
    let unit=checkpoint["units"].as_array().unwrap().iter().find(|u|u["unit_instance_id"]==*caster).ok_or("same actual caster missing")?;
    assert_eq!(unit["owner"],"Player");assert_eq!(unit["stats"]["current_health"],full);assert_eq!(unit["stats"]["max_health"],full);
    let pools:Vec<_>=checkpoint["sustained_effects"]["active"].as_array().unwrap().iter()
        .filter(|p|p["skill_id"]=="u11_terminal_chain").collect();assert_eq!(pools.len(),1);let pool=pools[0];
    assert_eq!(pool["caster"],*caster);assert_eq!(pool["target"],*caster);
    assert_eq!(pool["authored_site"]["effect_id"],"shield");assert_eq!(pool["shield"]["capacity"],10);assert_eq!(pool["shield"]["remaining"],5);
    assert_eq!(pool["expires_at_ms"],step["time_ms"].as_u64().unwrap()+250);
    assert!(!report.is_complete(),"actual noops do not replace required positive consumption");
    normal::save(&out.join("actual_terminal_arithmetic.json"),&json!({"cast":casts[0],"step":step,"hp_events":hp,
        "actual_shield":pool,"same_actual_player":unit,"coverage":report,"approval_claimed":false}))?;
    Ok(())
}
