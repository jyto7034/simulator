//! Actual external immutable candidates; these observations do not claim Lab PASS.
use game_core::{authoring::CancellationToken,
    game::{ability::SkillEffectDef, data::content_pack::{ContentSelection, InputLimits}},
    mod_validation::{Candidate, ModValidationError, ModValidationErrorKind, TrustedBase},
    runtime_safety::RuntimeSafetySettings};
use serde::Serialize;
use serde_json::{json, Value};
use std::{error::Error, fs::{self, OpenOptions}, io::Write, path::Path,sync::Arc};

fn save(path: &Path, value: &impl Serialize) -> Result<(), Box<dyn Error>> {
    let mut bytes = serde_json::to_vec_pretty(value)?; bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;
    Ok(())
}
fn registry(root: &Path, name: &str) -> Vec<u8> {
    fs::read(root.join(format!("registries/{name}.json"))).unwrap()
}
fn acquire(root: &Path, name: &str, base: &TrustedBase) -> Result<Candidate, ModValidationError> {
    Candidate::acquire(root, &registry(root, name), base, &InputLimits::default(), &CancellationToken::default())
}
fn damage(candidate: &Candidate, id: &str) -> i32 {
    let skill = candidate.generation().database().skill_data.get_by_id(id).expect("authored skill");
    match &skill.steps[0].effects[0].op {
        SkillEffectDef::Damage { amount, .. } => *amount,
        _ => panic!("independent fixture's first authored effect is Damage"),
    }
}
fn observed(result: &Result<Candidate, ModValidationError>) -> Value {
    match result { Ok(candidate) => json!({"admitted":true,"candidate":candidate.view()}),
        Err(error) => json!({"admitted":false,"error":error}) }
}
fn copy_tree(source: &Path, target: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?; let kind = entry.file_type()?;
        if kind.is_dir() { copy_tree(&entry.path(), &target.join(entry.file_name()))?; }
        else if kind.is_file() { fs::copy(entry.path(), target.join(entry.file_name()))?; }
        else { return Err("fixture contains non-regular entry".into()); }
    }
    Ok(())
}
fn trusted(root: &Path,settings:Arc<RuntimeSafetySettings>) -> TrustedBase {
    TrustedBase::acquire(root, ContentSelection { packs:vec!["official".into()],winners:vec![] },
        &InputLimits::default(), settings.clone(), &CancellationToken::default()).unwrap()
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len()!=2&&args.len()!=3 { return Err("usage: candidate FROZEN_FIXTURE_ROOT NEW_OUTPUT_DIRECTORY [RUNTIME_SAFETY_RON]".into()); }
    let input = Path::new(&args[0]); let out = Path::new(&args[1]); fs::create_dir(out)?;
    let (settings,_profile_bytes)=u11_independent_public_probe::normal_preparation::runtime_profile(
        args.get(2).map(|p|Path::new(p)),&out.join("runtime_context.json"))?;
    let root = out.join("scratch"); copy_tree(input, &root)?;
    let base = trusted(&root,settings.clone());
    let a = acquire(&root, "a", &base)?;
    let b = acquire(&root, "b", &base)?;
    let ab = acquire(&root, "ab", &base)?;
    save(&out.join("normal_candidates.json"), &json!({"a":a.view(),"b":b.view(),"ab":ab.view()}))?;
    assert_eq!(damage(&a, "u11_damage_a"),13);
    assert_eq!(damage(&b, "u11_damage_b"),17);
    assert_eq!(damage(&ab, "u11_damage_a"),13);
    assert_eq!(damage(&ab, "u11_damage_b"),17);
    let keys = |c: &Candidate| c.view().changed_definitions.iter()
        .map(|k| format!("{:?}:{}",k.kind,k.id)).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(keys(&a), ["Equipment:u11_weapon_a", "Skill:u11_damage_a"].map(str::to_owned).into_iter().collect());
    assert_eq!(keys(&b), ["Equipment:u11_armor_b", "Skill:u11_damage_b"].map(str::to_owned).into_iter().collect());
    assert!(a.view().removed_definitions.is_empty() && b.view().removed_definitions.is_empty());
    let a_before = serde_json::to_value(a.view())?; let b_before = serde_json::to_value(b.view())?;
    let individual_a = ab.individual("a", &base, &CancellationToken::default())?;
    let individual_b = ab.individual("b", &base, &CancellationToken::default())?;
    save(&out.join("individuals.json"), &json!({"a":individual_a.view(),"b":individual_b.view()}))?;
    assert_eq!(individual_a.required_configuration(), a.required_configuration());
    assert_eq!(individual_b.required_configuration(), b.required_configuration());
    assert_eq!(damage(&individual_a,"u11_damage_a"),13);
    assert_eq!(damage(&individual_b,"u11_damage_b"),17);

    // These only assert publication and independent changed-definition sets.
    // Runtime family/site coverage must be established later by the actual Lab.
    for (name, expected) in [
        ("structural",vec!["Equipment:u11_structural_weapon","Relic:u11_structural_relic","Skill:u11_structural_strike"]),
        ("causal",vec!["Equipment:u11_causal_weapon","Relic:u11_causal_relic","Skill:u11_causal_strike"]),
        ("objects",vec!["Equipment:u11_object_weapon","IndependentObject:u11_finite_actor","Skill:u11_object_create"]),
        ("noop",vec!["Equipment:u11_heal_weapon","Equipment:u11_zero_armor","Skill:u11_full_health_heal","Skill:u11_zero_damage"]),
    ] {
        let family=acquire(&root,name,&base)?;
        save(&out.join(format!("{name}_candidate.json")),family.view())?;
        assert_eq!(keys(&family),expected.into_iter().map(str::to_owned).collect(),"{name}: exact independent inventory");
        assert_eq!(serde_json::to_value(a.view())?,a_before);
    }
    let host_a=acquire(&root,"host_a",&base)?;
    let host_b=acquire(&root,"host_b",&base)?;
    let host_ab=acquire(&root,"host_ab",&base)?;
    save(&out.join("shared_loadout_host_candidates.json"),&json!({"a":host_a.view(),"b":host_b.view(),"ab":host_ab.view()}))?;
    assert_eq!(keys(&host_a),["Equipment:standard_training_sword","Skill:u11_damage_a"].map(str::to_owned).into_iter().collect());
    assert_eq!(keys(&host_b),["Equipment:standard_armor","Skill:u11_damage_b"].map(str::to_owned).into_iter().collect());
    assert_eq!(host_ab.individual("host_a",&base,&CancellationToken::default())?.required_configuration(),host_a.required_configuration());
    assert_eq!(host_ab.individual("host_b",&base,&CancellationToken::default())?.required_configuration(),host_b.required_configuration());
    assert!(host_b.generation().database().skill_data.get_by_id("u11_damage_a").is_none());
    assert_eq!(damage(&host_a,"u11_damage_a"),13);
    assert_eq!(damage(&host_b,"u11_damage_b"),17);
    for (tier,count) in [("small",64),("expanded",256),("pathological",1024)] {
        for suffix in ["a","b","ab"] {
            let name=format!("host_load_{tier}_{suffix}");let workload=acquire(&root,&name,&base)?;
            save(&out.join(format!("{name}_candidate.json")),workload.view())?;
            for side in if suffix=="ab" {vec!["a","b"]}else{vec![suffix]} {
                let skill_id=format!("u11_load_{tier}_{side}");
                let skill=workload.generation().database().skill_data.get_by_id(&skill_id).expect("authored workload skill");
                assert_eq!(skill.steps.len(),1);assert_eq!(skill.steps[0].effects.len(),count);
                assert_eq!(damage(&workload,&skill_id),1,"first actual effect is the positive control");
                for (index,effect) in skill.steps[0].effects.iter().enumerate().skip(1) {
                    assert_eq!(effect.id,format!("effect_{index}"));
                    assert!(matches!(&effect.op,SkillEffectDef::Damage{amount:0,..}),"remaining explicit sites are legal zero damage");
                }
            }
        }
    }

    for name in ["broken_member","bad_parse_independent","writer_ambiguous","writer_stale"] {
        let result = acquire(&root,name,&base);
        save(&out.join(format!("{name}.json")), &observed(&result))?;
        let error = result.err().expect("invalid selected candidate must fail");
        assert!(!error.reason.is_empty() && !error.field_path.is_empty());
        assert_eq!(serde_json::to_value(a.view())?,a_before);
        assert_eq!(serde_json::to_value(b.view())?,b_before);
        let mut only_b: Value = serde_json::from_slice(&registry(&root,name))?;
        only_b["selection"] = json!({"mods":["base","b"],"winners":[]});
        let independent = Candidate::acquire(&root,&serde_json::to_vec(&only_b)?,&base,
            &InputLimits::default(),&CancellationToken::default())?;
        assert_eq!(damage(&independent,"u11_damage_b"),17,"{name}: independent B remains usable");
        assert_eq!(independent.required_configuration(),b.required_configuration());
    }
    let winner = acquire(&root,"writer_right",&base)?;
    save(&out.join("writer_right.json"), winner.view())?;
    assert_eq!(damage(&winner,"u11_damage_a"),31);
    let duplicate_writer = acquire(&root,"writer_duplicate_target",&base);
    save(&out.join("writer_duplicate_target.json"),&observed(&duplicate_writer))?;
    assert_eq!(duplicate_writer.err().expect("contradictory explicit winners must fail").kind,
        ModValidationErrorKind::InvalidInput);
    assert_eq!(damage(&winner,"u11_damage_a"),31);
    let base_a=TrustedBase::acquire(&root,
        ContentSelection{packs:vec!["official".into(),"a_skills".into(),"a_item".into()],winners:vec![]},
        &InputLimits::default(),settings.clone(),&CancellationToken::default())?;
    let removed=acquire(&root,"remove_a",&base_a)?;
    save(&out.join("removed_a.json"),removed.view())?;
    assert!(removed.generation().database().skill_data.get_by_id("u11_damage_a").is_none());
    assert!(!removed.generation().database().equipment_data.items.iter().any(|item|item.id=="u11_weapon_a"));
    let removed_keys:std::collections::BTreeSet<_>=removed.view().removed_definitions.iter()
        .map(|key|format!("{:?}:{}",key.kind,key.id)).collect();
    assert_eq!(removed_keys,["Skill:u11_damage_a","Equipment:u11_weapon_a"].map(str::to_owned).into_iter().collect());
    assert_eq!(damage(&a,"u11_damage_a"),13);

    // Exact source changes are meaningful misses even if resolved definitions
    // are unchanged. This mutation is confined to this probe's fresh scratch.
    let path = root.join("a_skills/catalog.ron"); let original = fs::read_to_string(&path)?;
    fs::write(&path,format!("{original}\n// U11 exact byte change only\n"))?;
    let comment = acquire(&root,"a",&base)?;
    save(&out.join("comment_only.json"), comment.view())?;
    assert_eq!(comment.generation().identity(),a.generation().identity());
    assert_ne!(comment.view().identity,a.view().identity);
    assert_ne!(serde_json::to_value(&comment.view().source_receipt)?,serde_json::to_value(&a.view().source_receipt)?);
    let changed = original.replace("amount: 13","amount: 29"); assert_ne!(changed,original);
    fs::write(&path,&changed)?;
    let replaced = acquire(&root,"a",&base)?;
    save(&out.join("actual_changed_amount.json"), replaced.view())?;
    assert_eq!(damage(&replaced,"u11_damage_a"),29);
    assert_eq!(damage(&a,"u11_damage_a"),13);
    assert_eq!(serde_json::to_value(a.view())?,a_before);
    fs::remove_file(&path)?;
    let old_closure = ab.individual("a",&base,&CancellationToken::default())?;
    save(&out.join("captured_individual_after_file_removal.json"),old_closure.view())?;
    assert_eq!(damage(&old_closure,"u11_damage_a"),13,"captured original definitions are immutable");
    let captured_subset = ab.selected_from_capture(vec!["base".into(),"a".into()],&CancellationToken::default())?;
    save(&out.join("selected_from_capture_after_file_removal.json"),captured_subset.view())?;
    assert_eq!(damage(&captured_subset,"u11_damage_a"),13);
    assert_eq!(captured_subset.required_configuration(),old_closure.required_configuration());
    assert_eq!(serde_json::to_value(captured_subset.view())?,serde_json::to_value(old_closure.view())?);
    fs::write(&path,original)?;

    let cancel = CancellationToken::default(); cancel.cancel();
    let cancelled = Candidate::acquire(&root,&registry(&root,"a"),&base,&InputLimits::default(),&cancel);
    save(&out.join("cancelled.json"), &observed(&cancelled))?;
    assert_eq!(cancelled.err().expect("cancelled candidate cannot publish").kind,ModValidationErrorKind::Cancelled);

    // Holding a trusted base must not authorize a replacement of its disk bytes.
    let official = root.join("official/data/skills/base.ron");
    let raw = fs::read_to_string(&official)?;
    fs::write(&official,format!("{raw}\n// replacement after TrustedBase acquisition\n"))?;
    let replaced_base = acquire(&root,"a",&base);
    save(&out.join("replaced_trusted_base.json"),&observed(&replaced_base))?;
    let mismatch=replaced_base.err().expect("old exact trusted base cannot bless newly acquired base bytes");
    assert_eq!(mismatch.kind,ModValidationErrorKind::ContentFailed);
    assert_eq!(mismatch.field_path,"candidate");
    assert_eq!(mismatch.reason,"mod_candidate [trusted_base]: trusted base bytes changed after host suite capture");
    assert_eq!(damage(&a,"u11_damage_a"),13);
    save(&out.join("RESULT.json"),&json!({"passed":true,"scope":"static immutable candidate boundaries only",
        "approval_claimed":false,"ab_variants":3,"family_variants":4,"host_variants":3,"load_variants":9,
        "selected_failures":4,"independent_recovery":4,
        "checks":["changed_definition_inventory","multipack","independent_closure","writer",
            "duplicate_writer_rejected","removed_definition_inventory","family_catalogs",
            "exact_source_change","held_token","captured_individual","cancellation","trusted_base_replacement"]}))?;
    println!("{}",json!({"passed":true,"approval_claimed":false,"output":out}));
    Ok(())
}
