/** Editable design document, deliberately separate from executable Core RON. */
export const blockDefinitions = {
  query: {
    label: "대상 조회",
    group: "조회·계산",
    fields: {
      target: "조회 대상",
      condition: "선정 조건",
      result: "결과 이름",
    },
  },
  aggregate: {
    label: "값 집계·계산",
    group: "조회·계산",
    fields: { expression: "계산식", result: "결과 이름" },
  },
  changeState: {
    label: "상태 값 변경",
    group: "상태",
    fields: { state: "상태 참조", target: "적용 대상", amount: "변경량" },
  },
  transferState: {
    label: "상태 이전",
    group: "상태",
    fields: {
      state: "이전할 상태",
      destination: "받을 상태",
      target: "이전 대상",
    },
  },
  wait: {
    label: "기다리기",
    group: "흐름",
    fields: { duration: "대기 시간 (초 또는 식)" },
  },
  branch: {
    label: "조건 분기",
    group: "흐름",
    fields: { condition: "분기 조건" },
    container: true,
  },
  repeat: {
    label: "유한 반복",
    group: "흐름",
    fields: {
      count: "횟수 또는 유한 목록",
      interval: "반복 간격 (초 또는 식)",
    },
    container: true,
  },
  parallel: {
    label: "동시에 실행",
    group: "흐름",
    fields: { completion: "합류 기준" },
    container: true,
  },
  input: {
    label: "플레이어 입력",
    group: "입력",
    fields: { request: "선택할 내용", result: "결과 이름" },
  },
  commit: {
    label: "소비·실행 확정",
    group: "입력",
    fields: { cost: "소비할 자원", operation: "함께 확정할 동작" },
  },
  projectile: {
    label: "탄환 발사",
    group: "효과",
    fields: {
      origin: "발사 위치",
      direction: "방향",
      delivery: "충돌·관통 규칙",
    },
  },
  damage: {
    label: "피해 적용",
    group: "효과",
    fields: {
      target: "피해 대상",
      amount: "피해량 또는 식",
      damageType: "피해 유형",
    },
  },
  control: {
    label: "제어 효과",
    group: "효과",
    fields: {
      target: "적용 대상",
      control: "제어 종류",
      duration: "지속 시간 (초 또는 식)",
    },
  },
} as const;
export type BlockKind = keyof typeof blockDefinitions;
export type SkillBlock = {
  id: string;
  kind: BlockKind;
  fields: Record<string, string>;
  note: string;
  children: SkillBlock[];
  otherwise: SkillBlock[];
};
export const eventNames = {
  use: "스킬을 사용했을 때",
  damage: "피해가 발생했을 때",
  death: "유닛이 사망했을 때",
  timer: "전투 시간 주기마다",
  impact: "탄환이 적중했을 때",
} as const;
export type SkillRoutine = {
  id: string;
  name: string;
  event: keyof typeof eventNames;
  condition: string;
  blocks: SkillBlock[];
};
export const scopeNames = {
  undecided: "미정",
  unit: "유닛",
  tile: "타일",
  source: "제공 사본",
  cast: "이번 시전",
} as const;
export type SkillState = {
  id: string;
  name: string;
  scope: keyof typeof scopeNames;
  lifetime: string;
  initial: string;
};
export type SkillDocument = {
  format: "skill-design";
  version: 1;
  id: string;
  name: string;
  description: string;
  states: SkillState[];
  routines: SkillRoutine[];
};
export type Selection =
  | { type: "document" }
  | { type: "state"; id: string }
  | { type: "routine"; id: string }
  | { type: "block"; id: string; routineId: string }
  | {
      type: "slot";
      parentId: string;
      routineId: string;
      lane: "children" | "otherwise";
    };
export const id = () => crypto.randomUUID();
export function makeBlock(
  kind: BlockKind,
  fields: Record<string, string> = {},
  children: SkillBlock[] = [],
  otherwise: SkillBlock[] = [],
): SkillBlock {
  return {
    id: id(),
    kind,
    fields: {
      ...Object.fromEntries(
        Object.keys(blockDefinitions[kind].fields).map((k) => [k, ""]),
      ),
      ...fields,
    },
    note: "",
    children,
    otherwise,
  };
}
export function makeRoutine(): SkillRoutine {
  return { id: id(), name: "새 동작", event: "use", condition: "", blocks: [] };
}
export function createSkill(): SkillDocument {
  return {
    format: "skill-design",
    version: 1,
    id: id(),
    name: "새 스킬",
    description: "",
    states: [],
    routines: [makeRoutine()],
  };
}
export function isContainer(kind: BlockKind) {
  return "container" in blockDefinitions[kind];
}
export function findBlock(
  blocks: SkillBlock[],
  target: string,
): { block: SkillBlock; siblings: SkillBlock[]; index: number } | undefined {
  for (let index = 0; index < blocks.length; index++) {
    const block = blocks[index];
    if (block.id === target) return { block, siblings: blocks, index };
    const child =
      findBlock(block.children, target) ?? findBlock(block.otherwise, target);
    if (child) return child;
  }
}
export function selectedRoutine(doc: SkillDocument, selection: Selection) {
  const key =
    selection.type === "routine"
      ? selection.id
      : "routineId" in selection
        ? selection.routineId
        : undefined;
  return doc.routines.find((r) => r.id === key);
}
export function validSelection(
  doc: SkillDocument,
  selection: Selection,
): Selection {
  if (selection.type === "document") return selection;
  if (selection.type === "state")
    return doc.states.some((s) => s.id === selection.id)
      ? selection
      : { type: "document" };
  const routine = selectedRoutine(doc, selection);
  if (!routine) return { type: "document" };
  if (selection.type === "routine") return selection;
  const found = findBlock(
    routine.blocks,
    selection.type === "slot" ? selection.parentId : selection.id,
  );
  if (!found) return { type: "routine", id: routine.id };
  return selection;
}
export function insertBlock(
  doc: SkillDocument,
  selection: Selection,
  block: SkillBlock,
): Selection {
  const routine = selectedRoutine(doc, selection);
  if (!routine) throw new Error("먼저 동작을 선택하세요.");
  if (selection.type === "slot") {
    const parent = findBlock(routine.blocks, selection.parentId)?.block;
    if (
      !parent ||
      !isContainer(parent.kind) ||
      (selection.lane === "otherwise" && parent.kind !== "branch")
    )
      throw new Error("추가할 위치가 없습니다.");
    parent[selection.lane].push(block);
  } else if (selection.type === "block") {
    const found = findBlock(routine.blocks, selection.id);
    if (!found) throw new Error("선택한 블록이 없습니다.");
    found.siblings.splice(found.index + 1, 0, block);
  } else routine.blocks.push(block);
  return { type: "block", id: block.id, routineId: routine.id };
}
export function duplicateBlock(block: SkillBlock): SkillBlock {
  return {
    ...structuredClone(block),
    id: id(),
    children: block.children.map(duplicateBlock),
    otherwise: block.otherwise.map(duplicateBlock),
  };
}
export type BlockEditAction = "move-up" | "move-down" | "duplicate";
export function editBlockSelection(
  doc: SkillDocument,
  selection: Selection,
  action: BlockEditAction,
): Selection | undefined {
  const routine = selectedRoutine(doc, selection);
  if (!routine || selection.type !== "block") return;
  const found = findBlock(routine.blocks, selection.id);
  if (!found) return;
  if (action === "duplicate") {
    const clone = duplicateBlock(found.block);
    found.siblings.splice(found.index + 1, 0, clone);
    return { type: "block", id: clone.id, routineId: routine.id };
  }
  const next = found.index + (action === "move-up" ? -1 : 1);
  if (next < 0 || next >= found.siblings.length) return;
  found.siblings.splice(found.index, 1);
  found.siblings.splice(next, 0, found.block);
}
export function removeSelection(doc: SkillDocument, selection: Selection) {
  if (selection.type === "state")
    doc.states = doc.states.filter((s) => s.id !== selection.id);
  if (selection.type === "routine")
    doc.routines = doc.routines.filter((r) => r.id !== selection.id);
  if (selection.type === "block") {
    const routine = selectedRoutine(doc, selection);
    const found = routine && findBlock(routine.blocks, selection.id);
    if (found) found.siblings.splice(found.index, 1);
  }
}
