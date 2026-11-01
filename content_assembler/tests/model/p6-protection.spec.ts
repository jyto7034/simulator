import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import { example } from "../../src/nodes/core/examples";
import { definition, makeNode } from "../../src/nodes/core/catalog";
import { diagnostics, parseDocument } from "../../src/nodes/core/validation";
import { GraphSession } from "../../src/nodes/core/session";
import { cardPresentation } from "../../src/nodes/core/card-presentation";
import type { GraphDocument } from "../../src/nodes/types";
import { inputSourceLists } from "../../src/nodes/core/connections";
const fixtures = [
  ["porccubus", 6],
  ["schadenfreude", 5],
  ["scarecrow-searching-for-wisdom", 4],
  ["queen-bee", 5],
  ["nameless-fetus", 5],
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
test("recipient selection and independent shield lifetime reject mismatched authoring", () => {
  const bee = assign(example("queen-bee"));
  find(bee, "target-selection").values.mode = "self";
  expect(codes(bee)).toContain("action-request-selection");
  const shield = assign(example("porccubus"));
  find(shield, "target-state").values.sourceExit = "independent";
  expect(codes(shield)).toContain("shield-source-exit");
  shield.edges = shield.edges.filter(
    (e) =>
      e.target.node !== find(shield, "recipient-buff-plan").id ||
      e.target.port !== "range",
  );
  expect(codes(shield)).toContain("p4-input-required");
});
test("aura accepts distinct attributes but rejects duplicate or non-attribute effects and wrong units", () => {
  const d = assign(example("nameless-fetus"));
  const attrs = inputSourceLists(d)(find(d, "aura-buff-plan"), "effects");
  attrs[1].values.attribute = attrs[0].values.attribute;
  expect(codes(d)).toContain("aura-effect-duplicate");
  const d2 = assign(example("nameless-fetus"));
  const aura = find(d2, "aura-buff-plan"),
    floor = makeNode("skill-hp-floor-effect", { x: 0, y: 0 });
  floor.values.minimum = 1;
  d2.nodes.push(floor);
  d2.edges.push({
    id: crypto.randomUUID(),
    source: { node: floor.id, port: "effect" },
    target: { node: aura.id, port: "effects" },
  });
  expect(codes(d2)).toContain("aura-effect-kind");
  const speed = d2.nodes.find((n) => n.values.attribute === "basic-speed")!;
  speed.values.operation = "percent";
  expect(codes(d2)).toContain("attribute-unit");
});
test("compact card keeps connected optional ports and hides only unused ones without modifying data", () => {
  const node = makeNode("skill-resonance-qualify", { x: 0, y: 0 }),
    def = definition(node.kind);
  const connected = new Set(["program"]);
  const compact = cardPresentation(node, def, connected, false),
    expanded = cardPresentation(node, def, connected, true);
  expect(compact.definition.inputs.some((p) => p.id === "program")).toBe(true);
  expect(compact.definition.inputs.filter((p) => p.required)).toEqual(
    def.inputs.filter((p) => p.required),
  );
  expect(expanded.definition.inputs).toEqual(def.inputs);
  expect(compact.definition.height).toBeLessThan(expanded.definition.height);
  expect(def).toEqual(definition(node.kind));
});
