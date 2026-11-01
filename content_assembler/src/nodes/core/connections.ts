import type { Endpoint, GraphDocument, GraphNode } from "../types";

// Resolve authored inputs through reroutes; never evaluate a runtime value.
export function inputEndpoints(doc: GraphDocument) {
  const nodes = new Map(doc.nodes.map((n) => [n.id, n]));
  const inputs = new Map(
    doc.edges.map((e) => [`${e.target.node}:${e.target.port}`, e.source]),
  );
  return (node: GraphNode, port: string): Endpoint | undefined => {
    let end = inputs.get(`${node.id}:${port}`);
    const visited = new Set<string>();
    while (end) {
      if (visited.has(end.node)) return;
      visited.add(end.node);
      const producer = nodes.get(end.node);
      if (!producer) return;
      if (!producer.kind.startsWith("reroute-")) return end;
      end = inputs.get(`${producer.id}:in`);
    }
  };
}

export function inputSources(doc: GraphDocument) {
  const nodes = new Map(doc.nodes.map((n) => [n.id, n]));
  const endpoint = inputEndpoints(doc);
  return (node: GraphNode, port: string) => {
    const end = endpoint(node, port);
    return end ? nodes.get(end.node) : undefined;
  };
}

export function stateConnections(doc: GraphDocument, stateId: string) {
  const source = inputSources(doc);
  const references = doc.nodes.filter(
    (n) => source(n, "state")?.id === stateId,
  );
  return {
    applications: references
      .filter((n) =>
        ["skill-overlay", "skill-begin", "skill-resonance-install"].includes(
          n.kind,
        ),
      )
      .map((node) => ({
        node,
        duration: source(node, "duration"),
        profile: source(node, "profile"),
      })),
    events: references.filter((n) =>
      [
        "skill-basic-event",
        "skill-reinput",
        "skill-lookup",
        "skill-expiry",
        "skill-owner-exit",
      ].includes(n.kind),
    ),
  };
}

// Preserve every authored edge for multi-input ports, resolving typed reroutes.
export function inputSourceLists(doc: GraphDocument) {
  const nodes = new Map(doc.nodes.map((n) => [n.id, n]));
  const source = inputSources(doc);
  return (node: GraphNode, port: string): GraphNode[] =>
    doc.edges
      .filter((e) => e.target.node === node.id && e.target.port === port)
      .map((e) => {
        const producer = nodes.get(e.source.node);
        return producer?.kind.startsWith("reroute-")
          ? source(producer, "in")
          : producer;
      })
      .filter((n): n is GraphNode => Boolean(n));
}
