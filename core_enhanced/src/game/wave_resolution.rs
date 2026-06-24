use crate::game::{
    data::{
        corroded_wave_data::{CorrodedWavePreset, CorrodedWaveRoleWeight},
        pve_data::{PveWaveData, PveWaveEnemyData, PveWaveSource},
        GameDataBase,
    },
    determinism,
};

const CORRODED_WAVE_SEED_NS: u64 = 0x434F_5257_4156_4547; // "CORWAVEG"

pub fn resolve_pve_wave_enemy_data(
    wave: &PveWaveData,
    game_data: &GameDataBase,
    preview_seed: u64,
    wave_index: usize,
) -> Vec<PveWaveEnemyData> {
    match &wave.source {
        PveWaveSource::Manual(enemies) => enemies.clone(),
        PveWaveSource::GeneratedCorroded {
            preset_id,
            budget_override,
            seed_salt,
        } => {
            let preset = game_data
                .corroded_wave_data
                .get_by_id(preset_id)
                .unwrap_or_else(|| {
                    panic!(
                        "generated corroded wave '{}' references missing preset '{}'",
                        wave.id, preset_id
                    )
                });
            generate_corroded_wave(
                preset,
                *budget_override,
                *seed_salt,
                preview_seed,
                wave_index,
            )
        }
    }
}

fn generate_corroded_wave(
    preset: &CorrodedWavePreset,
    budget_override: Option<u32>,
    seed_salt: Option<u64>,
    preview_seed: u64,
    wave_index: usize,
) -> Vec<PveWaveEnemyData> {
    let mut remaining_budget = budget_override.unwrap_or(preset.budget);
    let mut counts = vec![0_u32; preset.role_mix.len()];

    for (index, role) in preset.role_mix.iter().enumerate() {
        let min_count = role.min_count;
        counts[index] = min_count;
        remaining_budget = remaining_budget.saturating_sub(role.cost.saturating_mul(min_count));
    }

    let target_count = generated_corroded_target_count(preset, preview_seed, seed_salt, wave_index);
    while counts.iter().sum::<u32>() < target_count {
        let Some(index) = choose_corroded_role_index(
            &preset.role_mix,
            &counts,
            remaining_budget,
            preview_seed,
            seed_salt,
            wave_index,
            counts.iter().sum::<u32>(),
        ) else {
            break;
        };
        counts[index] += 1;
        remaining_budget = remaining_budget.saturating_sub(preset.role_mix[index].cost);
    }

    preset
        .role_mix
        .iter()
        .zip(counts)
        .filter(|(_, count)| *count > 0)
        .map(|(role, count)| role.to_enemy_data(count))
        .collect()
}

fn generated_corroded_target_count(
    preset: &CorrodedWavePreset,
    preview_seed: u64,
    seed_salt: Option<u64>,
    wave_index: usize,
) -> u32 {
    let min = preset.count_range.min;
    let max = preset.count_range.max.max(min);
    let span = max - min + 1;
    min + (generated_corroded_seed(preview_seed, seed_salt, wave_index, 0) % u64::from(span)) as u32
}

fn choose_corroded_role_index(
    roles: &[CorrodedWaveRoleWeight],
    counts: &[u32],
    remaining_budget: u32,
    preview_seed: u64,
    seed_salt: Option<u64>,
    wave_index: usize,
    pick_index: u32,
) -> Option<usize> {
    let candidates = roles
        .iter()
        .enumerate()
        .filter(|(index, role)| {
            remaining_budget >= role.cost
                && role
                    .max_count
                    .is_none_or(|max_count| counts[*index] < max_count)
        })
        .collect::<Vec<_>>();
    let total_weight = candidates
        .iter()
        .map(|(_, role)| u64::from(role.weight))
        .sum::<u64>();
    if total_weight == 0 {
        return None;
    }

    let mut roll = generated_corroded_seed(
        preview_seed,
        seed_salt,
        wave_index,
        u64::from(pick_index) + 1,
    ) % total_weight;
    for (index, role) in candidates {
        let weight = u64::from(role.weight);
        if roll < weight {
            return Some(index);
        }
        roll -= weight;
    }

    None
}

fn generated_corroded_seed(
    preview_seed: u64,
    seed_salt: Option<u64>,
    wave_index: usize,
    step: u64,
) -> u64 {
    let mut seed = determinism::seed_with_namespace(preview_seed, CORRODED_WAVE_SEED_NS);
    seed = determinism::seed_with_namespace(seed, seed_salt.unwrap_or(0));
    seed = determinism::seed_with_namespace(seed, wave_index as u64);
    determinism::seed_with_namespace(seed, step)
}
