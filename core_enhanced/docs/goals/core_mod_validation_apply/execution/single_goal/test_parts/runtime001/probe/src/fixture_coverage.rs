//! Literal required-site inventory derived from independent fixture RON, not
//! from Candidate.changed_definitions or the product coverage plan.
use game_core::mod_validation::{CoverageReport,CoverageStage};
use serde::Serialize;
use std::{collections::BTreeSet,error::Error};
use uuid::Uuid;

#[derive(Clone,Debug,Serialize,PartialEq,Eq,PartialOrd,Ord)]
pub struct Site {
    pub kind:String,pub definition:String,pub action:String,pub effect:String,pub family:String,
    pub required_stage:usize,
}
impl Site {
    fn new(kind:&str,definition:&str,action:&str,effect:&str,family:&str,stage:usize)->Self {
        Self{kind:kind.into(),definition:definition.into(),action:action.into(),effect:effect.into(),
            family:family.into(),required_stage:stage}
    }
}
fn skill(rows:&mut Vec<Site>,id:&str,action:&str,effect:&str,stage:usize) {
    rows.push(Site::new("Skill",id,action,effect,"skill_effect",stage));
}
fn equipment(rows:&mut Vec<Site>,id:&str,serial:u128,ability:&str) {
    let uuid=Uuid::from_u128(0xc0de1100000000000000000000000000|serial);
    rows.push(Site::new("Equipment",id,"","",&format!("equipment_owner/{uuid}"),4));
    rows.push(Site::new("Equipment",id,ability,"0",&format!("equipment_activation/{uuid}"),4));
}
pub fn expected(case:&str)->Result<Vec<Site>,Box<dyn Error>> {
    let mut rows=Vec::new();
    match case {
        "a"|"b"|"ab"|"host_a"|"host_b"|"host_ab"=>{
            let host=case.starts_with("host_");let sides=case.strip_prefix("host_").unwrap_or(case);
            if sides.contains('a') {skill(&mut rows,"u11_damage_a","u11_a_strike","u11_a_damage",4);
                equipment(&mut rows,if host{"standard_training_sword"}else{"u11_weapon_a"},if host{1001}else{1},"u11_damage_a");}
            if sides.contains('b') {skill(&mut rows,"u11_damage_b","u11_b_strike","u11_b_damage",4);
                equipment(&mut rows,if host{"standard_armor"}else{"u11_armor_b"},if host{1002}else{2},"u11_damage_b");}
        }
        "noop"=>{
            skill(&mut rows,"u11_full_health_heal","u11_heal","u11_heal_seven",4);
            skill(&mut rows,"u11_zero_damage","u11_zero","u11_zero_effect",3);
            equipment(&mut rows,"u11_heal_weapon",7,"u11_full_health_heal");
            equipment(&mut rows,"u11_zero_armor",8,"u11_zero_damage");
        }
        "terminal"=>{
            for effect in ["cost","heal_positive","heal_noop","shield","absorbed","zero_expr"] {
                skill(&mut rows,"u11_terminal_chain","ordered",effect,4);
            }
            equipment(&mut rows,"u11_terminal_weapon",9,"u11_terminal_chain");
        }
        "objects"=>{
            skill(&mut rows,"u11_object_create","place","create",4);
            equipment(&mut rows,"u11_object_weapon",6,"u11_object_create");
            rows.push(Site::new("IndependentObject","u11_finite_actor","","","object_spawn",4));
            rows.push(Site::new("IndependentObject","u11_finite_actor","","","object_terminal",5));
        }
        "structural"=>{
            skill(&mut rows,"u11_structural_strike","hit","original",4);
            equipment(&mut rows,"u11_structural_weapon",4,"u11_structural_strike");
            for id in ["original_add","bonus_add","bonus_scale"] {
                rows.push(Site::new("Relic","u11_structural_relic",id,"","relic_rule/Normal",4));
            }
            rows.push(Site::new("Relic","u11_structural_relic","hit","bonus","grant/Normal/grant",4));
        }
        "resources"=>{
            for effect in ["mutate","read"]{skill(&mut rows,"u11_resource_strike","spend_and_read",effect,4);}
            equipment(&mut rows,"u11_resources_weapon",6001,"u11_resource_strike");
            rows.push(Site::new("Relic","u11_resource_relic","spend","spend","causal/Normal/causal_spend/0/0/program",4));
        }
        "recipe"|"recipe_manual"=>{
            skill(&mut rows,"u11_recipe_strike","anchor","anchor",4);
            skill(&mut rows,"u11_recipe_strike","original_shield","original",4);
            if case=="recipe"{equipment(&mut rows,"u11_recipe_weapon",6002,"u11_recipe_strike");}
            else{rows.push(Site::new("SkillFragment","u11_recipe_fragment","base","u11_recipe_strike","fragment_effect",4));}
            for id in ["original_single","original_add","generated_single","generated_add"]{
                rows.push(Site::new("Relic","u11_recipe_relic",id,"","relic_rule/Normal",4));
            }
            rows.push(Site::new("Relic","u11_recipe_relic","child_shield","child","grant/Normal/finite_child",4));
        }
        "causal"=>{
            for id in ["first","second","third"]{skill(&mut rows,"u11_causal_strike",id,id,4);}
            equipment(&mut rows,"u11_causal_weapon",5,"u11_causal_strike");
            rows.push(Site::new("Relic","u11_causal_relic","wedge","","wedge_positive/Normal",4));
            for (id,family) in [("echo","causal/Normal/echo/0/0/program"),
                ("left","causal/Normal/joined/1/0/left"),("right","causal/Normal/joined/1/1/right"),
                ("joined","causal/Normal/joined/2/2/then"),("pulse","causal/Normal/pulse/0/0/program")] {
                rows.push(Site::new("Relic","u11_causal_relic",id,id,family,4));
            }
        }
        _ if case.starts_with("host_load_")=>{
            let parts:Vec<_>=case.split('_').collect();
            if parts.len()!=4{return Err("unknown load fixture label".into());}
            let (tier_index,count)=match parts[2]{"small"=>(0,64),"expanded"=>(1,256),"pathological"=>(2,1024),_=>return Err("unknown load tier".into())};
            for (side,index,item) in [("a",0,"standard_training_sword"),("b",1,"standard_armor")] {
                if !parts[3].contains(side){continue;}
                let id=format!("u11_load_{}_{side}",parts[2]);
                for effect in 0..count{skill(&mut rows,&id,"load",&format!("effect_{effect}"),if effect==0{4}else{3});}
                equipment(&mut rows,item,2000+tier_index*2+index,&id);
            }
        }
        _=>return Err("unknown independent coverage fixture".into()),
    }
    rows.sort();Ok(rows)
}
pub fn stage(stage:CoverageStage)->usize {
    match stage{CoverageStage::CatalogValidated=>0,CoverageStage::Triggered=>1,CoverageStage::Ineligible=>2,
        CoverageStage::LegalNoop=>3,CoverageStage::ConsumedOrEffect=>4,CoverageStage::TerminatedOrReclaimed=>5}
}
pub fn inventory(report:&CoverageReport,generation:&str,expected:&[Site])->Result<(),Box<dyn Error>> {
    if report.overflow||!report.unsupported.is_empty(){return Err("required collector overflow/unsupported".into());}
    let mut actual=BTreeSet::new();
    for row in &report.rows {
        let k=&row.requirement.key;
        if k.generation!=generation{return Err("foreign coverage generation".into());}
        let key=Site::new(&k.definition_kind,&k.definition_id,&k.action_id,&k.effect_id,&k.producer_family,stage(row.requirement.required_stage));
        if !actual.insert(key){return Err("duplicate coverage requirement".into());}
    }
    let expected:BTreeSet<_>=expected.iter().cloned().collect();
    if actual!=expected{return Err(format!("independent required inventory mismatch; missing={:?}; unexpected={:?}",
        expected.difference(&actual).collect::<Vec<_>>(),actual.difference(&expected).collect::<Vec<_>>()).into());}
    Ok(())
}
pub fn count(report:&CoverageReport,definition:&str,action:&str,effect:&str,family:&str,stage:usize)->u64 {
    let matching:Vec<_>=report.rows.iter().filter(|r|{
        let k=&r.requirement.key;k.definition_id==definition&&k.action_id==action&&k.effect_id==effect&&k.producer_family==family
    }).collect();
    assert_eq!(matching.len(),1,"exact independent producer key required");matching[0].counts[stage]
}
