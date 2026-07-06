use std::collections::HashSet;

use crate::game::{
    combat_setup::balance::{is_fast_breakthrough_speed, is_high_defense, is_high_magic_resist},
    data::GameDataBase,
    determinism,
};

use super::{
    EnemyKind, SpawnWave, SpawnWaveEnemyEntry, ThreatWarning, ThreatWarningSource,
    ThreatWarningStatus, ThreatWarningTag,
};

pub(super) const THREAT_RUMOR_CHANCE_PERCENT: u8 = 20;
const THREAT_RUMOR_SEED_NS: u64 = 0x5448_5254_5255_4D52; // "THRTRUMR"

#[derive(Debug, Clone, Copy)]
struct EnemyThreatStats {
    defense: i32,
    magic_resist: i32,
    speed_units_per_ms: u32,
    mobility_kind: crate::game::battle::types::MobilityKind,
}

pub(super) fn threat_warnings_for(
    spawn_waves: &[SpawnWave],
    game_data: &GameDataBase,
    preview_seed: u64,
) -> Vec<ThreatWarning> {
    let tags = required_briefing_warning_tags_for_spawn_waves(spawn_waves, game_data);
    let mut warnings = detectable_threat_warning_tags()
        .iter()
        .copied()
        .filter(|tag| tags.contains(tag))
        .map(|tag| ThreatWarning {
            tag,
            status: ThreatWarningStatus::Unverified,
            source: ThreatWarningSource::Briefing,
        })
        .collect::<Vec<_>>();

    if let Some(tag) = rumor_threat_warning_tag(preview_seed, &tags) {
        warnings.push(ThreatWarning {
            tag,
            status: ThreatWarningStatus::Unverified,
            source: ThreatWarningSource::Rumor,
        });
    }

    warnings
}

pub fn required_briefing_warning_tags_for_spawn_waves(
    spawn_waves: &[SpawnWave],
    game_data: &GameDataBase,
) -> HashSet<ThreatWarningTag> {
    let mut tags = HashSet::new();
    for entry in spawn_waves
        .iter()
        .flat_map(|wave| wave.enemy_entries.iter())
    {
        let Some(stats) = threat_stats_for_enemy(entry, game_data) else {
            continue;
        };
        if is_high_defense(stats.defense) {
            tags.insert(ThreatWarningTag::ArmoredEnemyPossible);
        }
        if is_high_magic_resist(stats.magic_resist) {
            tags.insert(ThreatWarningTag::HighMagicResistEnemyPossible);
        }
        if is_fast_breakthrough_speed(stats.speed_units_per_ms) {
            tags.insert(ThreatWarningTag::FastBreakthroughEnemyPossible);
        }
        if stats.mobility_kind.is_airborne() {
            tags.insert(ThreatWarningTag::AirEnemyPossible);
        }
    }

    tags
}

pub(super) fn detectable_threat_warning_tags() -> &'static [ThreatWarningTag] {
    &[
        ThreatWarningTag::ArmoredEnemyPossible,
        ThreatWarningTag::HighMagicResistEnemyPossible,
        ThreatWarningTag::AirEnemyPossible,
        ThreatWarningTag::FastBreakthroughEnemyPossible,
    ]
}

pub(super) fn rumor_threat_warning_tag(
    preview_seed: u64,
    real_tags: &HashSet<ThreatWarningTag>,
) -> Option<ThreatWarningTag> {
    let seed = determinism::seed_with_namespace(preview_seed, THREAT_RUMOR_SEED_NS);
    if (seed % 100) as u8 >= THREAT_RUMOR_CHANCE_PERCENT {
        return None;
    }

    let candidates = detectable_threat_warning_tags()
        .iter()
        .copied()
        .filter(|tag| !real_tags.contains(tag))
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return None;
    }

    let pick_seed = determinism::seed_with_namespace(seed, 1);
    Some(candidates[(pick_seed as usize) % candidates.len()])
}

fn threat_stats_for_enemy(
    entry: &SpawnWaveEnemyEntry,
    game_data: &GameDataBase,
) -> Option<EnemyThreatStats> {
    match entry.kind {
        EnemyKind::CorrodedEmployee => {
            let profile_id = entry.profile_id.as_deref()?;
            game_data
                .corroded_employee_data
                .get_by_id(profile_id)
                .map(|profile| EnemyThreatStats {
                    defense: profile.defense,
                    magic_resist: profile.magic_resist,
                    speed_units_per_ms: profile.movement.speed_units_per_ms,
                    mobility_kind: Default::default(),
                })
        }
        EnemyKind::Abnormality | EnemyKind::FacilityEntity => game_data
            .abnormality_data
            .get_by_id(&entry.abnormality_id)
            .map(|abnormality| EnemyThreatStats {
                defense: abnormality.defense,
                magic_resist: abnormality.magic_resist,
                speed_units_per_ms: abnormality.movement.speed_units_per_ms,
                mobility_kind: abnormality.mobility_kind,
            }),
    }
}
