import type { GraphDocument } from "../types";
import { p4ValidationContext } from "./p4-validation-context";
import { readTilePattern } from "./tile-pattern";

export function offensiveDiagnostics(doc: GraphDocument) {
  const { out, source, issue, requireKind, positive, ofKind, plans } =
    p4ValidationContext(doc);
  for (const n of doc.nodes) {
    if (n.kind === "skill-limited-range") {
      const base = requireKind(n, "base", ["range"]);
      if (
        base &&
        typeof n.values.tiles === "number" &&
        readTilePattern(String(base.values.pattern)).cells.some(
          (p) =>
            Math.max(Math.abs(p.x), Math.abs(p.y)) > Number(n.values.tiles),
        )
      )
        issue(
          n,
          "range-distance-limit",
          "기본 효과 타일이 지정한 최대 거리를 벗어났습니다.",
          "base",
        );
    }
    if (n.kind === "skill-delayed-area-plan") {
      positive(n, "delay", true);
      const select = requireKind(n, "selector", ["enemy-selector"]);
      if (n.values.selectionTime === "accepted")
        for (const entry of ofKind("action-entry")) {
          if (
            plans(source(entry, "plan")).some((p) => p.id === n.id) &&
            source(entry, "selector")?.id !== select?.id
          )
            issue(
              n,
              "impact-accepted-selector",
              "접수 때 고정할 위치는 행동 시작 설정과 같은 선택기를 사용해야 합니다.",
              "selector",
            );
        }
    }
    if (n.kind === "skill-scheduled-plan") {
      positive(n, "interval");
      requireKind(n, "count", ["count-positive"]);
      const body = requireKind(n, "body", ["delayed-area-plan"]);
      if (body && body.values.selectionTime !== "start")
        issue(
          n,
          "scheduled-selection",
          "유한 기회는 각 기회 시작에 새 목표를 선택해야 합니다.",
          "body",
        );
    }
    if (n.kind === "skill-on-hit-plan") {
      const body = source(n, "body");
      if (
        plans(body).some((p) =>
          [
            "skill-scheduled-plan",
            "skill-next-basic-plan",
            "skill-recipient-buff-plan",
            "skill-aura-buff-plan",
            "skill-reactive-heal-plan",
            "skill-parry-plan",
          ].includes(p.kind),
        )
      )
        issue(
          n,
          "hit-outcome-scope",
          "독립 구독·예약 설치를 직접 타격 완료 집계로 사용할 수 없습니다.",
          "body",
        );
    }
    if (n.kind === "skill-guard-charge") {
      const guard = requireKind(n, "guard", ["passive-guard"]);
      const parry = requireKind(n, "parry", ["parry-plan"]);
      requireKind(n, "condition", ["equipment-condition"]);
      requireKind(n, "range", ["range"]);
      if (guard && parry && source(parry, "guard")?.id !== guard.id)
        issue(
          n,
          "guard-charge-source",
          "축적과 받아치기는 같은 감면 출처를 참조해야 합니다.",
          "guard",
        );
      if (n.values.factor === 0)
        issue(
          n,
          "guard-charge-power",
          "주 대상 피해 배율은 0보다 커야 합니다.",
        );
      if (
        guard &&
        ofKind("guard-charge").filter(
          (c) => source(c, "guard")?.id === guard.id,
        ).length > 1
      )
        issue(
          n,
          "guard-charge-duplicate",
          "같은 감면 출처에는 축적 선언을 하나만 연결합니다.",
          "guard",
        );
    }
    if (n.kind === "skill-direct-reflection") {
      const guard = requireKind(n, "guard", ["passive-guard"]);
      if (
        guard &&
        ofKind("direct-reflection").filter(
          (p) => source(p, "guard")?.id === guard.id,
        ).length > 1
      )
        issue(
          n,
          "guard-reflection-duplicate",
          "같은 감면 출처의 반사 구독이 중복되었습니다.",
          "guard",
        );
    }
    if (n.kind === "skill-parry-plan") {
      requireKind(n, "guard", ["passive-guard"]);
      for (const port of ["window", "immunity", "failure"]) positive(n, port);
    }
  }
  return out;
}
