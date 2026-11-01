//! Authored battle-copy stocks, hostile cohorts and atomic directional volleys.
use super::{marks::MarkRatio, relation_action::PlanStatus, resonance::ResonanceAdmission};
use crate::game::battle::{damage::DamageType, tile_range::TileRangePattern};
use serde::{Deserialize, Serialize};
pub use super::hostile_cohort::HostileCohortDef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShotStockDef {
    pub definition: String,
    pub initial: u32,
    pub capacity: Option<u32>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShotProjectileDef {
    pub definition: String,
    pub power: u32,
    pub damage_type: DamageType,
    pub speed: MarkRatio,
    pub range: MarkRatio,
    pub width: MarkRatio,
    pub lifetime_ms: u32,
    pub max_hits: Option<u32>,
    pub air: bool,
    pub obstacle_stop: bool,
    pub after: Vec<PlanStatus>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ShotChange {
    KillGain {
        factor: MarkRatio,
    },
    Wound {
        profiles: Vec<String>,
        amount: u32,
        cap: u32,
        percent: MarkRatio,
        duration_ms: u32,
    },
    WoundGrant {
        definition: String,
        factor: MarkRatio,
    },
    Status {
        profile: String,
        status: PlanStatus,
    },
    Push {
        profile: String,
        tiles: MarkRatio,
    },
    Explosion {
        profile: String,
        power: u32,
        damage_type: DamageType,
        range: TileRangePattern,
        air: bool,
        bonus: MarkRatio,
    },
    ControlVulnerability {
        definition: String,
        percent: u32,
        duration_ms: u32,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShotModifier {
    pub id: String,
    pub relic_definition: String,
    pub requires: Vec<String>,
    pub change: ShotChange,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputPlanDef {
    pub definition: String,
    pub admission: ResonanceAdmission,
    pub normal: ShotStockDef,
    pub enhanced: ShotStockDef,
    pub stock_interval_ms: u32,
    pub stock_gain: u32,
    pub cohort: HostileCohortDef,
    pub placement_range: TileRangePattern,
    pub allow_duplicates: bool,
    pub normal_projectile: ShotProjectileDef,
    pub enhanced_projectile: ShotProjectileDef,
    pub modifiers: Vec<ShotModifier>,
}
impl InputPlanDef {
    pub fn validate(&self) -> Result<(), String> {
        fn identity(s: &str) -> bool {
            !s.trim().is_empty()
        }
        self.admission.validate().map_err(str::to_owned)?;
        self.placement_range.validate()?;
        if !identity(&self.definition)
            || self.normal.definition == self.enhanced.definition
            || self.stock_interval_ms == 0
            || self.stock_gain == 0
        {
            return Err("invalid input plan stocks".into());
        }
        for s in [&self.normal, &self.enhanced] {
            if !identity(&s.definition) || s.capacity.is_some_and(|c| c == 0 || s.initial > c) {
                return Err("invalid shot stock".into());
            }
        }
        self.cohort.validate()?;
        let valid_status = |s: &PlanStatus| -> Result<(), String> {
            if !identity(&s.definition)
                || s.duration_ms == 0
                || !matches!(s.count, super::relation_action::ActionCount::Constant(n) if n > 0)
            {
                return Err("invalid projectile hit status".into());
            }
            s.percent.validate(true)
        };
        if self.normal_projectile.definition == self.enhanced_projectile.definition {
            return Err("projectile identities must be distinct".into());
        }
        for p in [&self.normal_projectile, &self.enhanced_projectile] {
            if !identity(&p.definition) || p.lifetime_ms == 0 || p.max_hits == Some(0) {
                return Err("invalid projectile lifetime or hit limit".into());
            }
            p.speed.validate(false)?;
            p.range.validate(false)?;
            p.width.validate(false)?;
            for s in &p.after {
                valid_status(s)?;
            }
        }
        let profile = |s: &str| {
            s == self.normal_projectile.definition || s == self.enhanced_projectile.definition
        };
        let mut ids = std::collections::BTreeSet::new();
        for m in &self.modifiers {
            if !identity(&m.id)
                || !identity(&m.relic_definition)
                || !ids.insert(&m.id)
                || m.requires
                    .iter()
                    .any(|r| !identity(r) || r == &m.relic_definition)
            {
                return Err("invalid projectile modifier identity".into());
            }
            match &m.change {
                ShotChange::KillGain { factor } => factor.validate(false)?,
                ShotChange::Wound { profiles, amount, cap, percent, duration_ms } => {
                    if profiles.is_empty() || profiles.iter().any(|p| !profile(p)) || profiles.iter().collect::<std::collections::BTreeSet<_>>().len()!=profiles.len() || *amount == 0 || *cap == 0 || *duration_ms == 0 { return Err("invalid projectile wound".into()); }
                    percent.validate(true)?;
                }
                ShotChange::WoundGrant { definition, factor } => {
                    if !self.modifiers.iter().any(|w| &w.id == definition && matches!(w.change, ShotChange::Wound { .. })) { return Err("unknown wound definition".into()); }
                    factor.validate(false)?;
                }
                ShotChange::Status { profile: p, status } => { if !profile(p) { return Err("unknown projectile".into()); } valid_status(status)?; }
                ShotChange::Push { profile: p, tiles } => { if !profile(p) { return Err("unknown projectile".into()); } tiles.validate(false)?; }
                ShotChange::Explosion { profile: p, range, bonus, .. } => { if !profile(p) { return Err("unknown projectile".into()); } range.validate()?; bonus.validate(true)?; }
                ShotChange::ControlVulnerability { definition, duration_ms, .. } => {
                    if *duration_ms == 0 || !self.modifiers.iter().any(|m| matches!(&m.change,ShotChange::Status {status,..} if &status.definition==definition && status.kind==super::relation_action::PlanStatusKind::Stun)) { return Err("control modifier requires a declared stun".into()); }
                }
            }
        }
        Ok(())
    }
}
crate::authoring::clone_shape::struct_shape!(ShotStockDef {
    definition,
    initial,
    capacity
});

crate::authoring::clone_shape::struct_shape!(ShotProjectileDef {
    definition,
    power,
    damage_type,
    speed,
    range,
    width,
    lifetime_ms,
    max_hits,
    air,
    obstacle_stop,
    after
});
crate::authoring::clone_shape::struct_shape!(ShotModifier {
    id,
    relic_definition,
    requires,
    change
});
crate::authoring::clone_shape::struct_shape!(InputPlanDef {
    definition,
    admission,
    normal,
    enhanced,
    stock_interval_ms,
    stock_gain,
    cohort,
    placement_range,
    allow_duplicates,
    normal_projectile,
    enhanced_projectile,
    modifiers
});
impl crate::authoring::clone_shape::CloneShape for ShotChange {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        match self {
            Self::KillGain { .. } => Ok(()),
            Self::Wound { profiles, .. } => profiles.visit_clone_shape(m),
            Self::WoundGrant { definition, .. } | Self::ControlVulnerability { definition, .. } => {
                definition.visit_clone_shape(m)
            }
            Self::Status { profile, status } => {
                profile.visit_clone_shape(m)?;
                status.visit_clone_shape(m)
            }
            Self::Push { profile, .. } => profile.visit_clone_shape(m),
            Self::Explosion { profile, range, .. } => {
                profile.visit_clone_shape(m)?;
                range.visit_clone_shape(m)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShotKind {
    Normal,
    Enhanced,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShotPlacement {
    pub kind: ShotKind,
    pub tile: crate::game::resources::Position,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShotDirection {
    pub x: i32,
    pub y: i32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShotLayoutInput {
    Fresh(Vec<ShotPlacement>),
    Remembered,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShotAimInput {
    Precise(Vec<ShotDirection>),
    Focused(crate::game::resources::Position),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShotCommitInput {
    pub preparation: u64,
    pub request: u64,
    pub layout: ShotLayoutInput,
    pub aim: ShotAimInput,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShotCommitReceipt {
    pub copy: uuid::Uuid,
    pub preparation: u64,
    pub request: u64,
    pub batch: u64,
    pub normal: u32,
    pub enhanced: u32,
    pub released_at_ms: u64,
}
crate::authoring::clone_shape::copy_shape!(ShotKind, ShotDirection, ShotCommitReceipt);
crate::authoring::clone_shape::struct_shape!(ShotPlacement { kind, tile });
crate::authoring::clone_shape::struct_shape!(ShotCommitInput {
    preparation,
    request,
    layout,
    aim
});
impl crate::authoring::clone_shape::CloneShape for ShotLayoutInput {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        match self {
            Self::Fresh(v) => v.visit_clone_shape(m),
            Self::Remembered => Ok(()),
        }
    }
}
impl crate::authoring::clone_shape::CloneShape for ShotAimInput {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        match self {
            Self::Precise(v) => v.visit_clone_shape(m),
            Self::Focused(_) => Ok(()),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShotEnd {
    HitLimit,
    Range,
    Lifetime,
    Obstacle,
    Boundary,
    BattleEnded,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShotStatusApplication {
    pub definition: String,
    pub applied: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShotHitResult {
    pub statuses: Vec<ShotStatusApplication>,
    pub target: crate::game::battle::ids::UnitInstanceId,
    pub time_ms: u64,
    pub actual_hp_loss: u64,
    pub effective_hit: bool,
    pub killed: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShotFlightResult {
    pub id: u64,
    pub batch: u64,
    pub copy: uuid::Uuid,
    pub kind: ShotKind,
    pub ended_at_ms: u64,
    pub end: ShotEnd,
    pub hits: Vec<ShotHitResult>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShotOwnerView {
    pub copy: uuid::Uuid,
    pub employee: uuid::Uuid,
    pub caster: Option<crate::game::battle::ids::UnitInstanceId>,
    pub normal: u32,
    pub enhanced: u32,
    pub next_stock_ms: u64,
    pub next_hostile_ms: u64,
    pub memory: Vec<ShotPlacement>,
    pub applied: Vec<(String, uuid::Uuid)>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShotPreparationView {
    pub id: u64,
    pub copy: uuid::Uuid,
    pub caster: crate::game::battle::ids::UnitInstanceId,
    pub started_at_ms: u64,
    pub normal: u32,
    pub enhanced: u32,
    pub remembered_layout_valid: bool,
    pub placement_tiles: Vec<crate::game::resources::Position>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InputPlansSnapshot {
    pub owners: Vec<ShotOwnerView>,
    pub preparation: Option<ShotPreparationView>,
    pub disconnected: bool,
    pub receipts: Vec<ShotCommitReceipt>,
    pub active_projectiles: Vec<u64>,
    pub settled_projectiles: Vec<ShotFlightResult>,
    pub cohort_units: Vec<(crate::game::battle::ids::UnitInstanceId, uuid::Uuid)>,
    pub wounds: Vec<(
        uuid::Uuid,
        String,
        crate::game::battle::ids::UnitInstanceId,
        u32,
        u64,
    )>,
}
crate::authoring::clone_shape::copy_shape!(ShotEnd);
crate::authoring::clone_shape::struct_shape!(ShotStatusApplication {
    definition,
    applied
});
crate::authoring::clone_shape::struct_shape!(ShotHitResult {
    statuses,
    target,
    time_ms,
    actual_hp_loss,
    effective_hit,
    killed
});
crate::authoring::clone_shape::struct_shape!(ShotFlightResult {
    id,
    batch,
    copy,
    kind,
    ended_at_ms,
    end,
    hits
});
