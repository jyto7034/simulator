use std::cmp::Ordering;
use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnitInstanceId(pub Uuid);

impl UnitInstanceId {
    pub fn as_uuid(self) -> Uuid {
        self.0
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

impl Ord for UnitInstanceId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl PartialOrd for UnitInstanceId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<Uuid> for UnitInstanceId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<UnitInstanceId> for Uuid {
    fn from(value: UnitInstanceId) -> Self {
        value.0
    }
}

impl fmt::Display for UnitInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
