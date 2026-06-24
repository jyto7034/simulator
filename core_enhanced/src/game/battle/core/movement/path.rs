use crate::game::resources::Position;

use super::types::WorldVec2;

pub(in crate::game::battle::core::movement) fn next_cell_path_target(
    current: WorldVec2,
    cells: &[Position],
    reach_radius: f32,
) -> Option<WorldVec2> {
    if cells.is_empty() {
        return None;
    }

    let centers = cells
        .iter()
        .copied()
        .map(WorldVec2::from_tile_center)
        .collect::<Vec<_>>();
    if centers.len() == 1 {
        return centers.first().copied();
    }

    let first = centers[0];
    let first_segment = centers[1] - first;
    if dot(current - first, first_segment) < 0.0 && current.distance(first) > reach_radius {
        return Some(first);
    }

    let current_progress = route_progress_along_centers(&centers, current)?;
    let target_progress = current_progress + reach_radius.max(0.0);
    let mut cumulative = 0.0;

    for waypoint in 1..centers.len() {
        cumulative += centers[waypoint - 1].distance(centers[waypoint]);
        if cumulative > target_progress {
            return Some(centers[waypoint]);
        }
    }

    centers.last().copied()
}

pub(in crate::game::battle::core::movement) fn route_progress_along_cells(
    cells: &[Position],
    position: WorldVec2,
) -> Option<f32> {
    if cells.is_empty() {
        return None;
    }
    if cells.len() == 1 {
        return Some(0.0);
    }

    let centers = cells
        .iter()
        .copied()
        .map(WorldVec2::from_tile_center)
        .collect::<Vec<_>>();
    route_progress_along_centers(&centers, position)
}

pub(in crate::game::battle::core::movement) fn route_total_length(
    cells: &[Position],
) -> Option<f32> {
    if cells.len() < 2 {
        return Some(0.0);
    }

    let centers = cells
        .iter()
        .copied()
        .map(WorldVec2::from_tile_center)
        .collect::<Vec<_>>();
    Some(
        centers
            .windows(2)
            .map(|segment| segment[0].distance(segment[1]))
            .sum(),
    )
}

fn route_progress_along_centers(centers: &[WorldVec2], position: WorldVec2) -> Option<f32> {
    if centers.is_empty() {
        return None;
    }
    if centers.len() == 1 {
        return Some(0.0);
    }

    let mut cumulative = 0.0;
    let mut best_distance_sq = f32::MAX;
    let mut best_progress = 0.0;

    for segment in centers.windows(2) {
        let start = segment[0];
        let end = segment[1];
        let delta = end - start;
        let length_sq = delta.length_squared();
        if length_sq <= f32::EPSILON {
            continue;
        }

        let segment_length = length_sq.sqrt();
        let to_position = position - start;
        let t = (dot(to_position, delta) / length_sq).clamp(0.0, 1.0);
        let projected = start + delta * t;
        let distance_sq = position.distance_squared(projected);
        let progress = cumulative + segment_length * t;

        if distance_sq < best_distance_sq - f32::EPSILON
            || ((distance_sq - best_distance_sq).abs() <= f32::EPSILON && progress > best_progress)
        {
            best_distance_sq = distance_sq;
            best_progress = progress;
        }

        cumulative += segment_length;
    }

    Some(best_progress)
}

fn dot(a: WorldVec2, b: WorldVec2) -> f32 {
    a.x * b.x + a.y * b.y
}
