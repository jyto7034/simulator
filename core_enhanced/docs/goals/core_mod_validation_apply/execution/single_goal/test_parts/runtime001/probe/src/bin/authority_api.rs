//! External compile surface. Each invalid feature is checked separately after
//! the feature-free control; none uses unsafe or a private test fixture API.
#![allow(dead_code)]
use game_core::{application::CoreApplication,game::data::ValidatedGameData,
    mod_validation::{ApprovedGeneration,ApprovedHost,CalibrationRun,Candidate,CriterionAssessment,
        JobId,LabOutcome,ModValidationError,ValidationIssuer}};
use std::sync::Arc;

fn valid_issuer_to_host(issuer:Arc<ValidationIssuer>,job:JobId)
    ->Result<Arc<ApprovedHost>,ModValidationError> {
    let approved=issuer.approval(job)?;
    ApprovedHost::new(issuer,approved)
}
fn valid_active_and_future(host:&ApprovedHost,approved:ApprovedGeneration)
    ->Result<CoreApplication,ModValidationError> {
    host.apply(approved)?;
    host.new_application(11).map(|(application,_)|application)
}
fn trusted_embedding_remains_explicit(generation:ValidatedGameData)->CoreApplication {
    // Raw CoreApplication is a separately trusted embedding surface. The
    // normal mod host boundary is what must require an issued capability.
    CoreApplication::new(generation,11)
}

#[cfg(feature="invalid_deserialize_approval")]
fn deserialize_approval(blob:&str)->ApprovedGeneration {
    serde_json::from_str(blob).unwrap()
}
#[cfg(feature="invalid_static_candidate")]
fn static_candidate_is_not_approval(issuer:Arc<ValidationIssuer>,candidate:Candidate) {
    let _=ApprovedHost::new(issuer,candidate);
}
#[cfg(feature="invalid_result_dto")]
fn result_dto_is_not_approval(issuer:Arc<ValidationIssuer>,result:LabOutcome) {
    let _=ApprovedHost::new(issuer,result);
}
#[cfg(feature="invalid_comparator_result")]
fn comparator_result_is_not_approval(issuer:Arc<ValidationIssuer>,result:CriterionAssessment) {
    let _=ApprovedHost::new(issuer,result);
}
#[cfg(feature="invalid_direct_future")]
fn future_owner_is_private(host:&ApprovedHost,approved:ApprovedGeneration) {
    *host.future.lock().unwrap()=approved;
}
#[cfg(feature="invalid_issuer_authority")]
fn issuer_authority_is_private(issuer:&ValidationIssuer) {
    let _=&issuer.authority;
}
#[cfg(feature="invalid_deserialize_calibration")]
fn serialized_metrics_are_not_calibration(blob:&str)->CalibrationRun {
    serde_json::from_str(blob).unwrap()
}
fn main() {}
