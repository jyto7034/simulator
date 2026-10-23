
#[test]
fn u164_independent_reproduction_fifth_round_atomic_failure() {
    let (mut c,p,id)=audit_start(false,Some(1_000_000_000),false);
    let before=assets(&c);
    deploy(&mut c,p);
    for round in 1..=4 {
        until_choice_or_result(&mut c);
        assert_eq!(view(&c).completed_rounds,round);
        act(&mut c,p,false).unwrap();
    }
    let mut reproduced=false;
    for _ in 0..240 {
        let checkpoint=state(&c);
        let Err(error)=c.advance_active_battle_by(250) else {continue;};
        assert_eq!(view(&c).current_round,5);
        assert_eq!(view(&c).completed_rounds,4);
        assert_eq!(state(&c),checkpoint,"failed finalization rolls back the candidate");
        assert_eq!(assets(&c),before,"no partial reward or XP is committed");
        let retry=c.advance_active_battle_by(250);
        assert!(retry.is_err());
        assert_eq!(state(&c),checkpoint);
        println!("AUDIT_REPRODUCTION {{\"round\":5,\"completed_before_error\":4,\"error\":\"{error:?}\",\"authored_total\":5000000000,\"wallet_max\":4294967295,\"retry_failed\":true,\"roots_unchanged\":true}}");
        execute_current_battle_mutation(&mut c,p,PlayerBehavior::RetreatBattle).unwrap();
        assert_eq!(assets(&c),before);
        assert_eq!(c.state.run.as_ref().unwrap().map.node(id).unwrap().state,MapNodeState::Completed);
        reproduced=true;
        break;
    }
    assert!(reproduced,"supported authored oversized cumulative reward reproduces an atomic finalization failure");
}
