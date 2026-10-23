import { test, expect } from "@playwright/test";
import { makeNode } from "../../src/nodes/core/catalog";
import { createDocument } from "../../src/nodes/core/model";
import { diagnostics, parseDocument } from "../../src/nodes/core/validation";
import { GraphSession } from "../../src/nodes/core/session";
import type { GraphNode } from "../../src/nodes/types";
function setup() {
  const doc = createDocument();
  const add = (kind: string, values: GraphNode["values"] = {}) => {
    const n = makeNode(`skill-${kind}`, { x: 0, y: 0 });
    Object.assign(n.values, values);
    doc.nodes.push(n);
    return n;
  };
  const link = (a: GraphNode, ap: string, b: GraphNode, bp: string) =>
    doc.edges.push({
      id: crypto.randomUUID(),
      source: { node: a.id, port: ap },
      target: { node: b.id, port: bp },
    });
  const condition = add("equipment-condition", { definition: "long-breath" });
  return {
    doc,
    add,
    link,
    condition,
    codes: () => diagnostics(doc).map((d) => d.code),
  };
}
test("conditional duration preserves the original value through edits, history and files", () => {
  const { doc, add, link, condition, codes } = setup();
  const seconds = add("seconds", { value: 3 });
  const modifier = add("modifier-duration", { percent: 25 });
  link(condition, "condition", modifier, "condition");
  link(seconds, "value", modifier, "target");
  expect(codes()).toEqual([]);
  const session = new GraphSession(doc),
    original = session.getSnapshot().doc;
  session.edit({ type: "setNode", id: modifier.id, values: { percent: 50 } });
  expect(
    session.getSnapshot().doc.nodes.find((n) => n.id === seconds.id)?.values
      .value,
  ).toBe(3);
  expect(parseDocument(session.export().json)).toEqual(
    session.getSnapshot().doc,
  );
  session.travel(false);
  expect(session.getSnapshot().doc).toEqual(original);
});
test("equipment prerequisites reject cycles instead of bootstrapping each other", () => {
  const { add, link, condition, codes } = setup();
  const other = add("equipment-condition", { definition: "other" });
  link(condition, "condition", other, "requires");
  link(other, "condition", condition, "requires");
  expect(codes()).toContain("equipment-prerequisite-cycle");
});
test("square expansion requires an explicit replacement for non-square footprints", () => {
  const { add, link, condition, codes } = setup();
  const range = add("range", { pattern: "@", includeAnchor: true });
  const modifier = add("modifier-range");
  link(condition, "condition", modifier, "condition");
  link(range, "range", modifier, "target");
  expect(codes()).toEqual([]);
  range.values.pattern = "@X";
  expect(codes()).toContain("modifier-range-shape");
  modifier.values.mode = "replace";
  expect(codes()).toContain("modifier-range-required");
  const replacement = add("range", { pattern: "X@X", includeAnchor: true });
  link(replacement, "range", modifier, "replacement");
  expect(codes()).toEqual([]);
});
test("the same modifier cannot silently double itself and debt limits cannot be removed", () => {
  const { add, link, condition, codes } = setup();
  const seconds = add("seconds", { value: 3 });
  for (let i = 0; i < 2; i++) {
    const m = add("modifier-duration", { percent: 25 });
    link(condition, "condition", m, "condition");
    link(seconds, "value", m, "target");
  }
  expect(codes()).toContain("modifier-duplicate");
  const ledger = add("credit-ledger", { grant: 10, creditLimit: 20 });
  const cap = add("modifier-capacity");
  link(condition, "condition", cap, "condition");
  link(ledger, "bank", cap, "target");
  expect(codes()).toContain("p4-source-scope");
});

test("actual control success excludes pull, slow and ambiguous event sources", () => {
  const { add, link, condition, codes } = setup();
  const n = add("modifier-control-vulnerability", { percent: 12 });
  link(condition, "condition", n, "condition");
  link(add("seconds", { value: 5 }), "value", n, "duration");
  const status = add("reactive-status", {
    effect: "slow",
    percent: 30,
    refresh: "refresh",
  });
  link(add("seconds", { value: 3 }), "value", status, "duration");
  link(status, "status", n, "reactive");
  expect(codes()).toContain("modifier-hard-control");
  status.values.effect = "freeze";
  expect(codes()).toEqual([]);
  const hit = add("hit-status", { effect: "stun" });
  link(add("seconds", { value: 2 }), "value", hit, "duration");
  link(hit, "status", n, "hit");
  expect(codes()).toContain("modifier-control-choice");
});
