import type { GraphDocument, GraphNode } from "../types";

// Conservative dominance checks for authored flow, including reroutes and cycles.
export function flowPaths(doc: GraphDocument) {
  const nodes = new Map(doc.nodes.map((n) => [n.id, n]));
  const flowInputs = new Map<string, { node: string; port: string }[]>();
  const incomingFlow = new Map<string, string[]>();
  // The flow input convention is shared by the existing execution nodes and these nodes.
  for (const edge of doc.edges) {
    if (edge.target.port !== "in") continue;
    const kind = nodes.get(edge.target.node)?.kind ?? "";
    if (kind.startsWith("reroute-") && kind !== "reroute-flow") continue;
    const previous = incomingFlow.get(edge.target.node) ?? [];
    previous.push(edge.source.node);
    incomingFlow.set(edge.target.node, previous);
    const endpoints = flowInputs.get(edge.target.node) ?? [];
    endpoints.push(edge.source);
    flowInputs.set(edge.target.node, endpoints);
  }
  // Every incoming path must pass this exact output. Merely being an ancestor
  // would accept the opposite branch, a bypass, or a cyclic self-justification.
  const mustPass = (node: GraphNode, producer: GraphNode, port: string) => {
    const memo = new Map<string, boolean>();
    const visit = (id: string, path: Set<string>): boolean => {
      if (path.has(id)) return false;
      if (memo.has(id)) return memo.get(id)!;
      const nextPath = new Set(path).add(id);
      const edges = flowInputs.get(id) ?? [];
      const valid =
        edges.length > 0 &&
        edges.every((edge) =>
          edge.node === producer.id && edge.port === port
            ? true
            : edge.node === producer.id
              ? false
              : visit(edge.node, nextPath),
        );
      memo.set(id, valid);
      return valid;
    };
    return visit(node.id, new Set());
  };
  const ancestors = (node: GraphNode) => {
    const seen = new Set<string>();
    const pending = [...(incomingFlow.get(node.id) ?? [])];
    while (pending.length) {
      const id = pending.pop()!;
      if (seen.has(id)) continue;
      seen.add(id);
      pending.push(...(incomingFlow.get(id) ?? []));
    }
    return seen;
  };
  const mayPass = (node: GraphNode, producer: GraphNode, port: string) => {
    const seen = new Set<string>();
    const visit = (id: string): boolean => {
      if (seen.has(id)) return false;
      seen.add(id);
      return (flowInputs.get(id) ?? []).some((edge) =>
        edge.node === producer.id ? edge.port === port : visit(edge.node),
      );
    };
    return visit(node.id);
  };
  return { mustPass, mayPass, ancestors };
}
