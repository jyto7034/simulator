use game_core::game::{
    behavior::{
        BattlePlaybackState, LiveBattleEventDeltaDto, LiveBattleSetupBattlefieldDto,
        LiveBattleSetupCatalogRefsDto, LiveBattleSetupSnapshotDto,
        LiveBattleSetupSnapshotMessageType, LiveBattleStateCheckpointDto,
        LiveBattleTacticalResourceDto, LiveBattleUpdateDto, LiveBattleUpdateMessageType,
    },
    combat_preview::{CombatMissionVariant, CombatNodeType},
};
use game_server::game::player_game_actor::messages::PlayerGameServerMessage;
use serde_json::json;
use uuid::Uuid;

fn update(battle_uuid: Uuid, revision: u64, paused: bool) -> LiveBattleUpdateDto {
    LiveBattleUpdateDto {
        message_type: LiveBattleUpdateMessageType::BattleUpdate,
        battle_uuid,
        battle_state_revision: revision,
        action_capabilities: Vec::new(),
        server_battle_time_ms: 0,
        events_delta: LiveBattleEventDeltaDto {
            after_seq: 0,
            to_seq: 0,
            events: Vec::new(),
        },
        checkpoint: LiveBattleStateCheckpointDto {
            interaction: Default::default(),
            sustained_effects: Default::default(),
            independent_objects: Default::default(),
            at_seq: 0,
            battle_time_ms: 0,
            playback: BattlePlaybackState {
                paused,
                ..BattlePlaybackState::default()
            },
            playback_eligibility:
                game_core::game::behavior::LiveBattlePlaybackEligibilityDto::for_state(
                    BattlePlaybackState {
                        paused,
                        ..BattlePlaybackState::default()
                    },
                ),
            tactical_focus: None,
            field_work_objects: Vec::new(),
            units: Vec::new(),
            tactical_resource: LiveBattleTacticalResourceDto {
                current: 20,
                maximum: 99,
                recovery_per_second: 1,
            },
            deployment: None,
        },
        visual_novel: None,
    }
}

fn main() {
    let battle_uuid = Uuid::parse_str("11111111-2222-3333-4444-555555555555")
        .expect("fixture UUID must be valid");
    let setup = LiveBattleSetupSnapshotDto {
        message_type: LiveBattleSetupSnapshotMessageType::BattleSetupSnapshot,
        setup_version: 2,
        battle_uuid,
        encounter_id: "revision_contract_fixture".to_string(),
        node_type: CombatNodeType::Defense,
        mission_variant: CombatMissionVariant::Defense,
        survive_timer_ms: None,
        battlefield: LiveBattleSetupBattlefieldDto {
            width: 2,
            height: 1,
            tiles: Vec::new(),
            valid_tiles: Vec::new(),
            blocked_tiles: Vec::new(),
            static_obstacles: Vec::new(),
        },
        routes: Vec::new(),
        deployment_zones: Vec::new(),
        spawn_zones: Vec::new(),
        tactical_points: Vec::new(),
        static_objects: Vec::new(),
        field_work_objects: Vec::new(),
        initial_units: Vec::new(),
        catalog_refs: LiveBattleSetupCatalogRefsDto {
            battlefield_template_id: "revision_contract_fixture".to_string(),
            abnormality_ids: Vec::new(),
        },
    };
    let initial = update(battle_uuid, 1, false);
    let paused = update(battle_uuid, 2, true);
    let repaired = update(battle_uuid, 2, false);
    let fixture = json!({
        "generator": "game_server/examples/generate_battle_revision_fixture.rs",
        "setup": serde_json::to_value(PlayerGameServerMessage::battle_setup_snapshot(setup))
            .expect("setup must serialize"),
        "initial_update": serde_json::to_value(PlayerGameServerMessage::battle_update(initial))
            .expect("initial update must serialize"),
        "same_seq_time_higher_revision_update":
            serde_json::to_value(PlayerGameServerMessage::battle_update(paused.clone()))
                .expect("paused update must serialize"),
        "same_revision_resync":
            serde_json::to_value(PlayerGameServerMessage::battle_resync(repaired))
                .expect("resync must serialize"),
    });

    if let Some(path) = std::env::args().nth(1) {
        let checked_in: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&path).expect("checked-in fixture must be readable"),
        )
        .expect("checked-in fixture must be valid JSON");
        assert_eq!(
            checked_in, fixture,
            "checked-in fixture differs from typed Rust serialization: {path}"
        );
        return;
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&fixture).expect("fixture must serialize")
    );
}
