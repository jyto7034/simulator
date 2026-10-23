import { lockedChange, validSelection } from "./operations";
import {
  createDocument,
  layerNames,
  type Layer,
  type MapDocument,
  type Selection,
} from "./model";
import { validateDocument } from "./serialization";
export type EditorState = Readonly<{
  doc: MapDocument;
  past: readonly MapDocument[];
  future: readonly MapDocument[];
  selection: Selection;
  locked: ReadonlySet<Layer>;
  hidden: ReadonlySet<Layer>;
  saved: string;
  serialized: string;
  epoch: number;
  revision: number;
  message: string;
}>;

// Documents contain only validated JSON data. Freeze every nested object/array;
// callers edit a separate draft, never a committed document or history entry.
function freezeDocument<T>(value: T): T {
  if (value !== null && typeof value === "object") {
    for (const child of Object.values(value)) freezeDocument(child);
    Object.freeze(value);
  }
  return value;
}

export class EditorSession {
  private listeners = new Set<() => void>();
  private state: EditorState;
  private editing = false;
  constructor(doc = createDocument()) {
    validateDocument(doc);
    this.state = Object.freeze({
      doc: freezeDocument(structuredClone(doc)),
      serialized: JSON.stringify(doc),
      past: Object.freeze([]),
      future: Object.freeze([]),
      selection: null,
      locked: new Set<Layer>(),
      hidden: new Set<Layer>(),
      saved: "",
      epoch: 0,
      revision: 0,
      message: "새 문서 · JSON 파일로 다운로드하여 보관하세요.",
    });
  }
  getSnapshot = () => this.state;
  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };
  private update(patch: Partial<EditorState>) {
    this.assertNotEditing();
    if (patch.past) Object.freeze(patch.past);
    if (patch.future) Object.freeze(patch.future);
    if (patch.selection)
      patch = { ...patch, selection: Object.freeze({ ...patch.selection }) };
    this.state = Object.freeze({ ...this.state, ...patch });
    // Publication happens after commit. One broken observer must neither turn a
    // successful edit/export into a failure nor prevent other observers updating.
    for (const listener of [...this.listeners]) {
      try {
        listener();
      } catch (error) {
        console.error(
          "편집 상태는 반영되었지만 구독자 알림에 실패했습니다.",
          error,
        );
      }
    }
  }
  private assertNotEditing() {
    if (this.editing)
      throw new Error(
        "변경 준비 중에는 세션을 변경할 수 없습니다. 같은 초안에서 편집하세요.",
      );
  }
  message = (message: string) => this.update({ message });
  select = (selection: Selection) =>
    this.update({ selection: validSelection(this.state.doc, selection) });
  toggleLayer(layer: Layer, flag: "locked" | "hidden") {
    const set = new Set(this.state[flag]);
    if (set.has(layer)) set.delete(layer);
    else set.add(layer);
    this.update({ [flag]: set });
  }
  commit = (edit: (draft: MapDocument) => void): boolean => {
    this.assertNotEditing();
    const { doc, locked, past } = this.state;
    let prepared: { doc: MapDocument; serialized: string } | undefined;
    let failure = "";
    this.editing = true;
    try {
      const next = structuredClone(doc);
      const result: unknown = edit(next);
      if (
        result &&
        typeof (result as PromiseLike<unknown>).then === "function"
      ) {
        // The rejected async callback may still finish on its isolated draft.
        void Promise.resolve(result).catch((error) => {
          console.error("거부된 비동기 편집에서 오류가 발생했습니다.", error);
        });
        throw new Error(
          "편집 콜백은 동기 작업이어야 합니다. 비동기 준비를 마친 뒤 확정하세요.",
        );
      }
      validateDocument(next);
      const serialized = JSON.stringify(next);
      if (this.state.serialized === serialized) return false;
      const blocked = lockedChange(doc, next, locked);
      if (blocked)
        throw new Error(`${layerNames[blocked]} 레이어가 잠겨 있습니다.`);
      // Detach retained drafts and objects supplied by the caller before publish.
      prepared = { doc: freezeDocument(structuredClone(next)), serialized };
    } catch (e) {
      failure = e instanceof Error ? e.message : String(e);
    } finally {
      this.editing = false;
    }
    if (!prepared) {
      this.message(`변경하지 않았습니다: ${failure}`);
      return false;
    }
    this.update({
      ...prepared,
      past: [...past.slice(-99), doc],
      future: [],
      selection: validSelection(prepared.doc, this.state.selection),
      revision: this.state.revision + 1,
    });
    return true;
  };
  travel = (direction: "undo" | "redo") => {
    const s = this.state;
    const source = direction === "undo" ? s.past : s.future;
    const doc = source.at(-1);
    if (!doc) return;
    const blocked = lockedChange(s.doc, doc, s.locked);
    if (blocked) {
      this.message(
        `이력을 적용하려면 ${layerNames[blocked]} 잠금을 해제하세요.`,
      );
      return;
    }
    this.update({
      doc,
      serialized: JSON.stringify(doc),
      past: direction === "undo" ? source.slice(0, -1) : [...s.past, s.doc],
      future: direction === "redo" ? source.slice(0, -1) : [...s.future, s.doc],
      selection: validSelection(doc, s.selection),
      revision: s.revision + 1,
    });
  };
  replace(doc: MapDocument, opened: boolean) {
    validateDocument(doc);
    this.update({
      doc: freezeDocument(structuredClone(doc)),
      serialized: JSON.stringify(doc),
      past: [],
      future: [],
      selection: null,
      saved: opened ? JSON.stringify(doc) : "",
      epoch: this.state.epoch + 1,
      revision: this.state.revision + 1,
    });
  }
  acknowledgeExport(doc: MapDocument, epoch: number) {
    if (epoch === this.state.epoch) this.update({ saved: JSON.stringify(doc) });
  }
  isDirty = () => this.state.serialized !== this.state.saved;
}
