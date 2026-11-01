import type { Diagnostic, GraphDocument, GraphNode } from "../types";
import { inputSources } from "./connections";
import { flowPaths } from "./flow-paths";

const events = new Set([
  "skill-ground-enter",
  "skill-ground-exit",
  "skill-ground-dwell",
  "skill-ground-ramping-slow",
  "skill-ground-periodic",
  "skill-ground-ended",
]);
const targets = new Set([
  "skill-ground-enter",
  "skill-ground-exit",
  "skill-ground-dwell",
  "skill-ground-ramping-slow",
  "skill-ground-targets",
]);
const effects = new Set([
  "skill-ground-pull",
  "skill-ground-damage",
  "skill-ground-heal",
  "skill-ground-apply-status",
  "skill-ground-clear-status",
]);

export function groundDiagnostics(doc: GraphDocument): Diagnostic[] {
  const out: Diagnostic[] = [];
  const source = inputSources(doc);
  const { mustPass } = flowPaths(doc);
  const issue = (
    node: GraphNode,
    code: string,
    message: string,
    portId: string | null = null,
  ) =>
    out.push({
      code,
      nodeId: node.id,
      portId,
      message: `${node.title || node.kind} · ${message}`,
    });
  const requireKind = (node: GraphNode, port: string, kind: string) => {
    const from = source(node, port);
    if (from && from.kind !== kind)
      issue(
        node,
        "ground-source",
        "이 입력의 출처 종류가 올바르지 않습니다.",
        port,
      );
    return from?.kind === kind ? from : undefined;
  };
  const memberArea = (node: GraphNode) => {
    const relation = source(node, "member");
    return relation && source(relation, "area");
  };
  const memberFilter = (node: GraphNode) => {
    const relation = source(node, "member");
    return relation && source(relation, "filter");
  };
  const applications = doc.nodes.filter(
    (n) => n.kind === "skill-ground-apply-status",
  );
  for (const n of doc.nodes) {
    if (events.has(n.kind) || n.kind === "skill-ground-targets") {
      const area = source(n, "area");
      if (
        area &&
        ![
          "skill-ground-area",
          "skill-channel-area",
          "skill-phase-area",
        ].includes(area.kind)
      )
        issue(n, "ground-source", "생성한 장판 참조를 연결하세요.", "area");
    }
    if (
      [
        "skill-ground-enter",
        "skill-ground-exit",
        "skill-ground-dwell",
        "skill-ground-ramping-slow",
        "skill-ground-targets",
      ].includes(n.kind)
    ) {
      const filter = source(n, "filter");
      if (
        filter &&
        !["skill-ground-filter", "skill-corrupted-filter"].includes(filter.kind)
      )
        issue(n, "ground-source", "영역 대상 조건을 연결하세요.", "filter");
    }
    const timePort =
      n.kind === "skill-ground-periodic"
        ? "interval"
        : ["skill-ground-dwell", "skill-ground-ramping-slow"].includes(n.kind)
          ? "duration"
          : null;
    if (timePort) {
      const time = source(n, timePort);
      if (
        n.kind === "skill-ground-ramping-slow" &&
        time &&
        time.kind !== "skill-seconds"
      )
        issue(
          n,
          "ground-duration-source",
          "체류 진행에는 고정 시간 값을 연결하세요.",
          timePort,
        );
      if (time?.kind === "skill-seconds" && time.values.value === 0)
        issue(
          n,
          "ground-positive-time",
          "주기와 체류 시간은 0초보다 커야 합니다.",
          timePort,
        );
    }
    if (n.kind === "skill-ground-targets") {
      const area = source(n, "area");
      const roots = doc.nodes.filter(
        (p) =>
          (p.id === area?.id ||
            (events.has(p.kind) &&
              p.kind !== "skill-ground-ended" &&
              source(p, "area")?.id === area?.id)) &&
          mustPass(n, p, "out"),
      );
      if (area && roots.length !== 1)
        issue(
          n,
          "ground-event-scope",
          "같은 장판의 생성 완료 또는 살아 있는 장판 사건 흐름이 필요합니다.",
          "in",
        );
    }
    if (!effects.has(n.kind)) continue;
    const relation = source(n, "member");
    if (
      relation &&
      (!targets.has(relation.kind) || !mustPass(n, relation, "out"))
    )
      issue(
        n,
        "ground-member-scope",
        "대상 관계를 만든 사건·대상별 흐름에서만 이 관계를 사용할 수 있습니다.",
        "member",
      );
    if (n.kind === "skill-ground-damage") {
      const power = source(n, "power");
      const capture =
        power?.kind === "skill-time-power"
          ? source(power, "seconds")
          : undefined;
      if (
        capture?.kind === "skill-capture-time" &&
        !mustPass(n, capture, "out")
      )
        issue(
          n,
          "ground-power-scope",
          "이 흐름에서 고정하지 않은 피해 위력을 가져올 수 없습니다.",
          "power",
        );
    }
    if (
      !["skill-ground-apply-status", "skill-ground-clear-status"].includes(
        n.kind,
      )
    )
      continue;
    const status = requireKind(n, "status", "skill-ground-status");
    if (
      n.kind === "skill-ground-apply-status" &&
      relation?.kind === "skill-ground-exit"
    )
      issue(
        n,
        "ground-closed-member",
        "이미 이탈한 관계에는 새 상태를 부여할 수 없습니다.",
        "member",
      );
    const area = memberArea(n);
    const filter = memberFilter(n);
    if (!status || !area || !filter) continue;
    const matching = applications.filter(
      (a) =>
        source(a, "status")?.id === status.id && memberArea(a)?.id === area.id,
    );
    if (n.kind === "skill-ground-clear-status" && !matching.length)
      issue(
        n,
        "ground-status-source",
        "이 장판과 상태 정의를 함께 사용하는 적용 동작이 없습니다.",
        "status",
      );
    if (matching.some((a) => memberFilter(a)?.id !== filter.id))
      issue(
        n,
        "ground-status-filter",
        "같은 장판·상태의 적용과 해제는 같은 대상 조건의 관계를 사용해야 합니다.",
        "member",
      );
  }
  return out;
}
