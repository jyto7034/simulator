include!("review_probes.rs");

fn authored(member_value: i64, member: &str) -> String {
    format!(r#"Aggregate(source: AuthoredMembers([(identity: "a", value: (value: {member_value}, unit: Magnitude), owner: Some((domain: Side, identity: "left")), projected_tile: Some((x: 0, y: 0)))]), grouping: WholeSet, reduction: Sum, outer_reduction: None, member: {member})"#)
}

#[test]
fn external_static_dynamic_boundary_matrix() {
    use game_core::game::data::content_pack::{self, ContentSelection, InputLimits};
    let root=std::path::Path::new("/tmp/r06-wp01-correction-review.NBF56n/extra");
    std::fs::create_dir_all(root).unwrap();
    let pack=root.join("official");
    if !pack.exists() {
        content_pack::package_official(std::path::Path::new("/mnt/f/work/simulator/game_resources"),&pack,&InputLimits::default()).unwrap();
    }
    let path=pack.join("data/skills/base.ron");
    let original=std::fs::read_to_string(&path).unwrap();
    let selector=ContentSelection {packs:vec!["official".into()],winners:vec![]};
    let local="Read(AggregateMember(unit: Magnitude))";
    let cases=[
        ("grouped_negative", authored(-1,local).replace("grouping: WholeSet","grouping: ByOwner").replace("outer_reduction: None","outer_reduction: Some(Sum)"), "value_expr_overflow"),
        ("nested_negative", authored(2,&authored(-1,local)), "value_expr_overflow"),
        ("empty_max", format!("Aggregate(source: AuthoredMembers([]), grouping: WholeSet, reduction: Max, outer_reduction: None, member: {local})"), "value_expr_unavailable"),
        ("empty_sum", format!("Aggregate(source: AuthoredMembers([]), grouping: WholeSet, reduction: Sum, outer_reduction: None, member: {local})"), "content_capability_unsupported"),
        ("dynamic_ratio", "Mul(value: Const((value: 7, unit: Magnitude)), scalar: Ratio(numerator: Const((value: 1, unit: Magnitude)), denominator: Read(Stat(subject: Source, stat: Attack)), rounding: TowardZero))".to_string(), "content_capability_unsupported"),
        ("mixed_local_external", authored(2,&format!("Add([{local}, Read(Stat(subject: Source, stat: Attack))])")), "content_capability_unsupported"),
        ("nested_resolved", authored(2,&format!("Aggregate(source: ResolvedTargets, grouping: WholeSet, reduction: Sum, outer_reduction: None, member: {local})")), "content_capability_unsupported"),
    ];
    let mut errors=Vec::new();
    for (name,expr,expected) in cases {
        let op=format!("DamageExpr(amount: (moment: Snapshot, root: {expr}), damage_type: Magic)");
        std::fs::write(&path,original.replacen("Damage(amount: 60, damage_type: Magic)",&op,1)).unwrap();
        let error=content_pack::load(root,&selector,&InputLimits::default()).err().expect("authored expression must never publish");
        println!("{name}: expected={expected} actual={} cause={}",error.code,error.cause);
        assert_eq!(error.object_id.as_deref(),Some("unknown_distortion_strike"));
        assert_eq!(error.action_id.as_deref(),Some("strike"));
        assert_eq!(error.effect_id.as_deref(),Some("effect_01"));
        if error.code!=expected {errors.push((name,expected,error.code));}
    }
    std::fs::write(&path,&original).unwrap();
    content_pack::load(root,&selector,&InputLimits::default()).unwrap();
    assert!(errors.is_empty(),"{errors:?}");
}

#[test]
fn strict_unknown_field_position_matrix() {
    let roots=[
        "Mul(typo: 0, value: Const((value: 1, unit: Magnitude)), scalar: Const((value: 2, unit: Scalar)))",
        "Mul(value: Const((value: 1, unit: Magnitude)), typo: 0, scalar: Const((value: 2, unit: Scalar)))",
        "Read(Stat(typo: 0, subject: Source, stat: Attack))",
        "Read(Stat(subject: Source, typo: 0, stat: Attack))",
        "Read(HitPoints(subject: Source, field: Current, typo: (nested: [1,2,3])))",
        "Aggregate(source: AuthoredMembers([(identity: \"a\", value: (value: 1, unit: Magnitude), owner: Some((domain: Side, identity: \"a\", typo: 0)), projected_tile: None)]), grouping: WholeSet, reduction: Sum, outer_reduction: None, member: Read(AggregateMember(unit: Magnitude)))",
    ];
    for root in roots {
        let err=SkillDatabase::from_ron("matrix.ron",&skill(root)).unwrap_err();
        assert_eq!(err.code,"ron_parse","{root}: {err:?}");
        assert!(err.cause.contains("unknown field"),"{root}: {err:?}");
        assert_eq!(err.object_id.as_deref(),Some("review_expr"));
        assert_eq!(err.effect_id.as_deref(),Some("amount"));
    }
}
