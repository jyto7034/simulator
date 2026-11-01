import { freeze } from "../nodes/core/model";
import {
  parseSource,
  scalarPatch,
  sourceLimit,
  type ParsedSource,
  type Path,
} from "./source";

export type Version = Readonly<{ epoch: number; revision: number }>;
export type CompileTicket = Version &
  Readonly<{ request_id: number; source: string }>;
export type CoreDiagnostic = {
  code: string;
  document: string;
  source_digest: string;
  source: { definition: string; calls: string[]; inputs: string[] };
  reason: string;
};
export type CompileResult = Version & {
  request_id: number;
  source_digest: string;
  target: "composition-yaml-v1";
} & (
    | { status: "compiled"; artifact: string; artifact_digest: string }
    | { status: "rejected"; diagnostic: CoreDiagnostic }
  );
export type Definition = Readonly<{
  ticket: CompileTicket;
  sourceDigest: string;
  artifact: string;
  artifactDigest: string;
}>;
type FileState = Readonly<{
  name: string;
  revision: string;
  savedSource: string;
}>;
export type SaveTicket = Version &
  Readonly<{
    id: number;
    name: string;
    source: string;
    expected: string | null;
  }>;
export type Snapshot = Readonly<
  Version & {
    source: string;
    parsed: ParsedSource;
    file: FileState | null;
    dirty: boolean;
    canUndo: boolean;
    canRedo: boolean;
    saving: boolean;
    compilation: "idle" | "running" | "compiled" | "rejected" | "failed";
    diagnostic: CoreDiagnostic | null;
    definition: Definition | null;
    message: string;
  }
>;
const same = (a: Version, b: Version) =>
  a.epoch === b.epoch && a.revision === b.revision;
const bytes = (value: string) => new TextEncoder().encode(value).length;

/** Text is the sole authored state. Parse trees, properties and graphs are projections. */
export class CompositionSession {
  private state: Snapshot;
  private past: string[] = [];
  private future: string[] = [];
  private listeners = new Set<() => void>();
  private serial = 0;
  private compileTicket: CompileTicket | null = null;
  private saveTicket: SaveTicket | null = null;
  constructor(source: string) {
    this.checkSize(source);
    this.state = freeze({
      epoch: 0,
      revision: 0,
      source,
      parsed: parseSource(source),
      file: null,
      dirty: true,
      canUndo: false,
      canRedo: false,
      saving: false,
      compilation: "idle",
      diagnostic: null,
      definition: null,
      message: "원본과 속성은 같은 작성 내용을 편집합니다.",
    });
  }
  getSnapshot = () => this.state;
  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };
  private publish(patch: Partial<Snapshot>) {
    const next = { ...this.state, ...patch };
    next.dirty = next.file?.savedSource !== next.source;
    next.canUndo = this.past.length > 0;
    next.canRedo = this.future.length > 0;
    this.state = freeze(next);
    for (const listener of [...this.listeners]) {
      try {
        listener();
      } catch (error) {
        console.error("확정된 조합 편집의 화면 알림 실패", error);
      }
    }
  }
  private checkSize(source: string) {
    if (bytes(source) > sourceLimit)
      throw new Error("원본 크기 제한은 2MB입니다.");
  }
  private expected(version: Version) {
    if (!same(version, this.state))
      throw new Error("편집 대상이 바뀌었습니다. 다시 선택하세요.");
  }
  edit(source: string, version: Version = this.state) {
    this.expected(version);
    this.checkSize(source);
    if (source === this.state.source) return;
    const parsed = parseSource(source);
    const past = [...this.past, this.state.source];
    let size = past.reduce((sum, text) => sum + bytes(text), 0);
    while (size > 32_000_000 && past.length > 1) size -= bytes(past.shift()!);
    this.past = past;
    this.future = [];
    this.compileTicket = null;
    this.publish({
      source,
      parsed,
      revision: this.state.revision + 1,
      compilation: "idle",
      diagnostic: null,
      message: "편집했습니다. 실행하려면 이 원본을 컴파일하세요.",
    });
  }
  scalar(
    path: Path,
    value: null | boolean | number | string,
    version: Version,
  ) {
    this.expected(version);
    this.edit(scalarPatch(this.state.source, path, value), version);
  }
  travel(redo: boolean) {
    const from = redo ? this.future : this.past;
    const source = from.at(-1);
    if (source === undefined) return;
    const parsed = parseSource(source);
    if (redo) {
      this.future = from.slice(0, -1);
      this.past = [...this.past, this.state.source];
    } else {
      this.past = from.slice(0, -1);
      this.future = [...this.future, this.state.source];
    }
    this.compileTicket = null;
    this.publish({
      source,
      parsed,
      revision: this.state.revision + 1,
      compilation: "idle",
      diagnostic: null,
      message: redo ? "다시 실행했습니다." : "실행 취소했습니다.",
    });
  }
  replace(source: string, file: FileState | null, version: Version) {
    this.expected(version);
    this.checkSize(source);
    const parsed = parseSource(source);
    this.past = [];
    this.future = [];
    this.compileTicket = null;
    this.publish({
      source,
      parsed,
      file,
      epoch: this.state.epoch + 1,
      revision: 0,
      definition: null,
      compilation: "idle",
      diagnostic: null,
      message: "원본을 열었습니다. 미완성 내용도 그대로 보존합니다.",
    });
  }
  beginCompile(): CompileTicket {
    const ticket = freeze({
      epoch: this.state.epoch,
      revision: this.state.revision,
      request_id: ++this.serial,
      source: this.state.source,
    });
    this.compileTicket = ticket;
    this.publish({
      compilation: "running",
      diagnostic: null,
      message: "Core에서 검증 중…",
    });
    return ticket;
  }
  finishCompile(ticket: CompileTicket, result: CompileResult) {
    if (this.compileTicket !== ticket || !same(ticket, this.state))
      return false;
    if (!same(ticket, result) || result.request_id !== ticket.request_id)
      throw new Error("컴파일 응답의 리비전이 다릅니다.");
    this.compileTicket = null;
    if (result.status === "compiled")
      this.publish({
        compilation: "compiled",
        diagnostic: null,
        definition: {
          ticket,
          sourceDigest: result.source_digest,
          artifact: result.artifact,
          artifactDigest: result.artifact_digest,
        },
        message: "현재 원본을 Core에서 컴파일했습니다.",
      });
    else
      this.publish({
        compilation: "rejected",
        diagnostic: result.diagnostic,
        message: "실행할 수 없는 원본입니다. 진단 위치를 수정하세요.",
      });
    return true;
  }
  failCompile(ticket: CompileTicket, reason: string) {
    if (this.compileTicket !== ticket || !same(ticket, this.state)) return;
    this.compileTicket = null;
    this.publish({ compilation: "failed", message: reason });
  }
  beginSave(name: string): SaveTicket {
    if (this.saveTicket) throw new Error("저장이 진행 중입니다.");
    const ticket = freeze({
      id: ++this.serial,
      epoch: this.state.epoch,
      revision: this.state.revision,
      source: this.state.source,
      name,
      expected:
        this.state.file?.name === name ? this.state.file.revision : null,
    });
    this.saveTicket = ticket;
    this.publish({ saving: true, message: "저장 중…" });
    return ticket;
  }
  finishSave(ticket: SaveTicket, revision: string) {
    if (this.saveTicket !== ticket) return;
    this.saveTicket = null;
    this.publish({
      saving: false,
      ...(this.state.epoch === ticket.epoch
        ? {
            file: { name: ticket.name, revision, savedSource: ticket.source },
            message: same(ticket, this.state)
              ? "저장했습니다."
              : "저장 후 편집한 변경이 남아 있습니다.",
          }
        : {}),
    });
  }
  failSave(ticket: SaveTicket, reason: string) {
    if (this.saveTicket !== ticket) return;
    this.saveTicket = null;
    this.publish({ saving: false, message: reason });
  }
  message(message: string) {
    this.publish({ message });
  }
}
