use rand::Rng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

use crate::behaviors::BehaviorType;

use super::seed::rng_for;

#[derive(Debug, Clone, Deserialize)]
pub struct BehaviorMixTemplate {
    // InvalidMessages 비율
    pub invalid_ratio_min: f64,
    pub invalid_ratio_max: f64,

    // 행동별 비율/파라미터 범위
    pub slow_ratio_min: f64,
    pub slow_ratio_max: f64,
    pub slow_delay_seconds_min: u64,
    pub slow_delay_seconds_max: u64,

    pub spiky_ratio_min: f64,
    pub spiky_ratio_max: f64,
    pub spiky_delay_ms_min: u64,
    pub spiky_delay_ms_max: u64,

    pub timeout_ratio_min: f64,
    pub timeout_ratio_max: f64,

    pub quit_before_ratio_min: f64,
    pub quit_before_ratio_max: f64,

    pub quit_during_loading_ratio_min: f64,
    pub quit_during_loading_ratio_max: f64,

    // InvalidMessages 모드 비율(세부 분포). 합이 1.0이 아니어도 내부에서 정규화
    pub invalid_mode_unknown_weight: f64,
    pub invalid_mode_missing_weight: f64,
    pub invalid_mode_early_loading_complete_weight: f64,
    // InvalidMessages 모드 비율(추가)
    pub invalid_mode_duplicate_enqueue_weight: f64,
    pub invalid_mode_wrong_session_id_weight: f64,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorMixConfig {
    pub slow_ratio: f64,
    pub slow_delay_seconds: u64,
    pub spiky_ratio: f64,
    pub spiky_delay_ms: u64,
    pub timeout_ratio: f64,
    pub quit_before_ratio: f64,
    pub quit_during_loading_ratio: f64,

    // InvalidMessages 모드 weight (정규화는 선택 시점에 수행)
    pub invalid_mode_unknown_weight: f64,
    pub invalid_mode_missing_weight: f64,
    pub invalid_mode_early_loading_complete_weight: f64,
    pub invalid_mode_duplicate_enqueue_weight: f64,
    pub invalid_mode_wrong_session_id_weight: f64,

    pub invalid_ratio: f64,
}

pub fn gen_behavior_mix(seed: u64, tpl: &BehaviorMixTemplate) -> BehaviorMixConfig {

    let mut r = rng_for(seed, "behavior_mix");
    let pick = |min: f64, max: f64, r: &mut ChaCha20Rng| -> f64 {
        if (min - max).abs() < f64::EPSILON { min } else { r.gen_range(min..=max) }
    };
    let pick_u = |min: u64, max: u64, r: &mut ChaCha20Rng| -> u64 {
        if min == max { min } else { r.gen_range(min..=max) }
    };

    BehaviorMixConfig {
        slow_ratio: pick(tpl.slow_ratio_min, tpl.slow_ratio_max, &mut r),
        slow_delay_seconds: pick_u(tpl.slow_delay_seconds_min, tpl.slow_delay_seconds_max, &mut r),
        spiky_ratio: pick(tpl.spiky_ratio_min, tpl.spiky_ratio_max, &mut r),
        spiky_delay_ms: pick_u(tpl.spiky_delay_ms_min, tpl.spiky_delay_ms_max, &mut r),
        timeout_ratio: pick(tpl.timeout_ratio_min, tpl.timeout_ratio_max, &mut r),
        quit_before_ratio: pick(tpl.quit_before_ratio_min, tpl.quit_before_ratio_max, &mut r),
        quit_during_loading_ratio: pick(
            tpl.quit_during_loading_ratio_min,
            tpl.quit_during_loading_ratio_max,
            &mut r,
        ),
        invalid_ratio: pick(tpl.invalid_ratio_min, tpl.invalid_ratio_max, &mut r),
        invalid_mode_unknown_weight: tpl.invalid_mode_unknown_weight,
        invalid_mode_missing_weight: tpl.invalid_mode_missing_weight,
        invalid_mode_early_loading_complete_weight: tpl.invalid_mode_early_loading_complete_weight,
        invalid_mode_duplicate_enqueue_weight: tpl.invalid_mode_duplicate_enqueue_weight,
        invalid_mode_wrong_session_id_weight: tpl.invalid_mode_wrong_session_id_weight,
    }
}

/// 플레이어 인덱스별로 Behavior를 결정적으로 할당합니다.
pub fn behavior_for_index(
    seed: u64,
    idx: u64,
    mix: &BehaviorMixConfig,
) -> BehaviorType {
    let mut r: ChaCha20Rng = rng_for(seed, &format!("behavior/{}", idx));
    let v: f64 = r.gen::<f64>();

    // 누적 구간 선택 방식
    let mut acc = 0.0;
    let choose = |v: f64, p: f64, acc: &mut f64| -> bool { *acc += p; v < *acc };

    if choose(v, mix.quit_before_ratio, &mut acc) {
        return BehaviorType::QuitBeforeMatch;
    }
    if choose(v, mix.quit_during_loading_ratio, &mut acc) {
        return BehaviorType::QuitDuringLoading;
    }
    if choose(v, mix.timeout_ratio, &mut acc) {
        return BehaviorType::TimeoutLoader;
    }
    if choose(v, mix.spiky_ratio, &mut acc) {
        return BehaviorType::SpikyLoader { delay_ms: mix.spiky_delay_ms };
    }
    if choose(v, mix.slow_ratio, &mut acc) {
        return BehaviorType::SlowLoader { delay_seconds: mix.slow_delay_seconds };
    }
    if choose(v, mix.invalid_ratio, &mut acc) {
        // Invalid 모드는 weights로 세분화
        let mut r2: ChaCha20Rng = rng_for(seed, &format!("behavior/{}/invalid_mode", idx));
        let total = mix.invalid_mode_unknown_weight
            + mix.invalid_mode_missing_weight
            + mix.invalid_mode_early_loading_complete_weight
            + mix.invalid_mode_duplicate_enqueue_weight
            + mix.invalid_mode_wrong_session_id_weight;
        let (w_u, w_m, w_e, w_d, w_w) = if total > 0.0 {
            (
                mix.invalid_mode_unknown_weight / total,
                mix.invalid_mode_missing_weight / total,
                mix.invalid_mode_early_loading_complete_weight / total,
                mix.invalid_mode_duplicate_enqueue_weight / total,
                mix.invalid_mode_wrong_session_id_weight / total,
            )
        } else {
            (1.0, 0.0, 0.0, 0.0, 0.0)
        };
        let v2: f64 = r2.gen::<f64>();
        let mut a2 = 0.0;
        let mut pick_mode = |p: f64| -> bool { a2 += p; v2 < a2 };
        let mode = if pick_mode(w_u) {
            crate::behaviors::invalid::InvalidMode::UnknownType
        } else if pick_mode(w_m) {
            crate::behaviors::invalid::InvalidMode::MissingField
        } else if pick_mode(w_e) {
            crate::behaviors::invalid::InvalidMode::EarlyLoadingComplete
        } else if pick_mode(w_d) {
            crate::behaviors::invalid::InvalidMode::DuplicateEnqueue
        } else if pick_mode(w_w) {
            crate::behaviors::invalid::InvalidMode::WrongSessionId
        } else {
            // 가드: 떠남
            crate::behaviors::invalid::InvalidMode::UnknownType
        };
        return BehaviorType::Invalid { mode };
    }
    BehaviorType::Normal
}
