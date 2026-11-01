import {
  blockDefinitions,
  eventNames,
  isContainer,
  type Selection,
  type SkillBlock,
  type SkillDocument,
} from "./model";
import type { SkillDiagnostic } from "./validation";

export type GraphItem = {
  id: string;
  routineId: string;
  title: string;
  summary: string;
  category: string;
  selection: Selection;
  kind: "event" | "block" | "join";
  container: boolean;
  branch: boolean;
  expanded: boolean;
  issueCount: number;
  childCount: number;
  canMoveUp: boolean;
  canMoveDown: boolean;
};
export type GraphLink = {
  id: string;
  source: string;
  target: string;
  label: string;
};
export type SkillGraphModel = {
  items: GraphItem[];
  links: GraphLink[];
  routines: string[];
};
export const graphSize = {
  width: 176,
  height: 88,
  joinWidth: 76,
  joinHeight: 36,
};

export function blockAncestors(doc: SkillDocument, target: string): string[] {
  const visit = (
    blocks: SkillBlock[],
    path: string[],
  ): string[] | undefined => {
    for (const b of blocks) {
      if (b.id === target) return path;
      const found = visit([...b.children, ...b.otherwise], [...path, b.id]);
      if (found) return found;
    }
  };
  for (const r of doc.routines) {
    const found = visit(r.blocks, []);
    if (found) return found;
  }
  return [];
}
export function descendantIds(block: SkillBlock): string[] {
  return [
    block.id,
    ...block.children.flatMap(descendantIds),
    ...block.otherwise.flatMap(descendantIds),
  ];
}
function summary(block: SkillBlock, doc: SkillDocument): string {
  const f = block.fields;
  const state = (key: string) =>
    doc.states.find((s) => s.id === f[key])?.name || "상태 미지정";
  switch (block.kind) {
    case "changeState":
      return `${state("state")} · ${f.amount || "변경량 미정"}`;
    case "transferState":
      return `${state("state")} → ${state("destination")}`;
    case "query":
      return f.target || "대상 미지정";
    case "aggregate":
      return f.result || f.expression || "계산식 미지정";
    case "wait":
      return f.duration ? `${f.duration}초` : "시간 미지정";
    case "repeat":
      return f.count || "반복 횟수·목록 미지정";
    case "branch":
      return f.condition || "분기 조건 미지정";
    case "parallel":
      return f.completion || "합류 기준 미지정";
    case "input":
      return f.request || "입력 내용 미지정";
    case "commit":
      return f.operation || "확정할 동작 미지정";
    case "damage":
      return `${f.target || "대상 미정"} · ${f.amount || "피해량 미정"}`;
    case "control":
      return `${f.control || "효과 미정"} · ${f.duration || "시간 미정"}`;
    case "projectile":
      return `${f.origin || "위치 미정"} → ${f.direction || "방향 미정"}`;
  }
}

/** Project the authored sequence into a view. Synthetic joins never enter the saved document. */
export function projectSkillGraph(
  doc: SkillDocument,
  expanded: ReadonlySet<string>,
  diagnostics: SkillDiagnostic[],
): SkillGraphModel {
  const items: GraphItem[] = [],
    links: GraphLink[] = [];
  const counts = new Map<string, number>();
  for (const d of diagnostics)
    if ("id" in d.selection)
      counts.set(d.selection.id, (counts.get(d.selection.id) ?? 0) + 1);
  const connect = (source: string, target: string, label = "") => {
    links.push({
      id: `edge:${links.length}:${source}:${target}`,
      source,
      target,
      label,
    });
  };
  for (const routine of doc.routines) {
    const routineId = routine.id;
    items.push({
      id: routineId,
      routineId,
      title: eventNames[routine.event],
      summary: routine.name,
      category: "시작 사건",
      selection: { type: "routine", id: routineId },
      kind: "event",
      container: false,
      branch: false,
      expanded: false,
      issueCount: counts.get(routineId) ?? 0,
      childCount: 0,
      canMoveUp: false,
      canMoveDown: false,
    });
    const sequence = (
      blocks: SkillBlock[],
      source: string,
      label = "",
      indexOffset = 0,
      siblingCount = blocks.length,
    ): string => {
      let tail = source;
      blocks.forEach((b, index) => {
        const container = isContainer(b.kind),
          open = container && expanded.has(b.id);
        const owned = descendantIds(b);
        items.push({
          id: b.id,
          routineId,
          title: `${index + indexOffset + 1}. ${blockDefinitions[b.kind].label}`,
          summary: summary(b, doc),
          category: blockDefinitions[b.kind].group,
          selection: { type: "block", id: b.id, routineId },
          kind: "block",
          container,
          branch: b.kind === "branch",
          expanded: open,
          issueCount: (open ? [b.id] : owned).reduce(
            (n, id) => n + (counts.get(id) ?? 0),
            0,
          ),
          childCount: owned.length - 1,
          canMoveUp: index + indexOffset > 0,
          canMoveDown: index + indexOffset < siblingCount - 1,
        });
        connect(tail, b.id, index === 0 ? label : "");
        tail = b.id;
        if (open) {
          const join = `join:${b.id}`;
          items.push({
            id: join,
            routineId,
            title: b.kind === "repeat" ? "반복 종료" : "합류",
            summary: "",
            category: "흐름",
            selection: { type: "block", id: b.id, routineId },
            kind: "join",
            container: false,
            branch: false,
            expanded: false,
            issueCount: 0,
            childCount: 0,
            canMoveUp: false,
            canMoveDown: false,
          });
          const lane = (
            children: SkillBlock[],
            label: string,
            offset = 0,
            count = children.length,
          ) => {
            const end = sequence(children, b.id, label, offset, count);
            connect(end, join, children.length ? "" : label);
          };
          if (b.kind === "branch") {
            lane(b.children, "참");
            lane(b.otherwise, "아니면");
          } else if (b.kind === "parallel" && b.children.length) {
            b.children.forEach((child, i) =>
              lane([child], `동시 ${i + 1}`, i, b.children.length),
            );
          } else lane(b.children, b.kind === "repeat" ? "반복 본문" : "안쪽");
          tail = join;
        }
      });
      return tail;
    };
    sequence(routine.blocks, routineId);
  }
  return { items, links, routines: doc.routines.map((r) => r.id) };
}
