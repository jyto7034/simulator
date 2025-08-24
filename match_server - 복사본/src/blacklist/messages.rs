use actix::prelude::*;
use std::net::IpAddr;
use uuid::Uuid;

use super::ViolationType;

#[derive(Debug, Clone)]
pub enum BlockCheckResult {
    Allowed,
    Blocked {
        remaining_seconds: u64,
        reason: String,
    },
}

#[derive(Message)]
#[rtype(result = "Result<(), anyhow::Error>")]
pub struct RecordViolation {
    pub player_id: Uuid,
    pub violation_type: ViolationType,
    pub ip_addr: Option<IpAddr>,
}

#[derive(Message)]
#[rtype(result = "Result<BlockCheckResult, anyhow::Error>")]
pub struct CheckPlayerBlock {
    pub player_id: Uuid,
    pub ip_addr: Option<IpAddr>,
}

#[derive(Message)]
#[rtype(result = "Result<(), anyhow::Error>")]
pub struct ClearPlayerViolations {
    pub player_id: Uuid,
}