import { test, expect } from "@playwright/test";
import { GraphSession } from "../../src/nodes/core/session";
import { groundExample } from "../../src/nodes/core/ground-examples";
import { groundFieldActive } from "../../src/nodes/core/ground-catalog";
import { makeNode, definition } from "../../src/nodes/core/catalog";
import { parseDocument } from "../../src/nodes/core/validation";
import type { FieldValue, GraphDocument } from "../../src/nodes/types";

const find = (s: GraphSession, title: string) =>
  s.getSnapshot().doc.nodes.find((n) => n.title === title)!;
const codes = (s: GraphSession) =>
  s.getSnapshot().diagnostics.map((d) => d.code);
const set = (
  s: GraphSession,
  title: string,
  values: Record<string, FieldValue>,
) => s.edit({ type: "setNode", id: find(s, title).id, values });

function add(s: GraphSession, kind: string) {
  const before = new Set(s.getSnapshot().doc.nodes.map((n) => n.id));
  s.edit({ type: "add", kind, position: { x: 0, y: 0 } });
  return s.getSnapshot().doc.nodes.find((n) => !before.has(n.id))!;
}

function assigned(mode: "damage" | "heal" | "status") {
  const s = new GraphSession(groundExample(mode));
  for (const n of s.getSnapshot().doc.nodes) {
    const values: Record<string, FieldValue> = {};
    for (const f of definition(n.kind).fields) {
      if (f.valueType === "optional-number" && n.values[f.id] === "")
        values[f.id] = n.kind === "skill-seconds" ? 1 : 10;
    }
    if (n.kind === "skill-range") values.pattern = "@X";
    if (n.kind === "skill-ground-periodic") values.first = "after-interval";
    if (n.kind === "skill-ground-damage") values.damageType = "magic";
    if (n.kind === "skill-ground-status") {
      values.overlap = "strongest";
      values.lifetime = "membership";
    }
    if (Object.keys(values).length)
      s.edit({ type: "setNode", id: n.id, values });
  }
  set(s, "장판 유지 시간", { value: 3 });
  return s;
}

test("ground examples expose unresolved balance and policies while preserving complete typed connections", () => {
  for (const mode of ["damage", "heal", "status"] as const) {
    const unfinished = new GraphSession(groundExample(mode));
    expect(new Set(codes(unfinished))).toEqual(
      new Set(["skill-value-unset", "skill-policy-unset"]),
    );
    expect(parseDocument(unfinished.export().json)).toEqual(
      unfinished.getSnapshot().doc,
    );
    const s = assigned(mode);
    expect(codes(s)).toEqual([]);
    expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
    const create = find(s, "지정 타일 장판 생성");
    const end = s
      .getSnapshot()
      .doc.nodes.find((n) => n.kind === "skill-resonance-end")!;
    expect(s.getSnapshot().doc.edges).toContainEqual(
      expect.objectContaining({
        source: { node: create.id, port: "out" },
        target: { node: end.id, port: "in" },
      }),
    );
    // Ground subscriptions bind to the independent area, never to the now-ended use.
    for (const n of s
      .getSnapshot()
      .doc.nodes.filter((n) =>
        [
          "skill-ground-periodic",
          "skill-ground-enter",
          "skill-ground-dwell",
          "skill-ground-exit",
          "skill-ground-ended",
        ].includes(n.kind),
      )) {
      expect(s.getSnapshot().doc.edges).toContainEqual(
        expect.objectContaining({
          source: { node: create.id, port: "area" },
          target: { node: n.id, port: "area" },
        }),
      );
    }
  }
});

test("first pulse choice is an undoable authored policy, with missing or zero periods diagnosed", () => {
  const s = assigned("damage");
  const before = s.getSnapshot().doc;
  set(s, "장판 주기", { first: "immediate" });
  const saved = parseDocument(s.export().json);
  expect(find(s, "장판 주기").values.first).toBe("immediate");
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
  s.travel(true);
  expect(s.getSnapshot().doc).toEqual(saved);
  set(s, "장판 주기", { first: "unresolved" });
  expect(codes(s)).toContain("skill-policy-unset");
  s.travel(false);
  set(s, "장판 발동 주기", { value: 0 });
  expect(codes(s)).toContain("ground-positive-time");
  s.travel(false);
  const snapshot = s.getSnapshot();
  expect(() => set(s, "장판 발동 주기", { value: 0.0001 })).toThrow();
  expect(s.getSnapshot()).toBe(snapshot);
  expect(() => set(s, "장판 주기", { first: "unrecognized" })).toThrow();
  expect(s.getSnapshot()).toBe(snapshot);
});

test("each status definition independently retains overlap and lifetime through kind changes and history", () => {
  const s = assigned("status");
  const before = s.getSnapshot().doc;
  const stun = find(s, "체류 기절 정의");
  set(s, "진입 둔화 정의", { overlap: "sum", lifetime: "area", percent: 20 });
  expect(find(s, "체류 기절 정의")).toEqual(stun);
  expect(codes(s)).toEqual([]);
  set(s, "진입 둔화 정의", { effect: "stun" });
  const node = find(s, "진입 둔화 정의");
  expect(
    definition(node.kind)
      .fields.filter((f) => groundFieldActive(node, f))
      .map((f) => f.id),
  ).toEqual(["effect", "lifetime"]);
  expect(node.values.percent).toBe(20);
  set(s, "진입 둔화 정의", { effect: "slow" });
  expect(find(s, "진입 둔화 정의").values).toEqual({
    effect: "slow",
    percent: 20,
    overlap: "sum",
    lifetime: "area",
  });
  const saved = parseDocument(s.export().json);
  for (let i = 0; i < 3; i++) s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
  for (let i = 0; i < 3; i++) s.travel(true);
  expect(s.getSnapshot().doc).toEqual(saved);
  set(s, "필요 체류 시간", { value: 0 });
  expect(codes(s)).toContain("ground-positive-time");
});

test("closed or foreign event flows cannot supply live state targets or bypass per-target execution", () => {
  const s = assigned("status");
  const enter = find(s, "장판 진입"),
    exit = find(s, "장판 이탈");
  const apply = find(s, "진입 대상 둔화");
  s.edit({
    type: "connect",
    source: { node: exit.id, port: "member" },
    target: { node: apply.id, port: "member" },
  });
  expect(codes(s)).toContain("ground-member-scope");
  expect(codes(s)).toContain("ground-closed-member");
  s.travel(false);
  s.edit({
    type: "connect",
    source: { node: exit.id, port: "out" },
    target: { node: apply.id, port: "in" },
  });
  expect(codes(s)).toContain("ground-member-scope");
  s.travel(false);
  expect(codes(s)).toEqual([]);
  expect(find(s, "장판 진입").id).toBe(enter.id);
  const damage = assigned("damage");
  const end = find(damage, "이 장판 종료"),
    effect = find(damage, "대상별 피해");
  damage.edit({
    type: "connect",
    source: { node: end.id, port: "out" },
    target: { node: effect.id, port: "in" },
  });
  expect(codes(damage)).toContain("ground-member-scope");
});

test("target queries reject another area and an ended-area event even through reroutes", () => {
  const s = assigned("damage");
  const each = find(s, "현재 장판 대상");
  const other = add(s, "skill-ground-area");
  s.edit({
    type: "connect",
    source: { node: other.id, port: "area" },
    target: { node: each.id, port: "area" },
  });
  expect(codes(s)).toContain("ground-event-scope");
  s.travel(false);
  s.travel(false);
  const end = find(s, "이 장판 종료");
  s.edit({
    type: "connect",
    source: { node: end.id, port: "out" },
    target: { node: each.id, port: "in" },
  });
  expect(codes(s)).toContain("ground-event-scope");
  const edge = s
    .getSnapshot()
    .doc.edges.find(
      (e) => e.source.node === end.id && e.source.port === "out",
    )!;
  s.edit({ type: "reroute", edgeId: edge.id, position: { x: 0, y: 0 } });
  expect(codes(s)).toContain("ground-event-scope");
});

test("state cleanup is scoped to the same area, status definition and membership filter", () => {
  const s = assigned("status");
  const exit = find(s, "장판 이탈"),
    status = find(s, "진입 둔화 정의");
  const clear = add(s, "skill-ground-clear-status");
  for (const [node, from, to] of [
    [exit, "out", "in"],
    [exit, "member", "member"],
    [status, "status", "status"],
  ] as const)
    s.edit({
      type: "connect",
      source: { node: node.id, port: from },
      target: { node: clear.id, port: to },
    });
  expect(codes(s)).toEqual([]);
  const foreign = add(s, "skill-ground-status");
  s.edit({
    type: "connect",
    source: { node: foreign.id, port: "status" },
    target: { node: clear.id, port: "status" },
  });
  expect(codes(s)).toContain("ground-status-source");
  s.travel(false);
  s.travel(false);
  const filter = add(s, "skill-ground-filter");
  s.edit({
    type: "connect",
    source: { node: filter.id, port: "filter" },
    target: { node: exit.id, port: "filter" },
  });
  expect(codes(s)).toContain("ground-status-filter");
});

test("healing and damage remain distinct typed values; failed links preserve the whole edit transaction", () => {
  const s = assigned("heal");
  const power = add(s, "skill-damage-amount");
  const before = s.getSnapshot();
  expect(() =>
    s.edit({
      type: "connect",
      source: { node: power.id, port: "power" },
      target: { node: find(s, "대상별 회복").id, port: "power" },
    }),
  ).toThrow();
  expect(s.getSnapshot()).toBe(before);
  const invalid: GraphDocument = structuredClone(before.doc);
  invalid.nodes.push(makeNode("skill-ground-periodic", { x: 0, y: 0 }));
  invalid.nodes.at(-1)!.values.first = "now";
  expect(() => parseDocument(JSON.stringify(invalid))).toThrow();
  expect(s.getSnapshot()).toBe(before);
});

test("reroutes, copy/paste and JSON reload preserve all ground ownership and unfinished policies", () => {
  for (const mode of ["damage", "heal", "status"] as const) {
    const s = assigned(mode);
    for (const e of [...s.getSnapshot().doc.edges])
      s.edit({ type: "reroute", edgeId: e.id, position: { x: 0, y: 0 } });
    expect(codes(s)).toEqual([]);
    expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
    const other = new GraphSession();
    other.edit({
      type: "paste",
      fragment: s.copy(s.getSnapshot().doc.nodes.map((n) => n.id)),
      position: { x: 0, y: 0 },
    });
    expect(codes(other)).toEqual([]);
  }
});
