// Included with the unchanged reviewer probes in an isolated test module.
#[test]
fn reviewer_f02_r01_action_swap_stale_attempt_and_retry() {
    let (core, entry, first, second) = actual_cast_sources(true);
    let before = core.transaction_debug_snapshot();

    let mut wrong_action = first.clone();
    wrong_action.action = second.action.clone();
    assert_eq!(
        core.capture_value_expr_entry(&entry, &wrong_action, None, 12, None)
            .unwrap_err()
            .code,
        ValueExprFaultCode::Unavailable
    );

    let mut copied_cast = first.clone();
    copied_cast.cast = Some(Arc::new(first.cast.as_ref().unwrap().as_ref().clone()));
    assert_eq!(
        core.capture_value_expr_entry(&entry, &copied_cast, None, 12, None)
            .unwrap_err()
            .code,
        ValueExprFaultCode::Unavailable
    );

    let mut other_attempt = core.clone();
    other_attempt.runtime_facts = Default::default();
    assert_eq!(
        other_attempt.runtime_facts.validate_live_action(first.action.as_ref().unwrap()),
        Err(crate::game::battle::runtime_facts::ActionQueryError::DifferentAttempt)
    );
    assert_eq!(
        other_attempt.capture_value_expr_entry(&entry, &first, None, 12, None)
            .unwrap_err()
            .code,
        ValueExprFaultCode::DifferentAttempt
    );
    assert_eq!(core.transaction_debug_snapshot(), before);
    assert_eq!(
        core.evaluate_value_expr_capture(
            &core.capture_value_expr_entry(&entry, &first, None, 12, None).unwrap(),
            None,
            13,
        )
        .unwrap()
        .amount,
        RuntimeAmount::Damage(10)
    );
}

#[test]
fn reviewer_f02_r01_released_delivery_and_departed_source_snapshot() {
    let (mut core, entry, mut source, _) = actual_cast_sources(true);
    let action = source.action.clone().unwrap();
    let cast_seq = source.cast.as_ref().unwrap().cast_seq;
    source.delivery = core.runtime_facts.delivery(&action).unwrap();
    core.runtime_facts.dispatched(&action, 0, 12).unwrap();
    assert!(core.runtime_facts.query(&action).unwrap().terminal.is_some());
    assert_eq!(core.runtime_facts.query(&action).unwrap().outstanding_deliveries, 1);
    // Model the cast controller's cleanup; the action and delivery still own
    // the relationship needed by capture.
    core.active_skill_casts.remove(&cast_seq);
    assert_eq!(
        core.evaluate_value_expr_capture(
            &core.capture_value_expr_entry(&entry, &source, None, 13, None).unwrap(),
            None,
            14,
        )
        .unwrap()
        .amount,
        RuntimeAmount::Damage(10)
    );

    let snapshot = plan(
        EvalMoment::Snapshot,
        "released_cast_snapshot",
        ValueExpr::Read(ValueRef::Stat {
            subject: ValueSubject::Source,
            stat: StatSelector::Attack,
        }),
    );
    source.authored = Some(Arc::new(snapshot.source().clone()));
    let capture = core.capture_value_expr_plan(AmountPlanKind::Damage, snapshot, &source, None, 15, None).unwrap();
    core.withdraw_unit(source.actor.unwrap(), 16).unwrap();
    assert_eq!(
        core.evaluate_value_expr_capture(&capture, None, 17).unwrap().amount,
        RuntimeAmount::Damage(10)
    );
}

#[test]
fn reviewer_f02_r01_castless_basic_source_remains_valid() {
    let (mut core, actor, _, entry) = review_catalog("Read(Stat(subject: Source, stat: Attack))", vec![]);
    let source = occurrence(&mut core, actor, entry.plan());
    assert!(source.cast.is_none());
    assert_eq!(
        core.evaluate_value_expr_capture(
            &core.capture_value_expr_entry(&entry, &source, None, 12, None).unwrap(),
            None,
            13,
        )
        .unwrap()
        .amount,
        RuntimeAmount::Damage(10)
    );
}
