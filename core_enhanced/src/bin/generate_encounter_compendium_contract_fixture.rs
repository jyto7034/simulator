use game_core::game::behavior::{
    EncounterCompendiumAbnormalitySnapshotDto, EncounterCompendiumCorrosionSnapshotDto,
    EncounterCompendiumSnapshotDto,
};
use serde_json::json;

fn main() {
    let game_data = game_core::game::data::ValidatedGameData::load_live_embedded()
        .expect("embedded live gameplay data must load for the compendium fixture");
    let abnormality = game_data
        .abnormality_data
        .items
        .first()
        .expect("live abnormality catalog must not be empty");
    let undiscovered_abnormality = game_data
        .abnormality_data
        .items
        .get(1)
        .expect("live abnormality catalog must contain a placeholder fixture entry");
    let corrosion_profile = game_data
        .corroded_employee_data
        .profiles
        .first()
        .expect("live corrosion profile catalog must not be empty");
    let snapshot = json!({
        "game_state_context": { "type": "viewing_map" },
        "action_capabilities": [],
        "player_profile_persistence": {
            "state": "clean",
            "dirty": false,
            "recovered_from_backup": false,
            "reset_required": false,
            "system_issue": null,
        },
        "encounter_compendium": EncounterCompendiumSnapshotDto {
            abnormalities: vec![
                EncounterCompendiumAbnormalitySnapshotDto {
                    abnormality_id: abnormality.id.clone(),
                    registered: true,
                    name: Some(abnormality.name.clone()),
                    risk_level: Some(abnormality.risk_level),
                },
                EncounterCompendiumAbnormalitySnapshotDto {
                    abnormality_id: undiscovered_abnormality.id.clone(),
                    registered: false,
                    name: None,
                    risk_level: None,
                },
            ],
            corrosion_profiles: vec![EncounterCompendiumCorrosionSnapshotDto {
                profile_id: corrosion_profile.id.clone(),
                registered: true,
                name: Some(corrosion_profile.name.clone()),
            }],
        },
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&snapshot).expect("fixture JSON must serialize")
    );
}
