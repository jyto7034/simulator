import { test, expect } from "@playwright/test";
import { fixture } from "./fixtures";
import { EditorSession } from "../../src/editor/history";
import {
  paint,
  duplicateWave,
  removeObject,
  setZoneKind,
  insertRoutePoint,
  moveRoutePoint,
  resizeMap,
} from "../../src/editor/operations";
import {
  snap,
  screenToMap,
  segmentCells,
  inBounds,
} from "../../src/editor/coordinates";
import {
  parseDocument,
  serializeDocument,
} from "../../src/editor/serialization";
import { diagnose } from "../../src/editor/diagnostics";
import { DocumentFiles } from "../../src/editor/files";

test("map expansion enables new rows and columns atomically while preserving holes and content", () => {
  const doc = fixture("irregular");
  doc.valid.push("9,1");
  doc.terrain["9,1"] = "obstacle";
  const s = new EditorSession(doc);
  s.commit((d) => resizeMap(d, "width", 10));
  let d = s.getSnapshot().doc;
  expect(d.valid).toHaveLength(doc.valid.length + 11);
  expect(d.valid).toEqual(expect.arrayContaining(["8,0", "9,5"]));
  expect(d.valid).not.toContain("3,3");
  expect(d.terrain["9,1"]).toBe("obstacle");
  expect(s.getSnapshot().past).toHaveLength(1);
  s.travel("undo");
  expect(s.getSnapshot().doc).toEqual(doc);
  s.travel("redo");
  s.commit((d) => resizeMap(d, "height", 8));
  d = s.getSnapshot().doc;
  expect(d.valid).toEqual(expect.arrayContaining(["0,6", "9,7"]));
  expect(d.valid).not.toContain("3,3");
  s.commit((d) => paint(d, ["9,7"], "platform"));
  expect(s.getSnapshot().doc.terrain["9,7"]).toBe("platform");
  const expanded = s.getSnapshot().doc;
  s.commit((d) => resizeMap(d, "height", 4));
  expect(s.getSnapshot().doc.valid).toEqual(expanded.valid);
  expect(s.getSnapshot().doc.terrain).toEqual(expanded.terrain);
  s.travel("undo");
  expect(s.getSnapshot().doc).toEqual(expanded);
  expect(parseDocument(serializeDocument(expanded))).toEqual(expanded);
});

test("map size validation, area locks, maximum expansion and no-op preserve history", () => {
  const s = new EditorSession(fixture());
  const original = s.getSnapshot().doc;
  for (const size of [0, -1, 1.5, 129, 1e9, NaN, Infinity])
    expect(s.commit((d) => resizeMap(d, "width", size))).toBe(false);
  expect(s.getSnapshot().doc).toEqual(original);
  expect(s.getSnapshot().past).toHaveLength(0);
  s.toggleLayer("area", "locked");
  expect(s.commit((d) => resizeMap(d, "height", 8))).toBe(false);
  expect(s.getSnapshot().doc).toEqual(original);
  s.toggleLayer("area", "locked");
  s.commit((d) => {
    resizeMap(d, "width", 128);
    resizeMap(d, "height", 128);
  });
  expect(s.getSnapshot().doc.valid).toHaveLength(128 * 128);
  s.travel("undo");
  expect(s.commit((d) => resizeMap(d, "width", 8))).toBe(false);
  expect(s.getSnapshot().future).toHaveLength(1);
  s.travel("redo");
  expect(s.getSnapshot().doc.valid).toContain("127,127");
});

test("route cells reject duplicate insertion and movement without changing history or redo", () => {
  const doc = fixture();
  const s = new EditorSession(doc);
  s.acknowledgeExport(doc, 0);
  const add = (x: number, y: number) =>
    s.commit((d) =>
      insertRoutePoint(d, "route-main", { id: "new-point", x, y }),
    );
  expect(add(0.5, 2.5)).toBe(false);
  expect(add(0.25, 2.75)).toBe(false);
  expect(s.isDirty()).toBe(false);
  expect(s.getSnapshot().past).toHaveLength(0);
  expect(add(1.25, 2.75)).toBe(true);
  s.travel("undo");
  expect(add(6.75, 2.25)).toBe(false);
  expect(s.getSnapshot().future).toHaveLength(1);
  s.travel("redo");
  const before = s.getSnapshot().doc;
  expect(
    s.commit((d) =>
      moveRoutePoint(d, "route-main", "new-point", { x: 6.25, y: 2.75 }),
    ),
  ).toBe(false);
  expect(s.getSnapshot().doc).toEqual(before);
  expect(s.getSnapshot().past).toHaveLength(1);
  // A point can move within its own cell, and another route can share that cell.
  expect(
    s.commit((d) =>
      moveRoutePoint(d, "route-main", "new-point", { x: 1.75, y: 2.25 }),
    ),
  ).toBe(true);
  expect(
    s.commit((d) =>
      insertRoutePoint(d, "another-route", {
        id: "another-point",
        x: 1.75,
        y: 2.25,
      }),
    ),
  ).toBe(true);
  expect(
    diagnose(s.getSnapshot().doc).some((d) => d.code === "ROUTE_POINT_OVERLAP"),
  ).toBe(false);
  s.travel("undo");
  expect(s.getSnapshot().doc.routes).toHaveLength(1);
  s.toggleLayer("routes", "locked");
  expect(
    s.commit((d) =>
      moveRoutePoint(d, "route-main", "new-point", { x: 3.5, y: 2.5 }),
    ),
  ).toBe(false);
});

test("imported route overlaps remain editable and diagnosable without losing points or references", () => {
  const doc = fixture();
  doc.routes[0].points.push({ id: "duplicate-point", x: 0.25, y: 2.75 });
  const parsed = parseDocument(serializeDocument(doc));
  expect(parsed).toEqual(doc);
  expect(
    diagnose(parsed)
      .filter((d) => d.code === "ROUTE_POINT_OVERLAP")
      .map((d) => d.selection),
  ).toEqual([{ kind: "routes", id: "route-main", pointId: "duplicate-point" }]);
  const s = new EditorSession(parsed);
  s.commit((d) =>
    moveRoutePoint(d, "route-main", "duplicate-point", { x: 1.25, y: 2.75 }),
  );
  expect(
    diagnose(s.getSnapshot().doc).some((d) => d.code === "ROUTE_POINT_OVERLAP"),
  ).toBe(false);
  expect(s.getSnapshot().doc.waves).toEqual(doc.waves);
  s.travel("undo");
  expect(s.getSnapshot().doc).toEqual(doc);
});

test("opposite zone kinds are replaced atomically while same-kind membership and references survive", () => {
  const doc = fixture();
  doc.zones.push({
    id: "deploy-2",
    name: "두 번째 배치",
    kind: "deploy",
    cells: ["6,2"],
  });
  const session = new EditorSession(doc);
  session.commit((d) => paint(d, ["0,2", "6,2"], "zone-add", "zone-deploy"));
  expect(session.getSnapshot().doc.zones[1].cells).toEqual(["0,3"]);
  expect(session.getSnapshot().doc.zones[2].cells).toEqual(["6,2"]);
  expect(session.getSnapshot().doc.waves[0].zoneId).toBe("zone-spawn");
  session.travel("undo");
  expect(session.getSnapshot().doc).toEqual(doc);
  session.travel("redo");
  session.commit((d) => setZoneKind(d, "zone-deploy", "spawn"));
  expect(session.getSnapshot().doc.zones[2].cells).toEqual([]);
  expect(
    diagnose(session.getSnapshot().doc).some(
      (d) => d.code === "ZONE_KIND_OVERLAP",
    ),
  ).toBe(false);
  const before = session.getSnapshot().doc;
  session.toggleLayer("zones", "locked");
  expect(session.commit((d) => setZoneKind(d, "zone-deploy", "deploy"))).toBe(
    false,
  );
  expect(session.getSnapshot().doc).toEqual(before);
});

test("imported overlap is diagnosed without data loss and repainting an existing cell resolves it", () => {
  const doc = fixture();
  doc.zones[1].cells.push("6,2");
  const parsed = parseDocument(serializeDocument(doc));
  expect(parsed.zones).toEqual(doc.zones);
  expect(
    diagnose(parsed)
      .filter((d) => d.code === "ZONE_KIND_OVERLAP")
      .map((d) => d.targetId)
      .sort(),
  ).toEqual(["zone-deploy", "zone-spawn"]);
  const session = new EditorSession(parsed);
  session.commit((d) => paint(d, ["6,2"], "zone-add", "zone-deploy"));
  expect(session.getSnapshot().doc.zones[1].cells).toEqual(["0,2", "0,3"]);
  session.travel("undo");
  expect(session.getSnapshot().doc.zones).toEqual(doc.zones);
});

test("one stroke, no-op, undo branching, saved baseline and stable identity", () => {
  const s = new EditorSession(fixture());
  s.acknowledgeExport(s.getSnapshot().doc, 0);
  s.commit((d) => paint(d, ["2,2", "3,2"], "obstacle"));
  expect(s.getSnapshot().past).toHaveLength(1);
  s.travel("undo");
  expect(s.isDirty()).toBe(false);
  s.commit(() => {});
  expect(s.getSnapshot().future).toHaveLength(1);
  s.travel("redo");
  expect(s.getSnapshot().doc.terrain["3,2"]).toBe("obstacle");
  s.travel("undo");
  s.commit((d) => {
    d.description = "분기";
  });
  expect(s.getSnapshot().future).toHaveLength(0);
});

test("terrain erase is not area removal; holes, disconnected cells and resize preserve references", () => {
  const s = new EditorSession(fixture());
  s.commit((d) => paint(d, ["6,2"], "erase"));
  expect(s.getSnapshot().doc.valid).toContain("6,2");
  expect(s.getSnapshot().doc.terrain["6,2"]).toBeUndefined();
  s.commit((d) => paint(d, ["3,3", "4,3", "0,0"], "remove"));
  expect(s.getSnapshot().doc.valid).not.toContain("3,3");
  s.commit((d) => {
    d.width = 4;
  });
  expect(s.getSnapshot().doc.routes[0].points[1].x).toBe(6.5);
  expect(s.getSnapshot().doc.zones[0].cells).toContain("6,2");
  expect(diagnose(s.getSnapshot().doc).map((d) => d.code)).toEqual(
    expect.arrayContaining([
      "AREA_BOUNDS",
      "ZONE_OUTSIDE",
      "ROUTE_POINT_OUTSIDE",
    ]),
  );
});

test("locked layers guard every mutation and history; hiding remains independent", () => {
  const s = new EditorSession(fixture());
  s.commit((d) => paint(d, ["1,1"], "platform"));
  s.toggleLayer("terrain", "locked");
  expect(s.commit((d) => paint(d, ["1,1"], "erase"))).toBe(false);
  s.travel("undo");
  expect(s.getSnapshot().doc.terrain["1,1"]).toBe("platform");
  s.toggleLayer("terrain", "locked");
  s.toggleLayer("terrain", "hidden");
  s.commit((d) => paint(d, ["1,1"], "erase"));
  expect(s.getSnapshot().doc.terrain["1,1"]).toBeUndefined();
  for (const layer of [
    "area",
    "zones",
    "routes",
    "points",
    "slots",
    "waves",
  ] as const) {
    s.toggleLayer(layer, "locked");
    const before = serializeDocument(s.getSnapshot().doc);
    s.commit((d) => {
      if (layer === "area") d.width = 3;
      else d[layer].splice(0, 1);
    });
    expect(serializeDocument(s.getSnapshot().doc)).toBe(before);
  }
});

test("coordinates: half-open bounds, zoom/pan inversion, internal intersections and center", () => {
  expect(screenToMap({ x: 200, y: 120 }, { x: 40, y: 24 }, 2)).toEqual({
    x: 2.5,
    y: 1.5,
  });
  expect(snap({ x: 2.01, y: 3.99 }, null)).toEqual({ x: 2.5, y: 3.5 });
  expect(snap({ x: 2.01, y: 3.99 }, 4)).toEqual({ x: 2.25, y: 3.75 });
  expect(snap({ x: 2.5, y: 3.5 }, 8)).toEqual({ x: 2.5, y: 3.5 });
  expect(inBounds(fixture(), { x: 8, y: 1 })).toBe(false);
  expect(segmentCells({ x: 0.5, y: 0.5 }, { x: 3.5, y: 0.5 })).toEqual([
    "0,0",
    "3,0",
    "1,0",
    "2,0",
  ]);
});

test("all five conditions, fixtures and incomplete references survive roundtrip", () => {
  for (const kind of [
    "small",
    "irregular",
    "many",
    "empty",
    "incomplete",
  ] as const) {
    const d = fixture(kind);
    expect(parseDocument(serializeDocument(d))).toEqual(d);
  }
  expect(new Set(fixture("many").waves.map((w) => w.trigger.kind)).size).toBe(
    5,
  );
});

test("duplicate IDs, unsupported versions, invalid structures and nonfinite values reject without mutation", () => {
  const d = fixture();
  const candidates: unknown[] = [
    { ...d, version: 2 },
    { ...d, zones: [d.zones[0], d.zones[0]] },
    { ...d, extra: true },
    { ...d, width: 0 },
    { ...d, terrain: { "1,1": "water" } },
    { ...d, points: [{ id: "x", name: "x", x: null, y: 0 }] },
  ];
  for (const bad of candidates)
    expect(() => parseDocument(JSON.stringify(bad))).toThrow();
  expect(() => parseDocument("{oops")).toThrow();
  const s = new EditorSession(d);
  expect(
    s.commit((next) => {
      next.width = NaN;
    }),
  ).toBe(false);
  expect(s.getSnapshot().doc).toEqual(d);
  expect(s.getSnapshot().past).toHaveLength(0);
});

test("duplicate wave renews owned IDs and only remaps internal self reference; delete/undo restores selection validity", () => {
  const s = new EditorSession(fixture());
  s.commit((d) => {
    d.waves[0].trigger = { kind: "clear", waveId: "wave-first" };
    duplicateWave(d, "wave-first");
  });
  const [a, b] = s.getSnapshot().doc.waves;
  expect(a.id).not.toBe(b.id);
  expect(a.enemies[0].id).not.toBe(b.enemies[0].id);
  expect(b.zoneId).toBe(a.zoneId);
  expect(b.trigger).toEqual({ kind: "clear", waveId: b.id });
  s.select({ kind: "routes", id: "route-main", pointId: "route-point-a" });
  s.commit((d) => removeObject(d, "routes", "route-main"));
  expect(s.getSnapshot().selection).toBeNull();
  expect(
    diagnose(s.getSnapshot().doc).some((d) => d.code === "ROUTE_REFERENCE"),
  ).toBe(true);
  s.travel("undo");
  expect(s.getSnapshot().doc.routes[0].id).toBe("route-main");
});

test("diagnostics distinguish missing refs, cycles, invalid values and geometric warnings", () => {
  const d = fixture();
  d.waves[0].trigger = { kind: "clear", waveId: "wave-second" };
  const b = structuredClone(d.waves[0]);
  b.id = "wave-second";
  b.enemies[0].id = "enemy-second";
  b.trigger = { kind: "clear", waveId: "wave-first" };
  d.waves.push(b);
  d.waves[0].enemies[0].count = -1;
  d.waves[0].fallbackMs = -2;
  d.terrain["3,2"] = "obstacle";
  expect(diagnose(d).map((i) => i.code)).toEqual(
    expect.arrayContaining([
      "WAVE_CYCLE",
      "ENEMY_COUNT",
      "FALLBACK_VALUE",
      "SEGMENT_OBSTACLE",
      "CORE_UNBOUND",
    ]),
  );
  d.waves[0].trigger = { kind: "hp", slotId: "missing", percent: 101 };
  expect(diagnose(d).map((i) => i.code)).toEqual(
    expect.arrayContaining(["HP_VALUE", "SLOT_REFERENCE"]),
  );
  expect(diagnose(fixture("incomplete")).map((i) => i.code)).toEqual(
    expect.arrayContaining([
      "RADIUS_VALUE",
      "POINT_REFERENCE",
      "ROUTE_INCOMPLETE",
    ]),
  );
});
function deferred<T>() {
  let resolve!: (v: T) => void;
  let reject!: (e: Error) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}

test("save failure, overlap, editing during export, old completion and explicit file acknowledgement", async () => {
  const s = new EditorSession(fixture());
  const pending = deferred<void>();
  let calls = 0;
  const files = new DocumentFiles(s, async () => {
    calls++;
    await pending.promise;
  });
  const saving = files.save();
  s.commit((d) => {
    d.name = "저장 중 새 편집";
  });
  await files.save();
  expect(calls).toBe(1);
  pending.resolve();
  const exported = await saving;
  expect(s.isDirty()).toBe(true);
  s.acknowledgeExport(exported!.doc, exported!.epoch);
  expect(s.isDirty()).toBe(true);
  s.travel("undo");
  expect(s.isDirty()).toBe(false);
  const failure = new DocumentFiles(s, async () => {
    throw new Error("disk denied");
  });
  s.commit((d) => {
    d.description = "보존";
  });
  await failure.save();
  expect(s.isDirty()).toBe(true);
  expect(s.getSnapshot().message).toContain("저장 실패");
  const late = deferred<void>();
  const old = new DocumentFiles(s, async () => late.promise).save();
  s.replace(fixture("empty"), false);
  late.resolve();
  const oldResult = await old;
  s.acknowledgeExport(oldResult!.doc, oldResult!.epoch);
  expect(s.isDirty()).toBe(true);
  expect(s.getSnapshot().doc.valid).toEqual([]);
});

test("file reading validates before replacement and rejects stale or reversed results", async () => {
  const s = new EditorSession(fixture());
  const files = new DocumentFiles(s, async () => {});
  const before = serializeDocument(s.getSnapshot().doc);
  await expect(
    files.read({ size: 2, text: async () => "{}" }),
  ).rejects.toThrow();
  expect(serializeDocument(s.getSnapshot().doc)).toBe(before);
  const first = deferred<string>();
  const read = files.read({ size: 1, text: () => first.promise });
  s.commit((d) => {
    d.name = "수정";
  });
  first.resolve(before);
  await expect(read).rejects.toThrow("작업이 바뀌");
  const older = deferred<string>();
  const p = files.read({ size: 1, text: () => older.promise });
  const latest = await files.read({
    size: 1,
    text: async () => before,
  });
  expect(latest.doc.name).toBe(fixture().name);
  older.resolve(before);
  await expect(p).rejects.toThrow();
});

test("unordered imported cell sets do not create no-op history or false locked-layer changes", () => {
  const d = fixture();
  d.valid.reverse();
  d.zones[0].cells.reverse();
  const s = new EditorSession(d);
  s.commit((next) => paint(next, ["1,1"], "add"));
  expect(s.getSnapshot().past).toHaveLength(0);
  s.toggleLayer("area", "locked");
  expect(s.commit((next) => paint(next, ["1,1"], "platform"))).toBe(true);
  s.commit((next) => paint(next, ["6,2"], "zone-add", "zone-deploy"));
  expect(s.getSnapshot().past).toHaveLength(1);
});

test("UTF-8 export size uses the same limit as reopening; oversized download fails before output", async () => {
  const d = fixture();
  d.slots = Array.from({ length: 200 }, (_, i) => ({
    id: `slot-${i}`,
    name: "가".repeat(10000),
  }));
  const s = new EditorSession(d);
  let outputs = 0;
  await new DocumentFiles(s, async () => {
    outputs++;
  }).save();
  expect(outputs).toBe(0);
  expect(s.isDirty()).toBe(true);
  expect(s.getSnapshot().message).toContain("5MB");
  expect(() => parseDocument(JSON.stringify(d))).toThrow("5MB");
});
