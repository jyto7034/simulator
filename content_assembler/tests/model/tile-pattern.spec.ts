import { test, expect } from "@playwright/test";
import { example } from "../../src/nodes/core/examples";
import { GraphSession } from "../../src/nodes/core/session";
import { GraphError } from "../../src/nodes/core/model";
import { parseDocument } from "../../src/nodes/core/validation";
import {
  readTilePattern,
  togglePatternTile,
} from "../../src/nodes/core/tile-pattern";
import { definition } from "../../src/nodes/core/catalog";

const start = () => new GraphSession(example("hello-goodbye"));
const area = (s: GraphSession) =>
  s.getSnapshot().doc.nodes.find((n) => n.kind === "skill-area")!;
const areaDiagnostics = (s: GraphSession) =>
  s.getSnapshot().diagnostics.filter((d) => d.nodeId === area(s).id);

test("tile offsets preserve holes and off-center anchors without geometric approximation", () => {
  let pattern = togglePatternTile("", { x: -2, y: -1 });
  expect(pattern).toBe("X..\n..@");
  pattern = togglePatternTile(pattern, { x: 2, y: 1 });
  expect(pattern).toBe("X....\n..@..\n....X");
  expect(readTilePattern(pattern).cells).toEqual([
    { x: -2, y: -1 },
    { x: 2, y: 1 },
  ]);
  expect(togglePatternTile(pattern, { x: -2, y: -1 })).toBe("@..\n..X");
  expect(togglePatternTile("@", { x: 0, y: 0 })).toBe("@");
  expect(togglePatternTile("@", { x: 8, y: 0 })).toBe("@.......X");
  for (const x of [0.5, Infinity, NaN, 16_384])
    expect(() => togglePatternTile("", { x, y: 0 })).toThrow(GraphError);
});

test("tile authoring is atomic, undoable, copyable and roundtrips with anchor inclusion", () => {
  const s = start(),
    before = s.getSnapshot();
  expect(area(s).values).toEqual({
    pattern: "",
    includeAnchor: false,
    facing: "accepted",
  });
  expect(definition("skill-area").fields.map((f) => f.id)).toEqual([
    "pattern",
    "includeAnchor",
    "facing",
  ]);
  s.edit({
    type: "setNode",
    id: area(s).id,
    values: { pattern: "X.X\n.@.", includeAnchor: true, facing: "accepted" },
  });
  expect(areaDiagnostics(s)).toEqual([]);
  const authored = s.getSnapshot().doc;
  expect(parseDocument(s.export().json)).toEqual(authored);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before.doc);
  s.travel(true);
  expect(s.getSnapshot().doc).toEqual(authored);
  const copied = s.copy([area(s).id]);
  const [pasted] = s.edit({
    type: "paste",
    fragment: copied,
    position: { x: 0, y: 0 },
  });
  expect(
    s.getSnapshot().doc.nodes.find((n) => n.id === pasted)?.values,
  ).toEqual(area(s).values);
});

test("invalid pattern edits and files preserve current graph and redo", () => {
  const s = start();
  s.edit({
    type: "setNode",
    id: area(s).id,
    values: { pattern: "@", includeAnchor: true },
  });
  s.travel(false);
  const before = s.getSnapshot();
  for (const pattern of ["XX", "@@", "@\nXX", "@?\nXX", "@\n", " @", 42]) {
    expect(() =>
      s.edit({
        type: "setNode",
        id: area(s).id,
        title: "must not leak",
        values: { pattern },
      }),
    ).toThrow(GraphError);
    expect(s.getSnapshot()).toBe(before);
    const external = structuredClone(before.doc);
    external.nodes.find((n) => n.id === area(s).id)!.values.pattern = pattern;
    expect(() => parseDocument(JSON.stringify(external))).toThrow(GraphError);
  }
  s.travel(true);
  expect(area(s).values.pattern).toBe("@");
});

test("unfinished and explicitly empty footprints remain editable but diagnose missing effect tiles", () => {
  const s = start();
  for (const values of [
    { pattern: "", includeAnchor: false },
    { pattern: "@", includeAnchor: false },
    { pattern: "", includeAnchor: true },
  ]) {
    s.edit({
      type: "setNode",
      id: area(s).id,
      values: { ...values, facing: "release" },
    });
    expect(areaDiagnostics(s).map((d) => d.code)).toEqual([
      "skill-value-unset",
    ]);
    expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
  }
  s.edit({
    type: "setNode",
    id: area(s).id,
    values: { pattern: "@", includeAnchor: true },
  });
  expect(areaDiagnostics(s)).toEqual([]);
});

test("legacy area is preserved on load/save and only explicit reassignment replaces it with undo", () => {
  const doc = structuredClone(start().getSnapshot().doc);
  const old = doc.nodes.find((n) => n.kind === "skill-area")!;
  old.values = {
    shape: "sector",
    length: 2.5,
    width: "",
    angle: 90,
    facing: "accepted",
  };
  const s = new GraphSession(parseDocument(JSON.stringify(doc)));
  const before = s.getSnapshot();
  expect(area(s).values).toEqual(old.values);
  expect(areaDiagnostics(s).map((d) => d.code)).toEqual(["skill-area-legacy"]);
  expect(parseDocument(s.export().json)).toEqual(before.doc);
  // Partial replacement cannot lose the old shape.
  expect(() =>
    s.edit({ type: "setNode", id: old.id, values: { pattern: "X\n@" } }),
  ).toThrow(GraphError);
  expect(s.getSnapshot()).toBe(before);
  s.edit({
    type: "setNode",
    id: old.id,
    values: { pattern: "X\n@", includeAnchor: false, facing: "accepted" },
  });
  expect(areaDiagnostics(s)).toEqual([]);
  expect(area(s).values).not.toHaveProperty("shape");
  expect(s.getSnapshot().doc.edges).toEqual(before.doc.edges);
  s.travel(false);
  expect(area(s).values).toEqual(old.values);
  s.travel(true);
  expect(area(s).values.pattern).toBe("X\n@");
  expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
});
