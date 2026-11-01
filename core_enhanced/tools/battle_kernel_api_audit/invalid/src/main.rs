#[path = "../../../external_pack_probe.rs"]
mod external_pack_probe;

use game_core::game::battle::{
    core::{movement::types::WorldVec2, BattleCore},
    enums::BattleEvent,
    scenario::BattleScenario,
};

fn main() {
    let data = external_pack_probe::load_selected();
    let mut kernel = BattleCore::new_from_scenario(BattleScenario::empty((1, 1)), data, 1);

    // A default release caller must be unable to name the kernel, reach its
    // unit authority, inject raw events, or use the unchecked float constructor.
    kernel.units.clear();
    kernel.enqueue_event(BattleEvent::ContinuousMovementTick { time_ms: 0 });
    let _ = WorldVec2::new(f32::NAN, f32::INFINITY);
}
