import type { EditorSession } from "./history";
import type { MapDocument } from "./model";
import { parseDocument, serializeDocument } from "./serialization";
type OpenVersion = Readonly<{
  request: number;
  epoch: number;
  revision: number;
}>;
export type PreparedOpen = OpenVersion & { readonly doc: MapDocument };
export type ExportPort = (text: string, filename: string) => Promise<void>;
export const downloadJson: ExportPort = async (text, filename) => {
  const url = URL.createObjectURL(
    new Blob([text], { type: "application/json" }),
  );
  try {
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
  } finally {
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
};
export class DocumentFiles {
  busy = false;
  private openRequest = 0;
  constructor(
    private session: EditorSession,
    private output: ExportPort = downloadJson,
  ) {}
  async save() {
    if (this.busy) return;
    this.busy = true;
    const { doc, epoch } = this.session.getSnapshot();
    try {
      this.session.message("다운로드 요청 중…");
      await this.output(
        serializeDocument(doc),
        `${[...doc.name].map((c) => (c.charCodeAt(0) < 32 || '<>:"/\\|?*'.includes(c) ? "_" : c)).join("") || "battle-map"}.battle-map.json`,
      );
      // Downloads do not prove disk completion. Keep dirty until explicit user acknowledgement.
      if (epoch === this.session.getSnapshot().epoch)
        this.session.message(
          "다운로드를 요청했습니다. 디스크 저장 완료는 확인할 수 없습니다. 파일을 확인한 뒤 「다운로드 확인」을 누르세요.",
        );
      return { doc, epoch };
    } catch (error) {
      if (epoch === this.session.getSnapshot().epoch)
        this.session.message(
          `저장 실패: ${error instanceof Error ? error.message : String(error)}`,
        );
    } finally {
      this.busy = false;
    }
  }
  async read(file: {
    text(): Promise<string>;
    size: number;
  }): Promise<PreparedOpen> {
    const request = ++this.openRequest;
    const { epoch, revision } = this.session.getSnapshot();
    if (file.size > 5_000_000) throw new Error("파일 크기 제한은 5MB입니다.");
    const doc = parseDocument(await file.text());
    const prepared = { doc, request, epoch, revision };
    this.assertCurrent(prepared);
    return prepared;
  }
  open(prepared: PreparedOpen) {
    // Reading and user confirmation can finish in different turns. Check again
    // immediately before replacing the document, with no await between them.
    this.assertCurrent(prepared);
    this.session.replace(prepared.doc, true);
  }
  private assertCurrent({ request, epoch, revision }: OpenVersion) {
    const current = this.session.getSnapshot();
    if (
      request !== this.openRequest ||
      epoch !== current.epoch ||
      revision !== current.revision
    )
      throw new Error(
        "파일 교체 전에 작업이 바뀌었습니다. 현재 작업을 보존했습니다. 다시 열어주세요.",
      );
  }
}
