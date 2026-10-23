use game_core::mod_validation::LabCriterion;
use serde_json::json;

#[test]
fn json_or_a_claimed_pass_cannot_mint_actual_calibration_authority() {
    let mut value=json!({"revision":"untrusted","calibration_sha256":"ab".repeat(32),
        "max_elapsed_ns":1_000_000,"max_requested_heap_bytes":1_000_000,
        "max_rss_bytes":1_000_000,"max_native_work":1_000_000});
    assert!(serde_json::from_value::<LabCriterion>(value.clone()).is_err());
    value["binding"]=json!({"context_sha256":"ab".repeat(32),"sealed_sha256":"ab".repeat(32),
        "actual":{"status":"passed","metrics":{"complete":true}}});
    assert!(serde_json::from_value::<LabCriterion>(value).is_err());
}
