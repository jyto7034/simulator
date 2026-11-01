import type { GraphDocument, GraphNode } from "../types";
import { p4ValidationContext } from "./p4-validation-context";

const actionKinds = [
  "place-fusion-plan",
  "spawn-mobile-plan",
  "convert-status-plan",
  "self-heal-plan",
  "on-hit-plan",
  "delayed-area-plan",
  "scheduled-plan",
  "parry-plan",
  "strike-plan",
  "recipient-buff-plan",
  "next-basic-plan",
  "reactive-heal-plan",
  "aura-buff-plan",
  "sequence-plan",
  "repeat-plan",
  "spend-plan",
  "stance-plan",
  "timed-plan",
  "wait-plan",
  "mark-burst-plan",
  "path-burst-plan",
  "stack-burst-plan",
];
export function actionPlanDiagnostics(doc: GraphDocument) {
  const c = p4ValidationContext(doc);
  const {
    out,
    source,
    sources,
    issue,
    required,
    requireKind,
    positive,
    positiveField,
    ofKind,
    entries,
    plans,
    counts,
    planChildren,
    admittedProgram,
    requestFor,
  } = c;
  const checkCounts = (
    n: GraphNode,
    port: string,
    paid: GraphNode | undefined,
    allowStock = true,
  ) => {
    for (const p of counts(source(n, port))) {
      if (p.kind === "skill-count-paid" && !paid)
        issue(
          n,
          "action-paid-scope",
          "실제 소비량은 성공한 소비 단계의 본문/후속 상태 안에서만 읽을 수 있습니다.",
          port,
        );
      if (p.kind === "skill-count-stock" && !allowStock)
        issue(
          n,
          "action-snapshot-scope",
          "행동 시작 재고는 이 독립 투사체 효과의 문맥에 없습니다.",
          port,
        );
    }
  };
  const statusCounts = (
    status: GraphNode,
    paid: GraphNode | undefined,
    allowStock = true,
  ) => {
    if (status.values.effect === "vulnerability")
      checkCounts(status, "count", paid, allowStock);
  };
  const walkEntry = (entry: GraphNode) => {
    const visited = new Set<string>();
    const walk = (node: GraphNode | undefined, paid?: GraphNode) => {
      if (!node) return;
      const key = `${node.id}:${paid?.id ?? "none"}`;
      if (visited.has(key)) return;
      visited.add(key);
      if (node.kind === "skill-spend-plan") {
        if (node.values.spend === "amount") {
          checkCounts(node, "count", paid);
          const bank = source(node, "bank");
          if (
            bank &&
            counts(source(node, "count")).some(
              (p) =>
                p.kind === "skill-count-stock" &&
                source(p, "bank")?.id !== bank.id,
            )
          )
            issue(
              node,
              "action-cost-bank",
              "단계 비용을 고정한 재고와 소비할 재고가 다릅니다.",
              "count",
            );
        }
        const status = source(node, "after");
        if (status) statusCounts(status, node);
        walk(source(node, "body"), node);
        return;
      }
      if (node.kind === "skill-repeat-plan") checkCounts(node, "count", paid);
      if (node.kind === "skill-strike-plan")
        sources(node, "after").forEach((s) => statusCounts(s, paid));
      if (node.kind === "skill-timed-plan") {
        const duration = source(node, "duration");
        if (duration?.kind === "skill-count-duration")
          checkCounts(duration, "count", paid);
      }
      planChildren(node).forEach((p) => walk(p, paid));
    };
    walk(source(entry, "plan"));
  };
  for (const n of doc.nodes) {
    if (n.kind === "skill-hit-status") {
      positive(n, "duration");
      if (n.values.effect === "vulnerability") {
        required(n, "count");
        positiveField(n, "percent");
      } else if (source(n, "count"))
        issue(
          n,
          "action-status-count",
          "스택 개수는 받는 피해 증가 상태에서만 사용합니다.",
          "count",
        );
    }
    if (n.kind === "skill-hit-ramp") positiveField(n, "percent");
    if (n.kind === "skill-strike-plan") {
      if (
        ["selected-area", "forward-area", "selected-direction-area"].includes(
          String(n.values.target),
        )
      )
        required(n, "range");
      else if (source(n, "range"))
        issue(
          n,
          "action-strike-range",
          "주 대상 직접 공격은 범위 입력을 사용하지 않습니다.",
          "range",
        );
    }
    if (n.kind === "skill-sequence-plan") {
      requireKind(n, "first", actionKinds);
      requireKind(n, "next", actionKinds);
    }
    if (n.kind === "skill-repeat-plan") {
      requireKind(n, "body", ["strike-plan"]);
      positive(n, "time");
    }
    if (n.kind === "skill-spend-plan") {
      requireKind(n, "bank", ["stock"]);
      requireKind(n, "body", actionKinds);
      if (n.values.spend === "amount") {
        const count = required(n, "count");
        if (count?.kind === "skill-count-constant" && count.values.value === 0)
          issue(
            n,
            "action-zero-cost",
            "지정 개수 단계 비용은 양수여야 합니다.",
            "count",
          );
      } else if (source(n, "count"))
        issue(
          n,
          "action-all-cost",
          "전량 소비는 별도 예상 개수를 받지 않습니다.",
          "count",
        );
    }
    if (n.kind === "skill-stance-plan") {
      positive(n, "duration");
      positiveField(n, "percent");
    }
    if (n.kind === "skill-wait-plan") positive(n, "duration");
    if (n.kind === "skill-count-duration") {
      positive(n, "base");
      positive(n, "perCount");
    }
    if (n.kind === "skill-timed-plan") {
      const body = requireKind(n, "body", ["strike-plan"]);
      positive(n, "duration");
      positive(n, "interval");
      if (body && body.values.target !== "selected-area")
        issue(
          n,
          "action-timed-target",
          "유지형 타격은 접수 타일 범위의 현재 적을 조회해야 합니다.",
          "body",
        );
    }
    if (n.kind === "skill-action-entry") {
      const root = requireKind(n, "plan", actionKinds),
        body = plans(root);
      const target = body.filter((p) => p.kind === "skill-strike-plan");
      const primary = target.some((p) => p.values.target === "primary");
      const area =
        target.some((p) => p.values.target === "selected-area") ||
        body.some((p) => p.kind === "skill-place-fusion-plan");
      if (primary && area)
        issue(
          n,
          "action-target-context",
          "고정 주 대상 행동과 지정 타일 행동을 같은 시작 설정에 섞을 수 없습니다.",
        );
      if (n.values.start === "next-basic" && area)
        issue(
          n,
          "action-basic-context",
          "다음 기본 공격 교체는 그 공격의 주 대상 문맥을 사용해야 합니다.",
        );
      if (n.values.start === "immediate" && primary) required(n, "selector");
      if (source(n, "selector") && (n.values.start === "next-basic" || area))
        issue(
          n,
          "action-selector-context",
          "이 행동은 기본 공격 대상 또는 지정 타일을 사용하므로 별도 자동 선택을 받지 않습니다.",
          "selector",
        );
      requireKind(n, "selector", ["enemy-selector"]);
      walkEntry(n);
    }
    if (n.kind === "skill-mode-program") {
      requireKind(n, "mode", ["battle-mode"]);
      requireKind(n, "normal", ["action-entry"]);
      requireKind(n, "active", ["action-entry"]);
      if (
        source(n, "normal") &&
        source(n, "normal")?.id === source(n, "active")?.id
      )
        issue(
          n,
          "action-mode-branches",
          "서로 다른 상태의 행동 시작 설정을 연결하세요.",
        );
    }
    if (n.kind === "skill-run-action") {
      const { use, program } = admittedProgram(n, [
        "action-entry",
        "mode-program",
      ]);
      const request = use && requestFor(use),
        selection = request && source(request, "selection");
      for (const entry of entries(program)) {
        const target = plans(source(entry, "plan")).filter(
          (p) => p.kind === "skill-strike-plan",
        );
        const selectedAlly = plans(source(entry, "plan")).some(
          (p) =>
            p.kind === "skill-reactive-heal-plan" ||
            (p.kind === "skill-recipient-buff-plan" &&
              p.values.recipient === "selected"),
        );
        const area =
          target.some((p) => p.values.target === "selected-area") ||
          plans(source(entry, "plan")).some(
            (p) => p.kind === "skill-place-fusion-plan",
          );
        const direction = target.some(
          (p) => p.values.target === "selected-direction-area",
        );
        if (
          selection &&
          (selection.kind !== "skill-target-selection" ||
            selection.values.mode !==
              (direction
                ? "player-direction"
                : area
                  ? "player-tile"
                  : selectedAlly
                    ? "player-ally"
                    : "self"))
        )
          issue(
            n,
            "action-request-selection",
            "행동의 주 대상/지정 타일 문맥과 사용 요청의 선택 방식이 다릅니다.",
            "program",
          );
      }
    }
    if (n.kind === "skill-action-periodic-self-damage") {
      const run = requireKind(n, "run", ["run-action"]),
        entry = requireKind(n, "entry", ["action-entry"]);
      positive(n, "interval");
      positiveField(n, "percent");
      if (
        run &&
        entry &&
        (!entries(source(run, "program")).some((p) => p.id === entry.id) ||
          source(entry, "plan")?.kind !== "skill-timed-plan")
      )
        issue(
          n,
          "action-self-period-scope",
          "같은 실행에서 선택할 수 있는 유지형 행동 설정만 자기 피해에 연결하세요.",
          "entry",
        );
      if (
        run &&
        entry &&
        ofKind("action-periodic-self-damage").filter(
          (p) =>
            source(p, "run")?.id === run.id &&
            source(p, "entry")?.id === entry.id,
        ).length > 1
      )
        issue(
          n,
          "action-self-period-duplicate",
          "같은 행동 설정의 주기 자기 피해 구독이 중복됩니다.",
        );
    }
    if (n.kind === "skill-action-stop-input") {
      const run = requireKind(n, "run", ["run-action"]);
      if (
        run &&
        !entries(source(run, "program")).some((e) =>
          ["skill-timed-plan", "skill-aura-buff-plan"].includes(
            source(e, "plan")?.kind ?? "",
          ),
        )
      )
        issue(
          n,
          "action-stop-scope",
          "유지형 행동을 포함하는 실행에만 직접 중단 입력을 연결하세요.",
          "run",
        );
    }
    if (n.kind === "skill-modifier-projectile-status") {
      const status = source(n, "status");
      if (status) statusCounts(status, undefined, false);
    }
    if (n.kind === "skill-projectile-profile") {
      positive(n, "lifetime");
      positiveField(n, "speed");
      positiveField(n, "range");
      positiveField(n, "width");
      if (n.values.delivery === "impact") required(n, "area");
      if (n.values.delivery === "directional") {
        if (n.values.tracking !== "fixed")
          issue(
            n,
            "projectile-direction-tracking",
            "방향 관통 탄환은 계획에서 고정한 방향을 유지해야 합니다.",
            "tracking",
          );
        if (source(n, "area"))
          issue(
            n,
            "projectile-direction-area",
            "방향 관통의 직접 충돌과 착탄 범위 피해를 섞지 마세요.",
            "area",
          );
      }
      sources(n, "after").forEach((s) => statusCounts(s, undefined, false));
    }
    if (n.kind === "skill-enemy-selector") {
      if (n.values.order === "unmarked-route") required(n, "mark");
      else if (source(n, "mark"))
        issue(
          n,
          "selector-unused-mark",
          "이 우선순위는 회수 상태를 사용하지 않습니다.",
          "mark",
        );
      requireKind(n, "mark", ["damage-mark"]);
    }
    if (n.kind === "skill-projectile-results")
      requireKind(n, "batch", ["plan-commit", "serial-projectiles"]);
    if (n.kind === "skill-resonance-qualify" && source(n, "program")) {
      requireKind(n, "program", [
        "action-entry",
        "mode-program",
        "portal-program",
      ]);
      if (source(n, "plan"))
        issue(
          n,
          "program-plan-exclusive",
          "행동 시작 설정과 별도 정지 사격 계획을 한 접수에 섞지 마세요.",
        );
    }
  }
  return out;
}
