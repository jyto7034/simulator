import { readFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";
import { GraphSession } from "../../src/nodes/core/session";
import { GraphError } from "../../src/nodes/core/model";
import { parseDocument } from "../../src/nodes/core/validation";
import {
  resonanceExample,
  resonanceExamples,
} from "../../src/nodes/core/resonance-examples";
import { resonanceFieldActive } from "../../src/nodes/core/resonance-catalog";
import type { GraphNode } from "../../src/nodes/types";

const node = (s: GraphSession, title: string) =>
  s.getSnapshot().doc.nodes.find((n) => n.title === title)!;
const codes = (s: GraphSession) =>
  s.getSnapshot().diagnostics.map((d) => d.code);
const connect = (
  s: GraphSession,
  a: GraphNode | string,
  ap: string,
  b: GraphNode | string,
  bp: string,
) =>
  s.edit({
    type: "connect",
    source: { node: typeof a === "string" ? a : a.id, port: ap },
    target: { node: typeof b === "string" ? b : b.id, port: bp },
  });

for (const mode of resonanceExamples) {
  test(`${mode} costs retain typed semantics through files, reroutes, copy, undo and redo`, async () => {
    const s = new GraphSession(resonanceExample(mode));
    expect(codes(s)).toEqual([]);
    const saved = parseDocument(
      await readFile(`content/resonance-${mode}.node-graph.json`, "utf8"),
    );
    const file = new GraphSession(saved);
    expect(codes(file)).toEqual([]);
    const cost = node(s, "첫 지불 비용");
    const before = s.getSnapshot().doc;
    const [copy] = s.edit({ type: "duplicate", ids: [cost.id] });
    expect(
      s.getSnapshot().doc.nodes.find((n) => n.id === copy)?.values,
    ).toEqual(cost.values);
    s.travel(false);
    expect(s.getSnapshot().doc).toEqual(before);
    const edge = before.edges.find((e) => e.source.node === cost.id)!;
    s.edit({ type: "reroute", edgeId: edge.id, position: { x: 600, y: 800 } });
    expect(codes(s)).toEqual([]);
    const edited = s.export().json;
    expect(new GraphSession(parseDocument(edited)).getSnapshot().doc).toEqual(
      s.getSnapshot().doc,
    );
    s.travel(false);
    expect(s.getSnapshot().doc).toEqual(before);
    s.travel(true);
    expect(s.export().json).toBe(edited);
  });
}

test("changing the cost mode diagnoses only its required settings, preserves drafts and distinguishes zero", () => {
  const s = new GraphSession(resonanceExample("all"));
  const cost = node(s, "첫 지불 비용");
  s.edit({ type: "setNode", id: cost.id, values: { mode: "fixed" } });
  expect(codes(s)).toEqual(["resonance-input-required"]);
  s.edit({ type: "setNode", id: cost.id, values: { mode: "current-percent" } });
  expect(new Set(codes(s))).toEqual(new Set(["skill-value-unset"]));
  s.edit({
    type: "setNode",
    id: cost.id,
    values: { percent: 0, minimumCost: 0 },
  });
  expect(codes(s)).toEqual([]);
  const before = s.getSnapshot();
  expect(() =>
    s.edit({ type: "setNode", id: cost.id, values: { percent: 101 } }),
  ).toThrow(GraphError);
  expect(s.getSnapshot()).toBe(before);
  s.edit({ type: "setNode", id: cost.id, values: { mode: "all" } });
  expect(resonanceFieldActive(node(s, "첫 지불 비용"), { id: "percent" })).toBe(
    false,
  );
  expect(codes(s)).toEqual([]);
  s.travel(false);
  expect(node(s, "첫 지불 비용").values.percent).toBe(0);
  s.edit({ type: "setNode", id: cost.id, values: { percent: "" } });
  expect(codes(s)).toContain("skill-value-unset");
  expect(
    parseDocument(s.export().json).nodes.find((n) => n.id === cost.id)?.values
      .percent,
  ).toBe("");
});

test("conditional minimum, selection range and count constraints remain editable diagnostics", () => {
  const s = new GraphSession(resonanceExample("selected"));
  const min = node(s, "선택 최소량"),
    requirement = node(s, "사용 공명 조건");
  s.edit({ type: "setNode", id: requirement.id, values: { mode: "minimum" } });
  expect(codes(s)).toContain("resonance-input-required");
  connect(s, min, "value", requirement, "minimum");
  expect(codes(s)).toEqual([]);
  s.edit({ type: "setNode", id: min.id, values: { value: 101 } });
  expect(codes(s)).toContain("resonance-selection-range");
  s.travel(false);
  connect(s, min, "value", node(s, "첫 지불 비용"), "selected");
  expect(codes(s)).toContain("resonance-selection-source");
  s.travel(false);
  const c = new GraphSession(resonanceExample("count"));
  const [number] = c.edit({
    type: "add",
    kind: "number",
    position: { x: 0, y: 0 },
  });
  connect(c, number, "value", node(c, "첫 지불 비용"), "count");
  for (const value of [-1, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
    c.edit({ type: "setNode", id: number, values: { value } });
    expect(codes(c)).toContain("resonance-count");
  }
  c.edit({ type: "setNode", id: number, values: { value: 0 } });
  expect(codes(c)).toEqual([]);
});

test("integer resonance rejects fractional and unsafe edits and files without altering history", () => {
  const s = new GraphSession(resonanceExample("max-percent"));
  const cost = node(s, "첫 지불 비용");
  const amount = s
    .getSnapshot()
    .doc.nodes.find((n) => n.kind === "skill-resonance-amount")!;
  for (const [id, field] of [
    [amount.id, "value"],
    [cost.id, "percent"],
    [cost.id, "minimumCost"],
  ]) {
    for (const value of [0.5, Number.MAX_SAFE_INTEGER + 1]) {
      const before = s.getSnapshot();
      expect(() =>
        s.edit({ type: "setNode", id, values: { [field]: value } }),
      ).toThrow(/소수 없이 안전한 정수/);
      expect(s.getSnapshot()).toBe(before);
      const bad = JSON.parse(s.export().json);
      bad.nodes.find((n: GraphNode) => n.id === id).values[field] = value;
      expect(() => parseDocument(JSON.stringify(bad))).toThrow(
        /소수 없이 안전한 정수/,
      );
    }
  }
  const before = s.getSnapshot().doc;
  s.edit({ type: "setNode", id: amount.id, values: { value: 30 } });
  expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
  s.travel(true);
  expect(
    s.getSnapshot().doc.nodes.find((n) => n.id === amount.id)?.values.value,
  ).toBe(30);
  for (const rounding of ["floor", "ceil", "nearest", "exact", "unresolved"]) {
    const old = JSON.parse(s.export().json);
    old.nodes.find((n: GraphNode) => n.id === cost.id).values.rounding =
      rounding;
    expect(() => parseDocument(JSON.stringify(old))).toThrow(
      /노드 속성이 누락되었거나 지원하지 않는 속성/,
    );
  }
});

test("units reject atomically, while owner mismatches diagnose through reroutes", () => {
  const s = new GraphSession(resonanceExample("fixed"));
  const [seconds] = s.edit({
    type: "add",
    kind: "skill-seconds",
    position: { x: 0, y: 0 },
  });
  const before = s.getSnapshot();
  expect(() =>
    connect(s, seconds, "value", node(s, "첫 지불 비용"), "amount"),
  ).toThrow(GraphError);
  expect(s.getSnapshot()).toBe(before);
  s.travel(false);
  const [other] = s.edit({
    type: "add",
    kind: "skill-request",
    position: { x: 0, y: 0 },
  });
  connect(s, other, "request", node(s, "요청의 공명 조회"), "request");
  expect(codes(s)).toContain("resonance-owner-scope");
  const edge = s.getSnapshot().doc.edges.find((e) => e.source.node === other)!;
  s.edit({ type: "reroute", edgeId: edge.id, position: { x: 10, y: 10 } });
  expect(codes(s)).toContain("resonance-owner-scope");
  s.travel(false);
  s.travel(false);
  expect(codes(s)).toEqual([]);
});

test("receipts and use handles cannot escape success branches or be justified by bypasses", () => {
  const s = new GraphSession(resonanceExample("fixed"));
  const admit = node(s, "접수와 첫 지불"),
    result = node(s, "첫 지불 결과");
  connect(s, admit, "rejected", result, "in");
  expect(codes(s)).toContain("skill-path-scope");
  expect(codes(s)).toContain("resonance-use-scope");
  s.travel(false);
  const [bypass] = s.edit({
    type: "add",
    kind: "event",
    position: { x: 0, y: 0 },
  });
  connect(s, bypass, "out", result, "in");
  expect(codes(s)).toContain("skill-path-scope");
  s.travel(false);
  expect(codes(s)).toEqual([]);
});

test("duplicate admission and stage payment are diagnosed, not silently charged twice", () => {
  const s = new GraphSession(resonanceExample("stage"));
  const admit = node(s, "접수와 첫 지불"),
    qualify = node(s, "사용 자격 검사");
  const [copy] = s.edit({ type: "duplicate", ids: [admit.id] });
  connect(s, qualify, "permit", copy, "permit");
  expect(codes(s)).toContain("resonance-duplicate-admission");
  s.travel(false);
  s.travel(false);
  const pay = node(s, "단계의 공명 지불"),
    step = node(s, "후속 공격 단계");
  const [other] = s.edit({ type: "duplicate", ids: [pay.id] });
  connect(s, step, "step", other, "step");
  expect(codes(s)).toContain("resonance-duplicate-payment");
  s.travel(false);
  s.travel(false);
  expect(codes(s)).toEqual([]);
});

test("periodic costs require positive time, finite integer occurrences and a same-use payment", () => {
  const s = new GraphSession(resonanceExample("periodic"));
  const period = node(s, "유지 비용 간격"),
    event = node(s, "유지 비용 주기");
  s.edit({ type: "setNode", id: period.id, values: { value: 0 } });
  expect(codes(s)).toContain("resonance-period");
  s.travel(false);
  s.edit({ type: "setNode", id: event.id, values: { count: 1.5 } });
  expect(codes(s)).toContain("resonance-count");
  s.travel(false);
  const pay = node(s, "단계의 공명 지불");
  connect(s, event, "completed", pay, "in");
  expect(codes(s)).toContain("skill-path-scope");
  s.travel(false);
  s.edit({ type: "delete", ids: [pay.id] });
  expect(codes(s)).toContain("resonance-period-payment");
  s.travel(false);
  expect(codes(s)).toEqual([]);
});

test("ended uses cannot pay again and receipt-based later costs require the matching result path", () => {
  const s = new GraphSession(resonanceExample("stage"));
  const result = node(s, "첫 지불 결과"),
    cost = node(s, "후속 지불 비용");
  connect(s, result, "paid", cost, "amount");
  expect(codes(s)).toEqual([]);
  // Keep the receipt value connected but bypass the result's execution boundary.
  connect(s, node(s, "접수와 첫 지불"), "out", node(s, "후속 공격 단계"), "in");
  expect(codes(s)).toContain("resonance-result-scope");
  s.travel(false);
  const end = node(s, "사용 완료"),
    stage = node(s, "후속 공격 단계");
  const [early] = s.edit({ type: "duplicate", ids: [end.id] });
  connect(s, node(s, "접수와 첫 지불"), "use", early, "use");
  connect(s, result, "out", early, "in");
  connect(s, early, "out", stage, "in");
  expect(codes(s)).toContain("resonance-use-scope");
});

test("previous Hello admission and install contracts reopen unchanged", async () => {
  const text = await readFile(
    "tests/model/fixtures/hello-goodbye-before-resonance.json",
    "utf8",
  );
  const s = new GraphSession(parseDocument(text));
  expect(JSON.parse(s.export().json)).toEqual(JSON.parse(text));
  expect(new Set(codes(s))).toEqual(
    new Set(["skill-value-unset", "skill-policy-unset", "skill-area-legacy"]),
  );
});

test("periodic lifetime can follow use termination without inventing a tick limit", () => {
  const s = new GraphSession(resonanceExample("periodic"));
  const event = node(s, "유지 비용 주기");
  s.edit({
    type: "setNode",
    id: event.id,
    values: { limit: "use-end", count: "" },
  });
  expect(codes(s)).toEqual(["resonance-period-completion"]);
  const completed = s
    .getSnapshot()
    .doc.edges.find(
      (e) => e.source.node === event.id && e.source.port === "completed",
    )!;
  s.edit({ type: "disconnect", ids: [completed.id] });
  // Complete the explicit use after a successful payment; its subscription stops.
  connect(s, node(s, "후속 지불 결과"), "out", node(s, "사용 완료"), "in");
  expect(codes(s)).toEqual([]);
  expect(resonanceFieldActive(node(s, "유지 비용 주기"), { id: "count" })).toBe(
    false,
  );
});

test("inactive cost inputs retain their wires but do not impose another request's selection", () => {
  const s = new GraphSession(resonanceExample("selected"));
  const [other] = s.edit({
    type: "add",
    kind: "skill-request",
    position: { x: 0, y: 0 },
  });
  connect(s, other, "request", node(s, "요청에서 선택한 값"), "request");
  expect(codes(s)).toContain("resonance-owner-scope");
  s.edit({
    type: "setNode",
    id: node(s, "첫 지불 비용").id,
    values: { mode: "all" },
  });
  expect(codes(s)).toEqual([]);
  s.travel(false);
  expect(codes(s)).toContain("resonance-owner-scope");
});
