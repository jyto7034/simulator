import type {
  Command,
  Exported,
  GraphDocument,
  Snapshot,
  Version,
} from "../types";
import { example } from "./examples";
import { applyPatch, entry, type Entry } from "./history";
import {
  byteLength,
  canonicalize,
  freeze,
  GraphError,
  id,
  MAX_FILE_BYTES,
} from "./model";
import { apply, copy } from "./operations";
import {
  diagnostics,
  frameRects,
  parseDocument,
  validateDocument,
} from "./validation";

export class GraphSession {
  private state: Snapshot;
  private serialized: string;
  private saved: string | undefined;
  private past: Entry[] = [];
  private future: Entry[] = [];
  private exports: { token: string; serialized: string }[] = [];
  private lastAcknowledged?: string;
  constructor(doc = example("values")) {
    const prepared = this.prepare(doc);
    this.serialized = prepared.serialized;
    this.state = freeze({
      doc: prepared.doc,
      epoch: 0,
      revision: 0,
      dirty: true,
      canUndo: false,
      canRedo: false,
      undoCount: 0,
      ...prepared.derived,
    });
  }
  getSnapshot = () => this.state;
  private prepare(input: GraphDocument) {
    validateDocument(input);
    const doc = structuredClone(input);
    canonicalize(doc);
    const serialized = JSON.stringify(doc);
    if (byteLength(serialized) > MAX_FILE_BYTES)
      throw new GraphError("file-size", "문서 크기 제한은 5MB입니다.");
    return {
      doc: freeze(doc),
      serialized,
      derived: { diagnostics: diagnostics(doc), frameRects: frameRects(doc) },
    };
  }
  assertCurrent(version: Version) {
    if (
      version.epoch !== this.state.epoch ||
      version.revision !== this.state.revision
    )
      throw new GraphError(
        "stale",
        "요청 이후 문서가 바뀌었습니다. 현재 작업을 보존했습니다. 다시 시도하세요.",
      );
  }
  private snapshot(patch: Partial<Snapshot> = {}) {
    this.state = freeze({
      ...this.state,
      ...patch,
      dirty: this.saved !== this.serialized,
      canUndo: !!this.past.length,
      canRedo: !!this.future.length,
      undoCount: this.past.length,
    });
  }
  edit(command: Command, version: Version = this.state): string[] {
    this.assertCurrent(version);
    const draft = structuredClone(this.state.doc);
    const selected = apply(draft, command);
    const prepared = this.prepare(draft);
    if (prepared.serialized === this.serialized) return selected;
    const next = [...this.past, entry(this.state.doc, prepared.doc)];
    let bytes = next.reduce((sum, e) => sum + e.bytes, 0);
    while (next.length > 100 || bytes > 32_000_000)
      bytes -= next.shift()!.bytes;
    // Everything that may reject the edit has finished. Commit document and history together.
    this.past = next;
    this.future = [];
    this.serialized = prepared.serialized;
    this.snapshot({
      doc: prepared.doc,
      ...prepared.derived,
      revision: this.state.revision + 1,
    });
    return selected;
  }
  travel(redo: boolean, version: Version = this.state) {
    this.assertCurrent(version);
    const e = (redo ? this.future : this.past).at(-1);
    if (!e) return;
    const draft = structuredClone(this.state.doc);
    applyPatch(draft, redo ? e.forward : e.backward);
    const prepared = this.prepare(draft);
    if (redo) {
      this.future.pop();
      this.past.push(e);
    } else {
      this.past.pop();
      this.future.push(e);
    }
    this.serialized = prepared.serialized;
    this.snapshot({
      doc: prepared.doc,
      ...prepared.derived,
      revision: this.state.revision + 1,
    });
  }
  replace(doc: GraphDocument, opened: boolean, version: Version = this.state) {
    this.assertCurrent(version);
    const prepared = this.prepare(doc);
    this.saved = opened ? prepared.serialized : undefined;
    this.serialized = prepared.serialized;
    this.past = [];
    this.future = [];
    this.exports = [];
    this.lastAcknowledged = undefined;
    this.snapshot({
      doc: prepared.doc,
      ...prepared.derived,
      epoch: this.state.epoch + 1,
      revision: this.state.revision + 1,
    });
  }
  load(json: string, version: Version) {
    this.assertCurrent(version);
    this.replace(parseDocument(json), true, version);
  }
  copy(ids: string[]) {
    return copy(this.state.doc, ids);
  }
  export(): Exported {
    const json = JSON.stringify(this.state.doc, null, 2);
    if (byteLength(json) > MAX_FILE_BYTES)
      throw new GraphError("file-size", "출력 파일 크기 제한은 5MB입니다.");
    const token = id();
    // Character-code check avoids embedding control characters in a filename regexp.
    const name = Array.from(this.state.doc.name, (c) =>
      c.charCodeAt(0) < 32 ||
      (c.charCodeAt(0) >= 127 && c.charCodeAt(0) <= 159) ||
      '<>:"/\\|?*'.includes(c)
        ? "_"
        : c,
    ).join("");
    this.exports = [
      ...this.exports.slice(-7),
      { token, serialized: this.serialized },
    ];
    return {
      token,
      json,
      filename: `${name.trim() ? name : "graph"}.node-graph.json`,
    };
  }
  acknowledge(token: string) {
    if (token === this.lastAcknowledged) return;
    const index = this.exports.findIndex((e) => e.token === token);
    if (index < 0)
      throw new GraphError(
        "export",
        "확인할 출력 파일이 없거나 문서가 교체되었습니다.",
      );
    this.saved = this.exports[index].serialized;
    this.lastAcknowledged = token;
    this.exports = this.exports.slice(index + 1);
    this.snapshot();
  }
}
