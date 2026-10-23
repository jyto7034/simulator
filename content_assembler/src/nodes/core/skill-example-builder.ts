import { placeExample } from "./example-layout";
import type { GraphDocument, GraphNode } from "../types";
import { makeNode } from "./catalog";
import { canonicalize, createDocument, id } from "./model";

// Shared authoring fixture assembly. It does not interpret or execute a skill.
export function skillExampleBuilder(name: string, existing?: GraphDocument) {
  const doc = existing ?? createDocument();
  doc.name = name;
  let currentFrame: string | null = null,
    index = 0,
    baseY = existing
      ? Math.max(0, ...existing.nodes.map((n) => n.position.y)) + 600
      : 0;
  const frame = (title: string) => {
    if (currentFrame) baseY += Math.ceil(index / 5) * 600 + 300;
    const f = { id: id(), title };
    doc.frames.push(f);
    currentFrame = f.id;
    index = 0;
  };
  const add = (
    kind: string,
    title: string,
    values: GraphNode["values"] = {},
  ) => {
    const node = makeNode(kind === "number" ? kind : `skill-${kind}`, {
      x: 40 + (index % 5) * 340,
      y: baseY + Math.floor(index / 5) * 600 + 80,
    });
    index++;
    node.title = title;
    node.values = { ...node.values, ...values };
    node.frame = currentFrame;
    doc.nodes.push(node);
    return node;
  };
  const link = (a: GraphNode, ap: string, b: GraphNode, bp: string) =>
    doc.edges.push({
      id: id(),
      source: { node: a.id, port: ap },
      target: { node: b.id, port: bp },
    });
  const seconds = (title: string, value: number | "" = "") =>
    add("seconds", title, { value });
  const admission = (
    request: GraphNode,
    settings: GraphNode,
    free = false,
    terms?: GraphNode,
    program?: GraphNode,
  ) => {
    const read = add("resonance-read", "이번 요청의 공명 조회");
    const requirement = add(
      "resonance-requirement",
      free ? "자동 · 공명 조건 없음" : "수동 · 공명 만충",
      { mode: free ? "none" : "full" },
    );
    const cost = add(
      "resonance-cost",
      free ? "자동 · 추가 공명 없음" : "수동 · 공명 전량 소비",
      { mode: free ? "zero" : "all" },
    );
    const qualify = add("resonance-qualify", "대상·공명·자원 자격 검사");
    const admit = add("resonance-admit", "원자적 접수·비용 확정");
    link(request, "request", read, "request");
    link(settings, "settings", read, "settings");
    link(read, "snapshot", cost, "snapshot");
    link(read, "snapshot", qualify, "snapshot");
    link(requirement, "requirement", qualify, "requirement");
    link(cost, "cost", qualify, "cost");
    link(request, "request", qualify, "request");
    link(request, "out", qualify, "in");
    if (terms) link(terms, "terms", qualify, "terms");
    if (program) link(program, "program", qualify, "program");
    link(qualify, "out", admit, "in");
    link(qualify, "permit", admit, "permit");
    return admit;
  };
  const settings = () => {
    const capacity = add("resonance-amount", "최대 공명 · 미정");
    const result = add("resonance-settings", "사본 공명 저장 설정");
    link(capacity, "value", result, "capacity");
    return result;
  };
  const action = (
    plan: GraphNode,
    mode: "self" | "player-ally" | "player-tile" | "player-direction" = "self",
  ) => {
    frame("시작 → 대상·공명 자격 → 비용 확정 → 행동 → 종료");
    const entry = add("action-entry", "접수할 행동 정의", {
      start: "immediate",
      interruption: "standard",
    });
    link(plan, "plan", entry, "plan");
    const selection = add("target-selection", "입력 대상 선택", { mode });
    if (mode === "player-ally" || mode === "player-tile")
      link(add("range", "대상 선택 범위 · 미정"), "range", selection, "range");
    const request = add("request", "파편 시전 입력");
    link(selection, "selection", request, "selection");
    const use = admission(request, settings(), false, undefined, entry);
    const run = add("run-action", "접수한 행동 실행·종료");
    link(use, "out", run, "in");
    link(use, "use", run, "use");
    link(entry, "program", run, "program");
    return { run, entry, use, request };
  };
  const finish = () => {
    placeExample(doc);
    canonicalize(doc);
    return doc;
  };
  return {
    doc,
    frame,
    add,
    link,
    seconds,
    admission,
    settings,
    action,
    finish,
  };
}
