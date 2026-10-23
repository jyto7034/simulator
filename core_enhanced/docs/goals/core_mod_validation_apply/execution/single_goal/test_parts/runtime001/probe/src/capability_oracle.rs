//! Literal independent resource and structural-recipe consumer assertions.
use game_core::{authoring::TrialSession,game::{battle::runtime_facts::{CostFact,ResourceChangeFact,FactResource,ResourceOwner as FactOwner},
    resource::{ResourceId,ResourceLifetime,ResourceOwner}}};
use serde_json::{json,Value};
use std::{collections::BTreeMap,error::Error,path::Path};
use u11_independent_public_probe::normal_preparation as normal;
use uuid::Uuid;
type Result<T>=std::result::Result<T,Box<dyn Error>>;
pub struct Tracker {
    label:String,employee:Uuid,initial_scopes:BTreeMap<u64,u32>,
    grant:Option<Value>,cost:Option<Value>,shields:BTreeMap<u64,Value>,fragment_copy:Option<Value>,
    manual_targets:BTreeMap<String,Value>,manual_release:Option<Value>,
}
impl Tracker {
    pub fn begin(label:&str,trial:&TrialSession,out:&Path)->Result<Self>{
        let employee=trial.lab_preparation().ok_or("actual employee receipt required")?.employee_ids[0];
        let mut value=Self{label:label.into(),employee,initial_scopes:BTreeMap::new(),grant:None,cost:None,shields:BTreeMap::new(),fragment_copy:None,manual_targets:BTreeMap::new(),manual_release:None};
        if label=="recipe_manual" {
            let snapshot=trial.snapshot(game_core::application::ApplicationSnapshotKind::Run,
                &trial.runtime_safety().authoring_execution(),&game_core::authoring::CancellationToken::default())?;
            let snapshot:Value=serde_json::from_str(snapshot.document())?;
            normal::save(&out.join("manual_provider_run_snapshot.json"),&snapshot)?;
            let actual=snapshot.pointer("/read_model/roster/employees").and_then(Value::as_array)
                .ok_or("actual manual roster absent")?.iter().find(|e|e["uuid"]==employee.to_string()).ok_or("manual employee absent")?;
            let copy=actual["skill_fragments"]["active_copy_uuid"].clone();assert!(copy.is_string());
            let equipped=actual["skill_fragments"]["equipped"].as_array().ok_or("actual equipped fragment list absent")?;
            assert!(equipped.iter().any(|f|f["id"]=="u11_recipe_fragment"));
            let compatibility=actual["skill_fragments"]["compatibility"].as_array().ok_or("actual fragment compatibility rows absent")?;
            let selected:Vec<_>=compatibility.iter().filter(|f|f["copy_uuid"]==copy&&f["definition_id"]=="u11_recipe_fragment").collect();
            assert_eq!(selected.len(),1);assert_eq!(selected[0]["selected_form"],"base");assert_eq!(selected[0]["is_compatible"],true);
            value.fragment_copy=Some(copy);
        }
        if label=="resources" {
            let observation=trial.observation()?;
            let view=observation.active_typed_resources().ok_or("actual resources prepared before the skill executes")?;
            let id=ResourceId::new("u11_meter".into())?;
            assert!(!view.entries().any(|(k,_)|k.resource==id),"initial value must be sparse and unmutated");
            for(scope,kind,live,_)in view.scopes(){if kind==ResourceLifetime::Deployment&&live{
                let balance=view.value(&id,ResourceOwner::Employee(employee),scope).ok_or("real initial owner and scope required")?;
                assert_eq!(balance.balance,9);assert_eq!(balance.reserved,0);assert_eq!(balance.revision,0);
                value.initial_scopes.insert(scope,balance.balance);
            }}
            assert!(!value.initial_scopes.is_empty());
            normal::save(&out.join("typed_resource_initial.json"),&json!({"employee":employee,"initial_scopes":value.initial_scopes,"actual_view":view}))?;
        }
        Ok(value)
    }
    pub fn capture_manual_targets(&mut self,projection:&Value,out:&Path)->Result<()> {
        assert_eq!(self.label,"recipe_manual");assert!(self.manual_targets.is_empty());
        let checkpoint=projection.pointer("/payload/battle_update/checkpoint").ok_or("pre-release checkpoint absent")?;
        assert_eq!(checkpoint["battle_time_ms"],8000);
        let units=checkpoint["units"].as_array().ok_or("pre-release units absent")?;
        for unit in units.iter().filter(|u|u["owner"]=="Opponent"&&u["position"]==json!({"x":4,"y":6})) {
            assert!(unit["stats"]["current_health"].as_u64().unwrap()>17);
            let id=unit["unit_instance_id"].as_str().ok_or("pre-release target identity absent")?.to_owned();
            assert!(self.manual_targets.insert(id,unit.clone()).is_none());
        }
        normal::save(&out.join("manual_pre_release_targets.json"),&json!({"checkpoint":checkpoint,
            "authored_enemy_footprint":[{"x":4,"y":6}],"expected_targets":self.manual_targets,
            "captured_before_begin_and_activate":true}))?;
        assert_eq!(self.manual_targets.len(),2,"fixed patrol places two distinct living enemies in the one-tile footprint");
        Ok(())
    }
    pub fn manual_anchor_target_count(&self)->u64 {
        assert_eq!(self.manual_targets.len(),2);assert!(self.manual_release.is_some());
        self.manual_targets.len() as u64
    }
    fn grant(&mut self,fact:&ResourceChangeFact)->Result<()> {
        if let FactResource::Typed{id,lifetime,..}=&fact.resource {
            if id.as_str()=="u11_meter" {
                assert_eq!(*lifetime,ResourceLifetime::Deployment);
                assert_eq!(fact.owner,FactOwner::Employee(self.employee));
                assert_eq!((fact.before,fact.after,fact.requested_delta),(4,7,3));
                self.grant=Some(serde_json::to_value(fact)?);
            }
        }Ok(())
    }
    fn cost(&mut self,fact:&CostFact)->Result<()> {
        if let FactResource::Typed{id,lifetime,..}=&fact.resource {
            if id.as_str()=="u11_meter"&&fact.before==7 {
                assert_eq!(*lifetime,ResourceLifetime::Deployment);
                assert_eq!(fact.owner,FactOwner::Employee(self.employee));
                assert_eq!((fact.before,fact.after,fact.requested_delta),(7,5,-2));
                assert_eq!(fact.actual_spent(),2);
                self.cost=Some(serde_json::to_value(fact)?);
            }
        }Ok(())
    }
    pub fn observe(&mut self,trial:&TrialSession,projection:&Value)->Result<()> {
        if self.label=="resources" {
            let observation=trial.observation()?;
            let facts=observation.active_runtime_facts().ok_or("actual active resource consumer facts required")?;
            let summary=facts.summary();
            if let Some(f)=&summary.latest_resource_change{self.grant(f)?;}
            if let Some(f)=&summary.latest_cost{self.cost(f)?;}
            for(_,actor)in facts.actors(){
                if let Some(f)=&actor.caused.latest_resource_change{self.grant(f)?;}
                if let Some(f)=&actor.caused.latest_cost{self.cost(f)?;}
            }
            for(_,provider)in facts.providers(){
                if let Some(f)=&provider.latest_resource_change{self.grant(f)?;}
                if let Some(f)=&provider.latest_cost{self.cost(f)?;}
            }
        }
        if self.label=="recipe_manual" {
            let events=projection.pointer("/payload/battle_update/events_delta/events").and_then(Value::as_array).ok_or("manual event projection absent")?;
            if self.manual_release.is_none() {
                if let Some(area)=events.iter().find(|e|e["event"]["type"]=="SkillAreaDeclared"&&e["event"]["skill_id"]=="u11_recipe_strike") {
                    let checkpoint=projection.pointer("/payload/battle_update/checkpoint").ok_or("release checkpoint absent")?;
                    // Public admitted casts apply max(focus_time_ms,1); compare membership
                    // again at the actual area time, not merely at the prior checkpoint.
                    assert_eq!(area["time_ms"],8001);assert_eq!(checkpoint["battle_time_ms"],area["time_ms"]);
                    let actual:std::collections::BTreeSet<_>=checkpoint["units"].as_array().ok_or("release units absent")?.iter()
                        .filter(|u|u["owner"]=="Opponent"&&u["position"]==json!({"x":4,"y":6})&&u["stats"]["current_health"].as_u64().unwrap()>0)
                        .map(|u|u["unit_instance_id"].as_str().unwrap().to_owned()).collect();
                    assert_eq!(actual,self.manual_targets.keys().cloned().collect(),"the exact two pre-release targets remain on the actual area tile at release");
                    self.manual_release=Some(checkpoint.clone());
                }
            }
            let pools=projection.pointer("/payload/battle_update/checkpoint/sustained_effects/active")
                .and_then(Value::as_array).ok_or("actual sustained pool projection required")?;
            for pool in pools {
                let effect=&pool["personal_grant"]["effect"];
                let original=effect["kind"]=="original"&&effect["site"]["action"]["definition_id"]=="u11_recipe_strike";
                let generated=effect["kind"]=="generated"&&effect["grant_rule_id"]=="finite_child";
                if !original&&!generated{continue;}
                let amount=if original{48}else{64};
                assert_eq!(pool["shield"]["capacity"],amount);assert_eq!(pool["shield"]["remaining"],amount);
                assert_eq!(pool["caster"],pool["target"],"personal-single retains the actual self recipient");
                assert_eq!(Some(&pool["personal_grant"]["fragment_copy_uuid"]),self.fragment_copy.as_ref(),"exact normal selected fragment copy is retained");
                assert!(pool["personal_grant"]["equipment_copy_uuid"].is_null());
                if original{assert_eq!(effect["site"]["effect_id"],"original");}
                else{assert_eq!(effect["local_action_id"],"child_shield");assert_eq!(effect["local_effect_id"],"child");}
                let activation=pool["activation"].as_u64().ok_or("actual pool activation required")?;
                if let Some(prior)=self.shields.insert(activation,pool.clone()){
                    assert_eq!(prior["expires_at_ms"],pool["expires_at_ms"]);
                    assert_eq!(prior["personal_grant"],pool["personal_grant"]);
                }
            }
        }Ok(())
    }
    pub fn finish(&self,trial:&TrialSession,events:&BTreeMap<u64,Value>,out:&Path)->Result<()> {
        if self.label=="resources" {
            let id=ResourceId::new("u11_meter".into())?;let observation=trial.observation()?;
            let view=observation.active_typed_resources().ok_or("actual surviving resource owner required")?;
            let rows:Vec<_>=view.entries().filter(|(k,_)|k.resource==id).collect();assert_eq!(rows.len(),1);
            let(key,amount)=rows[0];assert_eq!(key.owner,ResourceOwner::Employee(self.employee));
            assert_eq!(self.initial_scopes.get(&key.scope),Some(&9));assert_eq!(amount.balance,5);assert_eq!(amount.reserved,0);assert!(amount.revision>0);
            assert!(self.grant.is_some()&&self.cost.is_some(),"positive mutation and authored cost facts must both be observed");
            let damage:Vec<_>=events.values().filter(|e|e["event"]["type"]=="HpChanged"&&e["event"]["damage_source"]=="Ability").collect();
            assert_eq!(damage.len(),1);let damage=&damage[0]["event"];
            assert_eq!(damage["raw_damage"],14);assert_eq!(damage["final_damage"],14);
            assert_eq!(damage["hp_before"].as_u64().unwrap()-damage["hp_after"].as_u64().unwrap(),14);
            assert!(damage["hp_after"].as_u64().unwrap()>0);
            normal::save(&out.join("typed_resource_consumption.json"),&json!({"initial":9,"direct_consumed":5,"direct_grant_fact":self.grant,
                "live_read_balance":7,"damage":damage,"causal_cost_fact":self.cost,"actual_final":view,"final_balance":5}))?;
        }
        if self.label=="recipe_manual" {
            assert_eq!(self.shields.len(),2,"one original and one finite generated actual shield");
            let original=self.shields.values().find(|p|p["personal_grant"]["effect"]["kind"]=="original").ok_or("actual original recipe missing")?;
            let child=self.shields.values().find(|p|p["personal_grant"]["effect"]["kind"]=="generated").ok_or("actual generated finite child missing")?;
            assert_eq!(original["personal_grant"]["fragment_copy_uuid"],child["personal_grant"]["fragment_copy_uuid"]);
            let anchor:Vec<_>=events.values().filter(|e|e["event"]["type"]=="HpChanged"&&e["event"]["damage_source"]=="Ability"&&e["event"]["raw_damage"]==17).collect();
            assert_eq!(anchor.len(),self.manual_anchor_target_count() as usize);let anchor_events=anchor;let anchor=anchor_events[0];let time=anchor["time_ms"].as_u64().unwrap();
            let parent=|event:&Value|->Result<&Value>{
                let seq=event["cause"]["seq"].as_u64().ok_or("actual recipe parent sequence required")?;
                events.get(&seq).ok_or_else(||"actual recipe parent event absent".into())
            };
            let area=parent(anchor)?;let step=parent(area)?;let cast=parent(step)?;
            assert_eq!(area["event"]["type"],"SkillAreaDeclared");
            assert_eq!(area["event"]["skill_id"],"u11_recipe_strike");assert_eq!(area["event"]["step_id"],"anchor");
            assert_eq!(area["event"]["hit_targets"],"Enemies");assert_eq!(area["event"]["include_caster"],false);
            assert_eq!(area["event"]["duration_ms"],0);assert_eq!(area["event"]["shape"]["type"],"tile_pattern");
            assert_eq!(area["event"]["shape"]["affected_tiles"],json!([{ "x":4,"y":6 }]));
            assert_eq!(step["event"]["type"],"AbilityStepTriggered");
            assert_eq!(step["event"]["skill_id"],"u11_recipe_strike");assert_eq!(step["event"]["step_id"],"anchor");
            assert_eq!(cast["event"]["type"],"AbilityCast");assert_eq!(cast["event"]["skill_id"],"u11_recipe_strike");
            let caster=&cast["event"]["caster_instance_id"];
            assert_eq!(cast["event"]["target_instance_id"],*caster,"actual manual cast retains selected self");
            assert_eq!(step["event"]["caster_instance_id"],*caster);assert_eq!(step["event"]["target_instance_id"],*caster);
            assert_eq!(area["event"]["caster_instance_id"],*caster);
            assert_eq!(area["event"]["target"]["type"],"Unit");assert_eq!(area["event"]["target"]["unit_instance_id"],*caster);
            let mut actual_targets=std::collections::BTreeSet::new();
            for hit in &anchor_events {
                let target=hit["event"]["target_instance_id"].as_str().ok_or("actual area damage target absent")?;
                assert!(actual_targets.insert(target.to_owned()),"one actual HP mutation per target, never duplicate terminals");
                let before=self.manual_targets.get(target).ok_or("area hit was not a pre-release eligible target")?;
                assert_eq!(hit["event"]["source_instance_id"],*caster);assert_eq!(hit["cause"]["seq"],area["seq"]);
                assert_eq!(hit["time_ms"],time);assert_eq!(hit["event"]["raw_damage"],17);assert_eq!(hit["event"]["final_damage"],17);
                assert_eq!(hit["event"]["hp_before"],before["stats"]["current_health"]);
                assert_eq!(hit["event"]["hp_before"].as_u64().unwrap()-hit["event"]["hp_after"].as_u64().unwrap(),17);
            }
            assert_eq!(actual_targets,self.manual_targets.keys().cloned().collect());
            for event in [area,step,cast]{assert_eq!(event["time_ms"],time);}
            assert_eq!(original["caster"],*caster);assert_eq!(child["caster"],*caster);
            assert_eq!(original["expires_at_ms"],time+250);assert_eq!(child["expires_at_ms"],time+10+350);
            let end=normal::reconnect(trial)?;
            let pools=end.pointer("/payload/battle_update/checkpoint/sustained_effects/active").and_then(Value::as_array).ok_or("final pool state required")?;
            assert!(pools.iter().all(|p|!self.shields.contains_key(&p["activation"].as_u64().unwrap())),"both actual finite shields must expire");
            let units=end.pointer("/payload/battle_update/checkpoint/units").and_then(Value::as_array).ok_or("actual recipe units absent")?;
            let player=units.iter().find(|u|u["unit_instance_id"]==*caster).ok_or("actual recipe caster absent")?;
            let enemies:Vec<_>=units.iter().filter(|u|u["unit_instance_id"].as_str().is_some_and(|id|actual_targets.contains(id))).collect();
            assert_eq!(enemies.len(),self.manual_targets.len());assert!(enemies.iter().all(|u|u["owner"]=="Opponent"));
            assert_eq!(player["owner"],"Player");
            let deployed=end.pointer("/payload/battle_update/checkpoint/deployment/deployed_units").and_then(Value::as_array).ok_or("actual recipe deployment absent")?;
            assert!(deployed.iter().any(|u|u["unit_instance_id"]==*caster&&u["employee_uuid"]==self.employee.to_string()));
            normal::save(&out.join("actual_recipe_finite_child.json"),&json!({"actual_pools":self.shields,"anchor_events":anchor_events,
                "pre_release_targets":self.manual_targets,"actual_release_checkpoint":self.manual_release,
                "actual_area":area,"actual_step":step,"actual_manual_cast":cast,"actual_player":player,"actual_enemies":enemies,
                "original_capacity":48,"generated_capacity":64,"both_expired":true,"actual_final":end}))?;
        }Ok(())
    }
}
