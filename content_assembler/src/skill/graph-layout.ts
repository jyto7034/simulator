import type { ELK, ElkNode } from "elkjs/lib/elk-api";
import { graphSize, type SkillGraphModel } from "./graph-model";

export function layoutRequest(model: SkillGraphModel): ElkNode[] {
  return model.routines.map((id) => {
    const children = model.items
      .filter((n) => n.routineId === id)
      .map((n) => ({
        id: n.id,
        width: n.kind === "join" ? graphSize.joinWidth : graphSize.width,
        height: n.kind === "join" ? graphSize.joinHeight : graphSize.height,
      }));
    const ids = new Set(children.map((n) => n.id));
    return {
      id,
      children,
      edges: model.links
        .filter((e) => ids.has(e.source))
        .map((e) => ({ id: e.id, sources: [e.source], targets: [e.target] })),
      layoutOptions: {
        "elk.algorithm": "layered",
        "elk.direction": "RIGHT",
        "elk.spacing.nodeNode": "44",
        "elk.layered.spacing.nodeNodeBetweenLayers": "48",
        "elk.layered.considerModelOrder.strategy": "NODES_AND_EDGES",
        "elk.padding": "[top=0,left=0,bottom=0,right=0]",
      },
    };
  });
}
export async function arrangeSkillGraphs(
  elk: Pick<ELK, "layout">,
  requests: ElkNode[],
) {
  const positions: Record<string, { x: number; y: number }> = {};
  let top = 0;
  for (const request of requests) {
    const result = await elk.layout(request);
    for (const node of result.children ?? []) {
      if (!Number.isFinite(node.x) || !Number.isFinite(node.y))
        throw new Error("노드 위치를 계산하지 못했습니다.");
      positions[node.id] = { x: node.x!, y: node.y! + top };
    }
    top += (result.height ?? graphSize.height) + 28;
  }
  return positions;
}
