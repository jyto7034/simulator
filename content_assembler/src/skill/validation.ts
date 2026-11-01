import {
  blockDefinitions,
  eventNames,
  isContainer,
  scopeNames,
  type SkillDocument,
  type Selection,
  type SkillBlock,
} from "./model";
function record(
  value: unknown,
  keys: string[],
  path: string,
): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value))
    throw new Error(`${path}: 객체가 필요합니다.`);
  const object = value as Record<string, unknown>;
  if (
    Object.keys(object).some((k) => !keys.includes(k)) ||
    keys.some((k) => !(k in object))
  )
    throw new Error(
      `${path}: 필드가 누락되었거나 지원하지 않는 필드가 있습니다.`,
    );
  return object;
}
function text(value: unknown, path: string): asserts value is string {
  if (typeof value !== "string")
    throw new Error(`${path}: 문자열이 필요합니다.`);
}
function list(value: unknown, path: string): asserts value is unknown[] {
  if (!Array.isArray(value)) throw new Error(`${path}: 목록이 필요합니다.`);
}
export function validateSkill(input: unknown): asserts input is SkillDocument {
  const d = record(
    input,
    ["format", "version", "id", "name", "description", "states", "routines"],
    "문서",
  );
  if (d.format !== "skill-design" || d.version !== 1)
    throw new Error("지원하는 스킬 설계 문서(version 1)가 아닙니다.");
  const ids = new Set<string>();
  const identity = (value: unknown) => {
    text(value, "ID");
    if (!value || ids.has(value))
      throw new Error("ID가 비어 있거나 중복됩니다.");
    ids.add(value);
  };
  identity(d.id);
  text(d.name, "이름");
  text(d.description, "설명");
  list(d.states, "상태");
  list(d.routines, "동작");
  for (const value of d.states) {
    const s = record(
      value,
      ["id", "name", "scope", "lifetime", "initial"],
      "상태",
    );
    identity(s.id);
    text(s.name, "상태 이름");
    text(s.lifetime, "상태 수명");
    text(s.initial, "초기값");
    if (typeof s.scope !== "string" || !Object.hasOwn(scopeNames, s.scope))
      throw new Error("지원하지 않는 상태 대상입니다.");
  }
  let count = 0;
  const blocks = (value: unknown, depth: number) => {
    list(value, "블록");
    if (depth > 16) throw new Error("블록 중첩은 16단계까지 지원합니다.");
    for (const item of value) {
      if (++count > 1000)
        throw new Error("한 문서는 블록 1,000개까지 지원합니다.");
      const b = record(
        item,
        ["id", "kind", "fields", "note", "children", "otherwise"],
        "블록",
      );
      identity(b.id);
      text(b.note, "블록 메모");
      if (
        typeof b.kind !== "string" ||
        !Object.hasOwn(blockDefinitions, b.kind)
      )
        throw new Error("지원하지 않는 블록 종류입니다.");
      const kind = b.kind as SkillBlock["kind"];
      const fields = record(
        b.fields,
        Object.keys(blockDefinitions[kind].fields),
        "블록 속성",
      );
      for (const value of Object.values(fields)) text(value, "블록 속성");
      list(b.children, "안쪽 블록");
      list(b.otherwise, "아니면 블록");
      if (
        (!isContainer(kind) && b.children.length) ||
        (kind !== "branch" && b.otherwise.length)
      )
        throw new Error("이 블록에는 하위 절차를 넣을 수 없습니다.");
      blocks(b.children, depth + 1);
      blocks(b.otherwise, depth + 1);
    }
  };
  for (const value of d.routines) {
    const r = record(
      value,
      ["id", "name", "event", "condition", "blocks"],
      "동작",
    );
    identity(r.id);
    text(r.name, "동작 이름");
    text(r.condition, "시작 조건");
    if (typeof r.event !== "string" || !Object.hasOwn(eventNames, r.event))
      throw new Error("지원하지 않는 시작 사건입니다.");
    blocks(r.blocks, 0);
  }
}
export function parseSkill(content: string): SkillDocument {
  const doc: unknown = JSON.parse(content);
  validateSkill(doc);
  return doc;
}
export type SkillDiagnostic = {
  code: "required" | "missing-reference" | "empty-procedure";
  selection: Selection;
  field: string;
  message: string;
};
export function diagnoseSkill(doc: SkillDocument): SkillDiagnostic[] {
  const out: SkillDiagnostic[] = [];
  const add = (
    selection: Selection,
    field: string,
    message: string,
    code: SkillDiagnostic["code"] = "required",
  ) => out.push({ selection, field, message, code });
  if (!doc.name.trim())
    add({ type: "document" }, "name", "스킬 이름을 입력하세요.");
  if (!doc.routines.length)
    add({ type: "document" }, "routines", "시작 사건과 동작을 추가하세요.");
  for (const s of doc.states) {
    const selection: Selection = { type: "state", id: s.id };
    if (!s.name.trim()) add(selection, "name", "상태 이름이 비어 있습니다.");
    if (s.scope === "undecided")
      add(selection, "scope", `${s.name}: 상태가 붙을 대상을 정하세요.`);
    if (!s.lifetime.trim())
      add(selection, "lifetime", `${s.name}: 수명·종료 규칙이 미정입니다.`);
    if (!s.initial.trim())
      add(selection, "initial", `${s.name}: 초기값이 미정입니다.`);
  }
  for (const r of doc.routines) {
    if (!r.name.trim() || !r.blocks.length)
      add(
        { type: "routine", id: r.id },
        "name",
        "동작 이름과 실행 블록을 작성하세요.",
      );
    const visit = (blocks: SkillBlock[]) => {
      for (const b of blocks) {
        const selection: Selection = {
          type: "block",
          id: b.id,
          routineId: r.id,
        };
        for (const [key, label] of Object.entries(
          blockDefinitions[b.kind].fields,
        )) {
          if (!b.fields[key].trim())
            add(
              selection,
              key,
              `${blockDefinitions[b.kind].label}: ${label} 미지정`,
            );
          else if (
            ["state", "destination"].includes(key) &&
            !doc.states.some((s) => s.id === b.fields[key])
          )
            add(
              selection,
              key,
              `${label}: 삭제되었거나 없는 상태를 참조합니다.`,
              "missing-reference",
            );
        }
        if (isContainer(b.kind) && !b.children.length)
          add(
            selection,
            "children",
            "안쪽 절차가 비어 있습니다.",
            "empty-procedure",
          );
        visit(b.children);
        visit(b.otherwise);
      }
    };
    visit(r.blocks);
  }
  return out;
}
