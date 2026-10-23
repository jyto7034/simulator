import type { Diagnostic, GraphDocument, GraphNode } from "../types";
import { inputSources } from "./connections";
import { flowPaths } from "./flow-paths";

export function spatialDiagnostics(doc: GraphDocument): Diagnostic[] {
  const out: Diagnostic[] = [];
  const source = inputSources(doc);
  const { ancestors, mustPass } = flowPaths(doc);
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
  const selectedTile = (request: GraphNode | undefined) => {
    const selection = request && source(request, "selection");
    return (
      selection?.kind === "skill-target-selection" &&
      selection.values.mode === "player-tile"
    );
  };
  for (const n of doc.nodes) {
    if (
      n.kind === "skill-target-selection" &&
      ["player-tile", "player-ally"].includes(String(n.values.mode)) &&
      !source(n, "range")
    )
      issue(
        n,
        "spatial-selection-range",
        "지정 대상의 선택 범위를 연결하세요.",
        "range",
      );
    if (n.kind === "skill-ground-area") {
      const area = source(n, "area");
      if (
        area &&
        (area.kind !== "skill-range-placement" ||
          area.values.origin !== "selected-tile")
      )
        issue(
          n,
          "spatial-ground-origin",
          "지정 타일 장판에는 접수한 지정 타일 기준의 배치 범위를 연결하세요.",
          "area",
        );
      const use = source(n, "use");
      const permit = use && source(use, "permit");
      const request = permit && source(permit, "request");
      if (use && !selectedTile(request))
        issue(
          n,
          "spatial-target-context",
          "이번 사용의 요청에 플레이어 지정 타일 선택 설정이 필요합니다.",
          "use",
        );
      const duration = source(n, "duration");
      if (duration?.kind === "skill-seconds" && duration.values.value === 0)
        issue(
          n,
          "spatial-duration",
          "장판 유지 시간은 0초보다 커야 합니다.",
          "duration",
        );
    }
    if (n.kind === "skill-area-damage") {
      const area = source(n, "area");
      if (
        area?.kind === "skill-range-placement" &&
        area.values.origin === "selected-tile"
      ) {
        const path = ancestors(n);
        const requests = doc.nodes.filter(
          (p) => p.kind === "skill-request" && path.has(p.id),
        );
        if (
          requests.length !== 1 ||
          !selectedTile(requests[0]) ||
          !mustPass(n, requests[0], "out")
        )
          issue(
            n,
            "spatial-target-context",
            "접수한 지정 타일을 보존한 하나의 요청 문맥이 필요합니다.",
            "area",
          );
      }
    }
  }
  return out;
}
