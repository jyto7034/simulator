
#[test]
fn audit_u03_trusted_goodbye_lookup_and_ordinary_reinput_controls() {
    use crate::application::{CoreApplication, ApplicationCommand, ApplicationExecutionContext, ApplicationBattleAuthority};
    fn input(employee: Uuid, activation: u64, target: serde_json::Value) -> ApplicationCommand {
        ApplicationCommand::from_document(serde_json::json!({"type":"reinput_sustained_skill", "employee_uuid":employee, "skill_id":"r08_strike", "activation":activation, "target":target}).into()).unwrap()
    }
    fn context(core: &GameCore) -> ApplicationExecutionContext {
        let authority = core.active_battle_authority().unwrap();
        ApplicationExecutionContext { expected_battle_authority:Some(ApplicationBattleAuthority {battle_uuid:authority.battle_uuid, battle_state_revision:authority.battle_state_revision}), ..Default::default() }
    }
    let (core, player, employee, activation) = GameCore::u03_expression_fault_fixture();
    let cx = context(&core);
    let mut app = CoreApplication::from_core(core);
    let stale = app.execute(player, input(employee, activation + 1, serde_json::Value::Null), cx.clone()).unwrap_err();
    assert_eq!(stale.code(), "invalid_action"); assert!(app.battle_fault().is_none());
    let target = app.execute(player, input(employee, activation, serde_json::json!({"type":"Unit", "unit_instance_id":Uuid::nil()})), cx).unwrap_err();
    assert_eq!(target.code(), "invalid_action"); assert!(app.battle_fault().is_none());
    app.advance_active_battle_by(1, 0).unwrap();
    println!("ordinary_stale_activation={} ordinary_invalid_target={} subsequent_advance=ok fault_present={}",stale.code(),target.code(),app.battle_fault().is_some());
    let (mut core, player, employee, activation) = GameCore::u03_expression_fault_fixture();
    core.state.active_battle.as_mut().unwrap().battle.audit_u03_missing_goodbye_definition();
    let cx = context(&core);
    let mut app = CoreApplication::from_core(core);
    let first = app.execute(player, input(employee, activation, serde_json::Value::Null), cx.clone()).unwrap_err();
    println!("trusted_lookup_first={} fault_present={}",first.code(),app.battle_fault().is_some());
    let second = app.execute(player, input(employee, activation, serde_json::Value::Null), cx).unwrap_err();
    println!("trusted_lookup_second={} fault_present={}",second.code(),app.battle_fault().is_some());
    assert!(app.battle_fault().is_some(),"missing child of trusted sustained record must latch terminal fault");
    assert_eq!(second.code(), "battle_faulted");
}
