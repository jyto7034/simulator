use serde::Deserialize;

use super::behavior_mix::BehaviorMixTemplate;

#[derive(Debug, Clone, Deserialize)]
pub struct SwarmTemplate {
    // 전체 테스트 지속 시간 (초)
    pub duration_secs: u64,

    // 총 플레이어 수 범위
    pub player_count_min: u64,
    pub player_count_max: u64,

    // 평균 초당 생성량(CPS) 범위
    pub cps_min: f64,
    pub cps_max: f64,

    // Behavior Mix 템플릿(예: Slow 비율/지연)
    pub behavior_mix: BehaviorMixTemplate,
    // 향후 확장: spikes, groups 등
}
