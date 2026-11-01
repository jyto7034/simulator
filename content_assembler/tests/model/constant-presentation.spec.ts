import { test, expect } from "@playwright/test";
import { catalog } from "../../src/nodes/core/catalog";
import { example } from "../../src/nodes/core/examples";
import {
  privateConstants,
  includePrivateConstants,
} from "../../src/nodes/core/constant-presentation";
import { GraphSession } from "../../src/nodes/core/session";
import { inputSources } from "../../src/nodes/core/connections";

test("private constant projection preserves source identity and excludes shared or computed values", () => {
  const d = example("king-of-greed"),
    before = structuredClone(d),
    source = inputSources(d);
  const defs = new Map(catalog.map((n) => [n.kind, n]));
  const folded = privateConstants(d, defs);
  const volley = d.nodes.find(
    (n) => n.kind === "skill-modifier-continuous-volley",
  )!;
  expect(folded.has(source(volley, "costInterval")!.id)).toBe(true);
  const modified = d.nodes.find((n) => n.kind === "skill-modifier-duration")!;
  expect(folded.has(source(modified, "target")!.id)).toBe(false);
  expect(folded.has(source(volley, "target")!.id)).toBe(false);
  expect(d).toEqual(before);
  const constant = source(volley, "costInterval")!;
  d.edges.push({
    id: crypto.randomUUID(),
    source: { node: constant.id, port: "value" },
    target: { node: volley.id, port: "interval" },
  });
  expect(privateConstants(d, defs).has(constant.id)).toBe(false);
});

test("folded-node duplication includes private values and undo restores the whole graph", () => {
  const d = example("king-of-greed"),
    s = new GraphSession(d),
    source = inputSources(d);
  const volley = d.nodes.find(
    (n) => n.kind === "skill-modifier-continuous-volley",
  )!;
  const ids = includePrivateConstants(
    [volley.id],
    privateConstants(d, new Map(catalog.map((n) => [n.kind, n]))),
  );
  expect(ids).toContain(source(volley, "costInterval")!.id);
  expect(ids).toContain(source(volley, "interval")!.id);
  expect(ids).not.toContain(source(volley, "target")!.id);
  s.edit({ type: "duplicate", ids });
  const after = s.getSnapshot().doc;
  expect(after.nodes.length).toBe(d.nodes.length + 3);
  const copy = after.nodes.find(
    (n) => n.kind === volley.kind && n.id !== volley.id,
  )!;
  expect(inputSources(after)(copy, "costInterval")?.values.value).toBe(1);
  s.travel(false);
  expect(s.getSnapshot().doc).toEqual(d);
});
