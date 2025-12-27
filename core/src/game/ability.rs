use serde::{Deserialize, Serialize};

pub type SkillId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillKind {
    Targeted,
    Untargeted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryDef {
    Instant,
    Projectile { speed_units_per_ms: u32 },
}

impl Default for DeliveryDef {
    fn default() -> Self {
        Self::Instant
    }
}

fn default_cast_delay_ms() -> u32 {
    10
}

/// 데이터 기반 스킬 정의 (RON 로드 대상)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    pub id: SkillId,
    pub kind: SkillKind,
    pub range_tiles: u8,
    #[serde(default = "default_cast_delay_ms")]
    pub cast_delay_ms: u32,
    #[serde(default)]
    pub focus_time_ms: u32,
    pub delivery: DeliveryDef,
    #[serde(default)]
    pub effects: Vec<SkillEffectDef>,
}

/// 스킬 효과(초안): 구현 단계에서 커맨드/시스템으로 매핑될 수 있는 데이터 표현
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillEffectDef {
    Damage { amount: i32 },
    Heal { amount: i32 },
    ApplyBuff { buff_id: String, duration_ms: u32 },
    ExtraAttack { count: u8 },
}
