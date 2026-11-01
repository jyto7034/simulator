//! Trusted normal host admission adapters. The Core token remains opaque.
mod persistence;
mod session;
pub mod worker;
pub use persistence::ApprovedPersistence;

#[cfg(test)]
#[path = "../../core_enhanced/docs/goals/core_mod_validation_apply/execution/single_goal/test_parts/runtime001/shared/normal_save_tests.rs"]
mod u11_normal_save_tests;
#[cfg(test)]
#[path = "../../core_enhanced/docs/goals/core_expedition_lifecycle/execution/single_goal/test_parts/runtime001/shared/normal_host_tests.rs"]
mod u12_normal_host_tests;
pub use session::{administration, AdmissionConfig, AdmissionReport, AdmissionSession, IndividualValidation};
