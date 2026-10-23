import type { MapDocument, Position } from "./model";
// Top-left origin, +X right, +Y down. Cells own [x,x+1) × [y,y+1).
// Advanced snap uses interior subdivision intersections (1..N-1)/N, including .5.
export const TILE = 32;
export const cellKey = (p: Position) => `${Math.floor(p.x)},${Math.floor(p.y)}`;
export const cellPosition = (key: string): Position => {
  const [x, y] = key.split(",").map(Number);
  return { x, y };
};
// All edges use the same direction so adjacent cells share one stroke.
export function cellBorderPath(keys: Iterable<string>): string {
  const edges = new Set<string>();
  for (const key of keys) {
    const p = cellPosition(key);
    const x = p.x * TILE,
      y = p.y * TILE;
    edges.add(`M${x} ${y}h${TILE}`);
    edges.add(`M${x} ${y + TILE}h${TILE}`);
    edges.add(`M${x} ${y}v${TILE}`);
    edges.add(`M${x + TILE} ${y}v${TILE}`);
  }
  return [...edges].join(" ");
}
export const inBounds = (d: MapDocument, p: Position) =>
  p.x >= 0 && p.y >= 0 && p.x < d.width && p.y < d.height;
export const isValid = (d: MapDocument, p: Position) =>
  inBounds(d, p) && d.valid.includes(cellKey(p));
export function snap(p: Position, subdivisions: number | null): Position {
  const axis = (v: number) =>
    Math.floor(v) +
    (subdivisions
      ? Math.max(
          1,
          Math.min(
            subdivisions - 1,
            Math.round((v - Math.floor(v)) * subdivisions),
          ),
        ) / subdivisions
      : 0.5);
  return { x: axis(p.x), y: axis(p.y) };
}
export function screenToMap(
  p: Position,
  pan: Position,
  zoom: number,
): Position {
  return { x: (p.x - pan.x) / (TILE * zoom), y: (p.y - pan.y) / (TILE * zoom) };
}
export function rectangle(a: Position, b: Position): string[] {
  const cells: string[] = [];
  for (
    let y = Math.floor(Math.min(a.y, b.y));
    y <= Math.floor(Math.max(a.y, b.y));
    y++
  )
    for (
      let x = Math.floor(Math.min(a.x, b.x));
      x <= Math.floor(Math.max(a.x, b.x));
      x++
    )
      cells.push(`${x},${y}`);
  return cells;
}
// Inspect intervals between grid crossings. This is geometric inspection, not pathfinding.
export function segmentCells(a: Position, b: Position): string[] {
  const times = [0, 1];
  for (const axis of ["x", "y"] as const) {
    if (a[axis] === b[axis]) continue;
    for (
      let n = Math.ceil(Math.min(a[axis], b[axis]));
      n < Math.max(a[axis], b[axis]);
      n++
    )
      times.push((n - a[axis]) / (b[axis] - a[axis]));
  }
  times.sort((x, y) => x - y);
  const keys = new Set([cellKey(a), cellKey(b)]);
  for (let i = 1; i < times.length; i++) {
    const t = (times[i - 1] + times[i]) / 2;
    keys.add(cellKey({ x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t }));
  }
  return [...keys];
}

export function fitMapView(
  viewport: { width: number; height: number },
  map: { width: number; height: number },
) {
  const zoom = Math.max(
    0.1,
    Math.min(
      1.5,
      (viewport.width - 56) / (map.width * TILE),
      (viewport.height - 56) / (map.height * TILE),
    ),
  );
  return {
    zoom,
    pan: {
      x: (viewport.width - map.width * TILE * zoom) / 2,
      y: (viewport.height - map.height * TILE * zoom) / 2,
    },
  };
}
