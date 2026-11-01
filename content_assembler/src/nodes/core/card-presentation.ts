import type { Definition, GraphNode } from "../types";
import { resonanceFieldActive } from "./resonance-catalog";
import { spatialFieldActive } from "./spatial-catalog";
import { groundFieldActive } from "./ground-catalog";
import { p4FieldActive } from "./p4-fields";

// Presentation only: connected and required inputs always remain accessible.
// Expansion never edits the authored definition, document or undo history.
export function cardPresentation(
  node: GraphNode,
  definition: Definition,
  connected: ReadonlySet<string>,
  expanded: boolean,
) {
  const optional = definition.inputs.filter(
    (port) => !port.required && !connected.has(port.id),
  );
  const inputs = expanded
    ? definition.inputs
    : definition.inputs.filter((port) => !optional.includes(port));
  const fields = definition.fields.filter(
    (field) =>
      resonanceFieldActive(node, field) &&
      spatialFieldActive(node, field) &&
      groundFieldActive(node, field) &&
      p4FieldActive(node, field),
  );
  return {
    optionalCount: optional.length,
    definition: {
      ...definition,
      inputs,
      fields,
      height:
        62 +
        Math.max(inputs.length, definition.outputs.length) * 44 +
        fields.length * 36 +
        (definition.role ? 24 : 0) +
        (optional.length ? 32 : 0),
    },
  };
}
