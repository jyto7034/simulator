#[path = "../../../external_pack_probe.rs"]
mod external_pack_probe;

use game_core::game::{behavior::CombatResultEventLogAttachmentDto, world::GameCore};

fn main() {
    let data = external_pack_probe::load_selected();
    let core = GameCore::new(data, 1);

    // Default-release consumers must be unable to name the debug attachment
    // DTO or request a cloned terminal diagnostic history from GameCore.
    let _: Option<CombatResultEventLogAttachmentDto> =
        core.get_combat_result_event_log_attachment();
}
