#[test]
fn reviewer_bound_cast_cannot_be_omitted_and_survives_candidate_cow() {
    let (core,entry,first,_) = actual_cast_sources(true);
    let mut missing=first.clone(); missing.cast=None;
    assert!(core.capture_value_expr_entry(&entry,&missing,None,12,None).is_err());
    let before=core.transaction_debug_snapshot();
    let mut candidate=core.clone();
    // Existing actions are shared; admission of another action detaches them.
    candidate.runtime_facts.accept(first.actor.unwrap(),None,RuntimeFactSource{
        actor:first.actor,..Default::default()
    },12,false).unwrap();
    for battle in [&core,&candidate] {
        let capture=battle.capture_value_expr_entry(&entry,&first,None,13,None).unwrap();
        assert_eq!(battle.evaluate_value_expr_capture(&capture,None,14).unwrap().amount,RuntimeAmount::Damage(10));
        assert!(battle.capture_value_expr_entry(&entry,&missing,None,13,None).is_err());
    }
    assert_eq!(core.transaction_debug_snapshot(),before);
}

#[test]
fn reviewer_binding_rejects_wrong_actor_skill_and_rebinding() {
    let (mut core,entry,first,other) = actual_cast_sources(false);
    let cast=first.cast.as_ref().unwrap().clone();
    let basis=RuntimeFactSource{actor:first.actor,authored:first.authored.clone(),..Default::default()};
    let action=core.runtime_facts.accept(first.actor.unwrap(),Some(&cast.skill_id),basis.clone(),12,false).unwrap();
    let mut candidate=core.clone();
    assert!(candidate.runtime_facts.bind_cast(&action,other.cast.as_ref().unwrap()).is_err());
    let mut wrong_skill=cast.as_ref().clone();wrong_skill.skill_id=SkillId::new("different_skill");
    assert!(candidate.runtime_facts.bind_cast(&action,&Arc::new(wrong_skill)).is_err());
    candidate.runtime_facts.bind_cast(&action,&cast).unwrap();
    assert!(candidate.runtime_facts.bind_cast(&action,&cast).is_err());
    let source=RuntimeFactSource{action:Some(action),cast:Some(cast),..basis};
    assert!(candidate.capture_value_expr_entry(&entry,&source,None,13,None).is_ok());
    assert!(core.capture_value_expr_entry(&entry,&source,None,13,None).is_err(),"candidate binding leaked into original owner");
}

#[test]
fn reviewer_binding_retained_refusal_keeps_owner_unbound_and_allows_retry() {
    use crate::game::battle::runtime_control as native;
    let (mut core,entry,first,_) = actual_cast_sources(true);
    let cast=first.cast.as_ref().unwrap().clone();
    let basis=RuntimeFactSource{actor:first.actor,authored:first.authored.clone(),..Default::default()};
    let action=core.runtime_facts.accept(first.actor.unwrap(),Some(&cast.skill_id),basis.clone(),12,false).unwrap();
    let baseline=core.retained_runtime_fact_extent().unwrap();
    let before=core.transaction_debug_snapshot();
    let denied=native::run(native::RuntimeFactLimits{max_retained_bytes:baseline+1,..Default::default()},||{
        native::seed_retained_extent(baseline)?;
        core.runtime_facts.bind_cast(&action,&cast)
    });
    assert!(denied.is_err());
    assert_eq!(core.transaction_debug_snapshot(),before);
    let source=RuntimeFactSource{action:Some(action.clone()),cast:Some(cast.clone()),..basis};
    assert!(core.capture_value_expr_entry(&entry,&source,None,13,None).is_err());
    native::run(native::RuntimeFactLimits::default(),||{
        native::seed_retained_extent(baseline)?;
        core.runtime_facts.bind_cast(&action,&cast)
    }).unwrap();
    assert!(core.capture_value_expr_entry(&entry,&source,None,13,None).is_ok());
}
