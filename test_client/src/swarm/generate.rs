use rand::Rng;
use rand_chacha::ChaCha20Rng;

use super::{behavior_mix::gen_behavior_mix, concrete::ConcreteConfig, template::SwarmTemplate};
use crate::swarm::seed::rng_for;

pub fn generate(global_seed: u64, tpl: &SwarmTemplate) -> ConcreteConfig {
    let mut r_counts: ChaCha20Rng = rng_for(global_seed, "counts");
    let mut r_cps: ChaCha20Rng = rng_for(global_seed, "cps");

    let player_count = if tpl.player_count_min == tpl.player_count_max {
        tpl.player_count_min
    } else {
        r_counts.gen_range(tpl.player_count_min..=tpl.player_count_max)
    };

    let cps = if (tpl.cps_min - tpl.cps_max).abs() < f64::EPSILON {
        tpl.cps_min
    } else {
        r_cps.gen_range(tpl.cps_min..=tpl.cps_max)
    };

    let behavior_mix = gen_behavior_mix(global_seed, &tpl.behavior_mix);

    ConcreteConfig {
        duration_secs: tpl.duration_secs,
        player_count,
        cps,
        behavior_mix,
    }
}
