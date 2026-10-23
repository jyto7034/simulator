import { cellKey, cellPosition, inBounds } from "./coordinates";
import {
  uid,
  assertMapDimension,
  type MapDocument,
  type Collection,
  type Layer,
  type Selection,
  type Terrain,
  type Position,
  layers,
} from "./model";
export type Paint =
  "add" | "remove" | Terrain | "erase" | "zone-add" | "zone-remove";

export function resizeMap(
  d: MapDocument,
  axis: "width" | "height",
  size: number,
) {
  // Validate before enumerating cells, including direct numeric field input.
  assertMapDimension(size);
  const previous = d[axis];
  if (size === previous) return;
  d[axis] = size;
  if (size < previous) return;
  const valid = new Set(d.valid);
  for (let y = axis === "height" ? previous : 0; y < d.height; y++)
    for (let x = axis === "width" ? previous : 0; x < d.width; x++)
      valid.add(`${x},${y}`);
  d.valid = [...valid];
}

export function paint(
  d: MapDocument,
  keys: string[],
  mode: Paint,
  zoneId?: string,
) {
  const valid = new Set(d.valid);
  const zone = d.zones.find((z) => z.id === zoneId);
  const painted = new Set<string>();
  for (const key of keys) {
    if (!inBounds(d, cellPosition(key))) continue;
    if (mode === "add") valid.add(key);
    else if (mode === "remove") valid.delete(key);
    else if (mode === "zone-remove" && zone)
      zone.cells = zone.cells.filter((k) => k !== key);
    else if (valid.has(key)) {
      if (mode === "zone-add" && zone) {
        if (!zone.cells.includes(key)) zone.cells.push(key);
        painted.add(key);
      } else if (mode === "erase" || mode === "floor") delete d.terrain[key];
      else if (mode === "platform" || mode === "obstacle")
        d.terrain[key] = mode;
    }
  }
  if (zone && painted.size) removeOppositeZoneCells(d, zone.kind, painted);
  d.valid = [...valid];
}

function removeOppositeZoneCells(
  d: MapDocument,
  kind: "deploy" | "spawn",
  cells: ReadonlySet<string>,
) {
  for (const other of d.zones)
    if (other.kind !== kind)
      other.cells = other.cells.filter((key) => !cells.has(key));
}

export function setZoneKind(
  d: MapDocument,
  id: string,
  kind: "deploy" | "spawn",
) {
  const zone = d.zones.find((zone) => zone.id === id);
  if (!zone) return;
  zone.kind = kind;
  removeOppositeZoneCells(d, kind, new Set(zone.cells));
}

function assertRouteCellAvailable(
  route: MapDocument["routes"][number],
  position: Position,
  exceptId?: string,
) {
  const key = cellKey(position);
  if (route.points.some((p) => p.id !== exceptId && cellKey(p) === key))
    throw new Error(`같은 경로의 셀 (${key})에 이미 점이 있습니다.`);
}

export function insertRoutePoint(
  d: MapDocument,
  routeId: string,
  point: Position & { id: string },
  afterId?: string,
) {
  let route = d.routes.find((r) => r.id === routeId);
  if (!route) {
    route = { id: routeId, name: `경로 ${d.routes.length + 1}`, points: [] };
    d.routes.push(route);
  }
  assertRouteCellAvailable(route, point);
  const after = route.points.findIndex((p) => p.id === afterId);
  route.points.splice(after >= 0 ? after + 1 : route.points.length, 0, point);
}

export function moveRoutePoint(
  d: MapDocument,
  routeId: string,
  pointId: string,
  position: Position,
) {
  const route = d.routes.find((r) => r.id === routeId);
  const point = route?.points.find((p) => p.id === pointId);
  if (!route || !point || (point.x === position.x && point.y === position.y))
    return;
  assertRouteCellAvailable(route, position, pointId);
  Object.assign(point, position);
}
export function duplicateWave(d: MapDocument, id: string): string | null {
  const source = d.waves.find((w) => w.id === id);
  if (!source) return null;
  const wave = structuredClone(source);
  wave.id = uid();
  wave.name += " 복사";
  wave.enemies.forEach((e) => (e.id = uid()));
  if (wave.trigger.kind === "clear" && wave.trigger.waveId === source.id)
    wave.trigger.waveId = wave.id;
  d.waves.splice(d.waves.indexOf(source) + 1, 0, wave);
  return wave.id;
}
export function removeObject(d: MapDocument, kind: Collection, id: string) {
  const index = d[kind].findIndex((o) => o.id === id);
  if (index >= 0) d[kind].splice(index, 1);
}
function layerValue(d: MapDocument, layer: Layer): unknown {
  return layer === "area" ? [d.width, d.height, d.valid] : d[layer];
}
export function lockedChange(
  a: MapDocument,
  b: MapDocument,
  locked: ReadonlySet<Layer>,
): Layer | undefined {
  return layers.find(
    (l) =>
      locked.has(l) &&
      JSON.stringify(layerValue(a, l)) !== JSON.stringify(layerValue(b, l)),
  );
}
export function validSelection(d: MapDocument, s: Selection): Selection {
  if (!s) return null;
  if (s.kind === "cell")
    return inBounds(d, cellPosition(s.key)) || s.key in d.terrain ? s : null;
  const obj = d[s.kind].find((o) => o.id === s.id);
  if (!obj) return null;
  if (
    s.kind === "routes" &&
    s.pointId &&
    !d.routes.find((r) => r.id === s.id)?.points.some((p) => p.id === s.pointId)
  )
    return { kind: s.kind, id: s.id };
  return s;
}
