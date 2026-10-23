import type { GraphDocument } from "../types";

export const MAX_FILE_BYTES = 5_000_000;
export const byteLength = (text: string) =>
  new TextEncoder().encode(text).byteLength;
export const id = () => crypto.randomUUID();

export class GraphError extends Error {
  constructor(
    readonly code: string,
    message: string,
  ) {
    super(message);
  }
}

export function createDocument(): GraphDocument {
  return {
    format: "node-graph",
    version: 1,
    id: id(),
    name: "새 노드 그래프",
    nodes: [],
    edges: [],
    frames: [],
  };
}

export function canonicalize(doc: GraphDocument) {
  const byId = (a: { id: string }, b: { id: string }) =>
    a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
  doc.nodes.sort(byId);
  doc.edges.sort(byId);
  doc.frames.sort(byId);
  for (const node of doc.nodes)
    node.values = Object.fromEntries(
      Object.entries(node.values).sort(([a], [b]) =>
        a < b ? -1 : a > b ? 1 : 0,
      ),
    );
}

// Committed documents and snapshots are immutable; commands work on detached drafts.
export function freeze<T>(value: T): T {
  if (value && typeof value === "object" && !Object.isFrozen(value)) {
    Object.values(value).forEach(freeze);
    Object.freeze(value);
  }
  return value;
}
