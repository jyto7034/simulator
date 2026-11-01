import { downloadJson, type ExportPort } from "../editor/files";
import { parseSkill } from "./validation";
import type { SkillDocument } from "./model";
import type { SkillSession } from "./session";
type Prepared = {
  doc: SkillDocument;
  request: number;
  epoch: number;
  revision: number;
};
export class SkillFiles {
  busy = false;
  private request = 0;
  constructor(
    private session: SkillSession,
    private output: ExportPort = downloadJson,
  ) {}
  async save() {
    if (this.busy) return;
    this.busy = true;
    const { doc, epoch } = this.session.getSnapshot();
    try {
      const json = JSON.stringify(doc, null, 2);
      if (new TextEncoder().encode(json).byteLength > 5_000_000)
        throw new Error("파일 크기 제한은 5MB입니다.");
      const name = [...doc.name]
        .map((c) =>
          c.charCodeAt(0) < 32 || '<>:"/\\|?*'.includes(c) ? "_" : c,
        )
        .join("");
      await this.output(json, `${name || "skill"}.skill-design.json`);
      if (epoch === this.session.getSnapshot().epoch)
        this.session.message(
          "다운로드를 요청했습니다. 파일을 확인한 뒤 ‘다운로드 확인’을 누르세요.",
        );
      return { doc, epoch };
    } catch (e) {
      if (epoch === this.session.getSnapshot().epoch)
        this.session.message(`저장 실패: ${String(e)}`);
    } finally {
      this.busy = false;
    }
  }
  async read(file: {
    size: number;
    text(): Promise<string>;
  }): Promise<Prepared> {
    const request = ++this.request;
    const { epoch, revision } = this.session.getSnapshot();
    if (file.size > 5_000_000) throw new Error("파일 크기 제한은 5MB입니다.");
    const doc = parseSkill(await file.text());
    const prepared = { doc, request, epoch, revision };
    this.assertCurrent(prepared);
    return prepared;
  }
  open(prepared: Prepared) {
    this.assertCurrent(prepared);
    this.session.replace(prepared.doc, true);
  }
  private assertCurrent(p: Prepared) {
    const s = this.session.getSnapshot();
    if (
      p.request !== this.request ||
      p.epoch !== s.epoch ||
      p.revision !== s.revision
    )
      throw new Error(
        "파일을 읽는 동안 작업이 바뀌었습니다. 현재 문서를 보존했습니다.",
      );
  }
}
