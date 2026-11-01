import { test, expect } from "@playwright/test";
import { GraphSession } from "../../src/nodes/core/session";
import { spatialExample } from "../../src/nodes/core/spatial-examples";
import { definition } from "../../src/nodes/core/catalog";
import { parseDocument } from "../../src/nodes/core/validation";
import { spatialFieldActive } from "../../src/nodes/core/spatial-catalog";
import { readTilePattern } from "../../src/nodes/core/tile-pattern";

const node = (s: GraphSession, title: string) =>
  s.getSnapshot().doc.nodes.find((n) => n.title === title)!;
const codes = (s: GraphSession) =>
  s.getSnapshot().diagnostics.map((d) => d.code);
const set = (
  s: GraphSession,
  title: string,
  values: Record<string, string | boolean | number>,
) => s.edit({ type: "setNode", id: node(s, title).id, values });

test("common range is pure pattern; front and ground examples keep separate placement and unresolved balance", () => {
  expect(definition("skill-range").fields.map((f) => f.id)).toEqual([
    "pattern",
    "includeAnchor",
  ]);
  for (const mode of ["front-strike", "selected-ground"] as const) {
    const s = new GraphSession(spatialExample(mode));
    expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
    expect(new Set(codes(s))).toEqual(
      new Set(
        mode === "front-strike"
          ? ["skill-value-unset", "skill-policy-unset"]
          : ["skill-value-unset"],
      ),
    );
    for (const n of s
      .getSnapshot()
      .doc.nodes.filter((n) => n.kind === "skill-range"))
      expect(n.values.pattern).toBe("");
    expect(node(s, "효과 범위 배치").values.origin).toBe(
      mode === "front-strike" ? "caster" : "selected-tile",
    );
  }
});

test("asymmetric ground pattern is stored unchanged, independent from selection range and caster timing", () => {
  const s = new GraphSession(spatialExample("selected-ground"));
  const before = s.getSnapshot().doc;
  set(s, "대상 선택 범위", { pattern: "@...X" });
  set(s, "스킬 효과 범위", { pattern: "@X\n.X" });
  set(s, "장판 유지 시간", { value: 1.001 });
  set(s, "검증 저장 상한", { value: 100 });
  expect(codes(s)).toEqual([]);
  expect(
    readTilePattern(String(node(s, "스킬 효과 범위").values.pattern)).cells,
  ).toEqual([
    { x: 1, y: 0 },
    { x: 1, y: 1 },
  ]);
  const placement = node(s, "효과 범위 배치");
  const fields = definition(placement.kind).fields;
  expect(
    fields.filter((f) => spatialFieldActive(placement, f)).map((f) => f.id),
  ).toEqual(["origin"]);
  set(s, "효과 범위 배치", { positionTime: "release", facingTime: "accepted" });
  expect(node(s, "스킬 효과 범위").values.pattern).toBe("@X\n.X");
  expect(codes(s)).toEqual([]);
  const saved = parseDocument(s.export().json);
  expect(
    saved.nodes.find((n) => n.id === node(s, "대상 선택 범위").id)?.values
      .pattern,
  ).toBe("@...X");
  for (let i = 0; i < 5; i++) s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
  for (let i = 0; i < 5; i++) s.travel(true);
  expect(s.getSnapshot().doc).toEqual(saved);
});

test("target selection and ground placement require the same admitted request and live use path", () => {
  const s = new GraphSession(spatialExample("selected-ground"));
  set(s, "입력의 대상 선택", { mode: "self" });
  expect(codes(s)).toContain("spatial-target-context");
  s.travel(false);
  set(s, "효과 범위 배치", { origin: "caster" });
  expect(codes(s)).toContain("spatial-ground-origin");
  s.travel(false);
  const create = node(s, "지정 타일 장판 생성");
  const admit = node(s, "접수와 첫 지불");
  s.edit({
    type: "connect",
    source: { node: admit.id, port: "rejected" },
    target: { node: create.id, port: "in" },
  });
  expect(codes(s)).toContain("resonance-use-scope");
  s.travel(false);
  set(s, "장판 유지 시간", { value: 0 });
  expect(codes(s)).toContain("spatial-duration");
});

test("selection range cannot substitute a placed effect; removing reach diagnoses without dropping authored pattern", () => {
  const s = new GraphSession(spatialExample("selected-ground"));
  const range = node(s, "대상 선택 범위"),
    create = node(s, "지정 타일 장판 생성");
  const before = s.getSnapshot();
  expect(() =>
    s.edit({
      type: "connect",
      source: { node: range.id, port: "range" },
      target: { node: create.id, port: "area" },
    }),
  ).toThrow();
  expect(s.getSnapshot()).toBe(before);
  const edge = before.doc.edges.find((e) => e.source.node === range.id)!;
  s.edit({ type: "disconnect", ids: [edge.id] });
  expect(codes(s)).toContain("spatial-selection-range");
  expect(s.getSnapshot().doc.nodes.find((n) => n.id === range.id)).toEqual(
    range,
  );
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before.doc);
});

test("typed reroutes preserve range, selection, use and duration diagnostics through copy and reload", () => {
  const s = new GraphSession(spatialExample("selected-ground"));
  const before = s.getSnapshot().diagnostics;
  for (const edge of [...s.getSnapshot().doc.edges])
    s.edit({ type: "reroute", edgeId: edge.id, position: { x: 0, y: 0 } });
  expect(s.getSnapshot().diagnostics).toEqual(before);
  expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
  const fragment = s.copy(s.getSnapshot().doc.nodes.map((n) => n.id));
  const pasted = new GraphSession();
  pasted.edit({ type: "paste", fragment, position: { x: 0, y: 0 } });
  expect(new Set(codes(pasted))).toEqual(new Set(["skill-value-unset"]));
});
