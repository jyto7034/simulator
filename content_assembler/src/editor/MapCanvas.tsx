import { useEffect, useMemo, useRef, useState, type PointerEvent } from "react";
import { Button } from "@/components/ui/button";
import {
  fitMapView,
  cellKey,
  cellPosition,
  cellBorderPath,
  inBounds,
  rectangle,
  screenToMap,
  segmentCells,
  snap,
  TILE,
} from "./coordinates";
import { insertRoutePoint, moveRoutePoint, paint } from "./operations";
import { uid, type Position, type Selection } from "./model";
import type { EditorSession, EditorState } from "./history";
import { toolDefinitions, type Tool } from "./tools";
import { useCanvasTools } from "./useCanvasTools";
import { RadialTools } from "./RadialTools";
import { CanvasTargets } from "./CanvasTargets";
type Gesture = { button: number } & (
  | {
      kind: "paint";
      start: Position;
      last: Position;
      keys: string[];
      tool: Parameters<typeof paint>[2];
      zoneId?: string;
      newZoneKind?: "deploy" | "spawn";
      shape: "brush" | "rectangle";
    }
  | { kind: "move"; selection: NonNullable<Selection>; position: Position }
  | { kind: "pan"; start: Position; origin: Position }
  | {
      kind: "delete-point";
      selection:
        | { kind: "routes"; id: string; pointId: string }
        | { kind: "points"; id: string };
    }
);
export function MapCanvas({
  session,
  state,
  tool: selectedTool,
  setTool,
  shape,
  subdivisions,
  onStatus,
}: {
  session: EditorSession;
  state: EditorState;
  tool: Tool;
  setTool: (tool: Tool) => void;
  shape: "brush" | "rectangle";
  subdivisions: number | null;
  onStatus: (text: string) => void;
}) {
  const { doc, selection, hidden, locked } = state;
  const terrainBorders = useMemo(() => {
    const valid = new Set(doc.valid);
    const keys = { platform: [] as string[], obstacle: [] as string[] };
    for (const [key, terrain] of Object.entries(doc.terrain))
      if (
        terrain !== "floor" &&
        valid.has(key) &&
        inBounds(doc, cellPosition(key))
      )
        keys[terrain].push(key);
    return {
      platform: cellBorderPath(keys.platform),
      obstacle: cellBorderPath(keys.obstacle),
    };
  }, [doc]);
  const svg = useRef<SVGSVGElement>(null);
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState<Position>({ x: 28, y: 28 });
  const [hover, setHover] = useState<Position | null>(null);
  const [gesture, setGesture] = useState<Gesture | null>(null);
  const gestureRef = useRef<Gesture | null>(null);
  const begin = (g: Gesture | null) => {
    gestureRef.current = g;
    setGesture(g);
  };
  const quick = useCanvasTools(
    svg,
    () => !gestureRef.current,
    (next) => {
      if (next === "deploy" || next === "spawn") {
        const zones = doc.zones.filter((zone) => zone.kind === next);
        const zone =
          zones.find(
            (zone) => selection?.kind === "zones" && zone.id === selection.id,
          ) ?? zones[0];
        session.select(zone ? { kind: "zones", id: zone.id } : null);
      } else if (next === "route" && selection?.kind !== "routes") {
        session.select(
          doc.routes[0] ? { kind: "routes", id: doc.routes[0].id } : null,
        );
      }
      setHover(null);
      setTool(next);
    },
  );
  const tool = quick.temporary ?? selectedTool;
  const fit = () => {
    const rect = svg.current?.getBoundingClientRect();
    if (rect) {
      const view = fitMapView(rect, doc);
      setZoom(view.zoom);
      setPan(view.pan);
      onStatus(`셀 좌표: — · ${Math.round(view.zoom * 100)}%`);
    }
  };
  useEffect(() => {
    const el = svg.current;
    if (!el) return;
    const resize = new ResizeObserver(() => {
      const rect = el.getBoundingClientRect();
      if (!rect.width || !rect.height) return;
      const view = fitMapView(rect, { width: doc.width, height: doc.height });
      setZoom(view.zoom);
      setPan(view.pan);
      onStatus(`셀 좌표: — · ${Math.round(view.zoom * 100)}%`);
    });
    resize.observe(el);
    return () => resize.disconnect();
  }, [doc.width, doc.height, onStatus]);
  useEffect(() => {
    const cancel = (e: KeyboardEvent) => {
      if (svg.current?.closest("[hidden]")) return;
      if (
        e.key !== "Escape" ||
        e.isComposing ||
        e.defaultPrevented ||
        (e.target instanceof Element &&
          e.target.closest(
            'input, textarea, select, [contenteditable], [role="dialog"], [role="menu"]',
          )) ||
        document.querySelector('[role="dialog"], [role="menu"]')
      )
        return;
      if (gestureRef.current) {
        e.preventDefault();
        reset();
      } else if (session.getSnapshot().selection?.kind === "cell") {
        e.preventDefault();
        session.select(null);
      }
    };
    const reset = () => {
      gestureRef.current = null;
      setGesture(null);
    };
    window.addEventListener("keydown", cancel);
    window.addEventListener("editor-cancel", reset);
    return () => {
      window.removeEventListener("keydown", cancel);
      window.removeEventListener("editor-cancel", reset);
    };
  }, [session]);
  const local = (e: { clientX: number; clientY: number }) => {
    const rect = svg.current!.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
  };
  const map = (e: { clientX: number; clientY: number }) =>
    screenToMap(local(e), pan, zoom);
  const selectedRoute =
    selection?.kind === "routes"
      ? doc.routes.find((r) => r.id === selection.id)
      : undefined;
  const down = (e: PointerEvent<SVGSVGElement>) => {
    if (quick.menu || gestureRef.current) return;
    if (e.button !== 0 && e.button !== 1 && e.button !== 2) return;
    e.preventDefault();
    e.currentTarget.focus();
    e.currentTarget.setPointerCapture(e.pointerId);
    const p = map(e);
    if ((tool === "hand" && e.button === 0) || e.button === 1) {
      begin({ kind: "pan", start: local(e), origin: pan, button: e.button });
      return;
    }
    if (tool === "hand") return;
    if (!inBounds(doc, p)) {
      session.select(null);
      return;
    }
    if (tool === "select") {
      if (e.button === 2) return;
      const hit = (e.target as Element).closest<SVGElement>("[data-object]");
      let s: Selection = { kind: "cell", key: cellKey(p) };
      if (hit?.dataset.object === "routes")
        s = { kind: "routes", id: hit.dataset.id!, pointId: hit.dataset.point };
      if (hit?.dataset.object === "points")
        s = { kind: "points", id: hit.dataset.id! };
      if (hit?.dataset.object === "zones")
        s = { kind: "zones", id: hit.dataset.id! };
      session.select(s);
      if (
        ((s.kind === "routes" && s.pointId) || s.kind === "points") &&
        !locked.has(s.kind)
      )
        begin({
          kind: "move",
          selection: s,
          position: snap(p, subdivisions),
          button: e.button,
        });
      return;
    }
    if (tool === "route") {
      if (hidden.has("routes") || locked.has("routes")) {
        session.message("경로 레이어를 표시하고 잠금을 해제하세요.");
        return;
      }
      if (e.button === 2) {
        const hit = (e.target as Element).closest<SVGElement>(
          '[data-object="routes"][data-point]',
        );
        if (hit)
          begin({
            kind: "delete-point",
            button: e.button,
            selection: {
              kind: "routes",
              id: hit.dataset.id!,
              pointId: hit.dataset.point!,
            },
          });
        return;
      }
      const point = { id: uid(), ...snap(p, subdivisions) };
      const routeId = selectedRoute?.id ?? uid();
      if (
        session.commit((d) => {
          insertRoutePoint(
            d,
            routeId,
            point,
            selection?.kind === "routes" ? selection.pointId : undefined,
          );
        })
      )
        session.select({
          kind: "routes",
          id: routeId,
          pointId: point.id,
        });
      return;
    }
    if (tool === "point") {
      if (hidden.has("points") || locked.has("points")) return;
      if (e.button === 2) {
        const hit = (e.target as Element).closest<SVGElement>(
          '[data-object="points"]',
        );
        if (hit)
          begin({
            kind: "delete-point",
            button: e.button,
            selection: { kind: "points", id: hit.dataset.id! },
          });
        return;
      }
      const point = {
        id: uid(),
        name: `지점 ${doc.points.length + 1}`,
        ...snap(p, subdivisions),
      };
      if (
        session.commit((d) => {
          d.points.push(point);
        })
      )
        session.select({ kind: "points", id: point.id });
      return;
    }
    const zoneKind = tool === "deploy" || tool === "spawn" ? tool : undefined;
    const basePaint = tool === "deploy" || tool === "spawn" ? "zone-add" : tool;
    const paintTool =
      e.button !== 2
        ? basePaint
        : basePaint.startsWith("zone")
          ? "zone-remove"
          : basePaint === "add" || basePaint === "remove"
            ? "remove"
            : "erase";
    const layer = paintTool.startsWith("zone")
      ? "zones"
      : tool === "add" || tool === "remove"
        ? "area"
        : "terrain";
    if (locked.has(layer) || hidden.has(layer)) {
      session.message("편집할 레이어를 표시하고 잠금을 해제하세요.");
      return;
    }
    const zoneId =
      selection?.kind === "zones" &&
      doc.zones.some(
        (zone) =>
          zone.id === selection.id && (!zoneKind || zone.kind === zoneKind),
      )
        ? selection.id
        : undefined;
    if (
      paintTool.startsWith("zone") &&
      !zoneId &&
      (!zoneKind || paintTool === "zone-remove")
    ) {
      session.message("구역을 만들거나 선택한 뒤 칠하세요.");
      return;
    }
    begin({
      kind: "paint",
      button: e.button,
      shape: e.shiftKey ? "rectangle" : shape,
      start: p,
      last: p,
      keys: [cellKey(p)],
      tool: paintTool,
      zoneId,
      newZoneKind: !zoneId && paintTool === "zone-add" ? zoneKind : undefined,
    });
  };
  const move = (e: PointerEvent<SVGSVGElement>) => {
    const p = map(e);
    setHover(inBounds(doc, p) ? p : null);
    onStatus(
      `셀 ${Math.floor(p.x)}, ${Math.floor(p.y)} · 좌표 ${p.x.toFixed(2)}, ${p.y.toFixed(2)} · ${Math.round(zoom * 100)}%`,
    );
    const g = gestureRef.current;
    if (!g) return;
    if (!(e.buttons & (g.button === 0 ? 1 : g.button === 1 ? 4 : 2))) {
      begin(null);
      return;
    }
    if (g.kind === "pan") {
      const at = local(e);
      setPan({
        x: g.origin.x + at.x - g.start.x,
        y: g.origin.y + at.y - g.start.y,
      });
    } else if (g.kind === "move" && inBounds(doc, p))
      begin({ ...g, position: snap(p, subdivisions) });
    else if (g.kind === "paint") {
      if (g.shape !== "rectangle" && !inBounds(doc, p)) return;
      const end = {
        x: Math.max(0, Math.min(doc.width - 0.001, p.x)),
        y: Math.max(0, Math.min(doc.height - 0.001, p.y)),
      };
      begin({
        ...g,
        last: end,
        keys:
          g.shape === "rectangle"
            ? rectangle(g.start, end)
            : [...new Set([...g.keys, ...segmentCells(g.last, p)])],
      });
    }
  };
  const up = (e: PointerEvent<SVGSVGElement>) => {
    const g = gestureRef.current;
    begin(null);
    if (e.currentTarget.hasPointerCapture(e.pointerId))
      e.currentTarget.releasePointerCapture(e.pointerId);
    if (g && e.button !== g.button) return;
    if (g?.kind === "delete-point")
      session.commit((d) => {
        const s = g.selection;
        if (s.kind === "routes") {
          const route = d.routes.find((route) => route.id === s.id);
          if (route)
            route.points = route.points.filter(
              (point) => point.id !== s.pointId,
            );
        } else d.points = d.points.filter((point) => point.id !== s.id);
      });
    if (g?.kind === "paint") {
      const zoneId = g.zoneId ?? (g.newZoneKind ? uid() : undefined);
      const committed = session.commit((d) => {
        if (g.newZoneKind && zoneId) {
          if (!g.keys.some((key) => d.valid.includes(key))) return;
          d.zones.push({
            id: zoneId,
            name: `${g.newZoneKind === "deploy" ? "배치" : "등장"} 구역 ${d.zones.length + 1}`,
            kind: g.newZoneKind,
            cells: [],
          });
        }
        paint(d, g.keys, g.tool, zoneId);
      });
      if (committed && g.newZoneKind && zoneId)
        session.select({ kind: "zones", id: zoneId });
    }
    if (g?.kind === "move")
      session.commit((d) => {
        const s = g.selection;
        if (s.kind === "routes" && s.pointId)
          moveRoutePoint(d, s.id, s.pointId, g.position);
        else if (s.kind === "points") {
          const point = d.points.find((p) => p.id === s.id);
          if (point) Object.assign(point, g.position);
        }
      });
  };
  const shownPosition = (
    p: Position,
    kind: "routes" | "points",
    id: string,
    pointId?: string,
  ) =>
    gesture?.kind === "move" &&
    gesture.selection.kind === kind &&
    gesture.selection.id === id &&
    (kind === "points" || gesture.selection.pointId === pointId)
      ? gesture.position
      : p;
  const selected = (kind: string, id: string) =>
    selection?.kind === kind && "id" in selection && selection.id === id;
  const zoomTo = (z: number, anchor?: Position) => {
    const next = Math.max(0.1, Math.min(4, z));
    const a = anchor ?? { x: 28, y: 28 };
    setPan({
      x: a.x - ((a.x - pan.x) * next) / zoom,
      y: a.y - ((a.y - pan.y) * next) / zoom,
    });
    setZoom(next);
    setHover(null);
    onStatus(`셀 좌표: — · ${Math.round(next * 100)}%`);
  };
  const valid = new Set(doc.valid);
  const stroke = gesture?.kind === "paint" ? gesture : null;
  const previewKeys =
    stroke?.keys.filter(
      (key) =>
        ["add", "remove", "zone-remove"].includes(stroke.tool) ||
        valid.has(key),
    ) ?? [];
  const removing =
    !!stroke && ["remove", "erase", "zone-remove"].includes(stroke.tool);
  const previewLabel = stroke
    ? `${toolDefinitions.find((entry) => entry.id === stroke.tool)?.label}${removing ? "" : " 적용"} · ${previewKeys.length}셀`
    : "";
  return (
    <section className="canvas-panel" aria-label="전장 캔버스">
      <div className="canvas-bar">
        <h2>전장</h2>
        <span>
          {doc.width} × {doc.height} · 유효 {doc.valid.length}셀
        </span>
        <div className="spacer" />
        <Button
          variant="outline"
          size="sm"
          aria-label="빠른 도구 메뉴 (Q)"
          aria-haspopup="menu"
          aria-expanded={!!quick.menu}
          onClick={quick.openFromButton}
        >
          도구 Q
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => zoomTo(zoom / 1.25)}
          aria-label="축소"
        >
          −
        </Button>
        <output aria-label="캔버스 확대율">{Math.round(zoom * 100)}%</output>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => zoomTo(zoom * 1.25)}
          aria-label="확대"
        >
          +
        </Button>
        <Button variant="outline" size="sm" onClick={fit}>
          화면 맞춤
        </Button>
      </div>
      <CanvasTargets
        tool={tool}
        state={state}
        session={session}
        setTool={setTool}
        temporary={!!quick.temporary}
      />
      <svg
        ref={svg}
        className="map-canvas"
        role="application"
        aria-label="전투 맵 격자. Q를 누른 채 도구 선택, Space 화면 이동, D 임시 지우기."
        tabIndex={0}
        onPointerDown={down}
        onContextMenu={(e) => e.preventDefault()}
        onPointerMove={move}
        onPointerUp={up}
        onPointerCancel={() => begin(null)}
        onLostPointerCapture={() => begin(null)}
        onPointerLeave={() => {
          if (!gestureRef.current) setHover(null);
        }}
        onWheel={(e) => zoomTo(zoom * (e.deltaY < 0 ? 1.1 : 1 / 1.1), local(e))}
      >
        <defs>
          <pattern id="void" width="8" height="8" patternUnits="userSpaceOnUse">
            <path d="M0 8L8 0" stroke="var(--border)" strokeWidth="1" />
          </pattern>
          <marker
            id="arrow"
            viewBox="0 0 10 10"
            refX="8"
            refY="5"
            markerWidth="5"
            markerHeight="5"
            orient="auto-start-reverse"
          >
            <path d="M0 0L10 5L0 10Z" fill="var(--route-color)" />
          </marker>
        </defs>
        <g transform={`translate(${pan.x} ${pan.y}) scale(${zoom})`}>
          {Array.from({ length: doc.height }, (_, y) =>
            Array.from({ length: doc.width }, (_, x) => {
              const key = `${x},${y}`;
              const terrain = !hidden.has("terrain")
                ? (doc.terrain[key] ?? "floor")
                : "floor";
              return (
                <rect
                  key={key}
                  data-cell={key}
                  x={x * TILE}
                  y={y * TILE}
                  width={TILE}
                  height={TILE}
                  className={`map-cell ${valid.has(key) && !hidden.has("area") ? terrain : "void-cell"}`}
                />
              );
            }),
          )}
          {!hidden.has("terrain") && !hidden.has("area") && (
            <g className="terrain-borders" aria-hidden="true">
              <path className="obstacle-border" d={terrainBorders.obstacle} />
              <path className="platform-border" d={terrainBorders.platform} />
            </g>
          )}
          {Array.from({ length: doc.width }, (_, x) => (
            <text
              key={`x${x}`}
              className="axis-label"
              x={(x + 0.5) * TILE}
              y={-10}
              textAnchor="middle"
            >
              {x}
            </text>
          ))}
          {Array.from({ length: doc.height }, (_, y) => (
            <text
              key={`y${y}`}
              className="axis-label"
              x={-12}
              y={(y + 0.65) * TILE}
              textAnchor="middle"
            >
              {y}
            </text>
          ))}
          {!hidden.has("zones") &&
            doc.zones.map((z) => (
              <g
                key={z.id}
                data-object="zones"
                data-id={z.id}
                className={`zone ${z.kind} ${selected("zones", z.id) ? "is-selected" : ""}`}
              >
                {z.cells.map((k) => {
                  const p = cellPosition(k);
                  return (
                    inBounds(doc, p) && (
                      <rect
                        key={k}
                        x={p.x * TILE + 3}
                        y={p.y * TILE + 3}
                        width={TILE - 6}
                        height={TILE - 6}
                      />
                    )
                  );
                })}
              </g>
            ))}
          {!hidden.has("routes") &&
            doc.routes.map((r) => (
              <g
                key={r.id}
                className={`route ${selected("routes", r.id) ? "is-selected" : ""}`}
                data-object="routes"
                data-id={r.id}
              >
                {r.points.slice(1).map((p, i) => {
                  const a = shownPosition(
                    r.points[i],
                    "routes",
                    r.id,
                    r.points[i].id,
                  );
                  const b = shownPosition(p, "routes", r.id, p.id);
                  return (
                    <line
                      key={p.id}
                      x1={a.x * TILE}
                      y1={a.y * TILE}
                      x2={b.x * TILE}
                      y2={b.y * TILE}
                      markerEnd="url(#arrow)"
                    />
                  );
                })}
                {r.points.map((p, i) => {
                  const at = shownPosition(p, "routes", r.id, p.id);
                  return (
                    <g
                      key={p.id}
                      data-object="routes"
                      data-id={r.id}
                      data-point={p.id}
                    >
                      <circle
                        cx={at.x * TILE}
                        cy={at.y * TILE}
                        r={
                          selection?.kind === "routes" &&
                          selection.pointId === p.id
                            ? 7
                            : 5
                        }
                      />
                      <text x={at.x * TILE + 8} y={at.y * TILE - 7}>
                        {i === 0
                          ? "시작"
                          : i === r.points.length - 1
                            ? "도착"
                            : i + 1}
                      </text>
                    </g>
                  );
                })}
              </g>
            ))}
          {!hidden.has("points") &&
            doc.points.map((p) => {
              const at = shownPosition(p, "points", p.id);
              return (
                <g
                  key={p.id}
                  data-object="points"
                  data-id={p.id}
                  className={`landmark ${selected("points", p.id) ? "is-selected" : ""}`}
                >
                  <path
                    d={`M${at.x * TILE} ${at.y * TILE - 8}l8 8 -8 8 -8 -8Z`}
                  />
                  <text x={at.x * TILE + 10} y={at.y * TILE}>
                    {p.name}
                  </text>
                </g>
              );
            })}
          {stroke &&
            previewKeys.map((k) => {
              const p = cellPosition(k);
              return (
                <rect
                  key={k}
                  className={
                    removing ? "paint-preview remove-preview" : "paint-preview"
                  }
                  x={p.x * TILE + 1}
                  y={p.y * TILE + 1}
                  width={TILE - 2}
                  height={TILE - 2}
                />
              );
            })}
          {stroke?.shape === "rectangle" && (
            <rect
              className={
                removing ? "paint-bounds remove-preview" : "paint-bounds"
              }
              x={Math.floor(Math.min(stroke.start.x, stroke.last.x)) * TILE}
              y={Math.floor(Math.min(stroke.start.y, stroke.last.y)) * TILE}
              width={
                (Math.abs(
                  Math.floor(stroke.start.x) - Math.floor(stroke.last.x),
                ) +
                  1) *
                TILE
              }
              height={
                (Math.abs(
                  Math.floor(stroke.start.y) - Math.floor(stroke.last.y),
                ) +
                  1) *
                TILE
              }
            />
          )}
          {hover && (
            <g pointerEvents="none">
              <rect
                className="hover-cell"
                x={Math.floor(hover.x) * TILE}
                y={Math.floor(hover.y) * TILE}
                width={TILE}
                height={TILE}
              />
              {subdivisions &&
                Array.from({ length: subdivisions - 1 }, (_, i) => (
                  <path
                    className="subgrid"
                    key={i}
                    d={`M${(Math.floor(hover.x) + (i + 1) / subdivisions) * TILE} ${Math.floor(hover.y) * TILE}v${TILE}M${Math.floor(hover.x) * TILE} ${(Math.floor(hover.y) + (i + 1) / subdivisions) * TILE}h${TILE}`}
                  />
                ))}
              {(tool === "route" || tool === "point") && (
                <circle
                  className="snap-preview"
                  cx={snap(hover, subdivisions).x * TILE}
                  cy={snap(hover, subdivisions).y * TILE}
                  r={4}
                />
              )}
            </g>
          )}
          {selection?.kind === "cell" && (
            <rect
              data-selected-cell={selection.key}
              pointerEvents="none"
              className="hover-cell"
              x={cellPosition(selection.key).x * TILE}
              y={cellPosition(selection.key).y * TILE}
              width={TILE}
              height={TILE}
            />
          )}
        </g>
      </svg>
      {stroke && (
        <div
          className="paint-status"
          role="status"
          aria-label="범위 편집 미리보기"
        >
          {previewLabel} · Esc 취소
        </div>
      )}
      <div className="canvas-legend">
        <span>□ 바닥</span>
        <span>▤ 고지대</span>
        <span>▨ 장애물</span>
        <span className="deploy-legend">▦ 아군 배치</span>
        <span className="spawn-legend">▦ 적 등장</span>
        <span>빗금: 영역 없음</span>
      </div>
      {quick.menu && (
        <RadialTools
          menu={quick.menu}
          menuRef={quick.menuElement}
          choose={(next) => quick.close(next)}
          cancel={() => quick.close()}
          highlight={quick.highlight}
        />
      )}
    </section>
  );
}
