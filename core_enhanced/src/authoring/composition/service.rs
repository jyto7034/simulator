use super::{compile_yaml, Diagnostic};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SOURCE_TARGET: &str = "composition-yaml-v1";
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompileRequest {
    pub source: String,
    pub epoch: u64,
    pub revision: u64,
    pub request_id: u64,
}
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CompileOutcome {
    Compiled {
        artifact: String,
        artifact_digest: String,
    },
    Rejected {
        diagnostic: Diagnostic,
    },
}
#[derive(Debug, Serialize)]
pub struct CompileResponse {
    pub epoch: u64,
    pub revision: u64,
    pub request_id: u64,
    pub source_digest: String,
    pub target: &'static str,
    #[serde(flatten)]
    pub outcome: CompileOutcome,
}
pub fn compile_request(request: CompileRequest) -> Result<CompileResponse, String> {
    if [request.epoch, request.revision, request.request_id]
        .iter()
        .any(|n| *n > (1u64 << 53) - 1)
    {
        return Err("editor request identity exceeds exact integer range".into());
    }
    let outcome = match compile_yaml(&request.source).and_then(|artifact| artifact.to_ron()) {
        Ok(artifact) => CompileOutcome::Compiled {
            artifact_digest: format!("{:x}", Sha256::digest(artifact.as_bytes())),
            artifact,
        },
        Err(diagnostic) => CompileOutcome::Rejected { diagnostic },
    };
    Ok(CompileResponse {
        epoch: request.epoch,
        revision: request.revision,
        request_id: request.request_id,
        source_digest: format!("{:x}", Sha256::digest(request.source.as_bytes())),
        target: SOURCE_TARGET,
        outcome,
    })
}
