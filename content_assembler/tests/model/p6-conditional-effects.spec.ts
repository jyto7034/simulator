import { test, expect } from "@playwright/test";
import { example } from "../../src/nodes/core/examples";
import {
  inputSources,
  inputSourceLists,
} from "../../src/nodes/core/connections";
import { diagnostics } from "../../src/nodes/core/validation";
import type { GraphDocument } from "../../src/nodes/types";
import { makeNode } from "../../src/nodes/core/catalog";
const find = (d: GraphDocument, kind: string) =>
  d.nodes.find((n) => n.kind === `skill-${kind}`)!;
const codes = (d: GraphDocument) => diagnostics(d).map((d) => d.code);

test("continuous volley changes only the portal program and does not change recovery or HP cadence", () => {
  const d = example("king-of-greed"),
    source = inputSources(d);
  const n = find(d, "modifier-continuous-volley");
  expect(n.values.percent).toBe(5);
  expect(source(n, "target")?.kind).toBe("skill-portal-program");
  expect(source(n, "costInterval")?.values.value).toBe(1);
  const longBreath = find(d, "modifier-duration");
  expect(source(longBreath, "target")?.id).not.toBe(
    source(n, "costInterval")?.id,
  );
  source(n, "costInterval")!.values.value = 0;
  expect(codes(d)).toContain("p4-positive-time");
});
test("projectile wound shares only explicit normal/enhanced profiles and retains its prerequisite", () => {
  const d = example("little-red"),
    source = inputSources(d),
    sources = inputSourceLists(d);
  const wound = find(d, "modifier-projectile-wound");
  expect(sources(wound, "profiles").map((n) => n.kind)).toEqual([
    "skill-projectile-profile",
    "skill-projectile-profile",
  ]);
  expect(wound.values.amount).toBe(1);
  expect(wound.values.percent).toBe(30);
  const grant = find(d, "modifier-wound-grant");
  expect(source(grant, "target")?.id).toBe(wound.id);
  expect(grant.values.factor).toBe(2);
  expect(
    source(source(grant, "condition")!, "requires")?.values.definition,
  ).toBe("little-red-puncture-wound");
  expect(
    sources(wound, "profiles").every((n) => n.kind !== "skill-explosion-rule"),
  ).toBe(true);
});
test("slug control is independent of enhanced root and exposes only its actual stun to vulnerability", () => {
  const d = example("little-red"),
    source = inputSources(d);
  const weakness = find(d, "modifier-control-vulnerability");
  expect(source(weakness, "hit")?.values.effect).toBe("stun");
  expect(
    source(source(weakness, "condition")!, "requires")?.values.definition,
  ).toBe("little-red-slug-round");
  for (const n of d.nodes.filter(
    (n) => n.kind === "skill-modifier-projectile-push",
  ))
    source(n, "target")!.values.delivery = "impact";
  expect(codes(d)).toContain("modifier-directional-required");
});
test("single support keeps effect-specific replacement values and rejects shielding an aura", () => {
  const d = example("nameless-fetus"),
    source = inputSources(d),
    sources = inputSourceLists(d);
  const n = find(d, "modifier-single-support");
  expect(
    sources(n, "effects")
      .map((n) => n.values.amount)
      .sort((a, b) => Number(a) - Number(b)),
  ).toEqual([20, 40]);
  expect(
    sources(source(n, "target")!, "effects")
      .map((n) => n.values.amount)
      .sort((a, b) => Number(a) - Number(b)),
  ).toEqual([15, 20]);
  n.values.mode = "shield";
  expect(codes(d)).toContain("modifier-support-shield");
  const shield = example("porccubus");
  find(shield, "modifier-single-support").values.mode = "attributes";
  expect(codes(shield)).toContain("modifier-support-effects");
});
test("same-center replacement binds one pair definition and owns a separate range and delay", () => {
  const d = example("yin-yang"),
    source = inputSources(d);
  const n = find(d, "modifier-coincident-area-burst"),
    pair = source(n, "target")!;
  expect(pair.kind).toBe("skill-paired-areas");
  expect(n.values.percent).toBe(25);
  expect(source(n, "range")?.id).not.toBe(source(pair, "first")?.id);
  expect(source(n, "delay")?.values.value).toBe("");
  const edge = d.edges.find(
    (e) => e.target.node === n.id && e.target.port === "target",
  )!;
  edge.source = { node: find(d, "pair-pulse").id, port: "pulse" };
  expect(codes(d)).toContain("p4-source-scope");
});

test("conditionally attached projectile status cannot borrow a foreign paid-count receipt", () => {
  const d = example("king-of-greed"),
    source = inputSources(d);
  const status = source(find(d, "modifier-projectile-status"), "status")!;
  status.values.effect = "vulnerability";
  status.values.percent = 10;
  const paid = makeNode("skill-count-paid", { x: 0, y: 0 });
  d.nodes.push(paid);
  d.edges.push({
    id: crypto.randomUUID(),
    source: { node: paid.id, port: "count" },
    target: { node: status.id, port: "count" },
  });
  expect(codes(d)).toContain("action-paid-scope");
});
