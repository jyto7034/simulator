#[path = "../../../external_pack_probe.rs"]
mod external_pack_probe;

use game_core::application::{
    ApplicationCommand, ApplicationDocument, ApplicationExecutionContext, ApplicationResultKind,
    CoreApplication,
};
use serde_json::json;
use uuid::Uuid;

fn main() {
    let data = external_pack_probe::load_selected();
    let mut application = CoreApplication::new(data, 0xCA03_1001);
    let result = application
        .execute(
            Uuid::from_u128(0xCA03_1001),
            ApplicationCommand::from_document(ApplicationDocument::new(json!({
                "type": "start_new_game",
                "game_mode": "Standard",
                "replace_active_run": false,
            })))
            .expect("application command must validate"),
            ApplicationExecutionContext::default(),
        )
        .expect("registered application route must execute");
    assert_eq!(result.kind(), ApplicationResultKind::RunStartFlowStarted);
}
