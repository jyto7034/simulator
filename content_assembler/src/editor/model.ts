export type Position = { x: number; y: number };
export type Terrain = "floor" | "platform" | "obstacle";
export type Zone = {
  id: string;
  name: string;
  kind: "deploy" | "spawn";
  cells: string[];
};
export type RoutePoint = Position & { id: string };
export type Route = { id: string; name: string; points: RoutePoint[] };
export type Landmark = Position & { id: string; name: string };
export type Slot = { id: string; name: string };
export type Trigger =
  | { kind: "time"; timeMs: number }
  | { kind: "clear"; waveId: string }
  | { kind: "hp"; slotId: string; percent: number }
  | { kind: "death"; slotId: string }
  | { kind: "reach"; slotId: string; pointId: string; radius: number };
export type Enemy = {
  id: string;
  contentId: string;
  count: number;
  intervalMs: number;
};
export type Wave = {
  id: string;
  name: string;
  enemies: Enemy[];
  zoneId: string;
  routeId: string;
  trigger: Trigger;
  fallbackMs: number | null;
  priority: number;
  required: boolean;
};
export type MapDocument = {
  format: "battle-map";
  version: 1;
  id: string;
  name: string;
  description: string;
  width: number;
  height: number;
  valid: string[];
  terrain: Record<string, Terrain>;
  zones: Zone[];
  routes: Route[];
  points: Landmark[];
  slots: Slot[];
  waves: Wave[];
};
export type Collection = "zones" | "routes" | "points" | "slots" | "waves";
export type Layer = "area" | "terrain" | Collection;
export type Selection =
  | { kind: Collection; id: string; pointId?: string }
  | { kind: "cell"; key: string }
  | null;
export const layerNames: Record<Layer, string> = {
  area: "유효 영역",
  terrain: "지형",
  zones: "구역",
  routes: "경로",
  points: "이름 있는 지점",
  slots: "임시 개체 슬롯",
  waves: "웨이브",
};
export const layers = Object.keys(layerNames) as Layer[];
export const uid = () => crypto.randomUUID();
export function assertMapDimension(value: unknown): asserts value is number {
  if (
    typeof value !== "number" ||
    !Number.isInteger(value) ||
    value < 1 ||
    value > 128
  )
    throw new Error("맵 크기는 1~128 정수입니다.");
}
export function createDocument(): MapDocument {
  return {
    format: "battle-map",
    version: 1,
    id: uid(),
    name: "새 전투 맵",
    description: "",
    width: 16,
    height: 12,
    valid: Array.from(
      { length: 192 },
      (_, i) => `${i % 16},${Math.floor(i / 16)}`,
    ).sort(),
    terrain: {},
    zones: [],
    routes: [],
    points: [],
    slots: [],
    waves: [],
  };
}
export function createWave(): Wave {
  return {
    id: uid(),
    name: "새 웨이브",
    enemies: [{ id: uid(), contentId: "", count: 1, intervalMs: 1000 }],
    zoneId: "",
    routeId: "",
    trigger: { kind: "time", timeMs: 0 },
    fallbackMs: null,
    priority: 0,
    required: true,
  };
}
export function defaultTrigger(kind: Trigger["kind"]): Trigger {
  switch (kind) {
    case "time":
      return { kind, timeMs: 0 };
    case "clear":
      return { kind, waveId: "" };
    case "hp":
      return { kind, slotId: "", percent: 50 };
    case "death":
      return { kind, slotId: "" };
    case "reach":
      return { kind, slotId: "", pointId: "", radius: 1 };
  }
}
