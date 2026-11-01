import { readFileSync } from "node:fs";
import { test, expect } from "@playwright/test";
import { example } from "../../src/nodes/core/examples";
import { definition, makeNode } from "../../src/nodes/core/catalog";
import { GraphSession } from "../../src/nodes/core/session";
import { diagnostics, parseDocument } from "../../src/nodes/core/validation";
import { inputSources } from "../../src/nodes/core/connections";
import type { GraphDocument } from "../../src/nodes/types";
const fixtures = [
  ["melting-love", 12],
  ["funeral-of-dying-butterflies", 5],
  ["funeral-of-dying-butterflies-bloom", 8],
  ["funeral-of-dying-butterflies-orb", 12],
] as const;
function assigned(name = "melting-love", input = example(name)) {
  const doc = structuredClone(input);
  for (const n of doc.nodes) {
    for (const f of definition(n.kind).fields)
      if (f.valueType === "optional-number" && n.values[f.id] === "")
        n.values[f.id] = 10;
    if (n.kind === "skill-range" && n.values.pattern === "") {
      n.values.pattern = "@X";
      n.values.includeAnchor = true;
    }
  }
  return doc;
}
const find = (d: GraphDocument, k: string) =>
  d.nodes.find((n) => n.kind === `skill-${k}`)!;
const codes = (d: GraphDocument) => diagnostics(d).map((d) => d.code);
for (const [name, unset] of fixtures)
  test(`${name} real graph and factory preserve authored structure and explicit missing values`, () => {
    for (const doc of [
      example(name),
      parseDocument(readFileSync(`content/${name}.node-graph.json`, "utf8")),
    ]) {
      expect(codes(doc)).toEqual(Array(unset).fill("skill-value-unset"));
      expect(codes(assigned(name, doc))).toEqual([]);
      const s = new GraphSession(doc);
      expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
    }
  });
test("infection ownership, storage mode and duplicate clocks are checked independently", () => {
  const d = assigned();
  const power = find(d, "population-power");
  const foreign = makeNode("skill-target-marks", { x: 0, y: 0 });
  foreign.values = { ...find(d, "target-marks").values };
  d.nodes.push(foreign);
  d.edges.find(
    (e) => e.target.node === power.id && e.target.port === "marks",
  )!.source.node = foreign.id;
  expect(codes(d)).toContain("mark-source-owner");
  const m = assigned();
  find(m, "target-marks").values.mode = "stacks";
  expect(codes(m)).toContain("mark-storage-mode");
  const s = new GraphSession(assigned());
  const periodic = find(s.getSnapshot().doc, "mark-periodic-damage");
  s.edit({ type: "duplicate", ids: [periodic.id] });
  const duplicate = s
    .getSnapshot()
    .doc.nodes.find((n) => n.kind === periodic.kind && n.id !== periodic.id)!;
  const original = inputSources(s.getSnapshot().doc);
  for (const [port, output] of [
    ["marks", "marks"],
    ["power", "power"],
    ["interval", "value"],
  ])
    s.edit({
      type: "connect",
      source: { node: original(periodic, port)!.id, port: output },
      target: { node: duplicate.id, port },
    });
  expect(codes(s.getSnapshot().doc)).toContain("mark-subscription-duplicate");
});
test("illegal transfer, admission placement and filter drift remain editable but cannot validate", () => {
  const d = assigned();
  find(d, "target-marks").values.death = "tile";
  expect(codes(d)).toContain("mark-presence-tile");
  const b = assigned("funeral-of-dying-butterflies-bloom");
  find(b, "target-marks").values.death = "remove";
  expect(codes(b)).toContain("mark-stack-transfer");
  const delayed = assigned();
  find(delayed, "action-entry").values.start = "next-basic";
  expect(codes(delayed)).toContain("burst-admission-boundary");
  const f = assigned();
  const filter = makeNode("skill-mark-filter", { x: 0, y: 0 });
  f.nodes.push(filter);
  f.edges.find(
    (e) =>
      e.target.node === find(f, "mark-spread").id && e.target.port === "filter",
  )!.source.node = filter.id;
  expect(codes(f)).toContain("mark-filter-mismatch");
});
test("scalar edits reject invalid values atomically and preserve undo/save meaning", () => {
  const s = new GraphSession(assigned());
  const burst = find(s.getSnapshot().doc, "mark-burst-plan");
  const before = s.getSnapshot();
  expect(() =>
    s.edit({ type: "setNode", id: burst.id, values: { hostMultiplier: 5 } }),
  ).toThrow();
  expect(s.getSnapshot()).toBe(before);
  s.edit({ type: "setNode", id: burst.id, values: { hostMultiplier: 4 } });
  expect(
    find(s.getSnapshot().doc, "mark-burst-plan").values.hostMultiplier,
  ).toBe(4);
  expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before.doc);
});
