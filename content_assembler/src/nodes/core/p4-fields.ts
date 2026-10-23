import type { Field, FieldValue } from "../types";

// The inspector, canvas and diagnostics share conditional authoring fields.
export function p4FieldActive(
  node: { kind: string; values: Record<string, FieldValue> },
  field: Pick<Field, "id">,
) {
  if (node.kind === "skill-time-power" && field.id === "cap")
    return node.values.curve === "capped-linear";
  if (node.kind === "skill-modifier-single-support" && field.id === "factor")
    return node.values.mode === "shield";
  if (node.kind === "skill-modifier-population" && field.id === "percent")
    return node.values.mode === "growth";
  if (node.kind === "skill-reactive-status" && field.id === "percent")
    return node.values.effect === "slow";
  if (node.kind === "skill-mark-weapon-hit" && field.id === "weapon")
    return node.values.binding !== "fragment";
  if (node.kind === "skill-stock" && field.id === "capacity")
    return node.values.limit === "bounded";
  if (node.kind === "skill-repeat-plan" && field.id === "first")
    return node.values.timing === "interval";
  if (node.kind === "skill-hit-status" && field.id === "percent")
    return node.values.effect === "vulnerability";
  if (node.kind === "skill-projectile-profile") {
    if (field.id === "pierce") return node.values.delivery === "directional";
    if (field.id === "hits")
      return (
        node.values.delivery === "directional" && node.values.pierce === "count"
      );
  }
  return true;
}
