import { test, expect } from "@playwright/test";
import {
  createSkill,
  makeBlock,
  insertBlock,
  removeSelection,
  duplicateBlock,
} from "../../src/skill/model";
import { exampleSkill } from "../../src/skill/examples";
import { SkillSession } from "../../src/skill/session";
import { SkillFiles } from "../../src/skill/files";
import { diagnoseSkill, parseSkill } from "../../src/skill/validation";

test("skill nested editing, deletion and undo preserve IDs, references and blank policies", () => {
  const original = exampleSkill("butterfly");
  const s = new SkillSession(original);
  const removed = original.states[0];
  s.commit((d) => removeSelection(d, { type: "state", id: removed.id }));
  expect(
    diagnoseSkill(s.getSnapshot().doc).some(
      (d) => d.code === "missing-reference",
    ),
  ).toBe(true);
  s.travel("undo");
  expect(s.getSnapshot().doc).toEqual(original);
  const r = original.routines[2];
  const parent = r.blocks[1];
  const child = makeBlock("wait");
  s.commit((d) =>
    insertBlock(
      d,
      { type: "slot", routineId: r.id, parentId: parent.id, lane: "children" },
      child,
    ),
  );
  expect(s.getSnapshot().doc.routines[2].blocks[1].children.at(-1)?.id).toBe(
    child.id,
  );
  expect(parseSkill(JSON.stringify(s.getSnapshot().doc))).toEqual(
    s.getSnapshot().doc,
  );
  s.travel("undo");
  expect(s.getSnapshot().doc).toEqual(original);
  s.travel("redo");
  expect(
    s.getSnapshot().doc.routines[2].blocks[1].children.at(-1)?.fields.duration,
  ).toBe("");
  const clone = duplicateBlock(parent);
  expect(clone.id).not.toBe(parent.id);
  expect(clone.children[0].id).not.toBe(parent.children[0].id);
});

test("skill failed edits and invalid files preserve document, history and saved baseline", () => {
  const s = new SkillSession(exampleSkill("hood"));
  s.acknowledge(s.getSnapshot().doc, 0);
  s.commit((d) => {
    d.name = "수정";
  });
  s.travel("undo");
  const before = s.getSnapshot();
  expect(
    s.commit((d) => {
      d.name = "부분 수정";
      d.routines.push(d.routines[0]);
    }),
  ).toBe(false);
  expect(s.getSnapshot()).toEqual({
    ...before,
    message: s.getSnapshot().message,
  });
  expect(s.isDirty()).toBe(false);
  expect(() => parseSkill('{"format":"battle-map","version":1}')).toThrow();
  const invalid = { ...before.doc, version: 2 };
  expect(() => parseSkill(JSON.stringify(invalid))).toThrow();
  expect(s.getSnapshot().doc).toEqual(before.doc);
  const input = createSkill();
  s.replace(input, true);
  input.name = "호출자 변경";
  expect(s.getSnapshot().doc.name).toBe("새 스킬");
  let retained = input;
  s.commit((d) => {
    retained = d;
    d.name = "확정";
  });
  retained.name = "나중 변경";
  expect(s.getSnapshot().doc.name).toBe("확정");
});

test("skill file requests reject stale confirmations and preserve new edits during download", async () => {
  const s = new SkillSession(exampleSkill("hood"));
  let finish!: () => void;
  const output: string[] = [];
  const files = new SkillFiles(s, (text) => {
    output.push(text);
    return new Promise((resolve) => {
      finish = resolve;
    });
  });
  const pending = files.save();
  s.commit((d) => {
    d.name = "다운로드 중 편집";
  });
  finish();
  const exported = (await pending)!;
  s.acknowledge(exported.doc, exported.epoch);
  expect(s.isDirty()).toBe(true);
  expect(parseSkill(output[0]).name).toBe("두건 · 창 배치와 동시 사격");
  const body = JSON.stringify(createSkill());
  const prepared = await files.read({
    size: body.length,
    text: async () => body,
  });
  s.commit((d) => {
    d.description = "확인 대기 중 편집";
  });
  expect(() => files.open(prepared)).toThrow("작업이 바뀌었습니다");
  expect(s.getSnapshot().doc.description).toBe("확인 대기 중 편집");
  const latest = await files.read({
    size: body.length,
    text: async () => body,
  });
  files.open(latest);
  expect(s.getSnapshot().doc.name).toBe("새 스킬");
  expect(s.isDirty()).toBe(false);
  const failed = new SkillFiles(s, async () => {
    throw new Error("파일 전달 실패");
  });
  expect(await failed.save()).toBeUndefined();
  expect(failed.busy).toBe(false);
  expect(s.getSnapshot().message).toContain("저장 실패");
  s.commit((d) => {
    d.description = "가".repeat(1_700_000);
  });
  let delivered = false;
  const oversized = new SkillFiles(s, async () => {
    delivered = true;
  });
  expect(await oversized.save()).toBeUndefined();
  expect(delivered).toBe(false);
  expect(s.isDirty()).toBe(true);
  expect(s.getSnapshot().message).toContain("5MB");
});
