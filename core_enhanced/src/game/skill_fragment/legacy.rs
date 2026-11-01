//! Read-only V7 fragment progress retained by the explicit checkpoint converter.
//! New gameplay never reads rank, shared awakening progress or these flags.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyFragmentProgress {
    pub upgrade_level: u8,
    pub awakening_progress: u32,
    pub awakening_available: bool,
    pub awakened: bool,
}
crate::authoring::clone_shape::struct_shape!(LegacyFragmentProgress {
    upgrade_level,
    awakening_progress,
    awakening_available,
    awakened
});
