import { readFile } from "node:fs/promises";
import { skillDiagnostics } from "../../src/nodes/core/skill-validation";
import { test, expect } from "@playwright/test";
import { example } from "../../src/nodes/core/examples";
import { GraphSession } from "../../src/nodes/core/session";
import { GraphError } from "../../src/nodes/core/model";
import { parseDocument } from "../../src/nodes/core/validation";
import {
  inputSources,
  stateConnections,
} from "../../src/nodes/core/connections";
import type { GraphDocument } from "../../src/nodes/types";

const start = () => new GraphSession(example("hello-goodbye"));
function find(s: GraphSession, title: string) {
  const node = s.getSnapshot().doc.nodes.find((n) => n.title === title);
  if (!node) throw new Error(`missing ${title}`);
  return node;
}
const end = (node: string, port: string) => ({ node, port });
const codes = (s: GraphSession) =>
  s.getSnapshot().diagnostics.map((d) => d.code);

test("state summary follows references and reroutes, reflects edits and undo without writing state", () => {
  const s = start();
  const state = find(s, "Hello 상태 선언");
  const duration = find(s, "Hello 유지 시간");
  const summary = () => stateConnections(s.getSnapshot().doc, state.id);
  const before = s.getSnapshot();
  expect(summary().applications.map((a) => a.node.title)).toEqual([
    "Hello 적용",
  ]);
  expect(summary().applications[0].duration?.values.value).toBe("");
  expect(summary().applications[0].profile?.kind).toBe("skill-attack-profile");
  expect(summary().events).toHaveLength(4);
  expect(s.getSnapshot()).toBe(before);
  for (const edge of before.doc.edges.filter(
    (e) => e.source.node === state.id || e.source.node === duration.id,
  )) {
    s.edit({ type: "reroute", edgeId: edge.id, position: { x: 0, y: 0 } });
  }
  expect(summary()).toEqual(stateConnections(before.doc, state.id));
  s.edit({ type: "setNode", id: duration.id, values: { value: 0 } });
  expect(summary().applications[0].duration?.values.value).toBe(0);
  s.edit({ type: "delete", ids: [duration.id] });
  expect(summary().applications[0].duration).toBeUndefined();
  s.travel(false);
  expect(summary().applications[0].duration?.values.value).toBe(0);
  const overlay = find(s, "Hello 적용");
  const [other] = s.edit({ type: "duplicate", ids: [overlay.id] });
  s.edit({
    type: "connect",
    source: end(state.id, "state"),
    target: end(other, "state"),
  });
  expect(summary().applications).toHaveLength(2);
  s.edit({ type: "delete", ids: [overlay.id, other] });
  expect(summary().applications).toHaveLength(0);
  expect(summary().events).toHaveLength(4);
});

test("input summary stops at dangling or cyclic reroutes and keeps equal state titles distinct", () => {
  const s = start();
  const state = find(s, "Hello 상태 선언");
  const overlay = find(s, "Hello 적용");
  const [other] = s.edit({ type: "duplicate", ids: [state.id] });
  s.edit({ type: "setNode", id: other, title: state.title });
  expect(stateConnections(s.getSnapshot().doc, other)).toEqual({
    applications: [],
    events: [],
  });
  const edge = s
    .getSnapshot()
    .doc.edges.find(
      (e) => e.target.node === overlay.id && e.target.port === "state",
    )!;
  const [reroute] = s.edit({
    type: "reroute",
    edgeId: edge.id,
    position: { x: 0, y: 0 },
  });
  const doc = structuredClone(s.getSnapshot().doc);
  const upstream = doc.edges.find((e) => e.target.node === reroute)!;
  upstream.source.node = reroute;
  expect(inputSources(doc)(overlay, "state")).toBeUndefined();
  expect(stateConnections(doc, state.id).applications).toHaveLength(0);
  upstream.source.node = "missing";
  expect(inputSources(doc)(overlay, "state")).toBeUndefined();
});

test("Hello draft has complete typed connections and intentionally unresolved balance/policy fields", () => {
  const s = start();
  expect(s.getSnapshot().doc.nodes).toHaveLength(32);
  expect(s.getSnapshot().doc.frames).toHaveLength(3);
  expect(new Set(codes(s))).toEqual(
    new Set(["skill-value-unset", "skill-policy-unset"]),
  );
  expect(find(s, "Hello 유지 시간").values.value).toBe("");
  expect(find(s, "Goodbye 위력").values.curve).toBe("unresolved");
  expect(find(s, "Goodbye 재입력 허용").values.permission).toBe(
    "manual-followup",
  );
  expect(find(s, "Goodbye 마무리 권리").values.expiryOrder).toBe(
    "expiry-first",
  );
  const confirmed = new Set([
    find(s, "Goodbye 재입력 허용").id,
    find(s, "Goodbye 마무리 권리").id,
  ]);
  expect(
    s
      .getSnapshot()
      .diagnostics.filter(
        (d) => d.code === "skill-policy-unset" && confirmed.has(d.nodeId ?? ""),
      ),
  ).toEqual([]);
  expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
});

test("renaming a state and editing a duration preserve semantic links across history and file roundtrip", () => {
  const s = start(),
    state = find(s, "Hello 상태 선언"),
    duration = find(s, "Hello 유지 시간");
  const before = s.getSnapshot().doc;
  s.edit({ type: "setNode", id: state.id, title: "다른 표시 이름" });
  s.edit({ type: "setNode", id: duration.id, values: { value: 12.5 } });
  expect(s.getSnapshot().doc.edges).toEqual(before.edges);
  const saved = s.export();
  s.acknowledge(saved.token);
  expect(s.getSnapshot().dirty).toBe(false);
  const reopened = new GraphSession(parseDocument(saved.json));
  expect(reopened.getSnapshot().doc).toEqual(s.getSnapshot().doc);
  s.travel(false);
  expect(find(s, "Hello 유지 시간").values.value).toBe("");
  expect(s.getSnapshot().dirty).toBe(true);
  s.travel(true);
  expect(s.getSnapshot().dirty).toBe(false);
  s.travel(false);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
});

test("deleting a referenced state exposes missing inputs; deleting its installer exposes uninstalled state; undo restores all", () => {
  const s = start(),
    before = s.getSnapshot().doc;
  s.edit({ type: "delete", ids: [find(s, "Hello 상태 선언").id] });
  const missing = s
    .getSnapshot()
    .diagnostics.filter((d) => d.portId === "state");
  expect(missing).toHaveLength(5);
  expect(missing.every((d) => d.code === "unconnected-input")).toBe(true);
  expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
  s.edit({ type: "delete", ids: [find(s, "Hello 적용").id] });
  expect(codes(s).filter((c) => c === "skill-state-uninstalled")).toHaveLength(
    4,
  );
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
});

test("number-to-seconds and presentation-to-execution connections are rejected without changing history", () => {
  const s = start();
  const [number] = s.edit({
    type: "add",
    kind: "number",
    position: { x: 0, y: 0 },
  });
  const before = s.getSnapshot();
  for (const [source, target] of [
    [end(number, "value"), end(find(s, "Hello 적용").id, "duration")],
    [
      end(find(s, "Hello 적용").id, "cue"),
      end(find(s, "Goodbye 일격").id, "in"),
    ],
  ]) {
    expect(() => s.edit({ type: "connect", source, target })).toThrow(
      GraphError,
    );
    expect(s.getSnapshot()).toBe(before);
  }
});

test("activation from a different event cannot silently drive the reinput capture or end reason", () => {
  const s = start(),
    expiry = find(s, "Hello 자연 만료");
  s.edit({
    type: "connect",
    source: end(expiry.id, "activation"),
    target: end(find(s, "포기할 시간 고정").id, "activation"),
  });
  expect(codes(s)).toContain("skill-activation-scope");
  expect(codes(s)).toContain("skill-capture-source");
  s.travel(false);
  s.edit({
    type: "setNode",
    id: find(s, "만료 후 복귀").id,
    values: { reason: "manual" },
  });
  expect(codes(s)).toContain("skill-end-reason");
  expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
});

test("Goodbye diagnoses time captured outside its execution path, including rerouted values", () => {
  const s = start();
  const [other] = s.edit({
    type: "add",
    kind: "skill-capture-time",
    position: { x: 0, y: 0 },
  });
  s.edit({
    type: "connect",
    source: end(other, "seconds"),
    target: end(find(s, "Goodbye 위력").id, "seconds"),
  });
  expect(codes(s)).toContain("skill-result-scope");
  const edge = s.getSnapshot().doc.edges.find((e) => e.source.node === other)!;
  s.edit({ type: "reroute", edgeId: edge.id, position: { x: 300, y: 0 } });
  expect(codes(s)).toContain("skill-result-scope");
});

test("typed reroutes retain state ownership and valid result scope through undo", () => {
  const s = start(),
    before = s.getSnapshot().doc;
  for (const title of [
    "Hello 상태 선언",
    "포기할 시간 고정",
    "Goodbye 마무리 권리",
  ]) {
    const node = find(s, title);
    const edge = s
      .getSnapshot()
      .doc.edges.find(
        (e) => e.source.node === node.id && e.source.port !== "out",
      )!;
    s.edit({ type: "reroute", edgeId: edge.id, position: { x: 300, y: 500 } });
  }
  expect(new Set(codes(s))).toEqual(
    new Set(["skill-value-unset", "skill-policy-unset"]),
  );
  for (let i = 0; i < 3; i++) s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
});

test("copying the whole skill remaps internal state and cue references; partial copy remains diagnosable", () => {
  const s = start(),
    original = s.getSnapshot().doc;
  const fragment = s.copy(original.frames.map((f) => f.id));
  const selected = s.edit({
    type: "paste",
    fragment,
    position: { x: 3000, y: 100 },
  });
  expect(selected).toHaveLength(32);
  const pasted = new Set(selected);
  expect(
    s
      .getSnapshot()
      .doc.edges.filter((e) => pasted.has(e.source.node))
      .every((e) => pasted.has(e.target.node)),
  ).toBe(true);
  expect(codes(s).filter((c) => c === "skill-state-installers")).toHaveLength(
    0,
  );
  expect(codes(s).filter((c) => c === "skill-state-uninstalled")).toHaveLength(
    0,
  );
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(original);
  s.edit({
    type: "paste",
    fragment: s.copy([find(s, "Goodbye 마무리 권리").id]),
    position: { x: 3000, y: 100 },
  });
  expect(codes(s)).toContain("unconnected-input");
});

test("optional numbers remain unset instead of zero; invalid ranges and choices reject atomically", () => {
  const s = start(),
    duration = find(s, "Hello 유지 시간"),
    area = find(s, "Goodbye 전방 범위");
  const before = s.getSnapshot();
  for (const [id, values] of [
    [duration.id, { value: -1 }],
    [duration.id, { value: "twelve" }],
    [area.id, { angle: 361 }],
    [area.id, { shape: "unknown" }],
  ] as const) {
    expect(() =>
      s.edit({ type: "setNode", id, title: "must not leak", values }),
    ).toThrow(GraphError);
    expect(s.getSnapshot()).toBe(before);
  }
  s.edit({ type: "setNode", id: duration.id, values: { value: 0 } });
  expect(find(s, "Hello 유지 시간").values.value).toBe(0);
  s.edit({ type: "setNode", id: duration.id, values: { value: "" } });
  expect(find(s, "Hello 유지 시간").values.value).toBe("");
});

test("time-power accepts integer coefficients, rejects fractional files atomically and retains cap diagnostics", () => {
  const s = start(),
    power = find(s, "Goodbye 위력");
  const before = s.getSnapshot().doc;
  for (const field of ["base", "rate", "cap"]) {
    for (const value of [0.5, Number.MAX_SAFE_INTEGER + 1, -1]) {
      const snapshot = s.getSnapshot();
      expect(() =>
        s.edit({
          type: "setNode",
          id: power.id,
          title: "must not leak",
          values: { [field]: value },
        }),
      ).toThrow(GraphError);
      expect(s.getSnapshot()).toBe(snapshot);
      const invalid = structuredClone(before);
      invalid.nodes.find((n) => n.id === power.id)!.values[field] = value;
      expect(() => parseDocument(JSON.stringify(invalid))).toThrow(GraphError);
    }
  }
  s.edit({
    type: "setNode",
    id: power.id,
    values: { curve: "capped-linear", base: 100, rate: 10, cap: 110 },
  });
  expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
  s.travel(true);
  expect(find(s, "Goodbye 위력").values).toEqual({
    curve: "capped-linear",
    base: 100,
    rate: 10,
    cap: 110,
  });
  s.edit({ type: "setNode", id: power.id, values: { cap: 99 } });
  expect(codes(s)).toContain("skill-power-cap");
  s.edit({
    type: "setNode",
    id: power.id,
    values: { curve: "linear", base: 0, rate: 0, cap: "" },
  });
  expect(
    s.getSnapshot().diagnostics.filter((d) => d.nodeId === power.id),
  ).toEqual([]);
  expect(
    parseDocument(s.export().json).nodes.find((n) => n.id === power.id)?.values,
  ).toEqual({ curve: "linear", base: 0, rate: 0, cap: "" });
});

test("editing a presentation description changes no gameplay node, port or connection", () => {
  const s = start(),
    before = s.getSnapshot().doc,
    cue = find(s, "포신 변형 표시");
  s.edit({
    type: "setNode",
    id: cue.id,
    values: { description: "긴 한글 연출 설명\n팔의 모습만 변경합니다." },
  });
  const after = parseDocument(s.export().json);
  const gameplay = (d: GraphDocument) =>
    d.nodes.filter((n) => n.kind !== "skill-cue");
  expect(gameplay(after)).toEqual(gameplay(before));
  expect(after.edges).toEqual(before.edges);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
});

test("input branches require their own lookup result, including typed reroutes and bypass paths", () => {
  const s = start();
  const lookup = find(s, "Hello 활성 조회");
  const qualification = find(s, "Hello 첫 사용 자격");
  const permission = find(s, "Goodbye 재입력 허용");
  // The value is correctly typed, but the opposite flow branch cannot use it.
  s.edit({
    type: "connect",
    source: end(lookup.id, "present"),
    target: end(qualification.id, "in"),
  });
  expect(codes(s)).toContain("skill-path-scope");
  s.travel(false);
  const before = s.getSnapshot().doc;
  for (const edge of before.edges.filter((e) =>
    [lookup.id, qualification.id, permission.id].includes(e.source.node),
  ))
    s.edit({ type: "reroute", edgeId: edge.id, position: { x: 20, y: 20 } });
  expect(new Set(codes(s))).toEqual(
    new Set(["skill-value-unset", "skill-policy-unset"]),
  );
  const [bypass] = s.edit({
    type: "add",
    kind: "event",
    position: { x: 0, y: 0 },
  });
  s.edit({
    type: "connect",
    source: end(bypass, "out"),
    target: end(qualification.id, "in"),
  });
  expect(codes(s)).toContain("skill-path-scope");
  s.travel(false);
  expect(codes(s)).not.toContain("skill-path-scope");
});

test("start permits bind the queried state and followup permits bind the originating request", () => {
  const s = start();
  const [otherState] = s.edit({
    type: "duplicate",
    ids: [find(s, "Hello 상태 선언").id],
  });
  s.edit({
    type: "connect",
    source: end(otherState, "state"),
    target: end(find(s, "Hello 적용").id, "state"),
  });
  expect(codes(s)).toContain("skill-start-state");
  s.travel(false);
  const [otherRequest] = s.edit({
    type: "add",
    kind: "skill-request",
    position: { x: 0, y: 0 },
  });
  s.edit({
    type: "connect",
    source: end(otherRequest, "request"),
    target: end(find(s, "Goodbye 재입력 허용").id, "request"),
  });
  expect(codes(s)).toContain("skill-path-scope");
  expect(codes(s)).toContain("skill-request-scope");
});

test("capture and manual end require a claimed activation on every path", () => {
  const s = start();
  const lookup = find(s, "Hello 활성 조회");
  s.edit({
    type: "connect",
    source: end(lookup.id, "activation"),
    target: end(find(s, "포기할 시간 고정").id, "activation"),
  });
  expect(codes(s)).toContain("skill-capture-source");
  expect(codes(s)).toContain("skill-activation-scope");
  s.travel(false);
  const [bypass] = s.edit({
    type: "add",
    kind: "event",
    position: { x: 0, y: 0 },
  });
  s.edit({
    type: "connect",
    source: end(bypass, "out"),
    target: end(find(s, "Hello 조기 종료").id, "in"),
  });
  expect(codes(s)).toContain("skill-activation-scope");
  expect(codes(s)).toContain("skill-result-scope");
  s.travel(false);
  expect(new Set(codes(s))).toEqual(
    new Set(["skill-value-unset", "skill-policy-unset"]),
  );
});

test("lookup and permission cannot justify their values through a flow cycle", () => {
  const s = start();
  const doc = structuredClone(s.getSnapshot().doc);
  const qualify = find(s, "Hello 첫 사용 자격");
  doc.edges = doc.edges.filter(
    (e) => e.target.node !== qualify.id || e.target.port !== "in",
  );
  doc.edges.push({
    id: "cycle",
    source: end(qualify.id, "out"),
    target: end(qualify.id, "in"),
  });
  expect(skillDiagnostics(doc).map((d) => d.code)).toContain(
    "skill-path-scope",
  );
});

test("saved Hello example has the same semantic graph as the factory", async () => {
  const saved = parseDocument(
    await readFile("content/hello-goodbye.node-graph.json", "utf8"),
  );
  const semantic = (doc: GraphDocument) => ({
    nodes: doc.nodes
      .map((n) => ({
        kind: n.kind,
        title: n.title,
        values: n.values,
        note: n.note,
        // Saved examples may have a refined layout; positions are presentation.
        frame: doc.frames.find((f) => f.id === n.frame)?.title,
      }))
      .sort((a, b) => a.title.localeCompare(b.title)),
    edges: doc.edges
      .map(
        (e) =>
          `${doc.nodes.find((n) => n.id === e.source.node)!.title}:${e.source.port}->${doc.nodes.find((n) => n.id === e.target.node)!.title}:${e.target.port}`,
      )
      .sort(),
  });
  expect(semantic(saved)).toEqual(semantic(example("hello-goodbye")));
  expect(
    new Set(
      new GraphSession(saved).getSnapshot().diagnostics.map((d) => d.code),
    ),
  ).toEqual(new Set(["skill-value-unset", "skill-policy-unset"]));
});

test("Goodbye cannot bypass the matching manual end even when the captured time is available", () => {
  const s = start();
  s.edit({
    type: "connect",
    source: end(find(s, "포기할 시간 고정").id, "out"),
    target: end(find(s, "Goodbye 일격").id, "in"),
  });
  expect(codes(s)).toContain("skill-finish-required");
  s.travel(false);
  expect(codes(s)).not.toContain("skill-finish-required");
});

test("previous composite-node documents reopen without rewriting their stored contracts", async () => {
  const input = await readFile(
    "tests/model/fixtures/hello-goodbye-v1.json",
    "utf8",
  );
  const s = new GraphSession();
  s.load(input, s.getSnapshot());
  expect(JSON.parse(s.export().json)).toEqual(JSON.parse(input));
  expect(s.getSnapshot().dirty).toBe(false);
  expect(new Set(codes(s))).toEqual(
    new Set(["skill-value-unset", "skill-policy-unset", "skill-area-legacy"]),
  );
});
