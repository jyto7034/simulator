import { placeExample } from "./example-layout";
import type { GraphNode } from "../types";
import { makeNode } from "./catalog";
import { canonicalize, id } from "./model";
import { helloGoodbye } from "./hello-goodbye";
import { resonanceExample } from "./resonance-examples";

// Structural authoring examples. No attack range, duration or damage balance is chosen.
export function spatialExample(mode: "front-strike" | "selected-ground") {
  const doc =
    mode === "front-strike" ? helloGoodbye() : resonanceExample("all");
  doc.name =
    mode === "front-strike"
      ? "범위 구성 · 전방 일격"
      : "범위 구성 · 지정 타일 장판";
  const find = (kind: string) => doc.nodes.find((n) => n.kind === kind)!;
  const frame = { id: id(), title: "범위 구성 · 타일 배치와 수치는 미정" };
  doc.frames.push(frame);
  const add = (
    kind: string,
    title: string,
    x: number,
    y: number,
    values: GraphNode["values"] = {},
  ) => {
    const n = makeNode(`skill-${kind}`, { x, y });
    n.title = title;
    n.values = { ...n.values, ...values };
    n.frame = frame.id;
    doc.nodes.push(n);
    return n;
  };
  const link = (a: GraphNode, ap: string, b: GraphNode, bp: string) =>
    doc.edges.push({
      id: id(),
      source: { node: a.id, port: ap },
      target: { node: b.id, port: bp },
    });
  // Hello already declares its request selection. Reuse that declaration;
  // attaching another one would make the request's single input invalid.
  const request = find("skill-request");
  const existing = doc.edges.find(
    (e) => e.target.node === request.id && e.target.port === "selection",
  );
  const selection = existing
    ? doc.nodes.find((n) => n.id === existing.source.node)!
    : add("target-selection", "입력의 대상 선택", 40, 2700);
  selection.title = "입력의 대상 선택";
  selection.values.mode = mode === "front-strike" ? "self" : "player-tile";
  if (!existing) link(selection, "selection", request, "selection");
  const range = add("range", "스킬 효과 범위", 680, 2700);
  const placement = add("range-placement", "효과 범위 배치", 1000, 2700, {
    origin: mode === "front-strike" ? "caster" : "selected-tile",
  });
  link(range, "range", placement, "range");
  if (mode === "front-strike") {
    const old = find("skill-area");
    doc.edges = doc.edges.filter(
      (e) => e.source.node !== old.id && e.target.node !== old.id,
    );
    doc.nodes = doc.nodes.filter((n) => n.id !== old.id);
    link(placement, "area", find("skill-area-damage"), "area");
  } else {
    const reach = add("range", "대상 선택 범위", 40, 3100);
    link(reach, "range", selection, "range");
    const duration = add("seconds", "장판 유지 시간", 680, 3100);
    const create = add("ground-area", "지정 타일 장판 생성", 1320, 2700);
    create.note =
      "범위와 생성 수명을 확인하는 구조 예제입니다. 피해·회복·진입·체류·주기 효과는 아직 연결하지 않았습니다.";
    link(placement, "area", create, "area");
    link(duration, "value", create, "duration");
    link(find("skill-resonance-admit"), "use", create, "use");
    const end = find("skill-resonance-end");
    const beforeEnd = doc.edges.find(
      (e) => e.target.node === end.id && e.target.port === "in",
    )!;
    beforeEnd.target = { node: create.id, port: "in" };
    link(create, "out", end, "in");
    // The generic resonance fixture's numeric example must not become a skill balance default.
    find("skill-resonance-amount").values.value = "";
  }
  placeExample(doc);
  canonicalize(doc);
  return doc;
}
