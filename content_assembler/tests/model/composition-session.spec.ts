import { test, expect } from "@playwright/test";
import {
  CompositionSession,
  type CompileResult,
  type CompileTicket,
} from "../../src/composition/session";
import { contentOperation } from "../../server/content-files";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const success = (ticket: CompileTicket): CompileResult => ({
  ...ticket,
  target: "composition-yaml-v1",
  source_digest: "source-digest",
  artifact_digest: "artifact-digest",
  status: "compiled",
  artifact: "test artifact",
});

test("text and property edits share history; incomplete text remains saveable", () => {
  const original = "# 유지\na: 1 # 수치\nb: { Unresolved: 미정 }\n";
  const session = new CompositionSession(original);
  const before = session.getSnapshot();
  if (!before.parsed.ok) throw new Error("valid source expected");
  expect(Object.isFrozen(before.parsed.spans)).toBe(true);
  expect(Object.isFrozen(before.parsed.spans['["a"]'])).toBe(true);
  session.scalar(["a"], 2, before);
  expect(session.getSnapshot().source).toBe(original.replace("a: 1", "a: 2"));
  expect(() => session.scalar(["a"], 3, before)).toThrow("바뀌었습니다");
  session.travel(false);
  expect(session.getSnapshot().source).toBe(original);
  session.travel(true);
  expect(session.getSnapshot().source).toContain("a: 2");
  session.edit("a: [");
  expect(session.getSnapshot().parsed.ok).toBe(false);
  const ticket = session.beginSave("미완성.skill.yaml");
  expect(ticket.source).toBe("a: [");
  session.finishSave(ticket, "disk-v1");
  expect(session.getSnapshot().dirty).toBe(false);
});

test("late compile responses cannot change a new revision; running definitions keep their source", () => {
  const session = new CompositionSession("a: 1\n");
  const first = session.beginCompile();
  session.finishCompile(first, success(first));
  const runningDefinition = session.getSnapshot().definition!;
  const pending = session.beginCompile();
  session.edit("a: 2\n");
  expect(session.finishCompile(pending, success(pending))).toBe(false);
  expect(session.getSnapshot().compilation).toBe("idle");
  const next = session.beginCompile();
  session.finishCompile(next, success(next));
  expect(runningDefinition.ticket.source).toBe("a: 1\n");
  expect(session.getSnapshot().definition!.ticket.source).toBe("a: 2\n");
  const stale = session.beginCompile();
  const fresh = session.beginCompile();
  expect(session.finishCompile(stale, success(stale))).toBe(false);
  expect(session.finishCompile(fresh, success(fresh))).toBe(true);
});

test("save completion acknowledges only the written source and cannot rebind a replaced document", () => {
  const session = new CompositionSession("a: 1\n");
  const first = session.beginSave("one.skill.yaml");
  expect(() => session.beginSave("one.skill.yaml")).toThrow("진행 중");
  session.edit("a: 2\n");
  session.finishSave(first, "disk-1");
  expect(session.getSnapshot().dirty).toBe(true);
  const second = session.beginSave("one.skill.yaml");
  expect(second.expected).toBe("disk-1");
  session.replace("a: 3\n", null, session.getSnapshot());
  session.finishSave(second, "disk-2");
  expect(session.getSnapshot().file).toBeNull();
  expect(session.getSnapshot().source).toBe("a: 3\n");
});

test("actual content storage preserves incomplete YAML and rejects stale saves without changing graph files", async () => {
  const root = await mkdtemp(join(tmpdir(), "composition-files-"));
  const name = "한글 미완성.skill.yaml";
  try {
    const saved = await contentOperation(root, {
      kind: "composition",
      operation: "save",
      name,
      json: "# 유지\na: [",
      expected: null,
    });
    if (!("revision" in saved)) throw new Error("file expected");
    expect(
      await contentOperation(root, {
        kind: "composition",
        operation: "read",
        name,
      }),
    ).toEqual(saved);
    await expect(
      contentOperation(root, {
        kind: "composition",
        operation: "save",
        name,
        json: "a: 2",
        expected: null,
      }),
    ).rejects.toThrow("같은 이름");
    expect(await readFile(join(root, name), "utf8")).toBe("# 유지\na: [");
    await expect(
      contentOperation(root, {
        kind: "composition",
        operation: "read",
        name: "a.node-graph.json",
      }),
    ).rejects.toThrow("파일 이름");
    await expect(
      contentOperation(root, {
        kind: "composition",
        operation: "save-groups",
        json: "{}",
        expected: null,
      }),
    ).rejects.toThrow("그룹");
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
