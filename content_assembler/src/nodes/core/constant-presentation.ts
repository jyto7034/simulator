import type { Definition, GraphDocument, GraphEdge } from "../types";

// Fold only private scalar constants whose value is already editable on the
// receiving port. Shared values, modifier targets, reroutes and computations
// remain visible. The map is a view projection, never a document rewrite.
export function privateConstants(
  doc: GraphDocument,
  definitions: ReadonlyMap<string, Definition>,
) {
  const uses = new Map<string, GraphEdge[]>();
  const incoming = new Set(doc.edges.map((e) => e.target.node));
  const nodes = new Map(doc.nodes.map((n) => [n.id, n]));
  for (const e of doc.edges) {
    const list = uses.get(e.source.node) ?? [];
    list.push(e);
    uses.set(e.source.node, list);
  }
  const result = new Map<string, GraphEdge>();
  for (const node of doc.nodes) {
    const def = definitions.get(node.kind),
      edges = uses.get(node.id);
    if (
      !def ||
      def.inputs.length ||
      def.outputs.length !== 1 ||
      def.fields.length !== 1 ||
      def.fields[0].id !== "value" ||
      def.fields[0].valueType === "tile-pattern" ||
      incoming.has(node.id) ||
      edges?.length !== 1
    )
      continue;
    const edge = edges[0],
      consumer = nodes.get(edge.target.node);
    const port =
      consumer &&
      definitions
        .get(consumer.kind)
        ?.inputs.find((p) => p.id === edge.target.port);
    if (
      !port ||
      port.multiple ||
      port.valueType === "flow" ||
      consumer?.kind.startsWith("reroute-") ||
      (consumer?.kind.startsWith("skill-modifier-") && port.id === "target")
    )
      continue;
    result.set(node.id, edge);
  }
  return result;
}

export function includePrivateConstants(
  ids: string[],
  constants: ReadonlyMap<string, GraphEdge>,
) {
  const expanded = new Set(ids);
  for (const [id, edge] of constants)
    if (expanded.has(edge.target.node)) expanded.add(id);
  return [...expanded];
}
