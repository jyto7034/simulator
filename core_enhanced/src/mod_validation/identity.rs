use super::ModValidationError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{io::Read, path::Path};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildIdentity {
    #[serde(default)]
    pub dependency_graph_sha256: String,
    pub source_compiler_sha256: String,
    pub target: String,
    pub profile: String,
    pub features: String,
    pub executable_sha256: String,
}
impl BuildIdentity {
    pub(crate) fn check_current_core(
        pair: &ExecutionPairIdentity,
    ) -> Result<(), ModValidationError> {
        if env!("CMVA_BUILD_DEPENDENCY_GRAPH").is_empty()
            || pair.plain.dependency_graph_sha256 != env!("CMVA_BUILD_DEPENDENCY_GRAPH")
            || pair.plain.source_compiler_sha256 != env!("CMVA_BUILD_SHA256")
            || pair.plain.target != env!("CMVA_BUILD_TARGET")
            || pair.plain.profile != env!("CMVA_BUILD_PROFILE")
            || !matches!(env!("CMVA_BUILD_FEATURES"), "" | "RUNTIME_OBSERVATION")
        {
            return Err(ModValidationError::incomplete("issuer.build", "issuing Core lacks the exact supported actual dependency/build correspondence with workers"));
        }
        Ok(())
    }
    pub fn current(executable: &Path) -> Result<Self, ModValidationError> {
        let mut file = std::fs::File::open(executable)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(ModValidationError::incomplete(
                "executable",
                "not a regular binary",
            ));
        }
        let mut h = Sha256::new();
        let mut buf = [0; 65536];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            h.update(&buf[..n]);
        }
        Ok(Self {
            dependency_graph_sha256: env!("CMVA_BUILD_DEPENDENCY_GRAPH").into(),
            source_compiler_sha256: env!("CMVA_BUILD_SHA256").into(),
            target: env!("CMVA_BUILD_TARGET").into(),
            profile: env!("CMVA_BUILD_PROFILE").into(),
            features: env!("CMVA_BUILD_FEATURES").into(),
            executable_sha256: format!("{:x}", h.finalize()),
        })
    }
    pub(crate) fn valid(&self) -> bool {
        [
            &self.source_compiler_sha256,
            &self.executable_sha256,
            &self.dependency_graph_sha256,
        ]
        .iter()
        .all(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            && !self.target.is_empty()
            && self.profile == "release"
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionPairIdentity {
    pub plain: BuildIdentity,
    pub instrumented: BuildIdentity,
    pub environment: String,
    pub measurement_method: String,
}
impl ExecutionPairIdentity {
    pub(crate) fn validate(&self) -> Result<(), ModValidationError> {
        if !self.plain.valid()
            || !self.instrumented.valid()
            || !self.plain.features.is_empty()
            || self.instrumented.features != "RUNTIME_OBSERVATION"
            || self.plain.source_compiler_sha256 != self.instrumented.source_compiler_sha256
            || self.plain.dependency_graph_sha256 != self.instrumented.dependency_graph_sha256
            || self.plain.target != self.instrumented.target
            || self.environment.is_empty()
            || self.environment.len() > 1024
            || self.measurement_method.is_empty()
            || self.measurement_method.len() > 256
        {
            return Err(ModValidationError::incomplete("execution_pair","missing/unsupported actual default-release and instrumented build/environment identity"));
        }
        Ok(())
    }
}
