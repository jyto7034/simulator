use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModValidationErrorKind {
    InvalidInput,
    ContentFailed,
    Incomplete,
    DependencyHeld,
    Cancelled,
    Stale,
    ForeignIssuer,
    ProtectedOriginal,
    Io,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModValidationError {
    pub kind: ModValidationErrorKind,
    pub field_path: String,
    pub reason: String,
}
impl ModValidationError {
    pub fn new(
        kind: ModValidationErrorKind,
        field: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        let mut reason = reason.into();
        if reason.len() > 4096 {
            let mut end = 4096;
            while !reason.is_char_boundary(end) {
                end -= 1;
            }
            reason.truncate(end);
        }
        let mut field_path = field.into();
        if field_path.len() > 4096 {
            let mut end = 4096;
            while !field_path.is_char_boundary(end) {
                end -= 1;
            }
            field_path.truncate(end);
        }
        Self {
            kind,
            field_path,
            reason,
        }
    }
    pub(crate) fn invalid(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(ModValidationErrorKind::InvalidInput, field, reason)
    }
    pub(crate) fn incomplete(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(ModValidationErrorKind::Incomplete, field, reason)
    }
    pub(crate) fn protected(reason: impl Into<String>) -> Self {
        Self::new(
            ModValidationErrorKind::ProtectedOriginal,
            "required_configuration",
            reason,
        )
    }
    pub(crate) fn execution(
        field: impl Into<String>,
        error: crate::authoring::control::ExecutionFailure,
    ) -> Self {
        let kind = if error.code == "authoring_cancelled" {
            ModValidationErrorKind::Cancelled
        } else {
            ModValidationErrorKind::Incomplete
        };
        Self::new(kind, field, format!("{error:?}"))
    }
}
impl std::fmt::Display for ModValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {}: {}", self.kind, self.field_path, self.reason)
    }
}
impl std::error::Error for ModValidationError {}
impl From<std::io::Error> for ModValidationError {
    fn from(e: std::io::Error) -> Self {
        Self::new(ModValidationErrorKind::Io, "host_io", e.to_string())
    }
}
