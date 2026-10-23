import { readFileSync } from "node:fs";
import { test, expect } from "@playwright/test";
import { mountainOfSmilingBodies } from "../../src/nodes/core/mountain-of-smiling-bodies";
import { GraphSession } from "../../src/nodes/core/session";
import { parseDocument } from "../../src/nodes/core/validation";
import { inputSources } from "../../src/nodes/core/connections";
import type { GraphNode } from "../../src/nodes/types";

const find = (s: GraphSession, kind: string) =>
  s.getSnapshot().doc.nodes.find((n) => n.kind === `skill-${kind}`)!;
const codes = (s: GraphSession) =>
  s.getSnapshot().diagnostics.map((d) => d.code);
const connect = (
  s: GraphSession,
  a: GraphNode,
  ap: string,
  b: GraphNode,
  bp: string,
) =>
  s.edit({
    type: "connect",
    source: { node: a.id, port: ap },
    target: { node: b.id, port: bp },
  });
function add(s: GraphSession, kind: string) {
  const ids = new Set(s.getSnapshot().doc.nodes.map((n) => n.id));
  s.edit({ type: "add", kind, position: { x: 0, y: 0 } });
  return s.getSnapshot().doc.nodes.find((n) => !ids.has(n.id))!;
}
function assigned() {
  const s = new GraphSession(mountainOfSmilingBodies());
  // Test balance only. The shipped document keeps its unresolved content values.
  for (const n of s.getSnapshot().doc.nodes) {
    if (
      n.kind.startsWith("skill-modifier-") ||
      n.kind === "skill-weapon-kill-growth"
    ) {
      const values = Object.fromEntries(
        Object.entries(n.values)
          .filter(([, v]) => v === "")
          .map(([k]) => [k, 10]),
      );
      if (Object.keys(values).length)
        s.edit({ type: "setNode", id: n.id, values });
    }
    if (
      [
        "skill-seconds",
        "skill-resonance-amount",
        "skill-integer-amount",
      ].includes(n.kind)
    )
      s.edit({ type: "setNode", id: n.id, values: { value: 10 } });
    if (n.kind === "skill-target-speed")
      s.edit({ type: "setNode", id: n.id, values: { percent: 30 } });
    if (n.kind === "skill-range")
      s.edit({
        type: "setNode",
        id: n.id,
        values: { pattern: "@X", includeAnchor: true },
      });
  }
  return s;
}

test("real mountain fragment keeps explicit balance gaps and one shared per-target lifetime", () => {
  for (const doc of [
    mountainOfSmilingBodies(),
    parseDocument(
      readFileSync(
        "content/mountain-of-smiling-bodies.node-graph.json",
        "utf8",
      ),
    ),
  ]) {
    const s = new GraphSession(doc);
    expect(codes(s)).toEqual(Array(12).fill("skill-value-unset"));
    const source = inputSources(doc);
    const state = find(s, "target-state"),
      apply = find(s, "target-apply"),
      query = find(s, "range-targets");
    expect(source(apply, "state")?.id).toBe(state.id);
    expect(source(apply, "target")?.id).toBe(query.id);
    expect(source(find(s, "target-ended"), "instance")?.id).toBe(apply.id);
    expect(state.values.sourceExit).toBe("end");
    expect(
      doc.edges.filter(
        (e) => e.target.node === state.id && e.target.port === "effects",
      ),
    ).toHaveLength(3);
    expect(find(s, "target-attack-limit").values.tiles).toBe(1);
    expect(
      doc.nodes.some((n) =>
        [
          "skill-damage",
          "skill-area-damage",
          "skill-ground-create",
          "skill-channel-begin",
        ].includes(n.kind),
      ),
    ).toBe(false);
    expect(doc.edges).toContainEqual(
      expect.objectContaining({
        source: { node: query.id, port: "out" },
        target: { node: find(s, "resonance-end").id, port: "in" },
      }),
    );
    expect(parseDocument(s.export().json)).toEqual(doc);
  }
  expect(codes(assigned())).toEqual([]);
});

test("group settings reject duplicates, nonpositive values, friendly taunts and detached caster lifetimes", () => {
  const s = assigned();
  s.edit({
    type: "setNode",
    id: find(s, "target-state").id,
    values: { sourceExit: "independent" },
  });
  expect(codes(s)).toContain("target-taunt-owner");
  s.travel(false);
  for (const [kind, field, code] of [
    ["seconds", "value", "target-positive-time"],
    ["target-speed", "percent", "target-positive-speed"],
    ["target-attack-limit", "tiles", "target-positive-distance"],
  ]) {
    s.edit({ type: "setNode", id: find(s, kind).id, values: { [field]: 0 } });
    expect(codes(s)).toContain(code);
    s.travel(false);
  }
  s.edit({
    type: "setNode",
    id: find(s, "ground-filter").id,
    values: { side: "allies" },
  });
  expect(codes(s)).toContain("target-taunt-enemies");
  s.travel(false);
  const duplicate = add(s, "skill-target-taunt");
  connect(s, duplicate, "effect", find(s, "target-state"), "effects");
  expect(codes(s)).toContain("target-effect-duplicate");
  const edge = s
    .getSnapshot()
    .doc.edges.find((e) => e.source.node === duplicate.id)!;
  s.edit({ type: "reroute", edgeId: edge.id, position: { x: 0, y: 0 } });
  expect(codes(s)).toContain("target-effect-duplicate");
});

test("only the exact query item branch may apply a state, never its completion or a foreign item", () => {
  const s = assigned(),
    query = find(s, "range-targets"),
    apply = find(s, "target-apply");
  connect(s, query, "out", apply, "in");
  expect(codes(s)).toContain("target-item-scope");
  s.travel(false);
  const foreign = add(s, "skill-range-targets");
  connect(s, foreign, "target", apply, "target");
  expect(codes(s)).toContain("target-item-scope");
  s.travel(false);
  connect(s, query, "each", foreign, "in");
  expect(codes(s)).toContain("target-nested-query");
});

test("rejected and ended uses cannot query targets; selected tile placement requires player selection", () => {
  const s = assigned(),
    query = find(s, "range-targets");
  connect(s, find(s, "resonance-admit"), "rejected", query, "in");
  expect(codes(s)).toContain("target-use-scope");
  s.travel(false);
  const end = find(s, "resonance-end");
  connect(s, find(s, "resonance-result"), "out", end, "in");
  connect(s, end, "out", query, "in");
  expect(
    s
      .getSnapshot()
      .diagnostics.filter((d) => d.code === "target-use-closed")
      .map((d) => d.nodeId)
      .sort(),
  ).toEqual([query.id, find(s, "target-apply").id].sort());
  s.travel(false);
  s.travel(false);
  s.edit({
    type: "setNode",
    id: find(s, "range-placement").id,
    values: { origin: "selected-tile" },
  });
  expect(codes(s)).toContain("target-selection");
});

test("item processing cannot wait or finish its enclosing use, including a mixed incoming branch", () => {
  const s = assigned(),
    query = find(s, "range-targets");
  connect(s, query, "each", find(s, "resonance-end"), "in");
  expect(codes(s)).toContain("target-end-per-item");
  s.travel(false);
  const delay = add(s, "delay");
  connect(s, query, "each", delay, "in");
  expect(codes(s)).toContain("target-synchronous");
  connect(s, query, "out", delay, "in");
  expect(codes(s)).toContain("target-synchronous");
});

test("cleanup accepts only its successful item or own ended instance, and ended events cannot reapply", () => {
  const s = assigned(),
    apply = find(s, "target-apply"),
    ended = find(s, "target-ended");
  const clear = add(s, "skill-target-clear");
  connect(s, apply, "instance", clear, "instance");
  connect(s, apply, "applied", clear, "in");
  expect(codes(s)).toEqual([]);
  connect(s, apply, "rejected", clear, "in");
  expect(codes(s)).toContain("target-instance-scope");
  s.travel(false);
  // Remove the success branch to check the independent ended-event scope.
  const incoming = s
    .getSnapshot()
    .doc.edges.filter(
      (e) => e.target.node === clear.id && e.target.port === "in",
    )
    .map((e) => e.id);
  s.edit({ type: "delete", ids: incoming });
  connect(s, ended, "out", clear, "in");
  expect(codes(s)).toEqual([]);
  connect(s, ended, "out", apply, "in");
  expect(codes(s)).toContain("target-ended-reapply");
  s.travel(false);
  const other = add(s, "skill-target-apply");
  connect(s, other, "instance", clear, "instance");
  expect(codes(s)).toContain("target-instance-scope");
});

test("typed reroutes, copying and saved reload preserve all effect contributors and owners", () => {
  const s = assigned();
  for (const edge of [...s.getSnapshot().doc.edges])
    s.edit({ type: "reroute", edgeId: edge.id, position: { x: 0, y: 0 } });
  expect(codes(s)).toEqual([]);
  expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
  const copy = new GraphSession();
  copy.edit({
    type: "paste",
    fragment: s.copy(s.getSnapshot().doc.nodes.map((n) => n.id)),
    position: { x: 0, y: 0 },
  });
  expect(codes(copy)).toEqual([]);
  expect(
    copy
      .getSnapshot()
      .doc.edges.filter(
        (e) =>
          e.target.node === find(copy, "target-state").id &&
          e.target.port === "effects",
      ),
  ).toHaveLength(3);
});

test("invalid values leave document and history intact; valid state edits undo and reload together", () => {
  const s = assigned(),
    speed = find(s, "target-speed");
  const before = s.getSnapshot();
  expect(() =>
    s.edit({ type: "setNode", id: speed.id, values: { percent: -1 } }),
  ).toThrow();
  expect(s.getSnapshot()).toBe(before);
  const invalid = structuredClone(before.doc);
  invalid.nodes.find((n) => n.id === speed.id)!.values.percent = -1;
  expect(() => parseDocument(JSON.stringify(invalid))).toThrow();
  s.edit({ type: "setNode", id: speed.id, values: { percent: 55 } });
  const saved = parseDocument(s.export().json);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before.doc);
  s.travel(true);
  expect(s.getSnapshot().doc).toEqual(saved);
});
