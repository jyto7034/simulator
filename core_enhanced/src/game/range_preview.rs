use crate::game::{
    ability::{
        DeliveryDef, SkillAreaAnchorSource, SkillCastSelection, SkillDef, SkillTarget,
        StepTargetingMode,
    },
    battle::{
        battlefield::BattlefieldLayout,
        core::types::RuntimeUnit,
        tile_range::{FacingDirection, TileRangePattern, TileRangePolicy, TileReachQuery},
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

#[cfg(test)]
pub(crate) fn range_previews_for_player_runtime_unit(
    unit: &RuntimeUnit,
    game_data: &GameDataBase,
    battlefield: &BattlefieldLayout,
    position: Position,
) -> LiveBattleRangePreviewsDto {
    range_previews_for_runtime_unit_at(
        unit,
        game_data,
        battlefield,
        position,
        unit.facing_direction,
    )
}

pub(crate) fn range_previews_for_runtime_unit_at(
    unit: &RuntimeUnit,
    game_data: &GameDataBase,
    battlefield: &BattlefieldLayout,
    position: Position,
    facing: Option<FacingDirection>,
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
        facing,
    );
    let active_skill = active_skill_preview(
        unit.skill_id.as_ref(),
        game_data,
        battlefield,
        position,
        facing,
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
            match resolved_valid_cells_for_policy(
                TileRangePolicy::WholeFieldValidTiles,
                None,
                battlefield,
                position,
                None,
            ) {
                Ok(cells) => LiveBattleBasicAttackRangePreviewDto {
                    available: true,
                    reason: None,
                    cells,
                    source,
                    source_id,
                },
                Err(reason) => unavailable_basic_attack(reason),
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
    match resolved_valid_cells_for_policy(
        TileRangePolicy::Pattern,
        Some(pattern),
        battlefield,
        position,
        Some(facing),
    ) {
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
    let Some((_target, reach, target_range, _)) = skill.cast_target_definition() else {
        return unavailable_active_skill(
            Some(skill.id.clone()),
            None,
            LiveBattleRangePreviewReason::MissingRangeData,
        );
    };
    let range_policy = match reach {
        crate::game::ability::SkillTargetReach::Pattern => TileRangePolicy::Pattern,
        crate::game::ability::SkillTargetReach::Global => TileRangePolicy::WholeFieldValidTiles,
    };

    match skill.cast_selection() {
        SkillCastSelection::SelfUnit => self_target_skill_preview(
            skill,
            range_policy,
            target_range,
            battlefield,
            position,
            facing,
        ),
        SkillCastSelection::AutomaticUnit => unavailable_active_skill(
            Some(skill.id.clone()),
            Some(LiveBattleActiveSkillTargetingKind::AutoUnit),
            LiveBattleRangePreviewReason::RequiresRuntimeTarget,
        ),
        SkillCastSelection::PlayerUnit => manual_target_skill_preview(
            skill,
            LiveBattleActiveSkillTargetingKind::ManualUnit,
            range_policy,
            target_range,
            battlefield,
            position,
            facing,
        ),
        SkillCastSelection::PlayerTile => manual_target_skill_preview(
            skill,
            LiveBattleActiveSkillTargetingKind::ManualTile,
            range_policy,
            target_range,
            battlefield,
            position,
            facing,
        ),
    }
}

fn self_target_skill_preview(
    skill: &SkillDef,
    range_policy: TileRangePolicy,
    target_range: Option<&TileRangePattern>,
    battlefield: &BattlefieldLayout,
    position: Position,
    facing: Option<FacingDirection>,
) -> LiveBattleActiveSkillRangePreviewDto {
    if TileReachQuery::try_new(
        battlefield,
        range_policy,
        target_range,
        Some(position),
        facing,
    )
    .is_err()
    {
        return unavailable_active_skill(
            Some(skill.id.clone()),
            Some(LiveBattleActiveSkillTargetingKind::SelfTarget),
            LiveBattleRangePreviewReason::MissingRangeData,
        );
    }
    if battlefield.valid_positions().is_empty() {
        return unavailable_active_skill(
            Some(skill.id.clone()),
            Some(LiveBattleActiveSkillTargetingKind::SelfTarget),
            LiveBattleRangePreviewReason::NoValidCells,
        );
    }
    match self_target_effect_preview_cells(skill, battlefield, position, facing) {
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

fn self_target_effect_preview_cells(
    skill: &SkillDef,
    battlefield: &BattlefieldLayout,
    caster_position: Position,
    caster_facing: Option<FacingDirection>,
) -> Result<Vec<Position>, LiveBattleRangePreviewReason> {
    let mut union = Vec::new();
    let mut has_spatial_step = false;

    for step in &skill.steps {
        let DeliveryDef::TileArea { footprint, area } = &step.delivery else {
            continue;
        };
        has_spatial_step = true;

        let anchor = match area.anchor {
            SkillAreaAnchorSource::Caster => caster_position,
            SkillAreaAnchorSource::CastTarget | SkillAreaAnchorSource::CastTargetStart
                if matches!(step.targeting, StepTargetingMode::ReuseCastTarget)
                    && matches!(
                        &step.target,
                        SkillTarget::SelfUnit | SkillTarget::CastTarget
                    ) =>
            {
                caster_position
            }
            SkillAreaAnchorSource::CastTarget
            | SkillAreaAnchorSource::CastTargetStart
            | SkillAreaAnchorSource::ImpactContext
            | SkillAreaAnchorSource::ImpactContextStart => {
                return Err(LiveBattleRangePreviewReason::RequiresRuntimeTarget);
            }
        };
        let facing = caster_facing
            .or_else(|| {
                footprint
                    .is_rotation_invariant()
                    .then_some(FacingDirection::Up)
            })
            .ok_or(LiveBattleRangePreviewReason::MissingRangeData)?;
        let query = TileReachQuery::try_new(
            battlefield,
            TileRangePolicy::Pattern,
            Some(footprint),
            Some(anchor),
            Some(facing),
        )
        .map_err(|_| LiveBattleRangePreviewReason::MissingRangeData)?;
        union.extend(query.materialize_valid_cells());
    }

    union.sort_by_key(|cell| (cell.y, cell.x));
    union.dedup();
    if has_spatial_step && union.is_empty() {
        return Err(LiveBattleRangePreviewReason::NoValidCells);
    }
    Ok(union)
}

fn manual_target_skill_preview(
    skill: &SkillDef,
    targeting_kind: LiveBattleActiveSkillTargetingKind,
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
            targeting_kind: Some(targeting_kind),
            requires_manual_target: true,
            cast_cells: cells,
            effect_preview_cells: Vec::new(),
            source: LiveBattleRangePreviewSource::SkillDefinition,
            source_id: Some(skill.id.to_string()),
        },
        Err(reason) => {
            unavailable_active_skill(Some(skill.id.clone()), Some(targeting_kind), reason)
        }
    }
}

fn resolved_valid_cells_for_policy(
    range_policy: TileRangePolicy,
    defense_tile_range: Option<&TileRangePattern>,
    battlefield: &BattlefieldLayout,
    position: Position,
    facing: Option<FacingDirection>,
) -> Result<Vec<Position>, LiveBattleRangePreviewReason> {
    let query = TileReachQuery::try_new(
        battlefield,
        range_policy,
        defense_tile_range,
        Some(position),
        facing,
    )
    .map_err(|_| LiveBattleRangePreviewReason::MissingRangeData)?;
    let cells = query.materialize_valid_cells();
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
        ability::{
            DeliveryDef, SkillAreaAnchorSource, SkillCastTargetingDef, SkillDef, SkillId,
            SkillStepDef, SkillTarget, SkillTileAreaDeliveryDef, SkillTileAreaOrigin,
        },
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
    fn range_projection_characterization_covers_policy_facing_clipping_and_membership() {
        let directional = TileRangePattern {
            include_anchor_tile: false,
            rows: vec![".XX".to_string(), ".@.".to_string(), "X..".to_string()],
        };
        let anchor = Position::new(2, 2);
        let mut field = open_field();
        field.add_static_obstacle(Position::new(3, 2)).unwrap();
        let facing_cases = [
            (
                FacingDirection::Up,
                vec![
                    Position::new(2, 1),
                    Position::new(3, 1),
                    Position::new(1, 3),
                ],
            ),
            (
                FacingDirection::Right,
                vec![
                    Position::new(1, 1),
                    Position::new(3, 2),
                    Position::new(3, 3),
                ],
            ),
            (
                FacingDirection::Down,
                vec![
                    Position::new(3, 1),
                    Position::new(1, 3),
                    Position::new(2, 3),
                ],
            ),
            (
                FacingDirection::Left,
                vec![
                    Position::new(1, 1),
                    Position::new(1, 2),
                    Position::new(3, 3),
                ],
            ),
        ];

        for (facing, expected) in facing_cases {
            let cells = resolved_valid_cells_for_policy(
                TileRangePolicy::Pattern,
                Some(&directional),
                &field,
                anchor,
                Some(facing),
            )
            .unwrap();
            assert_eq!(cells, expected, "facing {facing:?}");
            for y in -1..=5 {
                for x in -1..=5 {
                    let tile = Position::new(x, y);
                    let current_membership = field.is_valid_tile(tile)
                        && directional
                            .contains_target_tile(anchor, facing, tile)
                            .unwrap();
                    assert_eq!(
                        current_membership,
                        cells.contains(&tile),
                        "facing {facing:?}, tile {tile:?}"
                    );
                }
            }
        }

        let anchor_including = TileRangePattern {
            include_anchor_tile: true,
            rows: vec!["X.".to_string(), ".@".to_string()],
        };
        let sparse_field = BattlefieldLayout::new_with_valid_tiles(
            3,
            3,
            vec![
                Position::new(2, 2),
                Position::new(0, 0),
                Position::new(1, 1),
            ],
        );
        assert_eq!(
            resolved_valid_cells_for_policy(
                TileRangePolicy::Pattern,
                Some(&anchor_including),
                &sparse_field,
                Position::new(0, 0),
                Some(FacingDirection::Down),
            )
            .unwrap(),
            vec![Position::new(0, 0), Position::new(1, 1)]
        );
        assert_eq!(
            resolved_valid_cells_for_policy(
                TileRangePolicy::WholeFieldValidTiles,
                None,
                &sparse_field,
                Position::new(99, 99),
                None,
            )
            .unwrap(),
            vec![
                Position::new(0, 0),
                Position::new(1, 1),
                Position::new(2, 2),
            ]
        );
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
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: skill_id.clone(),
            name: "Manual Tile Skill".to_string(),
            kind: Default::default(),
            cast_targeting: SkillCastTargetingDef::Explicit {
                selection: crate::game::ability::SkillCastSelection::PlayerTile,
                target: SkillTarget::CastTarget,
                reach: Default::default(),
                target_range: Some(tile_range.clone()),
                air_capable: false,
            },
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "hit".to_string(),
                delay_ms: 0,
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
        assert!(preview.active_skill.effect_preview_cells.is_empty());
    }

    fn self_target_skill_with_steps(id: &str, steps: Vec<SkillStepDef>) -> SkillDef {
        SkillDef {
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from(id),
            name: id.to_string(),
            kind: Default::default(),
            cast_targeting: SkillCastTargetingDef::Explicit {
                selection: SkillCastSelection::SelfUnit,
                target: SkillTarget::SelfUnit,
                reach: Default::default(),
                target_range: Some(TileRangePattern {
                    include_anchor_tile: false,
                    rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
                }),
                air_capable: false,
            },
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps,
        }
    }

    fn tile_area_step(id: &str, footprint: TileRangePattern) -> SkillStepDef {
        SkillStepDef {
            id: id.to_string(),
            delay_ms: 0,
            target: SkillTarget::SelfUnit,
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::TileArea {
                footprint,
                area: SkillTileAreaDeliveryDef {
                    anchor: SkillAreaAnchorSource::Caster,
                    tile_origin: SkillTileAreaOrigin::Caster,
                    ..Default::default()
                },
            },
            effects: Vec::new(),
            presentation: Default::default(),
        }
    }

    #[test]
    fn effect_preview_uses_deterministic_union_of_every_spatial_step() {
        let first = TileRangePattern {
            include_anchor_tile: false,
            rows: vec![".XX".to_string(), ".@.".to_string(), "X..".to_string()],
        };
        let second = TileRangePattern {
            include_anchor_tile: true,
            rows: vec![".X.".to_string(), "X@X".to_string(), ".X.".to_string()],
        };
        let first_step = tile_area_step("first", first.clone());
        let mut second_step = tile_area_step("second", second.clone());
        let DeliveryDef::TileArea { area, .. } = &mut second_step.delivery else {
            unreachable!();
        };
        area.anchor = SkillAreaAnchorSource::CastTarget;
        area.tile_origin = SkillTileAreaOrigin::Anchor;
        let skill =
            self_target_skill_with_steps("effect_preview_union", vec![first_step, second_step]);
        let mut reverse_second = tile_area_step("second", second);
        let DeliveryDef::TileArea { area, .. } = &mut reverse_second.delivery else {
            unreachable!();
        };
        area.anchor = SkillAreaAnchorSource::CastTarget;
        area.tile_origin = SkillTileAreaOrigin::Anchor;
        let reverse_skill = self_target_skill_with_steps(
            "effect_preview_union_reverse",
            vec![reverse_second, tile_area_step("first", first)],
        );
        let game_data = GameDataBuilder::empty()
            .with_skills(SkillDatabase::new(vec![skill, reverse_skill]))
            .build();
        let mut profile = UnitCombatProfile::employee_default();
        profile.skill_id = Some(SkillId::from("effect_preview_union"));
        let mut reverse_profile = profile.clone();
        reverse_profile.skill_id = Some(SkillId::from("effect_preview_union_reverse"));
        let cases = [
            (
                FacingDirection::Up,
                vec![
                    Position::new(2, 1),
                    Position::new(3, 1),
                    Position::new(1, 2),
                    Position::new(2, 2),
                    Position::new(3, 2),
                    Position::new(1, 3),
                    Position::new(2, 3),
                ],
            ),
            (
                FacingDirection::Right,
                vec![
                    Position::new(1, 1),
                    Position::new(2, 1),
                    Position::new(1, 2),
                    Position::new(2, 2),
                    Position::new(3, 2),
                    Position::new(2, 3),
                    Position::new(3, 3),
                ],
            ),
            (
                FacingDirection::Down,
                vec![
                    Position::new(2, 1),
                    Position::new(3, 1),
                    Position::new(1, 2),
                    Position::new(2, 2),
                    Position::new(3, 2),
                    Position::new(1, 3),
                    Position::new(2, 3),
                ],
            ),
            (
                FacingDirection::Left,
                vec![
                    Position::new(1, 1),
                    Position::new(2, 1),
                    Position::new(1, 2),
                    Position::new(2, 2),
                    Position::new(3, 2),
                    Position::new(2, 3),
                    Position::new(3, 3),
                ],
            ),
        ];

        for (facing, expected) in cases {
            let preview = range_previews_for_combat_profile(
                &profile,
                &game_data,
                &open_field(),
                Position::new(2, 2),
                facing,
            );
            assert_eq!(
                preview.active_skill.effect_preview_cells, expected,
                "facing {facing:?}"
            );
            assert!(preview.active_skill.cast_cells.is_empty());
            let reverse_preview = range_previews_for_combat_profile(
                &reverse_profile,
                &game_data,
                &open_field(),
                Position::new(2, 2),
                facing,
            );
            assert_eq!(
                reverse_preview.active_skill.effect_preview_cells, expected,
                "reverse step order facing {facing:?}"
            );
        }
    }

    #[test]
    fn effect_preview_3x3_and_5x5_union_clips_at_board_corner() {
        let area_3x3 = TileRangePattern {
            include_anchor_tile: true,
            rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
        };
        let area_5x5 = TileRangePattern {
            include_anchor_tile: true,
            rows: vec![
                "XXXXX".to_string(),
                "XXXXX".to_string(),
                "XX@XX".to_string(),
                "XXXXX".to_string(),
                "XXXXX".to_string(),
            ],
        };
        let skill = self_target_skill_with_steps(
            "effect_preview_3x3_5x5",
            vec![
                tile_area_step("area_3x3", area_3x3),
                tile_area_step("area_5x5", area_5x5),
            ],
        );
        let game_data = GameDataBuilder::empty()
            .with_skills(SkillDatabase::new(vec![skill]))
            .build();
        let mut profile = UnitCombatProfile::employee_default();
        profile.skill_id = Some(SkillId::from("effect_preview_3x3_5x5"));

        let preview = range_previews_for_combat_profile(
            &profile,
            &game_data,
            &open_field(),
            Position::new(0, 0),
            FacingDirection::Left,
        );

        assert_eq!(
            preview.active_skill.effect_preview_cells,
            vec![
                Position::new(0, 0),
                Position::new(1, 0),
                Position::new(2, 0),
                Position::new(0, 1),
                Position::new(1, 1),
                Position::new(2, 1),
                Position::new(0, 2),
                Position::new(1, 2),
                Position::new(2, 2),
            ]
        );
    }

    #[test]
    fn effect_preview_nonspatial_steps_contribute_no_area_cells() {
        let skill = self_target_skill_with_steps(
            "effect_preview_instant_only",
            vec![
                SkillStepDef {
                    id: "instant".to_string(),
                    delay_ms: 0,
                    target: SkillTarget::SelfUnit,
                    targeting: Default::default(),
                    when: Default::default(),
                    repeat: Default::default(),
                    delivery: DeliveryDef::Instant,
                    effects: Vec::new(),
                    presentation: Default::default(),
                },
                SkillStepDef {
                    id: "projectile".to_string(),
                    delay_ms: 1,
                    target: SkillTarget::SelfUnit,
                    targeting: Default::default(),
                    when: Default::default(),
                    repeat: Default::default(),
                    delivery: DeliveryDef::Projectile {
                        speed_units_per_ms: 1,
                        hit_policy: Default::default(),
                        allow_targetless_cast: false,
                        max_range_tiles: None,
                        max_lifetime_ms: None,
                        max_kills: None,
                        max_pierces: None,
                        collision: Default::default(),
                    },
                    effects: Vec::new(),
                    presentation: Default::default(),
                },
            ],
        );
        let game_data = GameDataBuilder::empty()
            .with_skills(SkillDatabase::new(vec![skill]))
            .build();
        let mut profile = UnitCombatProfile::employee_default();
        profile.skill_id = Some(SkillId::from("effect_preview_instant_only"));

        let preview = range_previews_for_combat_profile(
            &profile,
            &game_data,
            &open_field(),
            Position::new(2, 2),
            FacingDirection::Up,
        );

        assert!(preview.active_skill.available);
        assert!(preview.active_skill.effect_preview_cells.is_empty());
    }

    #[test]
    fn effect_preview_runtime_dependent_anchor_is_unavailable() {
        let retarget_projectile = SkillStepDef {
            id: "retarget_projectile".to_string(),
            delay_ms: 0,
            target: SkillTarget::EnemySingle {
                rule: Default::default(),
            },
            targeting: crate::game::ability::StepTargetingMode::RetargetOnStep,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 1,
                hit_policy: Default::default(),
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            },
            effects: Vec::new(),
            presentation: Default::default(),
        };
        let mut impact_area = tile_area_step(
            "impact_area",
            TileRangePattern {
                include_anchor_tile: true,
                rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
            },
        );
        let DeliveryDef::TileArea { area, .. } = &mut impact_area.delivery else {
            unreachable!();
        };
        area.anchor = SkillAreaAnchorSource::ImpactContext;
        area.tile_origin = SkillTileAreaOrigin::Anchor;
        let skill = self_target_skill_with_steps(
            "effect_preview_runtime_anchor",
            vec![retarget_projectile, impact_area],
        );
        let game_data = GameDataBuilder::empty()
            .with_skills(SkillDatabase::new(vec![skill]))
            .build();
        let mut profile = UnitCombatProfile::employee_default();
        profile.skill_id = Some(SkillId::from("effect_preview_runtime_anchor"));

        let preview = range_previews_for_combat_profile(
            &profile,
            &game_data,
            &open_field(),
            Position::new(2, 2),
            FacingDirection::Up,
        );

        assert!(!preview.active_skill.available);
        assert_eq!(
            preview.active_skill.reason,
            Some(LiveBattleRangePreviewReason::RequiresRuntimeTarget)
        );
        assert!(preview.active_skill.effect_preview_cells.is_empty());
    }

    #[test]
    fn effect_preview_missing_cast_data_preserves_unavailable_reason() {
        let mut skill = self_target_skill_with_steps(
            "effect_preview_missing_cast_data",
            vec![SkillStepDef {
                id: "instant".to_string(),
                delay_ms: 0,
                target: SkillTarget::SelfUnit,
                targeting: Default::default(),
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: Vec::new(),
                presentation: Default::default(),
            }],
        );
        let SkillCastTargetingDef::Explicit { target_range, .. } = &mut skill.cast_targeting;
        *target_range = None;
        let game_data = GameDataBuilder::empty()
            .with_skills(SkillDatabase::new(vec![skill]))
            .build();
        let mut profile = UnitCombatProfile::employee_default();
        profile.skill_id = Some(SkillId::from("effect_preview_missing_cast_data"));

        let preview = range_previews_for_combat_profile(
            &profile,
            &game_data,
            &open_field(),
            Position::new(2, 2),
            FacingDirection::Up,
        );

        assert!(!preview.active_skill.available);
        assert_eq!(
            preview.active_skill.reason,
            Some(LiveBattleRangePreviewReason::MissingRangeData)
        );
    }

    #[test]
    fn weapon_profile_range_overrides_fallback_basic_attack_preview() {
        let mut profile = UnitCombatProfile::employee_default();
        let mut weapon = WeaponCombatProfile::default();
        weapon.defense_tile_range = TileRangePattern {
            include_anchor_tile: false,
            rows: vec![".XX".to_string(), ".@.".to_string(), "...".to_string()],
        };
        profile.apply_weapon_profile(&weapon).unwrap();
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
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
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
