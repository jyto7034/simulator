use std::collections::{HashMap, HashSet};

use crate::game::resources::Position;

use super::{
    confidence_for_hidden, unique_positions, unique_tiles, BattlefieldRoute,
    BattlefieldRouteDefinition, BattlefieldTemplateDefinition, BattlefieldTile,
    BattlefieldTileKind, DeploymentZone, DeploymentZoneKind, ParsedBattlefieldTemplate, SpawnZone,
    SpawnZoneKind, ZoneConfidence,
};

pub(super) fn parse_battlefield_template(
    template: &BattlefieldTemplateDefinition,
) -> Result<ParsedBattlefieldTemplate, String> {
    if template.rows.is_empty() {
        return Err(format!(
            "battlefield template '{}' has no rows",
            template.id
        ));
    }

    let height = i32::try_from(template.rows.len()).map_err(|_| {
        format!(
            "battlefield template '{}' height does not fit i32",
            template.id
        )
    })?;
    let width = template
        .rows
        .iter()
        .map(|row| row.chars().count())
        .max()
        .and_then(|width| i32::try_from(width).ok())
        .ok_or_else(|| format!("battlefield template '{}' has invalid width", template.id))?;
    if width <= 0 || height <= 0 {
        return Err(format!(
            "battlefield template '{}' dimensions must be positive",
            template.id
        ));
    }

    let mut valid_tiles = Vec::new();
    let mut tiles = Vec::new();
    let mut ground_deployment_cells = Vec::new();
    let mut platform_deployment_cells = Vec::new();
    let mut obstacles = Vec::new();
    let mut north_entry = Vec::new();
    let mut north_west_entry = Vec::new();
    let mut north_east_entry = Vec::new();
    let mut side_ambush = Vec::new();
    let mut boss_anchor = Vec::new();
    let mut west_reinforcement = Vec::new();
    let mut east_reinforcement = Vec::new();

    for (y, row) in template.rows.iter().enumerate() {
        for (x, tile) in row.chars().enumerate() {
            if tile == ' ' {
                continue;
            }

            let position = Position::new(x as i32, y as i32);
            valid_tiles.push(position);
            let tile_kind = match tile {
                '.' | 'P' | 'N' | 'L' | 'Q' | 'A' | 'B' | 'W' | 'R' | 'X' | 'Y' | 'Z' => {
                    BattlefieldTileKind::Ground
                }
                'T' => BattlefieldTileKind::Platform,
                '#' => BattlefieldTileKind::Obstacle,
                other => {
                    return Err(format!(
                        "battlefield template '{}' contains unsupported tile '{}'",
                        template.id, other
                    ));
                }
            };
            tiles.push(BattlefieldTile {
                position,
                kind: tile_kind,
            });
            match tile {
                '.' => {}
                '#' => obstacles.push(position),
                'P' => ground_deployment_cells.push(position),
                'T' => platform_deployment_cells.push(position),
                'N' => north_entry.push(position),
                'L' => north_west_entry.push(position),
                'Q' => north_east_entry.push(position),
                'A' => side_ambush.push(position),
                'B' => boss_anchor.push(position),
                'W' => west_reinforcement.push(position),
                'R' => east_reinforcement.push(position),
                'X' | 'Y' | 'Z' => {}
                _ => unreachable!("unsupported battlefield tile was rejected above"),
            }
        }
    }

    let valid_tiles = unique_positions(valid_tiles);
    let tiles = unique_tiles(tiles);
    let ground_deployment_cells = unique_positions(ground_deployment_cells);
    let platform_deployment_cells = unique_positions(platform_deployment_cells);
    if ground_deployment_cells.is_empty() && platform_deployment_cells.is_empty() {
        return Err(format!(
            "battlefield template '{}' must contain at least one deployment tile",
            template.id
        ));
    }

    let mut deployment_zones = Vec::new();
    push_deployment_zone(
        &mut deployment_zones,
        "ground_deployment",
        "Ground Deployment",
        DeploymentZoneKind::Ground,
        ground_deployment_cells,
    );
    push_deployment_zone(
        &mut deployment_zones,
        "platform_deployment",
        "Platform Deployment",
        DeploymentZoneKind::Platform,
        platform_deployment_cells,
    );
    let mut spawn_zones = Vec::new();
    push_spawn_zone(
        &mut spawn_zones,
        "north_entry",
        "North Entry",
        SpawnZoneKind::Entry,
        ZoneConfidence::Confirmed,
        north_entry,
    );
    push_spawn_zone(
        &mut spawn_zones,
        "north_west_entry",
        "North-West Entry",
        SpawnZoneKind::Entry,
        ZoneConfidence::Likely,
        north_west_entry,
    );
    push_spawn_zone(
        &mut spawn_zones,
        "north_east_entry",
        "North-East Entry",
        SpawnZoneKind::Entry,
        ZoneConfidence::Likely,
        north_east_entry,
    );
    push_spawn_zone(
        &mut spawn_zones,
        "side_ambush",
        "Side Ambush",
        SpawnZoneKind::Ambush,
        confidence_for_hidden(),
        side_ambush,
    );
    push_spawn_zone(
        &mut spawn_zones,
        "boss_anchor",
        "Boss Anchor",
        SpawnZoneKind::BossAnchor,
        ZoneConfidence::Confirmed,
        boss_anchor,
    );
    push_spawn_zone(
        &mut spawn_zones,
        "west_reinforcement",
        "West Reinforcement",
        SpawnZoneKind::Entry,
        confidence_for_hidden(),
        west_reinforcement,
    );
    push_spawn_zone(
        &mut spawn_zones,
        "east_reinforcement",
        "East Reinforcement",
        SpawnZoneKind::Entry,
        confidence_for_hidden(),
        east_reinforcement,
    );
    if spawn_zones.is_empty() {
        return Err(format!(
            "battlefield template '{}' must contain at least one spawn tile",
            template.id
        ));
    }
    let obstacle_positions = obstacles.iter().copied().collect::<HashSet<_>>();
    let valid_positions = valid_tiles.iter().copied().collect::<HashSet<_>>();
    let routes = parse_battlefield_routes(
        template,
        width,
        height,
        &valid_positions,
        &obstacle_positions,
    )?;

    Ok(ParsedBattlefieldTemplate {
        id: template.id.clone(),
        archetype: template.archetype,
        size_class: template.size_class,
        width,
        height,
        tiles,
        valid_tiles,
        deployment_zones,
        spawn_zones,
        routes,
        obstacles: unique_positions(obstacles),
    })
}

fn parse_battlefield_routes(
    template: &BattlefieldTemplateDefinition,
    width: i32,
    height: i32,
    valid_tiles: &HashSet<Position>,
    obstacles: &HashSet<Position>,
) -> Result<Vec<BattlefieldRoute>, String> {
    let mut route_ids = HashSet::new();
    let mut routes = Vec::new();
    for route in &template.routes {
        if route.id.trim().is_empty() {
            return Err(format!(
                "battlefield template '{}' has route with empty id",
                template.id
            ));
        }
        if !route_ids.insert(route.id.as_str()) {
            return Err(format!(
                "battlefield template '{}' has duplicate route id '{}'",
                template.id, route.id
            ));
        }
        routes.push(parse_battlefield_route(
            template,
            route,
            width,
            height,
            valid_tiles,
            obstacles,
        )?);
    }
    Ok(routes)
}

fn parse_battlefield_route(
    template: &BattlefieldTemplateDefinition,
    route: &BattlefieldRouteDefinition,
    width: i32,
    height: i32,
    valid_tiles: &HashSet<Position>,
    obstacles: &HashSet<Position>,
) -> Result<BattlefieldRoute, String> {
    if route.overlay.len() != height as usize {
        return Err(format!(
            "route '{}' in battlefield template '{}' must have {} rows",
            route.id, template.id, height
        ));
    }

    let mut route_chars: HashMap<Position, char> = HashMap::new();
    for (y, row) in route.overlay.iter().enumerate() {
        let row_width = row.chars().count();
        if row_width != width as usize {
            return Err(format!(
                "route '{}' in battlefield template '{}' row {} width must be {} but was {}",
                route.id, template.id, y, width, row_width
            ));
        }
        for (x, ch) in row.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let position = Position::new(x as i32, y as i32);
            if !valid_tiles.contains(&position) {
                return Err(format!(
                    "route '{}' in battlefield template '{}' places '{}' outside valid terrain at ({}, {})",
                    route.id, template.id, ch, position.x, position.y
                ));
            }
            if obstacles.contains(&position) {
                return Err(format!(
                    "route '{}' in battlefield template '{}' places '{}' on obstacle at ({}, {})",
                    route.id, template.id, ch, position.x, position.y
                ));
            }
            if !is_route_arrow(ch) && !is_route_marker(ch) {
                return Err(format!(
                    "route '{}' in battlefield template '{}' contains unsupported marker '{}'",
                    route.id, template.id, ch
                ));
            }
            if is_route_marker(ch) {
                let terrain = terrain_char_at(template, position);
                if terrain != Some(ch) {
                    return Err(format!(
                        "route '{}' in battlefield template '{}' marker '{}' must match terrain marker at ({}, {})",
                        route.id, template.id, ch, position.x, position.y
                    ));
                }
            }
            route_chars.insert(position, ch);
        }
    }

    if route_chars.len() < 2 {
        return Err(format!(
            "route '{}' in battlefield template '{}' must contain at least a start and end marker",
            route.id, template.id
        ));
    }

    let mut outgoing: HashMap<Position, Position> = HashMap::new();
    let mut incoming_counts: HashMap<Position, usize> = HashMap::new();
    let mut marker_positions = Vec::new();

    for (&position, &ch) in &route_chars {
        if is_route_arrow(ch) {
            let next = route_arrow_destination(position, ch);
            if !route_chars.contains_key(&next) {
                return Err(format!(
                    "route '{}' in battlefield template '{}' arrow '{}' at ({}, {}) points outside route",
                    route.id, template.id, ch, position.x, position.y
                ));
            }
            outgoing.insert(position, next);
            *incoming_counts.entry(next).or_default() += 1;
        } else {
            marker_positions.push(position);
        }
    }

    if marker_positions.len() != 2 {
        return Err(format!(
            "route '{}' in battlefield template '{}' must contain exactly two endpoint markers",
            route.id, template.id
        ));
    }

    for &marker in &marker_positions {
        let candidates = route_marker_outgoing_candidates(marker, &route_chars);
        if candidates.len() > 1 {
            return Err(format!(
                "route '{}' in battlefield template '{}' marker at ({}, {}) branches to multiple arrows",
                route.id, template.id, marker.x, marker.y
            ));
        }
        if let Some(next) = candidates.first().copied() {
            outgoing.insert(marker, next);
            *incoming_counts.entry(next).or_default() += 1;
        }
    }

    let starts = marker_positions
        .iter()
        .copied()
        .filter(|position| {
            incoming_counts.get(position).copied().unwrap_or(0) == 0
                && outgoing.contains_key(position)
        })
        .collect::<Vec<_>>();
    let ends = marker_positions
        .iter()
        .copied()
        .filter(|position| {
            incoming_counts.get(position).copied().unwrap_or(0) == 1
                && !outgoing.contains_key(position)
        })
        .collect::<Vec<_>>();

    if starts.len() != 1 || ends.len() != 1 {
        return Err(format!(
            "route '{}' in battlefield template '{}' must resolve to one start and one end",
            route.id, template.id
        ));
    }

    for &position in route_chars.keys() {
        let incoming = incoming_counts.get(&position).copied().unwrap_or(0);
        if incoming > 1 {
            return Err(format!(
                "route '{}' in battlefield template '{}' branches into ({}, {})",
                route.id, template.id, position.x, position.y
            ));
        }
        if is_route_arrow(route_chars[&position]) && !outgoing.contains_key(&position) {
            return Err(format!(
                "route '{}' in battlefield template '{}' arrow at ({}, {}) has no outgoing edge",
                route.id, template.id, position.x, position.y
            ));
        }
    }

    let start = starts[0];
    let end = ends[0];
    let mut cells = Vec::new();
    let mut visited = HashSet::new();
    let mut current = start;
    loop {
        if !visited.insert(current) {
            return Err(format!(
                "route '{}' in battlefield template '{}' contains a loop",
                route.id, template.id
            ));
        }
        cells.push(current);
        if current == end {
            break;
        }
        current = *outgoing.get(&current).ok_or_else(|| {
            format!(
                "route '{}' in battlefield template '{}' is disconnected before reaching end",
                route.id, template.id
            )
        })?;
    }

    if cells.len() != route_chars.len() {
        return Err(format!(
            "route '{}' in battlefield template '{}' has disconnected cells",
            route.id, template.id
        ));
    }

    Ok(BattlefieldRoute {
        id: route.id.clone(),
        start,
        end,
        cells,
    })
}

fn terrain_char_at(template: &BattlefieldTemplateDefinition, position: Position) -> Option<char> {
    template
        .rows
        .get(position.y as usize)
        .and_then(|row| row.chars().nth(position.x as usize))
}

fn is_route_arrow(ch: char) -> bool {
    matches!(ch, '>' | '<' | '^' | 'v')
}

fn is_route_marker(ch: char) -> bool {
    ch.is_ascii_uppercase()
}

fn route_arrow_destination(position: Position, arrow: char) -> Position {
    match arrow {
        '>' => Position::new(position.x + 1, position.y),
        '<' => Position::new(position.x - 1, position.y),
        '^' => Position::new(position.x, position.y - 1),
        'v' => Position::new(position.x, position.y + 1),
        _ => position,
    }
}

fn route_marker_outgoing_candidates(
    marker: Position,
    route_chars: &HashMap<Position, char>,
) -> Vec<Position> {
    [
        (Position::new(marker.x + 1, marker.y), '>'),
        (Position::new(marker.x - 1, marker.y), '<'),
        (Position::new(marker.x, marker.y - 1), '^'),
        (Position::new(marker.x, marker.y + 1), 'v'),
    ]
    .into_iter()
    .filter_map(|(position, expected)| {
        route_chars
            .get(&position)
            .is_some_and(|ch| *ch == expected)
            .then_some(position)
    })
    .collect()
}

fn push_deployment_zone(
    zones: &mut Vec<DeploymentZone>,
    id: &str,
    label: &str,
    kind: DeploymentZoneKind,
    cells: Vec<Position>,
) {
    if cells.is_empty() {
        return;
    }

    zones.push(DeploymentZone {
        id: id.to_string(),
        label: label.to_string(),
        kind,
        cells: unique_positions(cells),
    });
}

fn push_spawn_zone(
    zones: &mut Vec<SpawnZone>,
    id: &str,
    label: &str,
    kind: SpawnZoneKind,
    confidence: ZoneConfidence,
    cells: Vec<Position>,
) {
    if cells.is_empty() {
        return;
    }

    zones.push(SpawnZone {
        id: id.to_string(),
        label: label.to_string(),
        kind,
        confidence,
        cells: unique_positions(cells),
        revealed_details: Vec::new(),
    });
}
