//! Transport-independent compiler request. Request identity is echoed for the
//! editor's stale-result checks; it is never treated as execution authority.
use super::{compile, Diagnostic, NATIVE_TARGET};
use crate::game::ability::SkillId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompileRequest {
    pub source: String,
    pub skill_id: SkillId,
    pub epoch: u64,
    pub revision: u64,
    pub request_id: u64,
}
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CompileOutcome {
    Compiled { ron: String },
    Rejected { diagnostics: Vec<Diagnostic> },
}
#[derive(Debug, Serialize)]
pub struct CompileResponse {
    pub epoch: u64,
    pub revision: u64,
    pub request_id: u64,
    pub skill_id: SkillId,
    pub source_digest: String,
    pub target: &'static str,
    #[serde(flatten)]
    pub outcome: CompileOutcome,
}
pub fn compile_request(request: CompileRequest) -> Result<CompileResponse, String> {
    const SAFE_INTEGER: u64 = (1u64 << 53) - 1;
    if [request.epoch, request.revision, request.request_id]
        .iter()
        .any(|n| *n > SAFE_INTEGER)
    {
        return Err("editor request identity exceeds exact integer range".into());
    }
    if request.source.len() > 5_000_000 {
        return Err("graph input byte limit exceeded".into());
    }
    let outcome = match compile(&request.source, request.skill_id.clone()) {
        Ok(artifact) => CompileOutcome::Compiled {
            ron: artifact.to_ron()?,
        },
        Err(diagnostics) => CompileOutcome::Rejected { diagnostics },
    };
    Ok(CompileResponse {
        epoch: request.epoch,
        revision: request.revision,
        request_id: request.request_id,
        skill_id: request.skill_id,
        source_digest: format!("{:x}", Sha256::digest(request.source.as_bytes())),
        target: NATIVE_TARGET,
        outcome,
    })
}
