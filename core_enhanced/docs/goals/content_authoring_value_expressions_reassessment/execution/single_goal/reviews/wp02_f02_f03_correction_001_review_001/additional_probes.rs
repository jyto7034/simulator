fn actual_cast_sources(same_actor: bool) -> (BattleCore, CompiledAmountEntry, RuntimeFactSource, RuntimeFactSource) {
    let (mut core, actor, other, entry)=review_catalog("Read(Stat(subject: Source, stat: Attack))",vec![]);
    core.invoke_ability(10,actor,"wp02_runtime_frame",None,Default::default(),false);
    core.invoke_ability(11,if same_actor {actor} else {other},"wp02_runtime_frame",None,Default::default(),false);
    assert_eq!(core.active_skill_casts.len(),2,"actual cast producer did not start both casts");
    let mut sources:Vec<_>=core.active_skill_casts.values().map(|c|c.fact_source.clone()).collect();
    sources.sort_by_key(|s|s.cast.as_ref().unwrap().cast_seq);
    let mut second=sources.pop().unwrap();let mut first=sources.pop().unwrap();
    first.authored=Some(Arc::new(entry.plan().source().clone()));
    second.authored=first.authored.clone();
    assert_ne!(first.action,second.action);
    assert_ne!(first.cast,second.cast);
    (core,entry,first,second)
}

#[test]
fn reviewer_actual_cast_producer_positive_control() {
    for same in [false,true] {
        let (core,entry,first,second)=actual_cast_sources(same);
        for source in [first,second] {
            let capture=core.capture_value_expr_entry(&entry,&source,None,12,None).unwrap();
            let result=core.evaluate_value_expr_capture(&capture,None,13).unwrap();
            assert_eq!(result.amount,RuntimeAmount::Damage(if source.cast.as_ref().unwrap().cast_seq==0 || same {10} else {3}));
        }
    }
}

#[test]
fn reviewer_late_cast_from_other_actor_must_not_bind_to_first_action() {
    let (core,entry,mut first,second)=actual_cast_sources(false);
    assert_ne!(first.actor,second.actor);
    first.cast=second.cast;
    let before=core.transaction_debug_snapshot();
    let result=core.capture_value_expr_entry(&entry,&first,None,12,None);
    eprintln!("OTHER_ACTOR_CAST accepted={} source_actor={:?} cast_actor={:?}",result.is_ok(),first.actor,first.cast.as_ref().unwrap().caster_instance_id);
    assert_eq!(core.transaction_debug_snapshot(),before);
    assert!(result.is_err(),"a real cast of another actor was accepted under the first action");
}

#[test]
fn reviewer_late_cast_from_other_action_same_actor_must_not_bind() {
    let (core,entry,mut first,second)=actual_cast_sources(true);
    assert_eq!(first.actor,second.actor);
    let original_seq=first.cast.as_ref().unwrap().cast_seq;
    first.cast=second.cast;
    let before=core.transaction_debug_snapshot();
    let result=core.capture_value_expr_entry(&entry,&first,None,12,None);
    eprintln!("SAME_ACTOR_OTHER_CAST accepted={} original_seq={original_seq} substituted_seq={}",result.is_ok(),first.cast.as_ref().unwrap().cast_seq);
    assert_eq!(core.transaction_debug_snapshot(),before);
    assert!(result.is_err(),"another real cast occurrence was accepted under the first action");
}
