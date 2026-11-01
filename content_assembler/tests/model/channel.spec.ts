import { readFileSync } from "node:fs";
import { test, expect } from "@playwright/test";
import { burrowingHeaven } from "../../src/nodes/core/burrowing-heaven";
import { resonanceExample } from "../../src/nodes/core/resonance-examples";
import { GraphSession } from "../../src/nodes/core/session";
import { parseDocument } from "../../src/nodes/core/validation";
import { inputSources } from "../../src/nodes/core/connections";
import type { GraphDocument, GraphNode } from "../../src/nodes/types";

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
function assigned() {
  const s = new GraphSession(burrowingHeaven());
  // Explicit test-only balance, never written to the shipped authoring graph.
  for (const n of s.getSnapshot().doc.nodes) {
    if (n.kind.startsWith("skill-modifier-")) {
      const values = Object.fromEntries(
        Object.entries(n.values)
          .filter(([, v]) => v === "")
          .map(([k]) => [k, 10]),
      );
      if (Object.keys(values).length)
        s.edit({ type: "setNode", id: n.id, values });
    }
    if (n.kind === "skill-seconds" || n.kind === "skill-resonance-amount")
      s.edit({ type: "setNode", id: n.id, values: { value: 10 } });
    if (n.kind === "skill-range")
      s.edit({
        type: "setNode",
        id: n.id,
        values: { pattern: "@X", includeAnchor: true },
      });
    if (n.kind === "skill-ground-ramping-slow")
      s.edit({ type: "setNode", id: n.id, values: { maxSlow: 60 } });
  }
  return s;
}

test("the real fragment and builder retain only explicit unassigned content values", () => {
  for (const doc of [
    burrowingHeaven(),
    parseDocument(
      readFileSync("content/burrowing-heaven.node-graph.json", "utf8"),
    ),
  ]) {
    const s = new GraphSession(doc);
    expect(codes(s)).toEqual(Array(8).fill("skill-value-unset"));
    expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
    expect(
      doc.nodes.some((n) =>
        ["skill-ground-damage", "skill-damage", "skill-area-damage"].includes(
          n.kind,
        ),
      ),
    ).toBe(false);
    const source = inputSources(doc);
    const begin = find(s, "channel-begin");
    expect(begin.values).toEqual({
      basicAttack: "pause",
      interruption: "standard",
    });
    expect(source(find(s, "channel-area"), "channel")?.id).toBe(begin.id);
    expect(source(find(s, "channel-cancel"), "channel")?.id).toBe(begin.id);
    expect(source(find(s, "channel-ended"), "channel")?.id).toBe(begin.id);
    const placement = find(s, "range-placement");
    expect(placement.values).toEqual({
      origin: "caster",
      positionTime: "accepted",
      facingTime: "accepted",
    });
    const statuses = doc.nodes.filter((n) => n.kind === "skill-ground-status");
    expect(
      statuses.find((n) => n.values.effect === "damage-taken")?.values,
    ).toEqual({
      effect: "damage-taken",
      percent: 20,
      overlap: "strongest",
      lifetime: "membership",
    });
    expect(
      statuses.find((n) => n.values.effect === "stun")?.values.lifetime,
    ).toBe("area");
    const apply = doc.nodes.filter(
      (n) => n.kind === "skill-ground-apply-status",
    );
    expect(apply.map((n) => source(n, "member")?.kind).sort()).toEqual([
      "skill-ground-enter",
      "skill-ground-ramping-slow",
    ]);
  }
  expect(codes(assigned())).toEqual([]);
});

test("channel settings, maximum time and progression survive undo and file reload; invalid edits are atomic", () => {
  const s = assigned();
  const before = s.getSnapshot().doc;
  const begin = find(s, "channel-begin");
  s.edit({
    type: "setNode",
    id: begin.id,
    values: { interruption: "protected", basicAttack: "allow" },
  });
  const changed = parseDocument(s.export().json);
  expect(find(s, "channel-begin").values.interruption).toBe("protected");
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
  s.travel(true);
  expect(s.getSnapshot().doc).toEqual(changed);
  const snapshot = s.getSnapshot();
  expect(() =>
    s.edit({
      type: "setNode",
      id: find(s, "ground-ramping-slow").id,
      values: { maxSlow: 101 },
    }),
  ).toThrow();
  expect(s.getSnapshot()).toBe(snapshot);
  const source = inputSources(snapshot.doc);
  const duration = source(begin, "duration")!;
  expect(() =>
    s.edit({ type: "setNode", id: duration.id, values: { value: 0.0005 } }),
  ).toThrow();
  expect(s.getSnapshot()).toBe(snapshot);
  s.edit({ type: "setNode", id: duration.id, values: { value: 0 } });
  expect(codes(s)).toContain("channel-positive-time");
  s.travel(false);
  const rampTime = source(find(s, "ground-ramping-slow"), "duration")!;
  s.edit({ type: "setNode", id: rampTime.id, values: { value: 0 } });
  expect(codes(s)).toContain("ground-positive-time");
});

test("ended or foreign channel flows cannot create owned areas or cancel a different instance", () => {
  const s = assigned();
  connect(s, find(s, "channel-ended"), "out", find(s, "channel-area"), "in");
  expect(codes(s)).toContain("channel-flow-scope");
  s.travel(false);
  connect(
    s,
    find(s, "channel-cancel-request"),
    "out",
    find(s, "channel-area"),
    "in",
  );
  expect(codes(s)).toContain("channel-flow-scope");
  s.travel(false);
  const original = find(s, "channel-begin").id;
  s.edit({
    type: "add",
    kind: "skill-channel-begin",
    position: { x: 0, y: 0 },
  });
  const other = s
    .getSnapshot()
    .doc.nodes.find(
      (n) => n.kind === "skill-channel-begin" && n.id !== original,
    )!;
  connect(s, other, "channel", find(s, "channel-cancel"), "channel");
  expect(codes(s)).toContain("channel-flow-scope");
  expect(codes(s)).toContain("channel-cancel-unhandled");
});

test("channel admission rejects wrong branches, closed uses and competing lifecycle owners", () => {
  const s = assigned();
  const admit = find(s, "resonance-admit"),
    begin = find(s, "channel-begin");
  connect(s, admit, "rejected", begin, "in");
  expect(codes(s)).toContain("channel-use-scope");
  s.travel(false);
  s.edit({
    type: "add",
    kind: "skill-resonance-end",
    position: { x: 0, y: 0 },
  });
  const end = find(s, "resonance-end");
  connect(s, admit, "use", end, "use");
  connect(s, find(s, "resonance-result"), "out", end, "in");
  connect(s, end, "out", begin, "in");
  expect(codes(s)).toContain("channel-use-end");
  expect(codes(s)).toContain("channel-use-closed");
});

test("ramp thresholds obey member scope and placement policy; imported invalid numbers are rejected", () => {
  const s = assigned();
  const stun = s
    .getSnapshot()
    .doc.nodes.find((n) => n.title === "체류 도달 기절 적용")!;
  connect(s, find(s, "ground-enter"), "member", stun, "member");
  expect(codes(s)).toContain("ground-member-scope");
  s.travel(false);
  s.edit({
    type: "setNode",
    id: find(s, "range-placement").id,
    values: { positionTime: "release" },
  });
  expect(codes(s)).toContain("channel-area-origin");
  s.travel(false);
  const doc: GraphDocument = structuredClone(s.getSnapshot().doc);
  doc.nodes.find(
    (n) => n.kind === "skill-ground-ramping-slow",
  )!.values.maxSlow = -1;
  expect(() => parseDocument(JSON.stringify(doc))).toThrow();
  expect(codes(s)).toEqual([]);
});

test("reroutes and copy/paste keep channel and per-target ownership across every typed edge", () => {
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
});

test("a failed stop-use payment cannot restart a channel even when merged with its success branch", () => {
  const s = new GraphSession(resonanceExample("stage"));
  s.edit({ type: "delete", ids: [find(s, "resonance-end").id] });
  s.edit({
    type: "add",
    kind: "skill-channel-begin",
    position: { x: 0, y: 0 },
  });
  s.edit({ type: "add", kind: "skill-seconds", position: { x: 0, y: 0 } });
  const begin = find(s, "channel-begin"),
    time = find(s, "seconds");
  s.edit({
    type: "setNode",
    id: begin.id,
    values: { basicAttack: "pause", interruption: "standard" },
  });
  s.edit({ type: "setNode", id: time.id, values: { value: 5 } });
  connect(s, time, "value", begin, "duration");
  connect(s, find(s, "resonance-admit"), "use", begin, "use");
  const laterResult = s
    .getSnapshot()
    .doc.nodes.find((n) => n.title === "후속 지불 결과")!;
  connect(s, laterResult, "out", begin, "in");
  expect(codes(s)).toEqual([]);
  connect(s, find(s, "resonance-pay"), "rejected", begin, "in");
  expect(codes(s)).toContain("channel-use-closed");
  const edge = s
    .getSnapshot()
    .doc.edges.find(
      (e) =>
        e.source.node === find(s, "resonance-pay").id &&
        e.source.port === "rejected",
    )!;
  s.edit({ type: "reroute", edgeId: edge.id, position: { x: 0, y: 0 } });
  expect(codes(s)).toContain("channel-use-closed");
});
