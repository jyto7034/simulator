import { readFileSync } from "node:fs";
import { test, expect } from "@playwright/test";
import { catalog, makeNode } from "../../src/nodes/core/catalog";
import { example } from "../../src/nodes/core/examples";
import { createDocument, GraphError } from "../../src/nodes/core/model";
import { GraphSession } from "../../src/nodes/core/session";
import {
  parseDocument,
  parseFragment,
  validateDocument,
} from "../../src/nodes/core/validation";
import { NodeStore } from "../../src/nodes/store";
import type { Definition, GraphDocument } from "../../src/nodes/types";

const endpoint = (node: string, port: string) => ({ node, port });
function rejectsUnchanged(
  s: GraphSession,
  action: () => unknown,
  code?: string,
) {
  const before = s.getSnapshot();
  let failure: unknown;
  try {
    action();
  } catch (e) {
    failure = e;
  }
  expect(failure).toBeInstanceOf(GraphError);
  if (code) expect((failure as GraphError).code).toBe(code);
  expect(s.getSnapshot()).toBe(before);
}

test("Rust v1 exports preserve document data and semantic catalog changes only declared time validation metadata", () => {
  const input = readFileSync("tests/model/fixtures/node-rust-v1.json", "utf8");
  const previous = JSON.parse(input);
  const s = new GraphSession();
  s.load(input, s.getSnapshot());
  expect(JSON.parse(s.export().json)).toEqual(previous);
  expect(s.getSnapshot().dirty).toBe(false);
  const originalCatalog: Definition[] = JSON.parse(
    readFileSync("tests/model/fixtures/node-catalog-v1.json", "utf8"),
  );
  // 2026-10-20 time agreement adds validation metadata, not serialized fields.
  const expected = originalCatalog.map<Definition>((d) =>
    d.kind === "delay"
      ? {
          ...d,
          fields: d.fields.map((f) =>
            f.id === "seconds" ? { ...f, min: 0, unit: "seconds" } : f,
          ),
        }
      : d,
  );
  // Node dimensions are editor presentation, enlarged for readable titles and
  // editable values. Every semantic field/port and the saved document stay exact.
  const semantic = (d: Definition) =>
    Object.fromEntries(
      Object.entries(d).filter(([key]) => !["width", "height"].includes(key)),
    );
  expect(catalog.slice(0, originalCatalog.length).map(semantic)).toEqual(
    expected.map(semantic),
  );
  expect(new Set(catalog.map((d) => d.kind)).size).toBe(catalog.length);
  const framed = s.getSnapshot().doc.frames[0];
  s.edit({ type: "renameFrame", id: framed.id, title: "수정 후 재열기" });
  const reopened = new GraphSession(parseDocument(s.export().json));
  expect(reopened.getSnapshot().doc).toEqual(s.getSnapshot().doc);
});

test("multi-move is one history entry and positions roundtrip through undo, redo and files", () => {
  const s = new GraphSession(),
    before = s.getSnapshot().doc;
  const positions = before.nodes.slice(0, 2).map((n) => ({
    id: n.id,
    position: { x: n.position.x + 93.5, y: n.position.y - 18.25 },
  }));
  s.edit({ type: "move", positions });
  const after = s.getSnapshot();
  expect(after.undoCount).toBe(1);
  for (const p of positions)
    expect(after.doc.nodes.find((n) => n.id === p.id)!.position).toEqual(
      p.position,
    );
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
  s.travel(true);
  expect(s.getSnapshot().doc).toEqual(after.doc);
  const output = s.export();
  expect(parseDocument(output.json)).toEqual(after.doc);
  s.acknowledge(output.token);
  expect(s.getSnapshot().dirty).toBe(false);
  s.travel(false);
  expect(s.getSnapshot().dirty).toBe(true);
  s.travel(true);
  expect(s.getSnapshot().dirty).toBe(false);
});

test("later failure in a multi-edit preserves document, revision, dirty and redo", () => {
  const s = new GraphSession();
  s.edit({ type: "rename", name: "after" });
  s.travel(false);
  rejectsUnchanged(
    s,
    () =>
      s.edit({
        type: "move",
        positions: [
          { id: s.getSnapshot().doc.nodes[0].id, position: { x: 500, y: 500 } },
          { id: "missing", position: { x: 0, y: 0 } },
        ],
      }),
    "selection",
  );
  s.travel(true);
  expect(s.getSnapshot().doc.name).toBe("after");
});

test("unchanged edits preserve redo and saved state; a new edit clears redo", () => {
  const s = new GraphSession();
  s.acknowledge(s.export().token);
  s.edit({ type: "rename", name: "changed" });
  s.travel(false);
  const before = s.getSnapshot();
  s.edit({
    type: "move",
    positions: before.doc.nodes.map((n) => ({
      id: n.id,
      position: n.position,
    })),
  });
  expect(s.getSnapshot()).toBe(before);
  expect(before.dirty).toBe(false);
  s.edit({ type: "rename", name: "new branch" });
  expect(s.getSnapshot().canRedo).toBe(false);
});

test("input replacement and explicit reconnection are atomic and undo restores link IDs", () => {
  const s = new GraphSession(),
    original = s.getSnapshot().doc;
  const add = original.nodes.find((n) => n.kind === "add")!,
    multiply = original.nodes.find((n) => n.kind === "multiply")!;
  const old = original.edges.find(
    (e) => e.target.node === add.id && e.target.port === "a",
  )!;
  const number = original.nodes.find(
    (n) => n.kind === "number" && n.id !== old.source.node,
  )!;
  s.edit({
    type: "connect",
    source: endpoint(number.id, "value"),
    target: old.target,
  });
  expect(s.getSnapshot().undoCount).toBe(1);
  expect(s.getSnapshot().doc.edges).toHaveLength(3);
  expect(
    s
      .getSnapshot()
      .doc.edges.some(
        (e) =>
          e.source.node === number.id &&
          e.target.node === add.id &&
          e.target.port === "a",
      ),
  ).toBe(true);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(original);
  s.edit({
    type: "connect",
    source: old.source,
    target: endpoint(multiply.id, "b"),
    replaceEdge: old.id,
  });
  expect(s.getSnapshot().doc.edges.some((e) => e.id === old.id)).toBe(false);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(original);
});

test("incompatible reconnect and failed add-with-link leave no partial changes", () => {
  const s = new GraphSession();
  const [boolean] = s.edit({
    type: "add",
    kind: "boolean",
    position: { x: 0, y: 0 },
  });
  const doc = s.getSnapshot().doc,
    old = doc.edges[0],
    number = doc.nodes.find((n) => n.kind === "number")!;
  rejectsUnchanged(
    s,
    () =>
      s.edit({
        type: "connect",
        source: endpoint(boolean, "value"),
        target: old.target,
        replaceEdge: old.id,
      }),
    "port-type",
  );
  rejectsUnchanged(
    s,
    () =>
      s.edit({
        type: "add",
        kind: "action",
        position: { x: 0, y: 0 },
        link: { endpoint: endpoint(number.id, "value"), output: true },
      }),
    "port-type",
  );
  const [added] = s.edit({
    type: "add",
    kind: "add",
    position: { x: 900, y: 0 },
    link: { endpoint: endpoint(number.id, "value"), output: true },
  });
  expect(
    s
      .getSnapshot()
      .doc.edges.some(
        (e) => e.target.node === added && e.source.node === number.id,
      ),
  ).toBe(true);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(doc);
});

test("frame deletion restores members and links in one undo", () => {
  const s = new GraphSession();
  const [frame] = s.edit({
    type: "group",
    ids: s
      .getSnapshot()
      .doc.nodes.slice(0, 2)
      .map((n) => n.id),
    title: "묶음",
  });
  const before = s.getSnapshot();
  s.edit({ type: "delete", ids: [frame] });
  expect(s.getSnapshot().undoCount).toBe(before.undoCount + 1);
  expect(s.getSnapshot().doc.nodes).toHaveLength(2);
  expect(s.getSnapshot().doc.frames).toHaveLength(0);
  validateDocument(s.getSnapshot().doc);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before.doc);
});

test("copy/paste remaps frames and internal links, excludes external links and detaches caller data", () => {
  const s = new GraphSession(),
    doc = s.getSnapshot().doc,
    add = doc.nodes.find((n) => n.kind === "add")!;
  const number = doc.edges.find((e) => e.target.node === add.id)!.source.node;
  const [frame] = s.edit({
    type: "group",
    ids: [add.id, number],
    title: "수식",
  });
  const fragment = s.copy([frame]),
    before = s.getSnapshot().doc;
  expect(fragment.nodes).toHaveLength(2);
  expect(fragment.edges).toHaveLength(1);
  expect(fragment.frames).toHaveLength(1);
  const pasted = s.edit({
      type: "paste",
      fragment,
      position: { x: 1100, y: 400 },
    }),
    after = s.getSnapshot().doc;
  expect(after.nodes).toHaveLength(6);
  expect(after.frames).toHaveLength(2);
  const members = after.nodes.filter((n) => pasted.includes(n.id));
  expect(members[0].frame).toBe(members[1].frame);
  expect(members[0].frame).not.toBe(frame);
  const links = after.edges.filter(
    (e) => pasted.includes(e.source.node) || pasted.includes(e.target.node),
  );
  expect(links).toHaveLength(1);
  expect(pasted).toContain(links[0].target.node);
  fragment.nodes[0].title = "external mutation";
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
  s.travel(true);
  expect(s.getSnapshot().doc).toEqual(after);
  const partial = s.copy([add.id]);
  expect(partial.frames).toHaveLength(0);
  expect(partial.nodes[0].frame).toBeNull();
});

test("reroute, duplicate and ungroup retain independent undo boundaries", () => {
  const s = new GraphSession(),
    before = s.getSnapshot().doc;
  s.edit({
    type: "reroute",
    edgeId: before.edges[0].id,
    position: { x: 300, y: 30 },
  });
  expect(s.getSnapshot().doc.nodes).toHaveLength(5);
  expect(s.getSnapshot().doc.edges).toHaveLength(4);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(before);
  const [frame] = s.edit({
    type: "group",
    ids: before.nodes.map((n) => n.id),
    title: "전체",
  });
  s.edit({ type: "duplicate", ids: [frame] });
  expect(s.getSnapshot().doc.frames).toHaveLength(2);
  expect(s.getSnapshot().doc.nodes).toHaveLength(8);
  const grouped = s.getSnapshot().doc;
  s.edit({ type: "ungroup", ids: [frame] });
  expect(s.getSnapshot().doc.frames).toHaveLength(1);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(grouped);
});

test("missing inputs and cycles are saved as incomplete documents with diagnostics", () => {
  const nodes = [0, 300, 600].map((x) =>
    makeNode("reroute-number", { x, y: 0 }),
  );
  const s = new GraphSession({ ...createDocument(), nodes });
  expect(s.getSnapshot().diagnostics).toHaveLength(3);
  for (const [a, b] of [
    [0, 1],
    [1, 0],
    [1, 2],
  ])
    s.edit({
      type: "connect",
      source: endpoint(nodes[a].id, "out"),
      target: endpoint(nodes[b].id, "in"),
    });
  expect(s.getSnapshot().diagnostics.map((d) => d.code)).toEqual([
    "cycle-dependency",
    "cycle-dependency",
    "cycle-dependency",
  ]);
  expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
  s.travel(false);
  expect(
    s
      .getSnapshot()
      .diagnostics.some(
        (d) => d.code === "unconnected-input" && d.nodeId === nodes[2].id,
      ),
  ).toBe(true);
});

test("invalid values, nonfinite positions and duplicate position entries never commit", () => {
  const s = new GraphSession(),
    n = s.getSnapshot().doc.nodes.find((n) => n.kind === "number")!;
  rejectsUnchanged(
    s,
    () =>
      s.edit({
        type: "setNode",
        id: n.id,
        title: "partial",
        values: { value: "bad" },
      }),
    "field-type",
  );
  for (const x of [NaN, Infinity, 1_000_001])
    rejectsUnchanged(
      s,
      () =>
        s.edit({
          type: "move",
          positions: [{ id: n.id, position: { x, y: 0 } }],
        }),
      "position",
    );
  rejectsUnchanged(
    s,
    () =>
      s.edit({
        type: "move",
        positions: [
          { id: n.id, position: { x: 0, y: 0 } },
          { id: n.id, position: { x: 1, y: 1 } },
        ],
      }),
    "position",
  );
});

test("old and out-of-order save confirmations cannot clear new edits or revert saved state", () => {
  const s = new GraphSession(),
    old = s.export();
  s.edit({ type: "rename", name: "new" });
  s.acknowledge(old.token);
  expect(s.getSnapshot().dirty).toBe(true);
  const current = s.export();
  s.acknowledge(current.token);
  expect(s.getSnapshot().dirty).toBe(false);
  const before = s.getSnapshot();
  s.acknowledge(current.token);
  expect(s.getSnapshot()).toBe(before);
  rejectsUnchanged(s, () => s.acknowledge(old.token), "export");
  const a = s.export();
  s.edit({ type: "rename", name: "newest" });
  const b = s.export();
  s.acknowledge(b.token);
  rejectsUnchanged(s, () => s.acknowledge(a.token), "export");
  s.travel(false);
  expect(s.getSnapshot().dirty).toBe(true);
  s.travel(true);
  expect(s.getSnapshot().dirty).toBe(false);
});

test("replacement invalidates versions, exports and history; malformed files preserve all of them", () => {
  const s = new GraphSession(),
    old = s.getSnapshot(),
    exported = s.export();
  s.edit({ type: "rename", name: "edited" });
  rejectsUnchanged(s, () => s.load("{}", s.getSnapshot()), "parse");
  const incoming = createDocument();
  s.replace(incoming, true);
  expect(s.getSnapshot().dirty).toBe(false);
  expect(s.getSnapshot().canUndo).toBe(false);
  rejectsUnchanged(
    s,
    () => s.edit({ type: "rename", name: "stale" }, old),
    "stale",
  );
  rejectsUnchanged(s, () => s.acknowledge(exported.token), "export");
  expect(s.getSnapshot().doc).toEqual(incoming);
});

test("parser rejects unknown fields, references, duplicate IDs, formats, capacities and file limits", () => {
  const base = example("values");
  const changes: ((d: GraphDocument) => void)[] = [
    (d) => {
      d.version = 2;
    },
    (d) => {
      Object.assign(d, { unknown: true });
    },
    (d) => {
      Object.assign(d.nodes[0].position, { z: 0 });
    },
    (d) => {
      d.nodes[1].id = d.nodes[0].id;
    },
    (d) => {
      d.edges[0].source.port = "missing";
    },
    (d) => {
      d.edges[0].target.node = "missing";
    },
    (d) => {
      d.edges.push({ ...structuredClone(d.edges[0]), id: "extra" });
    },
    (d) => {
      d.nodes[0].position.x = 1_000_001;
    },
  ];
  for (const change of changes) {
    const doc = structuredClone(base);
    change(doc);
    expect(() => parseDocument(JSON.stringify(doc))).toThrow(GraphError);
  }
  expect(() => parseDocument("{")).toThrow(GraphError);
  expect(() => parseDocument(" ".repeat(5_000_001))).toThrow(GraphError);
  expect(() =>
    parseFragment(JSON.stringify({ ...base, format: "node-graph-fragment" })),
  ).toThrow(GraphError);
});

test("retained patch history is bounded and restores the retained edits", () => {
  const s = new GraphSession();
  for (let i = 0; i < 110; i++) s.edit({ type: "rename", name: `name ${i}` });
  expect(s.getSnapshot().undoCount).toBe(100);
  for (let i = 0; i < 100; i++) s.travel(false);
  expect(s.getSnapshot().doc.name).toBe("name 9");
  const before = s.getSnapshot();
  s.travel(false);
  expect(s.getSnapshot()).toBe(before);
  for (let i = 0; i < 100; i++) s.travel(true);
  expect(s.getSnapshot().doc.name).toBe("name 109");
});

test("committed state is immutable and independent sessions do not share document changes", () => {
  const input = example("flow"),
    s = new GraphSession(input),
    other = new GraphSession(input);
  input.nodes[0].title = "outside";
  expect(s.getSnapshot().doc.nodes.some((n) => n.title === "outside")).toBe(
    false,
  );
  expect(() => {
    s.getSnapshot().doc.nodes[0].position.x = 999;
  }).toThrow();
  s.edit({ type: "rename", name: "first" });
  expect(other.getSnapshot().doc.name).toBe("실행 연결 예제");
});

test("store publishes the committed state once, keeps errors recoverable and isolates observer failure", () => {
  const store = new NodeStore();
  const revisions: number[] = [];
  store.subscribe(() => revisions.push(store.getSnapshot().state.revision));
  store.edit({ type: "rename", name: "first" });
  expect(revisions).toEqual([1]);
  const before = store.getSnapshot().state;
  expect(store.load("{}", store.version())).toBeUndefined();
  expect(store.getSnapshot().state).toBe(before);
  expect(store.getSnapshot().error).toBe(true);
  store.edit({ type: "rename", name: "recovered" });
  expect(store.getSnapshot().error).toBe(false);
  const errors: unknown[] = [];
  const original = console.error;
  console.error = (...args: unknown[]) => {
    errors.push(args);
  };
  try {
    store.subscribe(() => {
      throw new Error("observer");
    });
    let observed = "";
    store.subscribe(() => {
      observed = store.getSnapshot().state.doc.name;
    });
    const result = store.edit({ type: "rename", name: "committed" });
    expect(result?.state.doc.name).toBe("committed");
    expect(observed).toBe("committed");
    expect(errors).toHaveLength(1);
  } finally {
    console.error = original;
  }
});
