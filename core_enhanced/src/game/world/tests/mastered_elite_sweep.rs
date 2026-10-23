//! Legacy commands remain decodable for an explicit rejection, never a live offer.
use super::*;
use crate::game::behavior::ActionDisabledReasonCode;

#[test]
fn u17_retired_commands_reject_without_mutating_state_or_appearing_in_queries() {
    let mut core = GameCore::new(live_game_data_from_ron(), 1717);
    let player = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player, GameMode::Expedition);
    let before = core.get_run_snapshot_json().unwrap();
    for command in [PlayerBehavior::SweepMasteredElite {}, PlayerBehavior::CompleteMasteredEliteSweep {},
        PlayerBehavior::ConvertMapNodeWithMonolith { node_id: MapNodeId(Uuid::from_u128(1)), abnormality_id: "elite".into() }] {
        let value = serde_json::to_value(&command).unwrap();
        let decoded: PlayerBehavior = serde_json::from_value(value).unwrap();
        assert!(matches!(core.execute(player, decoded), Err(GameError::ActionUnavailable { reason: ActionDisabledReasonCode::RetiredFeature, .. })));
        assert_eq!(core.get_run_snapshot_json().unwrap(), before);
    }
    assert!(before.get("monolith").is_none());
    assert!(core.mastered_elite_resolution_dto().is_none());
    assert!(core.staged_mastered_elite_sweep_result().is_none());
    let payload = MapNodePayload::Shop { shop_id: None, shop_pool_id: Some("default_shops".into()) };
    let shop = core.resolve_map_shop(MapNodeId(Uuid::from_u128(0x1717)), &payload).unwrap().unwrap();
    assert!(shop.offers.iter().all(|offer| !matches!(offer.merchandise, ShopMerchandise::Monolith { .. })));
}
