use game_core::game::{
    behavior::{BehaviorCommandResultContract, BehaviorResult},
    data::GameDataBase,
    map::{MapTemplateId, MapViewDto},
    skill_fragment::SkillFragmentResearchDelivery,
};
use serde_json::json;

fn main() {
    let game_data = GameDataBase::load_live_embedded()
        .expect("embedded live gameplay data must load for the research-delivery fixture");
    let fragment = game_data
        .skill_fragment_data
        .fragments
        .first()
        .expect("live skill fragment catalog must not be empty");
    let result = BehaviorResult::NodeCompleted {
        map: MapViewDto {
            map_template_id: MapTemplateId::new("research_delivery_fixture"),
            facility_template_version: game_core::game::map::FACILITY_TEMPLATE_VERSION,
            nodes: vec![],
        },
        outcome: None,
        research_deliveries: vec![SkillFragmentResearchDelivery {
            fragment_id: fragment.id.clone(),
            count: 1,
            total_count: 2,
        }],
    };
    let BehaviorCommandResultContract::CommandPayload(contract) =
        result.into_command_result_contract()
    else {
        panic!("planning return must serialize as a command payload");
    };
    let wire = json!({
        "type": "command_result",
        "request_id": "complete-node-research-delivery",
        "ok": true,
        "result_type": contract.result_type(),
        "payload": contract
            .payload_value()
            .expect("research-delivery payload must serialize"),
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&wire).expect("fixture JSON must serialize")
    );
}
