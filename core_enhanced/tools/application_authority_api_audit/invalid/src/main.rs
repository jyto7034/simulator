#[path = "../../../external_pack_probe.rs"]
mod external_pack_probe;

use game_core::game::world::GameCore;

fn main() {
    let data = external_pack_probe::load_selected();
    let mut core = GameCore::new(data, 1);

    // All three expressions must remain unavailable in a default release build:
    // direct test-currency mutation, direct root replacement, and construction of
    // a parallel lifecycle state through the private state module.
    core.set_enkephalin(999);
    let _ = &mut core.state;
    let _ = game_core::game::world::state::GameCoreState::new(1);
}
