//! Actual per-actor participation and lifecycle evidence, independent of aggregate coverage.
use game_core::authoring::{LabPreparation,TrialSession};
use serde_json::{json,Value};
use std::{collections::{BTreeMap,BTreeSet},error::Error};
type Result<T>=std::result::Result<T,Box<dyn Error>>;

pub fn observe(trial:&TrialSession,recipe:&LabPreparation,view:&Value,large:bool,strict_full_window:bool)->Result<Value>{
    let receipt=trial.lab_preparation().ok_or("actual workload preparation receipt absent")?;
    let update=view.pointer("/payload/battle_update").ok_or("actual battle update absent")?;
    let rows=update.pointer("/events_delta/events").and_then(Value::as_array).ok_or("actual workload events absent")?;
    if rows.len()>65_536{return Err("finite workload event inventory exceeded".into());}
    let mut events=BTreeMap::new();
    for event in rows {
        let seq=event["seq"].as_u64().ok_or("event sequence absent")?;
        if events.insert(seq,event).is_some(){return Err("duplicate actual event sequence".into());}
    }
    let complete_events=update["events_delta"]["after_seq"]==0
        &&events.keys().next().copied()==Some(1)
        &&events.keys().next_back().copied()==update["events_delta"]["to_seq"].as_u64()
        &&events.len() as u64==update["events_delta"]["to_seq"].as_u64().unwrap_or(u64::MAX);
    let spawned:Vec<_>=rows.iter().filter(|e|e["event"]["type"]=="UnitSpawned"&&e["event"]["owner"]=="Opponent").collect();
    let mut positions=BTreeSet::new();
    let mut guards=true;
    for event in &spawned {
        let body=&event["event"];
        let p=&body["world_position"];
        positions.insert((p["x_milli"].as_i64(),p["y_milli"].as_i64()));
        guards&=body["unit_source"]["profile_id"]=="corroded_guard"&&event["time_ms"]==5000
            &&body["stats"]["max_health"]==10000&&body["stats"]["attack"]==0;
    }
    let expected_positions:BTreeSet<_>=(1_i64..=8).map(|x|(Some(x*1000+500),Some(5500))).collect();
    let spawn_geometry_valid=!large||(spawned.len()==8&&positions==expected_positions&&guards);
    let mut actor_evidence=Vec::new();let mut all_positive=true;let mut actual_actor_ids=BTreeSet::new();
    for placement in &recipe.placements {
        let employee=receipt.employee_ids.get(placement.employee_index).ok_or("placement employee absent from actual receipt")?.to_string();
        let deployed:Vec<_>=rows.iter().filter(|e|e["event"]["type"]=="UnitDeployed"&&e["event"]["employee_uuid"]==employee).collect();
        if deployed.len()!=1{return Err("one actual UnitDeployed per requested employee required".into());}
        let deploy=deployed[0];let actor=&deploy["event"]["unit_instance_id"];
        actual_actor_ids.insert(actor.as_str().ok_or("actual actor ID absent")?.to_owned());
        let mut producer_rows=Vec::new();let mut actor_positive=true;
        for (skill,step,amount) in [("u11_damage_a","u11_a_strike",13_u64),("u11_damage_b","u11_b_strike",17_u64)] {
            let mut positives=Vec::new();
            for hp in rows.iter().filter(|e|e["event"]["type"]=="HpChanged"&&e["event"]["source_instance_id"]==*actor
                &&e["event"]["damage_source"]=="Ability"&&e["event"]["raw_damage"]==amount&&e["event"]["final_damage"]==amount) {
                let Some(step_event)=hp["cause"]["seq"].as_u64().and_then(|seq|events.get(&seq)) else{continue};
                let body=&step_event["event"];
                if body["type"]!="AbilityStepTriggered"||body["caster_instance_id"]!=*actor||body["skill_id"]!=skill||body["step_id"]!=step
                    ||body["target_instance_id"]!=hp["event"]["target_instance_id"]{continue;}
                let Some(cast)=step_event["cause"]["seq"].as_u64().and_then(|seq|events.get(&seq)) else{continue};
                if cast["event"]["type"]!="AbilityCast"||cast["event"]["caster_instance_id"]!=*actor||cast["event"]["skill_id"]!=skill
                    ||cast["event"]["target_instance_id"]!=body["target_instance_id"]{continue;}
                let before=hp["event"]["hp_before"].as_u64().ok_or("actual HP before absent")?;
                let after=hp["event"]["hp_after"].as_u64().ok_or("actual HP after absent")?;
                if before.checked_sub(after)!=Some(amount){continue;}
                let target_spawn=spawned.iter().find(|e|e["event"]["unit_instance_id"]==body["target_instance_id"]);
                if target_spawn.is_none(){continue;}
                positives.push(json!({"hp_event":hp,"step_event":step_event,"cast_event":cast,"enemy_spawn":target_spawn}));
            }
            actor_positive&=positives.len()==1;
            producer_rows.push(json!({"skill_id":skill,"step_id":step,"literal_amount":amount,"positive_events":positives}));
        }
        all_positive&=actor_positive;
        actor_evidence.push(json!({"employee_index":placement.employee_index,"employee_id":employee,
            "unit_instance_id":actor,"actual_deployment":deploy,"positive_a_and_b":actor_positive,"producers":producer_rows}));
    }
    let unique_actors=actual_actor_ids.len()==recipe.placements.len();
    let lifecycle=trial.lifecycle();let current=lifecycle.current().ok_or("actual workload lifecycle absent")?;
    let status=current.status();let actual_ms=current.last_battle_time_ms();
    let elapsed_valid=if strict_full_window {status=="running"&&actual_ms==60000}
        else {actual_ms<=60000&&((status=="running"&&actual_ms==60000)||status=="finished"||status=="exited")};
    Ok(json!({"requested_clock_ms":60000,"actual_battle_time_ms":actual_ms,"actual_lifecycle":lifecycle,
        "actual_status":status,"strict_full_window_required":strict_full_window,"terminal_authority":"actual lifecycle status, last_battle_time_ms and terminal_verdict; raw clock result saved separately","early_terminal":status!="running"&&actual_ms<60000,
        "full_60000_ms_active":status=="running"&&actual_ms==60000,"elapsed_contract_satisfied":elapsed_valid,
        "actual_spawn_events":spawned,"eight_distinct_guard_spawn_cells_verified":if large{Some(spawn_geometry_valid)}else{None},
        "complete_event_inventory":complete_events,"requested_deployments":recipe.placements.len(),
        "actual_unique_deployed_actors":actual_actor_ids.len(),"actor_evidence":actor_evidence,
        "every_actor_positive_a_and_b":all_positive,"valid":complete_events&&spawn_geometry_valid&&unique_actors&&all_positive&&elapsed_valid,
        "meaningful_participation_rule":"each exact prepared employee maps to one deployed actor and one actual positive A13 and B17 HP event through that actor's real step/cast provenance",
        "criterion_or_approval":false}))
}
