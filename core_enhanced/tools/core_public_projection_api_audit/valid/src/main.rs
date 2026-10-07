#[path = "../../../external_pack_probe.rs"]
mod external_pack_probe;

use game_core::application::{
    ApplicationCommand, ApplicationDocument, ApplicationExecutionContext, ApplicationResultKind,
    ApplicationSnapshotKind, CoreApplication,
};
use serde_json::json;
use uuid::Uuid;

fn main() {
    let data = external_pack_probe::load_selected();
    let mut application = CoreApplication::new(data, 0xCA07_1001);
    let before = application
        .snapshot(ApplicationSnapshotKind::Run)
        .expect("initial read model");
    let invalid = ApplicationCommand::from_document(ApplicationDocument::new(json!({
        "type": "unknown_core_command"
    })))
    .expect_err("unknown command must reject before authority access");
    assert_eq!(invalid.code(), "invalid_application_command");
    assert_eq!(
        before,
        application
            .snapshot(ApplicationSnapshotKind::Run)
            .expect("unchanged read model")
    );

    let result = application
        .execute(
            Uuid::from_u128(0xCA07_1001),
            ApplicationCommand::from_document(ApplicationDocument::new(json!({
                "type": "start_new_game",
                "game_mode": "Standard",
                "replace_active_run": false,
            })))
            .expect("known command must validate"),
            ApplicationExecutionContext::default(),
        )
        .expect("known command must execute");
    assert_eq!(result.kind(), ApplicationResultKind::RunStartFlowStarted);
}
