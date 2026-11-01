import { test, expect } from "@playwright/test";
import { fixture } from "./fixtures";
import { EditorSession } from "../../src/editor/history";
import { DocumentFiles } from "../../src/editor/files";
import { paint, resizeMap } from "../../src/editor/operations";
import type { MapDocument } from "../../src/editor/model";

test("constructor and replacement take ownership without retaining caller aliases", () => {
  const input = fixture();
  const s = new EditorSession(input);
  const original = structuredClone(input);
  input.routes[0].points[0].x = 3.5;
  expect(s.getSnapshot().doc).toEqual(original);
  const opened = fixture("irregular");
  const expected = structuredClone(opened);
  s.replace(opened, true);
  opened.zones[0].cells.length = 0;
  expect(s.getSnapshot().doc).toEqual(expected);
  expect(s.isDirty()).toBe(false);
});

test("committed documents and history cannot be changed through retained drafts or snapshots", () => {
  const s = new EditorSession(fixture());
  let draft!: MapDocument;
  const point = { id: "external-point", name: "지점", x: 2.5, y: 2.5 };
  s.commit((d) => {
    draft = d;
    d.points.push(point);
    d.routes[0].points[0].x = 1.5;
  });
  const committed = structuredClone(s.getSnapshot().doc);
  draft.routes[0].points[0].x = 5.5;
  point.name = "외부 수정";
  expect(s.getSnapshot().doc).toEqual(committed);
  expect(() => {
    s.getSnapshot().doc.routes[0].points[0].x = 4.5;
  }).toThrow();
  expect(() => {
    s.getSnapshot().past[0].valid.push("127,127");
  }).toThrow();
  expect(s.getSnapshot().serialized).toBe(JSON.stringify(committed));
  s.travel("undo");
  expect(s.getSnapshot().doc).toEqual(fixture());
  s.travel("redo");
  expect(s.getSnapshot().doc).toEqual(committed);
});

test("notification failures preserve committed outcomes, notify other subscribers and release export busy state", async () => {
  const s = new EditorSession(fixture());
  const errors: unknown[][] = [];
  const report = console.error;
  console.error = (...args: unknown[]) => {
    errors.push(args);
  };
  try {
    s.subscribe(() => {
      throw new Error("subscriber failed");
    });
    const observed: string[] = [];
    s.subscribe(() => {
      observed.push(s.getSnapshot().doc.name);
    });
    expect(
      s.commit((d) => {
        d.name = "확정된 이름";
      }),
    ).toBe(true);
    expect(observed).toEqual(["확정된 이름"]);
    expect(s.getSnapshot().past).toHaveLength(1);
    s.travel("undo");
    expect(observed.at(-1)).toBe(fixture().name);
    s.travel("redo");
    expect(observed.at(-1)).toBe("확정된 이름");
    let outputs = 0;
    const files = new DocumentFiles(s, async () => {
      outputs++;
    });
    const exported = await files.save();
    expect(exported?.doc.name).toBe("확정된 이름");
    expect(files.busy).toBe(false);
    expect(outputs).toBe(1);
    expect(s.getSnapshot().message).not.toContain("저장 실패");
    expect(s.isDirty()).toBe(true);
    s.acknowledgeExport(exported!.doc, exported!.epoch);
    expect(s.isDirty()).toBe(false);
    expect(errors.length).toBeGreaterThan(0);
  } finally {
    console.error = report;
  }
});

test("partial preparation failures leave document, redo, revision and saved baseline untouched", () => {
  const s = new EditorSession(fixture());
  s.acknowledgeExport(s.getSnapshot().doc, 0);
  s.commit((d) => {
    d.name = "다시 실행할 변경";
  });
  s.travel("undo");
  const before = s.getSnapshot();
  expect(
    s.commit((d) => {
      resizeMap(d, "width", 10);
      paint(d, ["0,2"], "zone-add", "zone-deploy");
      d.routes[0].points[0].x = NaN;
    }),
  ).toBe(false);
  expect(s.getSnapshot()).toEqual({
    ...before,
    message: s.getSnapshot().message,
  });
  expect(s.isDirty()).toBe(false);
  const observed: MapDocument[] = [];
  s.subscribe(() => {
    observed.push(s.getSnapshot().doc);
  });
  expect(
    s.commit((d) => {
      resizeMap(d, "width", 10);
      paint(d, ["0,2"], "zone-add", "zone-deploy");
    }),
  ).toBe(true);
  expect(observed).toHaveLength(1);
  expect(observed[0].width).toBe(10);
  expect(observed[0].valid).toContain("9,0");
  expect(observed[0].zones[0].cells).toContain("0,2");
  expect(observed[0].zones[1].cells).not.toContain("0,2");
  expect(s.getSnapshot().past).toHaveLength(before.past.length + 1);
  s.travel("undo");
  expect(s.getSnapshot().doc).toEqual(before.doc);
});

test("nested session edits cannot publish a partial transaction", () => {
  const s = new EditorSession(fixture());
  const before = s.getSnapshot();
  expect(
    s.commit((d) => {
      d.name = "바깥 변경";
      s.commit((inner) => {
        inner.description = "안쪽 변경";
      });
      throw new Error("outer failed");
    }),
  ).toBe(false);
  expect(s.getSnapshot()).toEqual({
    ...before,
    message: s.getSnapshot().message,
  });
  expect(
    s.commit((d) => {
      d.name = "다음 정상 변경";
    }),
  ).toBe(true);
});

test("async edit callbacks are rejected before any partial draft is committed", async () => {
  const s = new EditorSession(fixture());
  let resume!: () => void;
  const gate = new Promise<void>((resolve) => {
    resume = resolve;
  });
  const before = s.getSnapshot();
  expect(
    s.commit(async (d) => {
      d.name = "await 이전";
      await gate;
      d.description = "await 이후";
    }),
  ).toBe(false);
  resume();
  await gate;
  expect(s.getSnapshot()).toEqual({
    ...before,
    message: s.getSnapshot().message,
  });
  expect(
    s.commit((d) => {
      d.name = "다음 정상 변경";
    }),
  ).toBe(true);
});

test("prepared file opens recheck edits, replacement and newer requests at the actual apply boundary", async () => {
  const s = new EditorSession(fixture());
  const files = new DocumentFiles(s, async () => {});
  const text = JSON.stringify(fixture("empty"));
  const file = { size: text.length, text: async () => text };
  const prepared = await files.read(file);
  s.commit((d) => {
    d.name = "확인창 대기 중 편집";
  });
  const edited = s.getSnapshot();
  expect(() => files.open(prepared)).toThrow("작업이 바뀌");
  expect(s.getSnapshot()).toBe(edited);
  const older = await files.read(file);
  const newest = await files.read(file);
  expect(() => files.open(older)).toThrow("작업이 바뀌");
  expect(s.getSnapshot()).toBe(edited);
  files.open(newest);
  expect(s.getSnapshot().doc.valid).toEqual([]);
  expect(s.isDirty()).toBe(false);
  const opened = s.getSnapshot();
  expect(() => files.open(newest)).toThrow("작업이 바뀌");
  expect(s.getSnapshot()).toBe(opened);
  const late = await files.read(file);
  s.replace(fixture(), false);
  const replaced = s.getSnapshot();
  expect(() => files.open(late)).toThrow("작업이 바뀌");
  expect(s.getSnapshot()).toBe(replaced);
});
