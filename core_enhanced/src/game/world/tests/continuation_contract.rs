//! Reusable durable-boundary comparison. Callers establish a confirmed state
//! using supported commands; unfinished-node rollback needs a pre-entry oracle.
use super::*;
use crate::application::{
    ApplicationCommand, ApplicationExecutionContext, ApplicationSnapshotKind, CoreApplication,
};

pub(super) fn assert_fresh_application_next(
    core: &GameCore,
    player: Uuid,
    commands: &[Value],
) {
    let mut uninterrupted = CoreApplication::from_core(core.clone());
    let export = uninterrupted.export_active_run_checkpoint().unwrap().expect("durable boundary");
    let mut fresh = CoreApplication::new(core.game_data.clone(), 0xfeed);
    install_application_profile_for_continue(&mut fresh, &uninterrupted);
    fresh.continue_from_active_run_checkpoint(player, &export).unwrap();
    let compare = |a: &CoreApplication, b: &CoreApplication| {
        for kind in [ApplicationSnapshotKind::Inventory, ApplicationSnapshotKind::EmployeeRoster,
            ApplicationSnapshotKind::SelectedEvent, ApplicationSnapshotKind::PlayerProfile,
            ApplicationSnapshotKind::Run] {
            assert_eq!(a.snapshot(kind).unwrap().read_model().as_value(),
                b.snapshot(kind).unwrap().read_model().as_value(), "fresh {kind:?}");
        }
    };
    compare(&uninterrupted, &fresh);
    for command in commands {
        let execute = |app: &mut CoreApplication| app.execute(player,
            ApplicationCommand::from_document(command.clone().into()).unwrap(),
            ApplicationExecutionContext::default());
        let left = execute(&mut uninterrupted);
        let right = execute(&mut fresh);
        assert_eq!(serde_json::to_value(&left).unwrap(), serde_json::to_value(&right).unwrap(),
            "next command {command}");
        compare(&uninterrupted, &fresh);
    }
}
