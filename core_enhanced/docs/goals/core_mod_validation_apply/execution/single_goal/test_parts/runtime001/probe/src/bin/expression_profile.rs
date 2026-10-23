//! The public normal loader and immutable candidate/individual paths must honor
//! the same min(caller ingress limit, selected runtime profile) parse boundary.
use game_core::{authoring::CancellationToken,
    game::data::{ValidatedGameData,content_pack::{self,ContentSelection,InputLimits},live_ron::LiveRonError},
    mod_validation::{Candidate,ModValidationError,ModValidationErrorKind,TrustedBase},
    runtime_safety::{RuntimeSafetySettings,EMBEDDED_RUNTIME_SAFETY_RON}};
use serde::Serialize;
use serde_json::{json,Value};
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path,sync::Arc};

fn save(path:&Path,value:&impl Serialize)->Result<(),Box<dyn Error>> {
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
fn copy_tree(source:&Path,target:&Path)->Result<(),Box<dyn Error>> {
    fs::create_dir(target)?;
    for entry in fs::read_dir(source)? {let entry=entry?;let kind=entry.file_type()?;
        if kind.is_dir(){copy_tree(&entry.path(),&target.join(entry.file_name()))?;}
        else if kind.is_file(){fs::copy(entry.path(),target.join(entry.file_name()))?;}
        else{return Err("non-regular fixture member".into());}}
    Ok(())
}
fn normal_observed(result:&Result<ValidatedGameData,LiveRonError>)->Value {
    match result {Ok(g)=>json!({"admitted":true,"content":g.identity(),"source":g.source_receipt(),
        "effective_runtime_safety_sha256":g.runtime_safety().effective_sha256()}),
        Err(e)=>json!({"admitted":false,"error":e})}
}
fn candidate_observed(result:&Result<Candidate,ModValidationError>)->Value {
    match result {Ok(c)=>json!({"admitted":true,"candidate":c.view()}),Err(e)=>json!({"admitted":false,"error":e})}
}
fn expression(node:&str,padding:usize)->String {
    format!("DamageExpr(/*{}*/ amount: (moment: Snapshot, root: {node}), damage_type: Magic)","x".repeat(padding))
}
fn nested(depth:usize)->String {
    assert!(depth>=1);
    let mut node="Const((value: 1, unit: Magnitude))".to_owned();
    // Each binary Add adds exactly one level; neutral zero preserves value1.
    for _ in 1..depth {node=format!("Add([{node},Const((value: 0, unit: Magnitude))])");}node
}
fn wide(nodes:usize)->String {
    assert!(nodes>=2);
    format!("Add([{}])",vec!["Const((value: 1, unit: Magnitude))";nodes-1].join(","))
}
fn aggregate(count:usize)->String {
    let members=(0..count).map(|i|format!("(identity:\"m{i}\",value:(value:{i},unit:Magnitude),owner:None,projected_tile:None)"))
        .collect::<Vec<_>>().join(",");
    format!("Aggregate(source:AuthoredMembers([{members}]),grouping:WholeSet,reduction:Sum,outer_reduction:None,member:Read(AggregateMember(unit:Magnitude)))")
}
fn selected_settings(field:&str,old:usize,new:usize)->Arc<RuntimeSafetySettings> {
    let raw=std::str::from_utf8(EMBEDDED_RUNTIME_SAFETY_RON).unwrap();
    // First max_depth/max_nodes occurrences are in expression.parse; compilation
    // keeps its ordinary limits so a compile ceiling cannot mask this check.
    let changed=raw.replacen(&format!("{field}: {old}"),&format!("{field}: {new}"),1);
    assert_ne!(changed,raw,"independent profile edit must select a real field");
    Arc::new(RuntimeSafetySettings::from_ron("u11 independent expression parse profile",changed.as_bytes()).unwrap())
}

fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=2{return Err("usage: expression_profile FROZEN_FIXTURE_ROOT NEW_OUTPUT_DIRECTORY".into());}
    let input=Path::new(&args[0]);let out=Path::new(&args[1]);fs::create_dir(out)?;
    let root=out.join("scratch");copy_tree(input,&root)?;
    let registry=fs::read(root.join("registries/a.json"))?;
    let path=root.join("a_skills/catalog.ron");let original=fs::read_to_string(&path)?;
    let original_op="Damage(amount: 13, damage_type: Magic)";
    assert_eq!(original.matches(original_op).count(),1);
    let selection=ContentSelection{packs:vec!["official".into(),"a_skills".into(),"a_item".into()],winners:vec![]};
    let base_selection=ContentSelection{packs:vec!["official".into()],winners:vec![]};
    let small=expression("Const((value: 1, unit: Magnitude))",0);let bytes_limit=small.len()+32;
    let cases=[
        ("max_depth",64,4,expression(&nested(4),0),expression(&nested(5),0),"value_expr_depth_limit"),
        ("max_nodes",1024,4,expression(&wide(4),0),expression(&wide(5),0),"value_expr_node_limit"),
        ("max_authored_members",4096,3,expression(&aggregate(3),0),expression(&aggregate(4),0),"value_expr_member_limit"),
        ("max_input_bytes",65536,bytes_limit,expression("Const((value: 1, unit: Magnitude))",32),
            expression("Const((value: 1, unit: Magnitude))",33),"value_expr_input_limit"),
    ];
    for (field,default_limit,limit,exact,over,code) in cases {
        for profile_is_stricter in [true,false] {
            let label=format!("{field}_{}",if profile_is_stricter{"profile"}else{"caller"});
            let directory=out.join(&label);fs::create_dir(&directory)?;
            let settings=if profile_is_stricter{selected_settings(field,default_limit,limit)}else{RuntimeSafetySettings::embedded_shared()};
            let mut limits=InputLimits::default();
            if !profile_is_stricter {match field {
                "max_depth"=>limits.value_expr.max_depth=limit,"max_nodes"=>limits.value_expr.max_nodes=limit,
                "max_authored_members"=>limits.value_expr.max_authored_members=limit,
                "max_input_bytes"=>limits.value_expr.max_input_bytes=limit,_=>unreachable!(),
            }}
            let trusted=TrustedBase::acquire(&root,base_selection.clone(),&limits,settings.clone(),&CancellationToken::default())?;
            fs::write(&path,original.replacen(original_op,&exact,1))?;
            save(&directory.join("inputs.json"),&json!({"dimension":field,"limit":limit,
                "profile_is_stricter":profile_is_stricter,"exact_expression":exact,"one_over_expression":over,
                "selected_profile_sha256":settings.effective_sha256()}))?;
            let normal=content_pack::load_with_runtime_safety(&root,&selection,&limits,settings.clone());
            let candidate=Candidate::acquire(&root,&registry,&trusted,&limits,&CancellationToken::default());
            save(&directory.join("exact.json"),&json!({"normal":normal_observed(&normal),"candidate":candidate_observed(&candidate)}))?;
            let normal=normal?;let candidate=candidate?;
            assert_eq!(candidate.generation().identity(),normal.identity());
            assert_eq!(candidate.generation().source_receipt(),normal.source_receipt());
            assert_eq!(candidate.generation().runtime_safety().effective_sha256(),settings.effective_sha256());
            let captured=serde_json::to_value(candidate.view())?;
            let individual=candidate.individual("a",&trusted,&CancellationToken::default())?;
            save(&directory.join("exact_individual.json"),individual.view())?;
            assert_eq!(individual.required_configuration(),candidate.required_configuration());
            assert_eq!(individual.generation().identity(),normal.identity());
            fs::write(&path,original.replacen(original_op,&over,1))?;
            let normal_over=content_pack::load_with_runtime_safety(&root,&selection,&limits,settings);
            let candidate_over=Candidate::acquire(&root,&registry,&trusted,&limits,&CancellationToken::default());
            save(&directory.join("one_over.json"),&json!({"normal":normal_observed(&normal_over),"candidate":candidate_observed(&candidate_over)}))?;
            let normal_error=normal_over.err().expect("normal path rejects one-over authored expression");
            let candidate_error=candidate_over.err().expect("frozen candidate must enforce the same minimum parse ceiling");
            assert_eq!(normal_error.code,code,"{label}");
            assert_eq!(candidate_error.kind,ModValidationErrorKind::ContentFailed,"{label}");
            assert!(candidate_error.reason.contains(&normal_error.cause),"{label}: same parse failure must survive candidate diagnostics");
            // Individual validation consumes captured exact input and its bound
            // profile; a later one-over file must neither replace nor poison it.
            let held=candidate.individual("a",&trusted,&CancellationToken::default())?;
            save(&directory.join("captured_individual_after_disk_one_over.json"),held.view())?;
            assert_eq!(held.required_configuration(),individual.required_configuration());
            assert_eq!(held.generation().source_receipt(),individual.generation().source_receipt());
            assert_eq!(serde_json::to_value(candidate.view())?,captured);
            fs::write(&path,&original)?;
        }
    }
    save(&out.join("RESULT.json"),&json!({"passed":true,"scope":"public expression parser/profile parity",
        "dimensions":4,"minimum_directions":2,"approval_claimed":false,"runtime_coverage_claimed":false}))?;
    println!("{}",json!({"passed":true,"approval_claimed":false,"output":out}));Ok(())
}
