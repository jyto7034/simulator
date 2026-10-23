import { expect, test } from "@playwright/test";
import { createDocument } from "../../src/nodes/core/model";
import { GraphSession } from "../../src/nodes/core/session";
import { parseDocument } from "../../src/nodes/core/validation";
import { secondsToMilliseconds } from "../../src/nodes/core/time-values";
import type { GraphNode } from "../../src/nodes/types";

test("decimal seconds convert exactly to safe integer milliseconds without tolerance or rounding", () => {
  for (const [seconds, milliseconds] of [
    [0, 0],
    [0.001, 1],
    [1e-3, 1],
    [0.5, 500],
    [1.001, 1001],
    [1.25, 1250],
    [0.009, 9],
    [1e6, 1_000_000_000],
    [9_000_000_000_000, 9_000_000_000_000_000],
  ]) {
    expect(secondsToMilliseconds(seconds)).toBe(milliseconds);
  }
  for (const seconds of [
    -0.001,
    0.0005,
    1.0001,
    1e-7,
    0.1 + 0.2,
    9_007_199_254_741,
    1e100,
    NaN,
    Infinity,
  ]) {
    expect(secondsToMilliseconds(seconds)).toBeNull();
  }
});

for (const [kind, field] of [
  ["skill-seconds", "value"],
  ["delay", "seconds"],
]) {
  test(`${kind} enforces milliseconds through edits, files and history`, () => {
    const s = new GraphSession(createDocument());
    const [id] = s.edit({ type: "add", kind, position: { x: 0, y: 0 } });
    const before = s.getSnapshot().doc;
    for (const value of [-1, 0.0005, 1.2345, 9_007_199_254_741]) {
      const snapshot = s.getSnapshot();
      expect(() =>
        s.edit({ type: "setNode", id, values: { [field]: value } }),
      ).toThrow(/0.001초 단위/);
      expect(s.getSnapshot()).toBe(snapshot);
      const invalid = JSON.parse(s.export().json);
      invalid.nodes.find((n: GraphNode) => n.id === id).values[field] = value;
      expect(() => parseDocument(JSON.stringify(invalid))).toThrow(
        /0.001초 단위/,
      );
    }
    s.edit({ type: "setNode", id, values: { [field]: 1.001 } });
    expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
    s.travel(false);
    expect(s.getSnapshot().doc).toEqual(before);
    s.travel(true);
    expect(s.getSnapshot().doc.nodes[0].values[field]).toBe(1.001);
    s.edit({ type: "setNode", id, values: { [field]: 0 } });
    expect(s.getSnapshot().doc.nodes[0].values[field]).toBe(0);
    if (kind === "skill-seconds") {
      s.edit({ type: "setNode", id, values: { [field]: "" } });
      expect(parseDocument(s.export().json).nodes[0].values[field]).toBe("");
      expect(s.getSnapshot().diagnostics.map((d) => d.code)).toContain(
        "skill-value-unset",
      );
    }
  });
}

test("a general number remains editable but sub-millisecond delay inputs diagnose through reroutes", () => {
  const s = new GraphSession(createDocument());
  const add = (kind: string) =>
    s.edit({ type: "add", kind, position: { x: 0, y: 0 } })[0];
  const number = add("number"),
    delay = add("delay");
  s.edit({ type: "setNode", id: number, values: { value: 0.0005 } });
  s.edit({
    type: "connect",
    source: { node: number, port: "value" },
    target: { node: delay, port: "seconds" },
  });
  const edge = s.getSnapshot().doc.edges[0];
  s.edit({ type: "reroute", edgeId: edge.id, position: { x: 100, y: 0 } });
  expect(s.getSnapshot().diagnostics).toContainEqual(
    expect.objectContaining({
      code: "time-precision",
      nodeId: delay,
      portId: "seconds",
    }),
  );
  expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
  s.edit({ type: "setNode", id: number, values: { value: 0.001 } });
  expect(s.getSnapshot().diagnostics.map((d) => d.code)).not.toContain(
    "time-precision",
  );
  s.travel(false);
  expect(s.getSnapshot().diagnostics.map((d) => d.code)).toContain(
    "time-precision",
  );
});
