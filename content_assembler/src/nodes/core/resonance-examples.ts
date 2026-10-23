import type { GraphNode } from "../types";
import { makeNode } from "./catalog";
import { canonicalize, createDocument, id } from "./model";

export const resonanceExamples = [
  "all",
  "fixed",
  "max-percent",
  "current-percent",
  "selected",
  "count",
  "zero",
  "periodic",
  "stage",
] as const;
export type ResonanceExample = (typeof resonanceExamples)[number];
const names: Record<ResonanceExample, string> = {
  all: "만충·전량",
  fixed: "고정량",
  "max-percent": "최대량 비율",
  "current-percent": "현재량 비율",
  selected: "선택 투입량",
  count: "횟수 비례",
  zero: "공명 소비 없음",
  periodic: "유지 비용",
  stage: "단계 비용",
};

// Authoring fixtures, deliberately separate from playable content and its balance.
export function resonanceExample(mode: ResonanceExample) {
  const doc = createDocument();
  doc.name = `검증 예제 · 공명 ${names[mode]}`;
  const frame = {
    id: id(),
    title: "검증 전용 수치 · 게임 콘텐츠에 적용하지 않음",
  };
  doc.frames = [frame];
  const nodes = new Map<string, GraphNode>();
  const add = (
    key: string,
    kind: string,
    title: string,
    x: number,
    y: number,
    values: GraphNode["values"] = {},
  ) => {
    const node = makeNode(
      kind.startsWith("skill-") || kind === "number"
        ? kind
        : `skill-resonance-${kind}`,
      { x, y },
    );
    node.title = title;
    node.frame = frame.id;
    node.values = { ...node.values, ...values };
    nodes.set(key, node);
    doc.nodes.push(node);
  };
  const link = (a: string, ap: string, b: string, bp: string) =>
    doc.edges.push({
      id: id(),
      source: { node: nodes.get(a)!.id, port: ap },
      target: { node: nodes.get(b)!.id, port: bp },
    });
  const followup = mode === "periodic" || mode === "stage";
  add("request", "skill-request", "검증 사용 요청", 40, 80);
  add("qualify", "qualify", "사용 자격 검사", 1000, 80);
  add("admit", "admit", "접수와 첫 지불", 1320, 80);
  add("result", "result", "첫 지불 결과", 1640, 80);
  add("end", "end", "사용 완료", 1960, followup ? 1360 : 80);
  add("capacity", "amount", "검증 저장 상한", 40, 440, { value: 100 });
  add("settings", "settings", "공명 저장 설정", 360, 440);
  add("read", "read", "요청의 공명 조회", 680, 440);
  add("requirement", "requirement", "사용 공명 조건", 1000, 440, {
    mode:
      mode === "all"
        ? "full"
        : followup || mode === "zero"
          ? "none"
          : "payable",
  });
  const initialMode = followup ? "zero" : mode;
  add("cost", "cost", "첫 지불 비용", 1320, 440, {
    mode: initialMode,
    ...(mode.endsWith("percent") ? { percent: 25, minimumCost: 0 } : {}),
  });
  link("request", "out", "qualify", "in");
  link("request", "request", "qualify", "request");
  link("request", "request", "read", "request");
  link("capacity", "value", "settings", "capacity");
  link("settings", "settings", "read", "settings");
  link("read", "snapshot", "qualify", "snapshot");
  link("read", "snapshot", "cost", "snapshot");
  link("requirement", "requirement", "qualify", "requirement");
  link("cost", "cost", "qualify", "cost");
  link("qualify", "out", "admit", "in");
  link("qualify", "permit", "admit", "permit");
  link("admit", "out", "result", "in");
  link("admit", "receipt", "result", "receipt");
  link("admit", "use", "end", "use");
  if (!followup) link("result", "out", "end", "in");
  if (mode === "fixed" || mode === "count" || followup) {
    add("amount", "amount", "회당 공명 비용", 680, 800, { value: 20 });
    if (!followup) link("amount", "value", "cost", "amount");
  }
  if (mode === "selected" || mode === "count") {
    add("input", "input", "요청에서 선택한 값", 40, 800);
    link("request", "request", "input", "request");
    link(
      "input",
      mode === "selected" ? "amount" : "count",
      "cost",
      mode === "selected" ? "selected" : "count",
    );
  }
  if (mode === "selected") {
    add("min", "amount", "선택 최소량", 360, 800, { value: 10 });
    add("max", "amount", "선택 최대량", 680, 800, { value: 100 });
    link("min", "value", "cost", "minimum");
    link("max", "value", "cost", "maximum");
  }
  if (followup) {
    add(
      "step",
      mode === "periodic" ? "periodic" : "stage",
      mode === "periodic" ? "유지 비용 주기" : "후속 공격 단계",
      40,
      1120,
      mode === "periodic" ? { limit: "count", count: 3 } : {},
    );
    add("later-cost", "cost", "후속 지불 비용", 1000, 800, { mode: "fixed" });
    add("pay", "pay", "단계의 공명 지불", 1320, 1120, {
      insufficient: "stop-use",
    });
    add("later-result", "result", "후속 지불 결과", 1640, 1120);
    link("admit", "use", "step", "use");
    link("read", "snapshot", "later-cost", "snapshot");
    link("amount", "value", "later-cost", "amount");
    link("step", "out", "pay", "in");
    link("step", "step", "pay", "step");
    link("later-cost", "cost", "pay", "cost");
    link("pay", "out", "later-result", "in");
    link("pay", "receipt", "later-result", "receipt");
    if (mode === "periodic") {
      add("interval", "skill-seconds", "유지 비용 간격", 360, 1120, {
        value: 2,
      });
      link("interval", "value", "step", "interval");
      link("step", "completed", "end", "in");
    } else {
      link("result", "out", "step", "in");
      link("later-result", "out", "end", "in");
    }
  }
  canonicalize(doc);
  return doc;
}
