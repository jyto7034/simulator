#[path = "../../../external_pack_probe.rs"]
mod external_pack_probe;

use game_core::application::CoreApplication;

fn main() {
    let data = external_pack_probe::load_selected();
    let application = CoreApplication::new(data, 0xC2_02);
    drop(application);
}
