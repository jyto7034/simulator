import {
  cellKey,
  cellPosition,
  inBounds,
  isValid,
  segmentCells,
} from "./coordinates";
import type { MapDocument, Selection } from "./model";
export type Diagnostic = {
  code: string;
  severity: "error" | "warning" | "info";
  targetId: string;
  field: string;
  description: string;
  selection: Selection;
};
export function diagnose(d: MapDocument): Diagnostic[] {
  const out: Diagnostic[] = [];
  const add = (
    code: string,
    selection: Selection,
    field: string,
    description: string,
    severity: Diagnostic["severity"] = "error",
  ) =>
    out.push({
      code,
      selection,
      targetId: selection && "id" in selection ? selection.id : d.id,
      field,
      description,
      severity,
    });
  add(
    "CORE_UNBOUND",
    null,
    "map",
    "편집기 문서입니다. 적 카탈로그·개체 바인딩·실제 이동 가능성은 코어 미연동으로 확인하지 않습니다.",
    "info",
  );
  if (!d.name.trim())
    add("NAME_EMPTY", null, "name", "맵 이름이 비어 있습니다.");
  if (!d.valid.length) add("AREA_EMPTY", null, "area", "유효 영역이 없습니다.");
  if (d.valid.some((k) => !inBounds(d, cellPosition(k))))
    add(
      "AREA_BOUNDS",
      null,
      "size",
      "크기 밖 유효 셀이 보존되어 있습니다. 크기를 복원하거나 영역을 정리하세요.",
    );
  for (const k of Object.keys(d.terrain))
    if (!isValid(d, cellPosition(k)))
      add(
        "TERRAIN_OUTSIDE",
        { kind: "cell", key: k },
        "terrain",
        `${k}: 유효 영역 밖 지형이 보존되어 있습니다.`,
        "warning",
      );
  const zoneCells = { deploy: new Set<string>(), spawn: new Set<string>() };
  for (const zone of d.zones)
    for (const key of zone.cells) zoneCells[zone.kind].add(key);
  for (const z of d.zones) {
    const s: Selection = { kind: "zones", id: z.id };
    if (!z.cells.length)
      add("ZONE_EMPTY", s, "cells", `${z.name}: 구역에 셀이 없습니다.`);
    const overlap = z.cells.filter((key) =>
      zoneCells[z.kind === "deploy" ? "spawn" : "deploy"].has(key),
    );
    if (overlap.length)
      add(
        "ZONE_KIND_OVERLAP",
        s,
        "cells",
        `${z.name}: 배치·등장 구역이 ${overlap.length}셀 겹칩니다. 유지할 종류로 다시 칠하거나 구역 셀을 지우세요.`,
      );
    if (z.cells.some((k) => !isValid(d, cellPosition(k))))
      add("ZONE_OUTSIDE", s, "cells", `${z.name}: 유효 영역 밖 셀이 있습니다.`);
    if (z.cells.some((k) => d.terrain[k] === "obstacle"))
      add(
        "ZONE_OBSTACLE",
        s,
        "cells",
        `${z.name}: 장애물과 겹칩니다.`,
        "warning",
      );
  }
  for (const r of d.routes) {
    const s: Selection = { kind: "routes", id: r.id };
    if (r.points.length < 2)
      add(
        "ROUTE_INCOMPLETE",
        s,
        "points",
        `${r.name}: 시작·도착을 위해 점이 2개 이상 필요합니다.`,
      );
    const occupied = new Set<string>();
    for (const p of r.points) {
      const key = cellKey(p);
      if (occupied.has(key))
        add(
          "ROUTE_POINT_OVERLAP",
          { ...s, pointId: p.id },
          "position",
          `${r.name}: 셀 (${key})에 경로 점이 중복됩니다. 점을 옮기거나 삭제하세요.`,
        );
      occupied.add(key);
      if (!isValid(d, p))
        add(
          "ROUTE_POINT_OUTSIDE",
          { ...s, pointId: p.id },
          "position",
          `${r.name}: 경로 점이 유효 영역 밖입니다.`,
        );
    }
    for (let i = 1; i < r.points.length; i++) {
      const keys = segmentCells(r.points[i - 1], r.points[i]);
      if (keys.some((k) => !isValid(d, cellPosition(k))))
        add(
          "SEGMENT_OUTSIDE",
          s,
          "points",
          `${r.name}: ${i}번 선분이 빈 영역을 지납니다.`,
          "warning",
        );
      if (keys.some((k) => d.terrain[k] === "obstacle"))
        add(
          "SEGMENT_OBSTACLE",
          s,
          "points",
          `${r.name}: ${i}번 선분이 장애물 셀을 지납니다. 실제 이동 판정은 아닙니다.`,
          "warning",
        );
    }
  }
  for (const p of d.points)
    if (!isValid(d, p))
      add(
        "POINT_OUTSIDE",
        { kind: "points", id: p.id },
        "position",
        `${p.name}: 지점이 유효 영역 밖입니다.`,
      );
  const time = (n: number) => Number.isInteger(n) && n >= 0;
  for (const w of d.waves) {
    const s: Selection = { kind: "waves", id: w.id };
    const t = w.trigger;
    if (!d.zones.some((z) => z.id === w.zoneId && z.kind === "spawn"))
      add(
        "SPAWN_REFERENCE",
        s,
        "zoneId",
        `${w.name}: 적 등장 구역을 지정하세요.`,
      );
    if (!d.routes.some((r) => r.id === w.routeId))
      add(
        "ROUTE_REFERENCE",
        s,
        "routeId",
        `${w.name}: 경로 참조가 없거나 끊어졌습니다.`,
      );
    if (!w.enemies.length)
      add("ENEMIES_EMPTY", s, "enemies", `${w.name}: 적 구성이 비어 있습니다.`);
    for (const e of w.enemies) {
      if (!e.contentId.trim())
        add(
          "ENEMY_REFERENCE",
          s,
          "enemies",
          `${w.name}: 적 콘텐츠 ID가 비어 있습니다.`,
        );
      if (!Number.isInteger(e.count) || e.count < 1)
        add(
          "ENEMY_COUNT",
          s,
          "enemies",
          `${w.name}: 수량은 1 이상의 정수여야 합니다.`,
        );
      if (!time(e.intervalMs))
        add(
          "INTERVAL_VALUE",
          s,
          "enemies",
          `${w.name}: 등장 간격은 0 이상 정수 밀리초여야 합니다.`,
        );
    }
    if (!Number.isInteger(w.priority))
      add(
        "PRIORITY_VALUE",
        s,
        "priority",
        `${w.name}: 우선순위는 정수여야 합니다.`,
      );
    if (w.fallbackMs !== null && !time(w.fallbackMs))
      add(
        "FALLBACK_VALUE",
        s,
        "fallbackMs",
        `${w.name}: 대체 발동 시각이 올바르지 않습니다.`,
      );
    if (t.kind === "time" && (!time(t.timeMs) || w.fallbackMs !== null))
      add(
        "TIME_VALUE",
        s,
        "trigger",
        `${w.name}: 전투 시간은 0 이상 정수 밀리초이며 대체 시각을 사용하지 않습니다.`,
      );
    if ("slotId" in t && !d.slots.some((slot) => slot.id === t.slotId))
      add(
        "SLOT_REFERENCE",
        s,
        "trigger",
        `${w.name}: 임시 개체 슬롯을 지정하세요.`,
      );
    if (t.kind === "hp" && (t.percent < 0 || t.percent > 100))
      add("HP_VALUE", s, "trigger", `${w.name}: HP 비율은 0~100%입니다.`);
    if (t.kind === "reach") {
      if (!d.points.some((p) => p.id === t.pointId))
        add(
          "POINT_REFERENCE",
          s,
          "trigger",
          `${w.name}: 이름 있는 지점을 지정하세요.`,
        );
      if (!Number.isInteger(t.radius) || t.radius < 0)
        add(
          "RADIUS_VALUE",
          s,
          "trigger",
          `${w.name}: 도달 반경은 0 이상의 정수 셀입니다.`,
        );
    }
    if (t.kind === "clear") {
      if (!d.waves.some((other) => other.id === t.waveId))
        add(
          "WAVE_REFERENCE",
          s,
          "trigger",
          `${w.name}: 선행 웨이브 참조가 없거나 끊어졌습니다.`,
        );
      if (t.waveId === w.id)
        add("WAVE_SELF", s, "trigger", `${w.name}: 자기 자신을 참조합니다.`);
      const visited = new Set<string>([w.id]);
      let next = t.waveId;
      while (next) {
        if (visited.has(next)) {
          add(
            "WAVE_CYCLE",
            s,
            "trigger",
            `${w.name}: 웨이브 의존 순환이 있습니다. 대체 시각이 있어도 수정하세요.`,
          );
          break;
        }
        visited.add(next);
        const trigger = d.waves.find((o) => o.id === next)?.trigger;
        next = trigger?.kind === "clear" ? trigger.waveId : "";
      }
    }
  }
  return out;
}
