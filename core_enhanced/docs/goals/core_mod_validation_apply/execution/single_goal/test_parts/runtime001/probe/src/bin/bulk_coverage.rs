//! Real World preparation, exact independent required inventory, per-bucket
//! witnesses and gameplay effects. This is diagnostic and never issues approval.
use game_core::{authoring::{CancellationToken,WorldLabPreparation},
    game::data::content_pack::{ContentSelection,InputLimits},
    mod_validation::{Candidate,LabProfile,LabRun,LabSuite,ModValidationErrorKind,TrustedBase}};
use serde_json::{json,Value};
use std::{error::Error,fs,path::Path};
use u11_independent_public_probe::{fixture_coverage as oracle,normal_preparation as normal};
#[path="../capability_oracle.rs"] mod capability_oracle;
#[path="../extended_oracle.rs"] mod extended_oracle;

fn site(report:&game_core::mod_validation::CoverageReport,id:&str,action:&str,effect:&str,stage:usize)->u64 {
    oracle::count(report,id,action,effect,"skill_effect",stage)
}
fn actual_hp_loss(event:&Value,amount:u64){
    let hp=&event["event"];assert_eq!(hp["type"],"HpChanged");
    assert_eq!(hp["hp_before"].as_u64().unwrap()-hp["hp_after"].as_u64().unwrap(),amount);
    assert!(hp["hp_after"].as_u64().unwrap()>0,"positive target must remain nonlethal");
}
// Zero literal damage emits no HP mutation event. Tie actual LegalNoop to the
// live cast-selected enemy, its preceding HP event, and the bucket checkpoint.
fn zero_noop_evidence(events:&std::collections::BTreeMap<u64,Value>,projection:&Value)->Result<Value,Box<dyn Error>> {
    let steps:Vec<_>=events.values().filter(|e|e["event"]["type"]=="AbilityStepTriggered"
        &&e["event"]["skill_id"]=="u11_zero_damage"&&e["event"]["step_id"]=="u11_zero").collect();
    assert_eq!(steps.len(),1,"one actual zero-damage step required");let step=steps[0];
    let step_seq=step["seq"].as_u64().unwrap();let time=step["time_ms"].as_u64().unwrap();
    let cast=events.get(&step["cause"]["seq"].as_u64().ok_or("zero step parent absent")?).ok_or("zero cast absent")?;
    assert_eq!(cast["event"]["type"],"AbilityCast");assert_eq!(cast["event"]["skill_id"],"u11_zero_damage");
    let target=&step["event"]["target_instance_id"];assert_eq!(*target,cast["event"]["target_instance_id"]);
    let units=projection.pointer("/payload/battle_update/checkpoint/units").and_then(Value::as_array).ok_or("actual noop checkpoint absent")?;
    let unit=units.iter().find(|u|u["unit_instance_id"]==*target).ok_or("actual zero target absent")?;
    assert_eq!(unit["owner"],"Opponent");assert!(unit["stats"]["current_health"].as_u64().unwrap()>0);
    let prior=events.range(..step_seq).rev().map(|(_,event)|event).find(|e|e["event"]["type"]=="HpChanged"
        &&e["event"]["target_instance_id"]==*target).ok_or("actual target HP before zero step absent")?;
    assert_eq!(prior["time_ms"],time);assert_eq!(prior["event"]["reason"],"BasicAttack");
    assert_eq!(unit["stats"]["current_health"],prior["event"]["hp_after"],"zero operation leaves live enemy HP unchanged");
    assert!(!events.range(step_seq..).any(|(_,e)|e["event"]["type"]=="HpChanged"
        &&e["event"]["target_instance_id"]==*target),"zero step emits no HP mutation in its actual bucket");
    Ok(json!({"actual_zero_step":step,"actual_cast":cast,"same_live_enemy":unit,"prior_hp_event":prior,
        "hp_before_zero":prior["event"]["hp_after"],"hp_after_zero":unit["stats"]["current_health"],
        "zero_hp_delta":0,"no_hp_mutation_event_after_zero_step_in_bucket":true}))
}
fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=4&&args.len()!=5{return Err("usage: bulk_coverage FROZEN_LAB_FIXTURE_ROOT REGISTRY_JSON INDEPENDENT_CASE NEW_OUTPUT [RUNTIME_SAFETY_RON]".into());}
    let root=Path::new(&args[0]);let label=args[2].to_str().ok_or("UTF-8 fixture label required")?;
    let out=Path::new(&args[3]);fs::create_dir(out)?;
    let (settings,profile_bytes)=normal::runtime_profile(args.get(4).map(|p|Path::new(p)),&out.join("runtime_context.json"))?;
    // Fail before gameplay if the nonlethal/full-health fixture was not prepared.
    let lab_base:Value=serde_json::from_slice(&fs::read(root.join("LAB_BASE_ORACLE.json"))?)?;
    assert_eq!(lab_base["authored_target"]["max_health"],10_000);
    let base=TrustedBase::acquire(root,ContentSelection{packs:vec!["official".into()],winners:vec![]},
        &InputLimits::default(),settings.clone(),&CancellationToken::default())?;
    let registry=fs::read(&args[1])?;
    let candidate=Candidate::acquire(root,&registry,&base,&InputLimits::default(),&CancellationToken::default())?;
    let expected=oracle::expected(label)?;
    normal::save(&out.join("candidate_and_independent_inventory.json"),&json!({"candidate":candidate.view(),"expected":expected}))?;
    let mut document=normal::prefix(&candidate,&out.join("normal_prefix"))?;
    let mut recipe=normal::recipe(label,1)?;
    let offsets=if label=="objects" {vec![game_core::game::resources::Position::new(1,0)]}else{vec![]};
    normal::place_one_with_free_offsets(&candidate,&document,&mut recipe,&offsets,&out.join("placement_discovery.json"))?;
    let prefix_len=document["preparation_steps"].as_array().unwrap().len();
    // Each command advances exactly one real event bucket. Prefix+steps respects
    // the public 256-step ceiling; the experiment never bypasses it per actor.
    let clocks:Vec<_>=(0..254-prefix_len).map(|_|json!({"kind":"clock","clock":{"mode":"next_event_bucket"}})).collect();
    document["steps"]=json!(if label=="recipe_manual"{extended_oracle::manual_steps(prefix_len)}else{clocks});
    normal::save(&out.join("proposed_finite_input.json"),&json!({"scenario":document,"preparation":recipe}))?;
    let mut trial=normal::prepare(&candidate,&document,&recipe)?;
    let receipt=trial.lab_preparation().expect("actual World preparation receipt");
    assert_eq!(receipt.employee_ids.len(),1);assert_eq!(receipt.deployed_count,1);
    let current=trial.lifecycle().current().expect("actual battle lifecycle");
    assert!(current.time_zero_completed());assert_eq!(current.key().battle_uuid,receipt.battle_id);
    normal::save(&out.join("actual_preparation.json"),&json!({"receipt":receipt,"ready":trial.ready_receipt().document(),
        "lifecycle":trial.lifecycle(),"reconnect":normal::reconnect(&trial)?}))?;
    let generation=&candidate.generation().identity().semantic_sha256;
    let mut before=trial.lab_coverage()?;oracle::inventory(&before,generation,&expected)?;
    let mut capability=capability_oracle::Tracker::begin(label,&trial,out)?;
    normal::save(&out.join("coverage_before_execution.json"),&before)?;
    assert!(before.rows.iter().filter(|r|r.requirement.key.producer_family=="skill_effect")
        .all(|r|r.counts[1..].iter().all(|n|*n==0)),
        "a catalog/preparation receipt cannot fabricate an OnAttack witness");
    let mut same_bucket=None;let mut all_events=std::collections::BTreeMap::<u64,Value>::new();
    let mut consumed=0usize;let mut complete_at=None;let mut finished=false;let mut terminal_seen=false;
    while !trial.is_complete() {
        let index=trial.next_step_index();
        let result=trial.advance(&CancellationToken::default()).map_err(|e|format!("actual bucket {index}: {e:?}"))?;
        let public=result.public_result().cloned();let report=trial.lab_coverage()?;
        oracle::inventory(&report,generation,&expected)?;
        let projection=normal::reconnect(&trial)?;
        capability.observe(&trial,&projection)?;
        let events=projection.pointer("/payload/battle_update/events_delta/events").and_then(Value::as_array)
            .ok_or("actual event delta missing")?;
        for event in events {let seq=event["seq"].as_u64().ok_or("event sequence absent")?;
            if let Some(old)=all_events.insert(seq,event.clone()){assert_eq!(old,*event,"an event sequence cannot change payload");}}
        let lifecycle=trial.lifecycle();let current=lifecycle.current().expect("retained lifecycle after bucket");
        let time=current.last_battle_time_ms();
        normal::save(&out.join(format!("bucket_{consumed:03}.json")),&json!({"step":index,"public_result":public,
            "lifecycle":lifecycle,"coverage":report,"observation":trial.observation()?,"reconnect":projection}))?;
        if label=="recipe_manual"&&consumed==0{
            extended_oracle::manual_ready(&projection,out)?;capability.capture_manual_targets(&projection,out)?;
        }
        if label=="terminal"&&site(&report,"u11_terminal_chain","ordered","zero_expr",3)>
            site(&before,"u11_terminal_chain","ordered","zero_expr",3) {
            extended_oracle::terminal(&report,&all_events,&projection,out)?;terminal_seen=true;
        }
        if label=="noop"&&site(&report,"u11_zero_damage","u11_zero","u11_zero_effect",3)>
            site(&before,"u11_zero_damage","u11_zero","u11_zero_effect",3) {
            normal::save(&out.join("actual_zero_noop.json"),&zero_noop_evidence(&all_events,&projection)?)?;
        }
        if label=="ab"||label=="host_ab" {
            let a=site(&report,"u11_damage_a","u11_a_strike","u11_a_damage",4);
            let b=site(&report,"u11_damage_b","u11_b_strike","u11_b_damage",4);
            if a>site(&before,"u11_damage_a","u11_a_strike","u11_a_damage",4)
                &&b>site(&before,"u11_damage_b","u11_b_strike","u11_b_damage",4){same_bucket=Some((index,time));}
        }
        consumed+=1;
        let reached=if label=="noop" {
            site(&report,"u11_full_health_heal","u11_heal","u11_heal_seven",3)>0
                &&site(&report,"u11_zero_damage","u11_zero","u11_zero_effect",3)>0
        }else if label=="terminal"{terminal_seen}
        else if label=="recipe"{all_events.values().any(|e|e["event"]["type"]=="HpChanged"
            &&e["event"]["reason"]=="BasicAttack"&&e["event"]["raw_damage"]==10)}
        else{report.is_complete()};
        if reached&&complete_at.is_none(){complete_at=Some(time);}
        finished=current.status()=="finished"||current.status()=="exited";
        before=report;
        // A second full second after required observations detects late duplicate
        // triggers and allows the 250ms object terminal/reclaim path to settle.
        if finished||complete_at.is_some_and(|at|time>=at.saturating_add(1_000)){break;}
    }
    normal::save(&out.join("coverage_final.json"),&before)?;
    normal::save(&out.join("all_actual_events.json"),&all_events.values().collect::<Vec<_>>())?;
    assert!(complete_at.is_some(),"actual finite normal fixture did not reach its independent witnesses");
    for row in &before.rows {let k=&row.requirement.key;
        if k.producer_family=="skill_effect"&&label!="recipe" {
            assert!(row.counts[1]>0,"actual trigger witness missing at {}",k.definition_id);
            assert_eq!(row.counts[2],0,"positive fixture unexpectedly ineligible");
        }
    }
    if label=="ab"||label=="host_ab" {
        let (step,time)=same_bucket.expect("both definitions must be consumed in one actual NextEventBucket");
        for (id,action,effect,amount) in [("u11_damage_a","u11_a_strike","u11_a_damage",13),
            ("u11_damage_b","u11_b_strike","u11_b_damage",17)] {
            assert_eq!(site(&before,id,action,effect,4),1,"max_triggers_per_battle is one");
            let matching:Vec<_>=all_events.values().filter(|event|event["time_ms"]==time
                &&event["event"]["type"]=="HpChanged"&&event["event"]["damage_source"]=="Ability"
                &&event["event"]["raw_damage"]==amount&&event["event"]["final_damage"]==amount).collect();
            assert_eq!(matching.len(),1,"independent actual HP effect for {id}, bucket {step}");
            let hp=&matching[0]["event"];
            assert_eq!(hp["hp_before"].as_u64().unwrap()-hp["hp_after"].as_u64().unwrap(),amount);
            assert!(hp["hp_after"].as_u64().unwrap()>0,"nonlethal positive target required");
        }
    }
    if label=="noop" {
        assert_eq!(site(&before,"u11_full_health_heal","u11_heal","u11_heal_seven",3),1);
        assert_eq!(site(&before,"u11_full_health_heal","u11_heal","u11_heal_seven",4),0,
            "full-health no-op is observed but cannot substitute for required positive heal");
        assert_eq!(site(&before,"u11_zero_damage","u11_zero","u11_zero_effect",3),1);
        assert!(!before.is_complete());
    }else if label=="terminal"||label=="recipe"{assert!(!before.is_complete());}
    else{assert!(before.is_complete());}
    if label=="terminal" {
        assert!(terminal_seen);
        let end=normal::reconnect(&trial)?;
        let pools=end.pointer("/payload/battle_update/checkpoint/sustained_effects/active").and_then(Value::as_array).ok_or("actual final shield pools missing")?;
        assert!(pools.iter().all(|pool|pool["skill_id"]!="u11_terminal_chain"),"finite shield expired through real clock");
    }
    if label=="recipe" {
        assert!(all_events.values().any(|e|e["event"]["type"]=="HpChanged"&&e["event"]["reason"]=="BasicAttack"&&e["event"]["raw_damage"]==10));
        assert!(!all_events.values().any(|e|e["event"]["skill_id"]=="u11_recipe_strike"
            &&["TriggeredAbilityProc","AbilityCast","AbilityStepTriggered"].iter().any(|kind|e["event"]["type"]==*kind)),
            "OnAttack enemy hint cannot provide the required explicit player selection");
        for row in &before.rows {assert_eq!(row.counts[4],0,"automatic recipe cannot claim positive consumption");}
        normal::save(&out.join("automatic_recipe_held.json"),&json!({"coverage":before,"actual_events":all_events,
            "actual_basic_attack_reached":true,"manual_selection_absent":true,"approval_claimed":false}))?;
    }
    if label=="structural" {
        let mut target=None;
        for amount in [30_u64,75] {
            let rows:Vec<_>=all_events.values().filter(|e|e["event"]["type"]=="HpChanged"
                &&e["event"]["damage_source"]=="Ability"&&e["event"]["raw_damage"]==amount).collect();
            assert_eq!(rows.len(),1,"original30 and generated75 each execute once");
            actual_hp_loss(rows[0],amount);assert_eq!(rows[0]["event"]["final_damage"],amount);
            if let Some(old)=target.replace(rows[0]["event"]["target_instance_id"].clone()){
                assert_eq!(old,rows[0]["event"]["target_instance_id"]);
            }
        }
        assert_eq!(site(&before,"u11_structural_strike","hit","original",4),1);
        assert_eq!(oracle::count(&before,"u11_structural_relic","hit","bonus","grant/Normal/grant",4),1);
    }
    if label=="causal" {
        let damage:Vec<_>=all_events.values().filter(|e|e["event"]["type"]=="HpChanged"
            &&e["event"]["damage_source"]=="Ability").collect();
        let mut sum=0_u64;
        for(amount,count)in [(11_u64,1_usize),(12,1),(26,1),(7,3),(2,3),(3,3),(5,3)] {
            let rows:Vec<_>=damage.iter().copied().filter(|e|e["event"]["raw_damage"]==amount).collect();
            assert_eq!(rows.len(),count,"literal direct/wedge/source-filtered causal multiplicity for {amount}");
            for event in rows{actual_hp_loss(event,amount);sum+=amount;}
        }
        assert_eq!(damage.len(),15,"reaction descendants cannot retrigger the direct-only subscribers");
        assert_eq!(sum,100,"independent direct49 + echo21 + joins30 HP loss");
        for amount in [11_u64,12,26] {
            let direct=damage.iter().find(|e|e["event"]["raw_damage"]==amount).unwrap();
            let time=direct["time_ms"].as_u64().unwrap();
            let once=|amount:u64,time:u64|->&Value{
                let rows:Vec<_>=damage.iter().copied().filter(|e|e["event"]["raw_damage"]==amount&&e["time_ms"]==time).collect();
                assert_eq!(rows.len(),1,"one actual branch/continuation per direct fact");rows[0]
            };
            let echo=once(7,time);let left=once(2,time);let right=once(3,time+5);let joined=once(5,time+5);
            assert!(direct["seq"].as_u64().unwrap()<echo["seq"].as_u64().unwrap());
            assert!(left["seq"].as_u64().unwrap()<joined["seq"].as_u64().unwrap());
            assert!(right["seq"].as_u64().unwrap()<joined["seq"].as_u64().unwrap());
        }
        let pulses:Vec<_>=all_events.values().filter(|e|e["event"]["type"]=="HpChanged"&&e["event"]["reason"]=="HealthCost").collect();
        assert!(!pulses.is_empty());for pulse in pulses{actual_hp_loss(pulse,3);}
        assert_eq!(oracle::count(&before,"u11_causal_relic","wedge","","wedge_positive/Normal",4),3);
        for(id,family)in [("echo","causal/Normal/echo/0/0/program"),("left","causal/Normal/joined/1/0/left"),
            ("right","causal/Normal/joined/1/1/right"),("joined","causal/Normal/joined/2/2/then")] {
            assert_eq!(oracle::count(&before,"u11_causal_relic",id,id,family,4),3);
        }
    }
    if label=="objects" {
        assert_eq!(oracle::count(&before,"u11_finite_actor","","","object_spawn",4),1);
        assert_eq!(oracle::count(&before,"u11_finite_actor","","","object_terminal",5),1);
        let removed:Vec<_>=all_events.values().filter(|e|e["event"]["type"]=="IndependentObjectRemoved").collect();
        assert_eq!(removed.len(),1,"spawn must have an actual removal event, not merely final absence");
    }
    if label=="resources" {
        for effect in ["mutate","read"]{assert_eq!(site(&before,"u11_resource_strike","spend_and_read",effect,4),1);}
        assert_eq!(oracle::count(&before,"u11_resource_relic","spend","spend","causal/Normal/causal_spend/0/0/program",4),1);
    }
    if label=="recipe_manual" {
        assert_eq!(site(&before,"u11_recipe_strike","anchor","anchor",4),capability.manual_anchor_target_count());
        assert_eq!(site(&before,"u11_recipe_strike","original_shield","original",4),1);
        assert_eq!(oracle::count(&before,"u11_recipe_relic","child_shield","child","grant/Normal/finite_child",4),1);
        for rule in ["original_single","original_add","generated_single","generated_add"]{
            assert_eq!(oracle::count(&before,"u11_recipe_relic",rule,"","relic_rule/Normal",4),1);
        }
    }
    capability.finish(&trial,&all_events,out)?;
    document["steps"].as_array_mut().unwrap().truncate(consumed);
    if label=="causal"||label=="resources"||label=="recipe"||label=="recipe_manual" {
        assert!(!finished,"cleanup fixture must exercise retreat with the periodic owner still active");
        assert!(trial.observation()?.active_runtime_facts().is_some());
        let has_typed_resources=trial.observation()?.active_typed_resources().is_some();
        assert_eq!(has_typed_resources,label!="causal",
            "causal uses the empty official resource catalog; resources/recipe use the explicit seeded capability base");
        let retained_before=serde_json::to_value(trial.observation()?)?;
        let mut exit_document=document.clone();
        let source=prefix_len+consumed-1;
        exit_document["steps"].as_array_mut().unwrap().push(json!({"kind":"command",
            "command":{"type":"retreat_battle"},
            "context":{"expected_battle_authority":{"battle_uuid":null,"battle_state_revision":null}},
            "bindings":[
                {"from":{"step":source,"pointer":"/payload/battle_uuid"},"to":"/context/expected_battle_authority/battle_uuid","value_type":"string"},
                {"from":{"step":source,"pointer":"/payload/battle_update/battle_state_revision"},"to":"/context/expected_battle_authority/battle_state_revision","value_type":"number"}
            ]}));
        exit_document["steps"].as_array_mut().unwrap().push(json!({"kind":"snapshot","snapshot":"run"}));
        normal::save(&out.join("causal_retreat_input.json"),&exit_document)?;
        let mut exited=normal::prepare(&candidate,&exit_document,&recipe)?;
        exited.run_to_end(&CancellationToken::default()).map_err(|e|format!("actual causal retreat: {e:?}"))?;
        assert_eq!(exited.lifecycle().current().expect("retained lifecycle").status(),"exited");
        let observation=exited.observation()?;
        assert!(observation.active().is_none(),"retreat removes the active observation owner");
        assert!(observation.active_runtime_facts().is_none(),"retreat removes active actions/providers and causal owner");
        assert!(observation.active_typed_resources().is_none(),"retreat removes battle resource ownership");
        normal::save(&out.join("causal_retreat_result.json"),&json!({"lifecycle":exited.lifecycle(),
            "observation":observation,"coverage":exited.lab_coverage()?,
            "receipts":exited.receipt_documents().collect::<Vec<_>>(),
            "scope":"actual retreat removes active owners; retained terminal diagnostics are allowed"}))?;
        assert_eq!(serde_json::to_value(trial.observation()?)?,retained_before,
            "separate cleanup replay cannot mutate the accepted positive trial");
    }
    let suite:LabSuite=serde_json::from_value(json!({"revision":format!("u11-independent-{label}-normal-v1"),
        "cases":[{"id":label,"class":"normal","scenario":serde_json::to_vec(&document)?,"preparation":recipe}]}))?;
    normal::save(&out.join("normal_suite.json"),&suite)?;
    let profile=LabProfile{revision:"u11-independent-explicit-profile-diagnostic-v1".into(),repeats:3,
        timeout_ms:300_000,max_output_bytes:16*1024*1024,runtime_safety_ron:profile_bytes,criterion:None};
    normal::save(&out.join("diagnostic_profile.json"),&profile)?;
    let diagnostic=LabRun::diagnose(&candidate,&suite,&profile,&CancellationToken::default())?;
    normal::save(&out.join("normal_driver_replay.json"),diagnostic.outcome())?;
    assert!(!diagnostic.outcome().is_passed());assert!(!diagnostic.metrics().is_complete());
    assert_eq!(serde_json::to_value(diagnostic.coverage())?,serde_json::to_value(&before)?,
        "normal driver must consume the identical frozen bulk recipe and exact producer semantics");

    // Candidate and accepted trial remain independent of failed/cancelled draft
    // preparations. These are controlled orderings, not timing-dependent races.
    let accepted_receipts=trial.receipt_documents().map(str::to_owned).collect::<Vec<_>>();
    let accepted_work=trial.accepted_work();let accepted_view=normal::reconnect(&trial)?;
    let mut invalids=Vec::new();
    let mut invalid=recipe.clone();invalid.schema_version=2;invalids.push(("unsupported_version",invalid));
    let mut invalid=recipe.clone();invalid.employees[0].count=0;invalids.push(("zero_employee_group",invalid));
    let mut invalid=recipe.clone();invalid.employees[0].count=4097;invalids.push(("employee_one_over",invalid));
    let mut invalid=recipe.clone();invalid.placements[0].employee_index=1;invalids.push(("out_of_range_index",invalid));
    let mut invalid=recipe.clone();invalid.placements.push(invalid.placements[0].clone());invalids.push(("duplicate_placement",invalid));
    let mut invalid=recipe.clone();invalid.encounter_id="u11_missing_encounter".into();invalids.push(("missing_encounter",invalid));
    for (name,invalid) in invalids {
        let bad_suite:LabSuite=serde_json::from_value(json!({"revision":"u11-independent-invalid-preparation-v1",
            "cases":[{"id":name,"class":"normal","scenario":serde_json::to_vec(&document)?,"preparation":invalid}]}))?;
        let error=LabRun::diagnose(&candidate,&bad_suite,&profile,&CancellationToken::default()).err()
            .expect("invalid World preparation cannot execute or approve");
        normal::save(&out.join(format!("invalid_{name}.json")),&error)?;
        assert_eq!(error.kind,ModValidationErrorKind::Incomplete);
        assert_eq!(trial.receipt_documents().map(str::to_owned).collect::<Vec<_>>(),accepted_receipts);
        assert_eq!(trial.accepted_work(),accepted_work);assert_eq!(normal::reconnect(&trial)?,accepted_view);
    }
    let cancel=CancellationToken::default();cancel.cancel();
    let error=WorldLabPreparation::prepare(candidate.authoring_content(),normal::checked(&document)?,recipe.clone(),
        normal::limits(&candidate)?,&cancel).err().expect("cancelled preparation cannot publish");
    normal::save(&out.join("cancelled_preparation.json"),&json!({"error":format!("{error:?}")}))?;
    assert_eq!(trial.receipt_documents().map(str::to_owned).collect::<Vec<_>>(),accepted_receipts);
    assert_eq!(normal::reconnect(&trial)?,accepted_view);
    normal::save(&out.join("RESULT.json"),&json!({"passed":true,"case":label,"scope":"normal World diagnostic coverage",
        "actual_employee_count":1,"actual_deployed_count":1,"event_buckets":consumed,"battle_finished":finished,
        "exact_independent_inventory":true,"same_bucket":same_bucket,"criterion_adopted":false,"approval_claimed":false,
        "calibration_workload_claimed":false}))?;
    println!("{}",json!({"passed":true,"approval_claimed":false,"output":out}));Ok(())
}
