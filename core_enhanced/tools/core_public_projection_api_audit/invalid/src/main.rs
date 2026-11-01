#[path = "../../../external_pack_probe.rs"]
mod external_pack_probe;

use game_core::{
    application::CoreApplication,
    game::{
        behavior::{BehaviorResult, PlayerBehavior},
        world::{ActiveRunTransactionRollback, GameCore},
    },
};

fn main() {
    let data = external_pack_probe::load_selected();
    let raw = GameCore::new(data.clone(), 1);
    let _: Option<PlayerBehavior> = None;
    let _: Option<BehaviorResult> = None;
    let _: Option<ActiveRunTransactionRollback> = None;

    let application = CoreApplication::new(data, 2);
    let _ = application.core;
    drop(raw);
}
