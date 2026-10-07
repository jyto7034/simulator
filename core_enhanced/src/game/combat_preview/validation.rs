use std::collections::{HashSet, VecDeque};

use crate::game::resources::Position;

use super::{BattlefieldInstance, BattlefieldTileKind, CombatNodeType};

#[derive(Debug)]
pub(super) struct InstanceValidationError {
    pub message: &'static str,
    pub wave_field: Option<(usize, String)>,
}

impl From<&'static str> for InstanceValidationError {
    fn from(message: &'static str) -> Self {
        Self {
            message,
            wave_field: None,
        }
    }
}

#[cfg(test)]
pub(super) fn validate_instance(instance: &BattlefieldInstance) -> Result<(), &'static str> {
    validate_instance_with_location(instance).map_err(|error| error.message)
}

pub(super) fn validate_instance_with_location(
    instance: &BattlefieldInstance,
) -> Result<(), InstanceValidationError> {
    if instance.width <= 0 || instance.height <= 0 {
        return Err("battlefield dimensions must be positive".into());
    }
    if instance.deployment_zones.is_empty() {
        return Err("battlefield must have at least one deployment zone".into());
    }
    if instance.spawn_zones.is_empty() {
        return Err("battlefield must have at least one spawn zone".into());
    }

    let valid_cells = instance.valid_tiles.iter().copied().collect::<HashSet<_>>();
    if valid_cells.is_empty() {
        return Err("battlefield must have at least one valid tile".into());
    }
    if valid_cells.len() != instance.valid_tiles.len() {
        return Err("duplicate valid battlefield tile".into());
    }
    if instance
        .valid_tiles
        .iter()
        .any(|cell| !in_bounds(*cell, instance.width, instance.height))
    {
        return Err("valid tile is out of battlefield bounds".into());
    }
    let tile_positions = instance
        .tiles
        .iter()
        .map(|tile| tile.position)
        .collect::<HashSet<_>>();
    if tile_positions.is_empty() {
        return Err("battlefield must expose canonical tiles".into());
    }
    if tile_positions.len() != instance.tiles.len() {
        return Err("duplicate canonical battlefield tile".into());
    }
    if tile_positions != valid_cells {
        return Err("canonical battlefield tiles must match valid tiles".into());
    }
    if instance
        .tiles
        .iter()
        .any(|tile| !in_bounds(tile.position, instance.width, instance.height))
    {
        return Err("canonical battlefield tile is out of battlefield bounds".into());
    }

    let obstacle_cells = instance.obstacles.iter().copied().collect::<HashSet<_>>();
    if obstacle_cells.len() != instance.obstacles.len() {
        return Err("duplicate battlefield obstacle".into());
    }
    if instance
        .obstacles
        .iter()
        .any(|cell| !in_bounds(*cell, instance.width, instance.height))
    {
        return Err("obstacle is out of battlefield bounds".into());
    }
    if obstacle_cells
        .iter()
        .any(|cell| !valid_cells.contains(cell))
    {
        return Err("obstacle is outside valid battlefield tiles".into());
    }
    let obstacle_tile_cells = instance
        .tiles
        .iter()
        .filter(|tile| tile.kind == BattlefieldTileKind::Obstacle)
        .map(|tile| tile.position)
        .collect::<HashSet<_>>();
    if obstacle_tile_cells != obstacle_cells {
        return Err("canonical obstacle tiles must match obstacles".into());
    }
    if has_duplicate_ids(
        instance
            .deployment_zones
            .iter()
            .map(|zone| zone.id.as_str()),
    ) {
        return Err("duplicate deployment zone id".into());
    }
    if has_duplicate_ids(instance.spawn_zones.iter().map(|zone| zone.id.as_str())) {
        return Err("duplicate spawn zone id".into());
    }
    if has_duplicate_ids(instance.routes.iter().map(|route| route.id.as_str())) {
        return Err("duplicate route id".into());
    }
    let deployment_cells = instance
        .deployment_zones
        .iter()
        .flat_map(|zone| zone.cells.iter().copied())
        .collect::<HashSet<_>>();
    let spawn_cells = instance
        .spawn_zones
        .iter()
        .flat_map(|zone| zone.cells.iter().copied())
        .collect::<HashSet<_>>();

    if deployment_cells
        .iter()
        .any(|cell| !in_bounds(*cell, instance.width, instance.height))
    {
        return Err("deployment zone is out of battlefield bounds".into());
    }
    if spawn_cells
        .iter()
        .any(|cell| !in_bounds(*cell, instance.width, instance.height))
    {
        return Err("spawn zone is out of battlefield bounds".into());
    }
    if deployment_cells
        .iter()
        .any(|cell| !valid_cells.contains(cell))
    {
        return Err("deployment zone is outside valid battlefield tiles".into());
    }
    if spawn_cells.iter().any(|cell| !valid_cells.contains(cell)) {
        return Err("spawn zone is outside valid battlefield tiles".into());
    }
    if deployment_cells
        .iter()
        .any(|cell| obstacle_cells.contains(cell))
    {
        return Err("deployment zone overlaps obstacle".into());
    }
    if spawn_cells.iter().any(|cell| obstacle_cells.contains(cell)) {
        return Err("spawn zone overlaps obstacle".into());
    }
    if deployment_cells
        .iter()
        .any(|cell| spawn_cells.contains(cell))
    {
        return Err("deployment zone overlaps spawn zone".into());
    }
    for route in &instance.routes {
        if route.cells.is_empty() {
            return Err("route has no cells".into());
        }
        if route.cells.first().copied() != Some(route.start) {
            return Err("route first cell must be route start".into());
        }
        if route.cells.last().copied() != Some(route.end) {
            return Err("route last cell must be route end".into());
        }
        if route.cells.iter().any(|cell| !valid_cells.contains(cell)) {
            return Err("route is outside valid battlefield tiles".into());
        }
        if route.cells.iter().any(|cell| obstacle_cells.contains(cell)) {
            return Err("route overlaps obstacle".into());
        }
        if route
            .cells
            .windows(2)
            .any(|window| !is_cardinal_adjacent(window[0], window[1]))
        {
            return Err("route cells must be cardinal-adjacent".into());
        }
    }
    if instance.node_type == CombatNodeType::Defense && !instance.routes.is_empty() {
        let defense_endpoint = instance.routes[0].end;
        if instance
            .routes
            .iter()
            .any(|route| route.end != defense_endpoint)
        {
            return Err("defense routes must share one defense object endpoint".into());
        }
    }

    let spawn_zone_ids = instance
        .spawn_zones
        .iter()
        .map(|zone| zone.id.as_str())
        .collect::<HashSet<_>>();
    let route_ids = instance
        .routes
        .iter()
        .map(|route| route.id.as_str())
        .collect::<HashSet<_>>();
    for (wave_index, wave) in instance.spawn_waves.iter().enumerate() {
        if let Some(zone_index) = wave
            .spawn_zone_ids
            .iter()
            .position(|zone_id| !spawn_zone_ids.contains(zone_id.as_str()))
        {
            return Err(InstanceValidationError {
                message: "spawn wave references missing spawn zone",
                wave_field: Some((wave_index, format!("spawn_zone_ids[{zone_index}]"))),
            });
        }
        if let Some(route_id) = &wave.route_id {
            if !route_ids.contains(route_id.as_str()) {
                return Err(InstanceValidationError {
                    message: "spawn wave references missing route",
                    wave_field: Some((wave_index, "route_id".into())),
                });
            }
        } else if instance.node_type == CombatNodeType::Defense {
            return Err(InstanceValidationError {
                message: "defense spawn wave must reference a route",
                wave_field: Some((wave_index, "route_id".into())),
            });
        }
    }

    let start = deployment_cells
        .iter()
        .next()
        .copied()
        .ok_or("deployment zone has no cells")?;
    let target = spawn_cells
        .iter()
        .next()
        .copied()
        .ok_or("spawn zone has no cells")?;
    if !has_path(
        start,
        target,
        instance.width,
        instance.height,
        &valid_cells,
        &obstacle_cells,
    ) {
        return Err("deployment zone cannot path to spawn zone".into());
    }

    Ok(())
}

fn has_duplicate_ids<'a>(mut ids: impl Iterator<Item = &'a str>) -> bool {
    let mut seen = HashSet::new();
    ids.any(|id| !seen.insert(id))
}

fn in_bounds(position: Position, width: i32, height: i32) -> bool {
    position.x >= 0 && position.y >= 0 && position.x < width && position.y < height
}

fn is_cardinal_adjacent(left: Position, right: Position) -> bool {
    (left.x - right.x).abs() + (left.y - right.y).abs() == 1
}

fn has_path(
    start: Position,
    target: Position,
    width: i32,
    height: i32,
    valid_cells: &HashSet<Position>,
    blocked: &HashSet<Position>,
) -> bool {
    if !valid_cells.contains(&start)
        || !valid_cells.contains(&target)
        || blocked.contains(&start)
        || blocked.contains(&target)
    {
        return false;
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::from([start]);
    while let Some(position) = queue.pop_front() {
        if position == target {
            return true;
        }
        if !visited.insert(position) {
            continue;
        }
        for next in [
            Position::new(position.x + 1, position.y),
            Position::new(position.x - 1, position.y),
            Position::new(position.x, position.y + 1),
            Position::new(position.x, position.y - 1),
        ] {
            if next.x < 0
                || next.y < 0
                || next.x >= width
                || next.y >= height
                || !valid_cells.contains(&next)
                || blocked.contains(&next)
                || visited.contains(&next)
            {
                continue;
            }
            queue.push_back(next);
        }
    }

    false
}
