import { readFileSync } from "node:fs";
import ELK from "elkjs/lib/elk.bundled.js";
import { test, expect } from "@playwright/test";
import { arrangeGraph } from "../../src/nodes/core/layout";
import { definition } from "../../src/nodes/core/catalog";
import { GraphSession } from "../../src/nodes/core/session";
import { parseDocument } from "../../src/nodes/core/validation";

test("connected layout preserves graph meaning, avoids overlap and commits as one undoable edit", async () => {
  const doc = parseDocument(
    readFileSync("content/knight-of-despair.node-graph.json", "utf8"),
  );
  // Exercise a real edit even when the distributed example is already arranged.
  doc.nodes[0].position.x += 40;
  const before = structuredClone(doc);
  const positions = await arrangeGraph(doc, new ELK());
  expect(doc).toEqual(before);
  expect(positions).toHaveLength(doc.nodes.length);
  const s = new GraphSession(doc);
  s.edit({ type: "move", positions });
  const after = s.getSnapshot();
  expect(after.undoCount).toBe(1);
  expect(after.doc.edges).toEqual(before.edges);
  expect(
    after.doc.nodes.map((node) => ({ ...node, position: { x: 0, y: 0 } })),
  ).toEqual(
    before.nodes.map((node) => ({ ...node, position: { x: 0, y: 0 } })),
  );
  expect(after.diagnostics).toEqual(
    new GraphSession(before).getSnapshot().diagnostics,
  );
  for (const a of after.doc.nodes)
    for (const b of after.doc.nodes) {
      if (a.id >= b.id) continue;
      const ad = definition(a.kind),
        bd = definition(b.kind);
      expect(
        a.position.x + ad.width <= b.position.x ||
          b.position.x + bd.width <= a.position.x ||
          a.position.y + ad.height <= b.position.y ||
          b.position.y + bd.height <= a.position.y,
      ).toBe(true);
    }
  const request = after.doc.nodes.find((n) => n.kind === "skill-request")!;
  const qualify = after.doc.nodes.find(
    (n) => n.kind === "skill-resonance-qualify",
  )!;
  const admit = after.doc.nodes.find(
    (n) => n.kind === "skill-resonance-admit",
  )!;
  expect(request.position.x).toBeLessThan(qualify.position.x);
  expect(qualify.position.x).toBeLessThan(admit.position.x);
  expect(parseDocument(s.export().json)).toEqual(after.doc);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
  s.travel(true);
  expect(s.getSnapshot().doc).toEqual(after.doc);
  expect(await arrangeGraph(after.doc, new ELK())).toEqual(positions);
});

test("layout failure and a stale computed layout preserve the current document", async () => {
  const s = new GraphSession(
    parseDocument(
      readFileSync("content/hello-goodbye.node-graph.json", "utf8"),
    ),
  );
  const before = s.getSnapshot();
  await expect(
    arrangeGraph(before.doc, {
      layout: async () => {
        throw Error("worker failed");
      },
    }),
  ).rejects.toThrow("worker failed");
  expect(s.getSnapshot()).toBe(before);
  const positions = await arrangeGraph(before.doc, new ELK());
  s.edit({ type: "rename", name: "사용자 수정" });
  const current = s.getSnapshot();
  expect(() => s.edit({ type: "move", positions }, before)).toThrow(
    "요청 이후 문서가 바뀌었습니다",
  );
  expect(s.getSnapshot()).toBe(current);
});

test("arranging preserves the author's section order independently of frame IDs", async () => {
  const doc = parseDocument(
    readFileSync("content/laetitia.node-graph.json", "utf8"),
  );
  const top = (id: string) =>
    Math.min(
      ...doc.nodes.filter((n) => n.frame === id).map((n) => n.position.y),
    );
  const before = [...doc.frames]
    .sort((a, b) => top(a.id) - top(b.id))
    .map((f) => f.id);
  doc.frames.reverse();
  const positions = new Map(
    (await arrangeGraph(doc, new ELK())).map((p) => [p.id, p.position]),
  );
  for (const n of doc.nodes) n.position = positions.get(n.id)!;
  expect(
    [...doc.frames].sort((a, b) => top(a.id) - top(b.id)).map((f) => f.id),
  ).toEqual(before);
});
