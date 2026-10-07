
#[test]
fn audit_u03_admitted_manual_cast_time_overflow_must_fault() {
    use crate::application::{CoreApplication, ApplicationCommand, ApplicationExecutionContext, ApplicationBattleAuthority};
    let mut observed = Vec::new();
    for focus in [0u32, 100u32] {
        let authored = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r11/hello_world_result.ron")).replace("focus_time_ms: 100", &format!("focus_time_ms: {focus}"));
        let (mut core, player, employee) = r11_live_sustained_fixture(false, false, 0, Some(&authored));
        core.state.active_battle.as_mut().unwrap().execution.audit_u03_exhaust_simulation_time();
        let authority = core.active_battle_authority().unwrap();
        let cx = ApplicationExecutionContext { expected_battle_authority:Some(ApplicationBattleAuthority {battle_uuid:authority.battle_uuid, battle_state_revision:authority.battle_state_revision}), ..Default::default() };
        let mut app = CoreApplication::from_core(core);
        let make_input = || ApplicationCommand::from_document(serde_json::json!({"type":"activate_skill", "employee_uuid":employee, "skill_id":"r08_strike", "target":null}).into()).unwrap();
        let first = app.execute(player, make_input(), cx.clone()).unwrap_err();
        let second = app.execute(player, make_input(), cx).unwrap_err();
        println!("focus={focus} first={} second={} fault_present={}",first.code(),second.code(),app.battle_fault().is_some());
        observed.push(app.battle_fault().is_some() && second.code()=="battle_faulted");
    }
    assert!(observed.iter().all(|v|*v),"checked admitted cast deadline overflow must record first fault and gate repetition");
}
