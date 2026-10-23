import {
  createSkill,
  validSelection,
  type Selection,
  type SkillDocument,
} from "./model";
import { validateSkill } from "./validation";
function freeze<T>(value: T): T {
  if (value && typeof value === "object" && !Object.isFrozen(value)) {
    Object.values(value).forEach(freeze);
    Object.freeze(value);
  }
  return value;
}
export type SkillEditorState = Readonly<{
  doc: SkillDocument;
  past: readonly SkillDocument[];
  future: readonly SkillDocument[];
  selection: Selection;
  saved: string;
  serialized: string;
  epoch: number;
  revision: number;
  message: string;
}>;
export class SkillSession {
  private state: SkillEditorState;
  private listeners = new Set<() => void>();
  private editing = false;
  constructor(doc = createSkill()) {
    validateSkill(doc);
    this.state = freeze({
      doc: structuredClone(doc),
      past: [],
      future: [],
      selection: { type: "document" },
      saved: "",
      serialized: JSON.stringify(doc),
      epoch: 0,
      revision: 0,
      message: "설계 초안 · 미정인 항목도 저장할 수 있습니다.",
    });
  }
  getSnapshot = () => this.state;
  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };
  private publish(patch: Partial<SkillEditorState>) {
    if (this.editing)
      throw new Error("변경 준비 중에는 다른 편집을 확정할 수 없습니다.");
    this.state = freeze({ ...this.state, ...patch });
    for (const listener of [...this.listeners]) {
      try {
        listener();
      } catch (e) {
        console.error("확정된 스킬 편집의 화면 알림 실패", e);
      }
    }
  }
  message = (message: string) => this.publish({ message });
  select = (selection: Selection) =>
    this.publish({ selection: validSelection(this.state.doc, selection) });
  commit(edit: (doc: SkillDocument) => Selection | void) {
    if (this.editing) throw new Error("중첩 편집은 지원하지 않습니다.");
    const before = this.state;
    let prepared: Partial<SkillEditorState> | undefined;
    let failure: unknown;
    this.editing = true;
    try {
      const draft = structuredClone(before.doc);
      const selection = edit(draft);
      validateSkill(draft);
      const serialized = JSON.stringify(draft);
      if (serialized !== before.serialized)
        prepared = {
          doc: structuredClone(draft),
          serialized,
          past: [...before.past.slice(-99), before.doc],
          future: [],
          selection: validSelection(draft, selection ?? before.selection),
          revision: before.revision + 1,
        };
    } catch (e) {
      failure = e;
    } finally {
      this.editing = false;
    }
    if (failure) {
      this.message(`변경하지 않았습니다: ${String(failure)}`);
      return false;
    }
    if (prepared) this.publish(prepared);
    return !!prepared;
  }
  travel(direction: "undo" | "redo") {
    const s = this.state,
      source = direction === "undo" ? s.past : s.future,
      doc = source.at(-1);
    if (!doc) return;
    this.publish({
      doc,
      serialized: JSON.stringify(doc),
      past: direction === "undo" ? source.slice(0, -1) : [...s.past, s.doc],
      future: direction === "redo" ? source.slice(0, -1) : [...s.future, s.doc],
      selection: validSelection(doc, s.selection),
      revision: s.revision + 1,
    });
  }
  replace(doc: SkillDocument, opened: boolean) {
    validateSkill(doc);
    this.publish({
      doc: structuredClone(doc),
      serialized: JSON.stringify(doc),
      saved: opened ? JSON.stringify(doc) : "",
      past: [],
      future: [],
      selection: { type: "document" },
      epoch: this.state.epoch + 1,
      revision: this.state.revision + 1,
    });
  }
  acknowledge(doc: SkillDocument, epoch: number) {
    if (this.state.epoch === epoch)
      this.publish({ saved: JSON.stringify(doc) });
  }
  isDirty = () => this.state.serialized !== this.state.saved;
}
