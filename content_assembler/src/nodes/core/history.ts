import type { GraphDocument } from "../types";
import { byteLength, canonicalize } from "./model";

type Changes<T> = { put: T[]; remove: string[] };
type Patch = {
  nodes: Changes<GraphDocument["nodes"][number]>;
  edges: Changes<GraphDocument["edges"][number]>;
  frames: Changes<GraphDocument["frames"][number]>;
  name?: string;
};
export type Entry = { forward: Patch; backward: Patch; bytes: number };
function difference<T extends { id: string }>(a: T[], b: T[]): Changes<T> {
  const before = new Map(a.map((n) => [n.id, n])),
    after = new Set(b.map((n) => n.id));
  return {
    put: b.filter(
      (n) => JSON.stringify(before.get(n.id)) !== JSON.stringify(n),
    ),
    remove: a.filter((n) => !after.has(n.id)).map((n) => n.id),
  };
}
function between(a: GraphDocument, b: GraphDocument): Patch {
  return {
    nodes: difference(a.nodes, b.nodes),
    edges: difference(a.edges, b.edges),
    frames: difference(a.frames, b.frames),
    ...(a.name === b.name ? {} : { name: b.name }),
  };
}
export function entry(before: GraphDocument, after: GraphDocument): Entry {
  const forward = between(before, after),
    backward = between(after, before);
  return {
    forward,
    backward,
    bytes:
      byteLength(JSON.stringify(forward)) +
      byteLength(JSON.stringify(backward)),
  };
}
function applyChanges<T extends { id: string }>(
  items: T[],
  changes: Changes<T>,
): T[] {
  const replaced = new Set([
    ...changes.remove,
    ...changes.put.map((n) => n.id),
  ]);
  return [
    ...items.filter((n) => !replaced.has(n.id)),
    ...structuredClone(changes.put),
  ];
}
export function applyPatch(doc: GraphDocument, patch: Patch) {
  doc.nodes = applyChanges(doc.nodes, patch.nodes);
  doc.edges = applyChanges(doc.edges, patch.edges);
  doc.frames = applyChanges(doc.frames, patch.frames);
  if (patch.name !== undefined) doc.name = patch.name;
  canonicalize(doc);
}
