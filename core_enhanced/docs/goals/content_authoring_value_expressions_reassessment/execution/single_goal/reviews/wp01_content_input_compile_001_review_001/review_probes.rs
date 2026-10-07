use game_core::game::data::skill_data::SkillDatabase;
use game_core::game::value_expr::validate_expression_input;

fn skill(root: &str) -> String {
    format!(r#"SkillDatabase(skills: [SkillDef(id: "review_expr", name: "Review", kind: Targeted, focus_time_ms: 0,
      cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: false),
      steps: [SkillStepDef(id: "strike", delay_ms: 0, target: EnemySingle(rule: Nearest), delivery: Instant,
        effects: [(id: "amount", op: DamageExpr(amount: (moment: Snapshot, root: {root}), damage_type: True))])])])"#)
}

fn nested(depth: usize) -> String {
    let mut root = "Const((value: 1, unit: Magnitude))".to_string();
    for _ in 1..depth {
        root = format!("Mul(value: {root}, scalar: Const((value: 1, unit: Scalar)))");
    }
    root
}

#[test]
fn unknown_expression_fields_are_rejected() {
    let input=skill("Mul(value: Const((value: 1, unit: Magnitude)), scalar: Const((value: 2, unit: Scalar)), typo_parameter: 123)");
    let result=SkillDatabase::from_ron("review.ron",&input);
    println!("unknown expression field accepted={}",result.is_ok());
    assert!(result.is_err(),"unknown expression field was silently discarded");
}

#[test]
fn bounded_scanner_and_normal_skill_control() {
    let input=skill(&nested(8));
    validate_expression_input(&input,65536,128).unwrap();
    SkillDatabase::from_ron("review.ron",&input).unwrap();
    let result=SkillDatabase::from_ron("review.ron",&skill("NoSuchNode(1)"));
    assert!(result.is_err());
    let error=result.unwrap_err();
    assert_eq!(error.object_id.as_deref(),Some("review_expr"));
    assert_eq!(error.action_id.as_deref(),Some("strike"));
    assert_eq!(error.effect_id.as_deref(),Some("amount"));
}

#[test]
#[ignore]
fn parser_default_depth_on_worker_stack() {
    std::thread::Builder::new().stack_size(2*1024*1024).spawn(|| {
        let input=skill(&nested(64));
        validate_expression_input(&input,65536,128).unwrap();
        eprintln!("scanner passed, expression depth=64 nodes=127 bytes={} worker_stack=2097152; entering typed parse",input.len());
        let result=SkillDatabase::from_ron("review.ron",&input);
        eprintln!("typed parser returned: {:?}",result.as_ref().map(|_|()).map_err(|e|(&e.code,&e.cause)));
        assert!(result.is_ok());
    }).unwrap().join().unwrap();
}

#[test]
fn external_compile_diagnostics_precede_publication_gate() {
    use game_core::game::data::content_pack::{self,ContentSelection,InputLimits};
    let root=std::path::Path::new("/tmp/r06-wp01-review.J309ox");
    let pack=root.join("official");
    if !pack.exists() {
        content_pack::package_official(std::path::Path::new("/mnt/f/work/simulator/game_resources"),&pack,&InputLimits::default()).unwrap();
    }
    let path=pack.join("data/skills/base.ron");
    let original=std::fs::read_to_string("/mnt/f/work/simulator/game_resources/data/skills/base.ron").unwrap();
    let literal="Damage(amount: 60, damage_type: Magic)";
    assert!(original.contains(literal));
    let selector=ContentSelection {packs:vec!["official".into()],winners:vec![]};
    let cases=[
        ("constant_negative","value_expr_overflow", "DamageExpr(amount: (moment: Snapshot, root: Const((value: -1, unit: Magnitude))), damage_type: Magic)"),
        ("authored_negative_sum","value_expr_overflow", "DamageExpr(amount: (moment: Snapshot, root: Aggregate(source: AuthoredMembers([(identity: \"a\", value: (value: -1, unit: Magnitude), owner: None, projected_tile: None)]), grouping: WholeSet, reduction: Sum, outer_reduction: None, member: Read(AggregateMember(unit: Magnitude)))), damage_type: Magic)"),
        ("unknown_mul_field","ron_parse", "DamageExpr(amount: (moment: Snapshot, root: Mul(value: Const((value: 1, unit: Magnitude)), scalar: Const((value: 2, unit: Scalar)), typo_parameter: 123)), damage_type: Magic)"),
        ("valid_expression_gate","content_capability_unsupported", "DamageExpr(amount: (moment: Snapshot, root: Const((value: 1, unit: Magnitude))), damage_type: Magic)"),
    ];
    let mut failed=Vec::new();
    for(name,expected,op)in cases {
        std::fs::write(&path,original.replacen(literal,op,1)).unwrap();
        let result=content_pack::load(root,&selector,&InputLimits::default());
        let error=result.err().expect("expression must not publish");
        println!("external {name}: expected={expected} actual={} cause={} object={:?} action={:?} effect={:?}",error.code,error.cause,error.object_id,error.action_id,error.effect_id);
        if error.code!=expected {failed.push((name,expected,error.code));}
    }
    // Preserve repository bytes; restore only this isolated diagnostic fixture.
    std::fs::write(&path,&original).unwrap();
    content_pack::load(root,&selector,&InputLimits::default()).unwrap();
    assert!(failed.is_empty(),"external compile diagnostics: {failed:?}");
}
