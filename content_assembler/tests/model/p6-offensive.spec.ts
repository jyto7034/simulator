import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import { example } from "../../src/nodes/core/examples";
import { definition } from "../../src/nodes/core/catalog";
import { diagnostics, parseDocument } from "../../src/nodes/core/validation";
import { GraphSession } from "../../src/nodes/core/session";
import type { GraphDocument } from "../../src/nodes/types";
const fixtures = [
  ["warm-hearted-woodsman", 18],
  ["clouded-monk", 7],
  ["rudolta", 7],
  ["dream-of-black-swan", 11],
  ["laetitia", 19],
  ["laetitia-bloom", 19],
  ["snow-queen", 14],
] as const;
const find = (d: GraphDocument, kind: string) =>
  d.nodes.find((n) => n.kind === `skill-${kind}`)!;
const codes = (d: GraphDocument) => diagnostics(d).map((d) => d.code);
const assign = (d: GraphDocument) => {
  const doc = structuredClone(d);
  for (const n of doc.nodes) {
    for (const f of definition(n.kind).fields)
      if (f.valueType === "optional-number" && n.values[f.id] === "")
        n.values[f.id] = 10;
    if (n.kind === "skill-range") {
      n.values.pattern = "@X";
      n.values.includeAnchor = true;
    }
  }
  return doc;
};
for (const [name, missing] of fixtures)
  test(`${name}: factory and saved graph retain unresolved content and valid provenance`, () => {
    for (const original of [
      example(name),
      parseDocument(readFileSync(`content/${name}.node-graph.json`, "utf8")),
    ]) {
      expect(codes(original)).toEqual(Array(missing).fill("skill-value-unset"));
      expect(codes(assign(original))).toEqual([]);
      const session = new GraphSession(original);
      const node = original.nodes.find((n) => n.kind === "skill-seconds")!;
      session.edit({ type: "setNode", id: node.id, values: { value: 6 } });
      session.travel(false);
      expect(session.getSnapshot().doc).toEqual(original);
      expect(parseDocument(session.export().json)).toEqual(original);
    }
  });
test("accepted target and scheduled opportunity preserve distinct selection timing", () => {
  const monk = assign(example("clouded-monk"));
  const entry = find(monk, "action-entry");
  monk.edges = monk.edges.filter(
    (e) => e.target.node !== entry.id || e.target.port !== "selector",
  );
  expect(codes(monk)).toContain("impact-accepted-selector");
  const gifts = assign(example("rudolta"));
  find(gifts, "delayed-area-plan").values.selectionTime = "accepted";
  expect(codes(gifts)).toContain("scheduled-selection");
});
test("limited footprints reject out-of-range cells instead of silently trimming", () => {
  const d = assign(example("warm-hearted-woodsman"));
  const limit = find(d, "limited-range");
  const edge = d.edges.find(
    (e) => e.target.node === limit.id && e.target.port === "base",
  )!;
  d.nodes.find((n) => n.id === edge.source.node)!.values.pattern = "@..X";
  expect(codes(d)).toContain("range-distance-limit");
});
test("fusion tiers require an unambiguous base and variants cannot cycle", () => {
  const d = assign(example("laetitia"));
  const tiers = d.nodes.filter((n) => n.kind === "skill-fusion-tier");
  tiers.find((n) => n.values.minimum === 1)!.values.minimum = 2;
  expect(codes(d)).toContain("fusion-tier-base");
  expect(codes(d)).toContain("fusion-tier-duplicate");
  const variant = find(d, "mobile-variant");
  d.edges.find(
    (e) => e.target.node === variant.id && e.target.port === "base",
  )!.source.node = variant.id;
  expect(codes(d)).toContain("mobile-variant-cycle");
});
test("remnant generation shares its actual cap and status conversion cannot consume itself", () => {
  const d = assign(example("snow-queen"));
  const convert = find(d, "convert-status-plan");
  const from = d.edges.find(
    (e) => e.target.node === convert.id && e.target.port === "from",
  )!;
  d.edges.find(
    (e) => e.target.node === convert.id && e.target.port === "to",
  )!.source = { ...from.source };
  expect(codes(d)).toContain("status-conversion-identical");
  const cap = structuredClone(find(d, "count-positive"));
  cap.id = crypto.randomUUID();
  d.nodes.push(cap);
  const death = find(d, "status-death-mobile");
  d.edges.find(
    (e) => e.target.node === death.id && e.target.port === "limit",
  )!.source.node = cap.id;
  expect(codes(d)).toContain("mobile-shared-limit");
});
