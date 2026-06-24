use std::collections::{HashSet, VecDeque};

use crate::game::resources::Position;

use super::{BattlefieldInstance, BattlefieldTileKind, CombatNodeType};

pub(super) fn validate_instance(instance: &BattlefieldInstance) -> Result<(), &'static str> {
    if instance.width <= 0 || instance.height <= 0 {
        return Err("battlefield dimensions must be positive");
    }
    if instance.deployment_zones.is_empty() {
        return Err("battlefield must have at least one deployment zone");
    }
    if instance.spawn_zones.is_empty() {
        return Err("battlefield must have at least one spawn zone");
    }

    let valid_cells = instance.valid_tiles.iter().copied().collect::<HashSet<_>>();
    if valid_cells.is_empty() {
        return Err("battlefield must have at least one valid tile");
    }
    if valid_cells.len() != instance.valid_tiles.len() {
        return Err("duplicate valid battlefield tile");
    }
    if instance
        .valid_tiles
        .iter()
        .any(|cell| !in_bounds(*cell, instance.width, instance.height))
    {
        return Err("valid tile is out of battlefield bounds");
    }
    let tile_positions = instance
        .tiles
        .iter()
        .map(|tile| tile.position)
        .collect::<HashSet<_>>();
    if tile_positions.is_empty() {
        return Err("battlefield must expose canonical tiles");
    }
    if tile_positions.len() != instance.tiles.len() {
        return Err("duplicate canonical battlefield tile");
    }
    if tile_positions != valid_cells {
        return Err("canonical battlefield tiles must match valid tiles");
    }
    if instance
        .tiles
        .iter()
        .any(|tile| !in_bounds(tile.position, instance.width, instance.height))
    {
        return Err("canonical battlefield tile is out of battlefield bounds");
    }

    let obstacle_cells = instance.obstacles.iter().copied().collect::<HashSet<_>>();
    if obstacle_cells.len() != instance.obstacles.len() {
        return Err("duplicate battlefield obstacle");
    }
    if instance
        .obstacles
        .iter()
        .any(|cell| !in_bounds(*cell, instance.width, instance.height))
    {
        return Err("obstacle is out of battlefield bounds");
    }
    if obstacle_cells
        .iter()
        .any(|cell| !valid_cells.contains(cell))
    {
        return Err("obstacle is outside valid battlefield tiles");
    }
    let obstacle_tile_cells = instance
        .tiles
        .iter()
        .filter(|tile| tile.kind == BattlefieldTileKind::Obstacle)
        .map(|tile| tile.position)
        .collect::<HashSet<_>>();
    if obstacle_tile_cells != obstacle_cells {
        return Err("canonical obstacle tiles must match obstacles");
    }
    if has_duplicate_ids(
        instance
            .deployment_zones
            .iter()
            .map(|zone| zone.id.as_str()),
    ) {
        return Err("duplicate deployment zone id");
    }
    if has_duplicate_ids(instance.spawn_zones.iter().map(|zone| zone.id.as_str())) {
        return Err("duplicate spawn zone id");
    }
    if has_duplicate_ids(instance.routes.iter().map(|route| route.id.as_str())) {
        return Err("duplicate route id");
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
        return Err("deployment zone is out of battlefield bounds");
    }
    if spawn_cells
        .iter()
        .any(|cell| !in_bounds(*cell, instance.width, instance.height))
    {
        return Err("spawn zone is out of battlefield bounds");
    }
    if deployment_cells
        .iter()
        .any(|cell| !valid_cells.contains(cell))
    {
        return Err("deployment zone is outside valid battlefield tiles");
    }
    if spawn_cells.iter().any(|cell| !valid_cells.contains(cell)) {
        return Err("spawn zone is outside valid battlefield tiles");
    }
    if deployment_cells
        .iter()
        .any(|cell| obstacle_cells.contains(cell))
    {
        return Err("deployment zone overlaps obstacle");
    }
    if spawn_cells.iter().any(|cell| obstacle_cells.contains(cell)) {
        return Err("spawn zone overlaps obstacle");
    }
    if deployment_cells
        .iter()
        .any(|cell| spawn_cells.contains(cell))
    {
        return Err("deployment zone overlaps spawn zone");
    }
    for route in &instance.routes {
        if route.cells.is_empty() {
            return Err("route has no cells");
        }
        if route.cells.first().copied() != Some(route.start) {
            return Err("route first cell must be route start");
        }
        if route.cells.last().copied() != Some(route.end) {
            return Err("route last cell must be route end");
        }
        if route.cells.iter().any(|cell| !valid_cells.contains(cell)) {
            return Err("route is outside valid battlefield tiles");
        }
        if route.cells.iter().any(|cell| obstacle_cells.contains(cell)) {
            return Err("route overlaps obstacle");
        }
        if route
            .cells
            .windows(2)
            .any(|window| !is_cardinal_adjacent(window[0], window[1]))
        {
            return Err("route cells must be cardinal-adjacent");
        }
    }
    if instance.node_type == CombatNodeType::Defense && !instance.routes.is_empty() {
        let defense_endpoint = instance.routes[0].end;
        if instance
            .routes
            .iter()
            .any(|route| route.end != defense_endpoint)
        {
            return Err("defense routes must share one defense object endpoint");
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
    for wave in &instance.spawn_waves {
        if wave
            .spawn_zone_ids
            .iter()
            .any(|zone_id| !spawn_zone_ids.contains(zone_id.as_str()))
        {
            return Err("spawn wave references missing spawn zone");
        }
        if let Some(route_id) = &wave.route_id {
            if !route_ids.contains(route_id.as_str()) {
                return Err("spawn wave references missing route");
            }
        } else if instance.node_type == CombatNodeType::Defense {
            return Err("defense spawn wave must reference a route");
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
        return Err("deployment zone cannot path to spawn zone");
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
