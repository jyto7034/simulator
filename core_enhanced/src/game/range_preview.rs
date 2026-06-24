use crate::game::{
    ability::{SkillDef, SkillTarget},
    battle::{
        battlefield::BattlefieldLayout,
        core::types::RuntimeUnit,
        tile_range::{FacingDirection, TileRangePattern, TileRangePolicy},
        types::{effective_basic_attack_tile_range, UnitCombatProfile},
    },
    behavior::{
        LiveBattleActiveSkillRangePreviewDto, LiveBattleActiveSkillTargetingKind,
        LiveBattleBasicAttackRangePreviewDto, LiveBattleRangePreviewReason,
        LiveBattleRangePreviewSource, LiveBattleRangePreviewsDto,
    },
    data::GameDataBase,
    resources::Position,
};

#[cfg(test)]
fn default_basic_attack_tile_range() -> TileRangePattern {
    crate::game::battle::types::default_basic_attack_tile_range_for_role(
        crate::game::data::equipment_data::WeaponRangeRole::Melee,
    )
}

pub(crate) fn range_previews_for_runtime_unit(
    unit: &RuntimeUnit,
    game_data: &GameDataBase,
    battlefield: &BattlefieldLayout,
    position: Position,
) -> LiveBattleRangePreviewsDto {
    let basic_source = match unit.basic_attack.defense_tile_range.as_ref() {
        Some(_) => LiveBattleRangePreviewSource::WeaponProfile,
        _ => LiveBattleRangePreviewSource::FallbackBasicAttack,
    };
    let basic_attack = basic_attack_preview(
        &unit.basic_attack,
        basic_source,
        None,
        battlefield,
        position,
        unit.facing_direction,
    );
    let active_skill = active_skill_preview(
        unit.skill_id.as_ref(),
        game_data,
        battlefield,
        position,
        unit.facing_direction,
    );
    LiveBattleRangePreviewsDto {
        basic_attack,
        active_skill,
    }
}

pub(crate) fn range_previews_for_combat_profile(
    profile: &UnitCombatProfile,
    game_data: &GameDataBase,
    battlefield: &BattlefieldLayout,
    position: Position,
    facing: FacingDirection,
) -> LiveBattleRangePreviewsDto {
    let source = if profile.weapon_profile.is_some() {
        LiveBattleRangePreviewSource::WeaponProfile
    } else {
        LiveBattleRangePreviewSource::FallbackBasicAttack
    };
    let basic_attack = basic_attack_preview(
        &profile.basic_attack,
        source,
        None,
        battlefield,
        position,
        Some(facing),
    );
    let active_skill = active_skill_preview(
        profile.skill_id.as_ref(),
        game_data,
        battlefield,
        position,
        Some(facing),
    );
    LiveBattleRangePreviewsDto {
        basic_attack,
        active_skill,
    }
}

fn basic_attack_preview(
    basic_attack: &crate::game::data::abnormality_data::BasicAttackDef,
    source: LiveBattleRangePreviewSource,
    source_id: Option<String>,
    battlefield: &BattlefieldLayout,
    position: Position,
    facing: Option<FacingDirection>,
) -> LiveBattleBasicAttackRangePreviewDto {
    match basic_attack.range_policy {
        TileRangePolicy::Pattern => {
            let pattern = effective_basic_attack_tile_range(basic_attack);
            let Some(facing) = facing else {
                return unavailable_basic_attack(LiveBattleRangePreviewReason::MissingRangeData);
            };
            basic_attack_preview_from_pattern(
                &pattern,
                source,
                source_id,
                battlefield,
                position,
                facing,
            )
        }
        TileRangePolicy::WholeFieldValidTiles => {
            let cells = battlefield.valid_positions();
            if cells.is_empty() {
                return unavailable_basic_attack(LiveBattleRangePreviewReason::NoValidCells);
            }
            LiveBattleBasicAttackRangePreviewDto {
                available: true,
                reason: None,
                cells,
                source,
                source_id,
            }
        }
    }
}

fn basic_attack_preview_from_pattern(
    pattern: &TileRangePattern,
    source: LiveBattleRangePreviewSource,
    source_id: Option<String>,
    battlefield: &BattlefieldLayout,
    position: Position,
    facing: FacingDirection,
) -> LiveBattleBasicAttackRangePreviewDto {
    match resolved_valid_cells(pattern, battlefield, position, facing) {
        Ok(cells) => LiveBattleBasicAttackRangePreviewDto {
            available: true,
            reason: None,
            cells,
            source,
            source_id,
        },
        Err(reason) => LiveBattleBasicAttackRangePreviewDto {
            available: false,
            reason: Some(reason),
            cells: Vec::new(),
            source: LiveBattleRangePreviewSource::Unavailable,
            source_id: None,
        },
    }
}

fn active_skill_preview(
    skill_id: Option<&crate::game::ability::SkillId>,
    game_data: &GameDataBase,
    battlefield: &BattlefieldLayout,
    position: Position,
    facing: Option<FacingDirection>,
) -> LiveBattleActiveSkillRangePreviewDto {
    let Some(skill_id) = skill_id else {
        return unavailable_active_skill(None, None, LiveBattleRangePreviewReason::NoSkill);
    };
    let Some(skill) = game_data.skill_data.get_by_id(skill_id.as_ref()) else {
        return unavailable_active_skill(
            Some(skill_id.clone()),
            None,
            LiveBattleRangePreviewReason::MissingRangeData,
        );
    };
    let Some((target, range_policy, defense_tile_range, _)) = skill.cast_target_definition() else {
        return unavailable_active_skill(
            Some(skill.id.clone()),
            None,
            LiveBattleRangePreviewReason::MissingRangeData,
        );
    };

    match target {
        SkillTarget::SelfUnit => self_target_skill_preview(
            skill,
            range_policy,
            defense_tile_range,
            battlefield,
            position,
            facing,
        ),
        SkillTarget::EnemySingle { .. } => unavailable_active_skill(
            Some(skill.id.clone()),
            Some(LiveBattleActiveSkillTargetingKind::AutoUnit),
            LiveBattleRangePreviewReason::RequiresRuntimeTarget,
        ),
        SkillTarget::CastTarget => manual_tile_skill_preview(
            skill,
            range_policy,
            defense_tile_range,
            battlefield,
            position,
            facing,
        ),
    }
}

fn self_target_skill_preview(
    skill: &SkillDef,
    range_policy: TileRangePolicy,
    defense_tile_range: Option<&TileRangePattern>,
    battlefield: &BattlefieldLayout,
    position: Position,
    facing: Option<FacingDirection>,
) -> LiveBattleActiveSkillRangePreviewDto {
    match resolved_valid_cells_for_policy(
        range_policy,
        defense_tile_range,
        battlefield,
        position,
        facing,
    ) {
        Ok(cells) => LiveBattleActiveSkillRangePreviewDto {
            available: true,
            reason: None,
            skill_id: Some(skill.id.clone()),
            targeting_kind: Some(LiveBattleActiveSkillTargetingKind::SelfTarget),
            requires_manual_target: false,
            cast_cells: Vec::new(),
            effect_preview_cells: cells,
            source: LiveBattleRangePreviewSource::SkillDefinition,
            source_id: Some(skill.id.to_string()),
        },
        Err(reason) => unavailable_active_skill(
            Some(skill.id.clone()),
            Some(LiveBattleActiveSkillTargetingKind::SelfTarget),
            reason,
        ),
    }
}

fn manual_tile_skill_preview(
    skill: &SkillDef,
    range_policy: TileRangePolicy,
    defense_tile_range: Option<&TileRangePattern>,
    battlefield: &BattlefieldLayout,
    position: Position,
    facing: Option<FacingDirection>,
) -> LiveBattleActiveSkillRangePreviewDto {
    match resolved_valid_cells_for_policy(
        range_policy,
        defense_tile_range,
        battlefield,
        position,
        facing,
    ) {
        Ok(cells) => LiveBattleActiveSkillRangePreviewDto {
            available: true,
            reason: None,
            skill_id: Some(skill.id.clone()),
            targeting_kind: Some(LiveBattleActiveSkillTargetingKind::ManualTile),
            requires_manual_target: true,
            cast_cells: cells,
            effect_preview_cells: Vec::new(),
            source: LiveBattleRangePreviewSource::SkillDefinition,
            source_id: Some(skill.id.to_string()),
        },
        Err(reason) => unavailable_active_skill(
            Some(skill.id.clone()),
            Some(LiveBattleActiveSkillTargetingKind::ManualTile),
            reason,
        ),
    }
}

fn resolved_valid_cells_for_policy(
    range_policy: TileRangePolicy,
    defense_tile_range: Option<&TileRangePattern>,
    battlefield: &BattlefieldLayout,
    position: Position,
    facing: Option<FacingDirection>,
) -> Result<Vec<Position>, LiveBattleRangePreviewReason> {
    match range_policy {
        TileRangePolicy::Pattern => {
            let pattern =
                defense_tile_range.ok_or(LiveBattleRangePreviewReason::MissingRangeData)?;
            let facing = facing.ok_or(LiveBattleRangePreviewReason::MissingRangeData)?;
            resolved_valid_cells(pattern, battlefield, position, facing)
        }
        TileRangePolicy::WholeFieldValidTiles => {
            let cells = battlefield.valid_positions();
            if cells.is_empty() {
                Err(LiveBattleRangePreviewReason::NoValidCells)
            } else {
                Ok(cells)
            }
        }
    }
}

fn resolved_valid_cells(
    pattern: &TileRangePattern,
    battlefield: &BattlefieldLayout,
    position: Position,
    facing: FacingDirection,
) -> Result<Vec<Position>, LiveBattleRangePreviewReason> {
    let cells = pattern
        .affected_tiles(position, facing)
        .map_err(|_| LiveBattleRangePreviewReason::MissingRangeData)?
        .into_iter()
        .filter(|cell| battlefield.is_valid_tile(*cell))
        .collect::<Vec<_>>();
    if cells.is_empty() {
        return Err(LiveBattleRangePreviewReason::NoValidCells);
    }
    Ok(cells)
}

fn unavailable_basic_attack(
    reason: LiveBattleRangePreviewReason,
) -> LiveBattleBasicAttackRangePreviewDto {
    LiveBattleBasicAttackRangePreviewDto {
        available: false,
        reason: Some(reason),
        cells: Vec::new(),
        source: LiveBattleRangePreviewSource::Unavailable,
        source_id: None,
    }
}

fn unavailable_active_skill(
    skill_id: Option<crate::game::ability::SkillId>,
    targeting_kind: Option<LiveBattleActiveSkillTargetingKind>,
    reason: LiveBattleRangePreviewReason,
) -> LiveBattleActiveSkillRangePreviewDto {
    LiveBattleActiveSkillRangePreviewDto {
        available: false,
        reason: Some(reason),
        skill_id,
        targeting_kind,
        requires_manual_target: matches!(
            targeting_kind,
            Some(LiveBattleActiveSkillTargetingKind::ManualTile)
        ),
        cast_cells: Vec::new(),
        effect_preview_cells: Vec::new(),
        source: LiveBattleRangePreviewSource::Unavailable,
        source_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        ability::{SkillCastTargetingDef, SkillDef, SkillId, SkillStepDef, SkillTarget},
        battle::types::UnitCombatProfile,
        data::{equipment_data::WeaponCombatProfile, skill_data::SkillDatabase, GameDataBuilder},
    };

    fn open_field() -> BattlefieldLayout {
        BattlefieldLayout::new_with_valid_tiles(
            5,
            5,
            (0..5)
                .flat_map(|y| (0..5).map(move |x| Position::new(x, y)))
                .collect(),
        )
    }

    #[test]
    fn default_basic_attack_preview_includes_own_tile_and_forward_tile() {
        let profile = UnitCombatProfile::employee_default();
        let game_data = GameDataBuilder::empty().build();

        let preview = range_previews_for_combat_profile(
            &profile,
            &game_data,
            &open_field(),
            Position::new(2, 2),
            FacingDirection::Right,
        );

        assert!(preview.basic_attack.available);
        assert_eq!(preview.basic_attack.reason, None);
        assert_eq!(
            preview.basic_attack.cells,
            vec![Position::new(2, 2), Position::new(3, 2)]
        );
        assert_eq!(
            preview.basic_attack.source,
            LiveBattleRangePreviewSource::FallbackBasicAttack
        );
        assert_eq!(
            preview.active_skill.reason,
            Some(LiveBattleRangePreviewReason::NoSkill)
        );
    }

    #[test]
    fn basic_attack_preview_rotates_by_facing() {
        let profile = UnitCombatProfile::employee_default();
        let game_data = GameDataBuilder::empty().build();
        let field = open_field();

        let up = range_previews_for_combat_profile(
            &profile,
            &game_data,
            &field,
            Position::new(2, 2),
            FacingDirection::Up,
        );
        let left = range_previews_for_combat_profile(
            &profile,
            &game_data,
            &field,
            Position::new(2, 2),
            FacingDirection::Left,
        );

        assert_eq!(
            up.basic_attack.cells,
            vec![Position::new(2, 1), Position::new(2, 2)]
        );
        assert_eq!(
            left.basic_attack.cells,
            vec![Position::new(1, 2), Position::new(2, 2)]
        );
    }

    #[test]
    fn basic_attack_preview_filters_out_of_bounds_cells() {
        let profile = UnitCombatProfile::employee_default();
        let game_data = GameDataBuilder::empty().build();

        let preview = range_previews_for_combat_profile(
            &profile,
            &game_data,
            &open_field(),
            Position::new(0, 0),
            FacingDirection::Left,
        );

        assert!(preview.basic_attack.available);
        assert_eq!(preview.basic_attack.cells, vec![Position::new(0, 0)]);
    }

    #[test]
    fn basic_attack_preview_keeps_static_obstacle_cells() {
        let profile = UnitCombatProfile::employee_default();
        let game_data = GameDataBuilder::empty().build();
        let mut field = open_field();
        let obstacle = Position::new(3, 2);
        field.add_static_obstacle(obstacle).unwrap();

        let preview = range_previews_for_combat_profile(
            &profile,
            &game_data,
            &field,
            Position::new(2, 2),
            FacingDirection::Right,
        );

        assert!(preview.basic_attack.available);
        assert_eq!(
            preview.basic_attack.cells,
            vec![Position::new(2, 2), obstacle]
        );
    }

    #[test]
    fn active_skill_preview_clips_pattern_to_valid_tiles() {
        let skill_id = SkillId::from("manual_tile_skill");
        let tile_range = TileRangePattern {
            include_anchor_tile: false,
            rows: vec!["X@X".to_string()],
        };
        let skill = SkillDef {
            id: skill_id.clone(),
            name: "Manual Tile Skill".to_string(),
            kind: Default::default(),
            cast_targeting: SkillCastTargetingDef::Explicit {
                target: SkillTarget::CastTarget,
                range_policy: Default::default(),
                defense_tile_range: Some(tile_range.clone()),
                air_capable: false,
            },
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "hit".to_string(),
                delay_ms: 0,
                range_policy: Default::default(),
                defense_tile_range: Some(tile_range),
                air_capable: false,
                target: SkillTarget::CastTarget,
                targeting: Default::default(),
                when: Default::default(),
                repeat: Default::default(),
                delivery: Default::default(),
                effects: Vec::new(),
                presentation: Default::default(),
            }],
        };
        let game_data = GameDataBuilder::empty()
            .with_skills(SkillDatabase::new(vec![skill]))
            .build();
        let mut profile = UnitCombatProfile::employee_default();
        profile.skill_id = Some(skill_id);
        let field = BattlefieldLayout::new_with_valid_tiles(
            4,
            3,
            vec![Position::new(1, 1), Position::new(2, 1)],
        );

        let preview = range_previews_for_combat_profile(
            &profile,
            &game_data,
            &field,
            Position::new(1, 1),
            FacingDirection::Up,
        );

        assert!(preview.active_skill.available);
        assert_eq!(preview.active_skill.cast_cells, vec![Position::new(2, 1)]);
    }

    #[test]
    fn weapon_profile_range_overrides_fallback_basic_attack_preview() {
        let mut profile = UnitCombatProfile::employee_default();
        let mut weapon = WeaponCombatProfile::default();
        weapon.defense_tile_range = TileRangePattern {
            include_anchor_tile: false,
            rows: vec![".XX".to_string(), ".@.".to_string(), "...".to_string()],
        };
        profile.apply_weapon_profile(&weapon);
        let game_data = GameDataBuilder::empty().build();

        let preview = range_previews_for_combat_profile(
            &profile,
            &game_data,
            &open_field(),
            Position::new(2, 2),
            FacingDirection::Up,
        );

        assert_eq!(
            preview.basic_attack.source,
            LiveBattleRangePreviewSource::WeaponProfile
        );
        assert_eq!(
            preview.basic_attack.cells,
            vec![Position::new(2, 1), Position::new(3, 1)]
        );
    }

    #[test]
    fn auto_unit_active_skill_preview_is_separate_and_requires_runtime_target() {
        let skill_id = SkillId::from("auto_unit_skill");
        let skill = SkillDef {
            id: skill_id.clone(),
            name: "Auto Unit Skill".to_string(),
            kind: Default::default(),
            cast_targeting: SkillCastTargetingDef::explicit(
                SkillTarget::EnemySingle {
                    rule: Default::default(),
                },
                Default::default(),
                Some(default_basic_attack_tile_range()),
                false,
            ),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "hit".to_string(),
                delay_ms: 0,
                range_policy: Default::default(),
                defense_tile_range: Some(default_basic_attack_tile_range()),
                air_capable: false,
                target: SkillTarget::EnemySingle {
                    rule: Default::default(),
                },
                targeting: Default::default(),
                when: Default::default(),
                repeat: Default::default(),
                delivery: Default::default(),
                effects: Vec::new(),
                presentation: Default::default(),
            }],
        };
        let game_data = GameDataBuilder::empty()
            .with_skills(SkillDatabase::new(vec![skill]))
            .build();
        let mut profile = UnitCombatProfile::employee_default();
        profile.skill_id = Some(skill_id.clone());

        let preview = range_previews_for_combat_profile(
            &profile,
            &game_data,
            &open_field(),
            Position::new(2, 2),
            FacingDirection::Right,
        );

        assert!(preview.basic_attack.available);
        assert!(!preview.active_skill.available);
        assert_eq!(preview.active_skill.skill_id, Some(skill_id));
        assert_eq!(
            preview.active_skill.targeting_kind,
            Some(LiveBattleActiveSkillTargetingKind::AutoUnit)
        );
        assert_eq!(
            preview.active_skill.reason,
            Some(LiveBattleRangePreviewReason::RequiresRuntimeTarget)
        );
        assert!(preview.active_skill.cast_cells.is_empty());
        assert!(preview.active_skill.effect_preview_cells.is_empty());
    }
}
