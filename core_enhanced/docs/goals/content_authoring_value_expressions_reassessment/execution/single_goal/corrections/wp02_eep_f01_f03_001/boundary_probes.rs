
#[test]
fn correction_prefix_buffer_exact_and_one_byte_short() {
    use crate::game::battle::runtime_control as native;
    let (core, _, target, skill, source) = ordered_fixture(r#"
        (id:"d",op:Damage(amount:1,damage_type:True)),
        (id:"h",op:HealExpr(amount:(moment:Live,root:Read(Result(selector:EarlierEffect("d"),field:DamageHpLost))))),
    "#);
    let mut carrier=core.build_value_expr_delivery_capture(&skill.steps[0],&source,&[target],10,None).unwrap().unwrap();
    let ids=(0..128).map(|i|crate::game::resource::ResourceId::new(format!("r{i}")).unwrap()).collect();
    Arc::get_mut(&mut carrier).unwrap().resource_ids=Arc::new(ids);
    let required=carrier.effect_ids.len()*size_of::<Option<EffectResultProjection>>()
        +128*size_of::<i128>()+carrier.action_site.definition_id.as_str().len()+carrier.action_site.action_id.len();
    for cap in [required-1,required] {
        TRACK.with(|s|s.set((true,0,0)));
        let result=native::run(native::RuntimeFactLimits{max_retained_bytes:cap,..Default::default()},||{
            let p=EffectResultPrefix::from_capture(&carrier,&source)?;
            Ok((p,native::baseline_usage().unwrap().2.unwrap()))
        });
        let (_,sum,max)=TRACK.with(|s|{let v=s.get();s.set((false,0,0));v});
        println!("prefix cap={cap} required={required} allocation={sum} max={max}");
        if cap<required {assert!(result.is_err());assert!(max<128*size_of::<i128>());} else {
            let (_,admitted)=result.unwrap();assert_eq!(admitted,required);assert!(sum<=admitted);
        }
    }
}

#[test]
fn correction_carrier_exact_budget_and_one_byte_short() {
    use crate::game::battle::runtime_control as native;
    let (core, _, target, skill, source)=ordered_fixture(r#"(id:"h",op:HealExpr(amount:(moment:Live,root:Const((value:1,unit:Magnitude)))))"#);
    let basis=core.retained_runtime_fact_extent().unwrap();
    let needed=native::run(native::RuntimeFactLimits::default(),||{
        native::seed_retained_extent(basis)?;
        core.build_value_expr_delivery_capture(&skill.steps[0],&source,&[target],10,None)?;
        Ok(native::baseline_usage().unwrap().2.unwrap())
    }).unwrap();
    for cap in [needed-1,needed] {
        let result=native::run(native::RuntimeFactLimits{max_retained_bytes:cap,..Default::default()},||{
            native::seed_retained_extent(basis)?;
            core.build_value_expr_delivery_capture(&skill.steps[0],&source,&[target],10,None)
        });
        println!("carrier cap={cap} needed={needed} success={}",result.is_ok());
        assert_eq!(result.is_ok(),cap==needed);
    }
}
