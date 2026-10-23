import type { ELK, ElkNode } from "elkjs/lib/elk-api";
import type { GraphDocument, Point } from "../types";
import { definition } from "./catalog";

// Layout changes positions only. Frames are authoring sections, never execution scopes.
export async function arrangeGraph(
  doc: GraphDocument,
  elk: Pick<ELK, "layout">,
): Promise<{ id: string; position: Point }[]> {
  const positions: { id: string; position: Point }[] = [];
  let top = 80;
  const frameTop = (id: string) =>
    Math.min(
      ...doc.nodes.filter((n) => n.frame === id).map((n) => n.position.y),
    );
  const frames = [...doc.frames].sort(
    (a, b) => frameTop(a.id) - frameTop(b.id),
  );
  for (const frame of [null, ...frames.map((f) => f.id)]) {
    const members = doc.nodes.filter((n) => n.frame === frame);
    if (!members.length) continue;
    const ids = new Set(members.map((n) => n.id));
    const request: ElkNode = {
      id: frame ?? "unframed",
      children: members.map((n) => {
        const d = definition(n.kind);
        return {
          id: n.id,
          width: d.width,
          height: d.height,
          layoutOptions: { "elk.portConstraints": "FIXED_ORDER" },
          ports: [
            ...d.inputs.map((p) => ({
              id: `${n.id}:in:${p.id}`,
              layoutOptions: { "elk.port.side": "WEST" },
            })),
            ...d.outputs.map((p) => ({
              id: `${n.id}:out:${p.id}`,
              layoutOptions: { "elk.port.side": "EAST" },
            })),
          ],
        };
      }),
      edges: doc.edges
        .filter((e) => ids.has(e.source.node) && ids.has(e.target.node))
        .map((e) => ({
          id: e.id,
          sources: [`${e.source.node}:out:${e.source.port}`],
          targets: [`${e.target.node}:in:${e.target.port}`],
        })),
      layoutOptions: {
        "elk.algorithm": "layered",
        "elk.direction": "RIGHT",
        "elk.spacing.nodeNode": "56",
        "elk.layered.spacing.nodeNodeBetweenLayers": "100",
        "elk.layered.considerModelOrder.strategy": "NODES_AND_EDGES",
        "elk.padding": "[top=0,left=0,bottom=0,right=0]",
      },
    };
    const result = await elk.layout(request);
    if (
      !Number.isFinite(result.height) ||
      result.children?.length !== members.length
    )
      throw new Error("그래프 배치를 계산하지 못했습니다.");
    for (const n of result.children) {
      if (!Number.isFinite(n.x) || !Number.isFinite(n.y))
        throw new Error("노드 위치를 계산하지 못했습니다.");
      positions.push({ id: n.id, position: { x: n.x! + 40, y: n.y! + top } });
    }
    top += result.height! + 160;
  }
  return positions;
}
