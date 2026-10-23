import { registerHooks } from "node:module";
import { readFile, writeFile } from "node:fs/promises";

// Node 24's type stripping loads the framework-independent editor catalog.
registerHooks({
  resolve(specifier, context, nextResolve) {
    try {
      return nextResolve(specifier, context);
    } catch (error) {
      if (specifier.startsWith(".") && !/\.[a-z]+$/.test(specifier))
        return nextResolve(`${specifier}.ts`, context);
      throw error;
    }
  },
});
const { catalog } = await import("../src/nodes/core/catalog.ts");
const { resonanceFieldActive, resonanceInputActive, resonanceRequiredPorts } =
  await import("../src/nodes/core/resonance-catalog.ts");
const { spatialFieldActive } =
  await import("../src/nodes/core/spatial-catalog.ts");
const { groundFieldActive } =
  await import("../src/nodes/core/ground-catalog.ts");
const { p4FieldActive } = await import("../src/nodes/core/p4-fields.ts");
const active = (node, field) =>
  [
    resonanceFieldActive,
    spatialFieldActive,
    groundFieldActive,
    p4FieldActive,
  ].every((f) => f(node, field));
function conditions(def, predicate) {
  const defaults = Object.fromEntries(def.fields.map((f) => [f.id, f.default]));
  const controls = def.fields.filter(
    (f) => f.options || f.valueType === "boolean",
  );
  const assignments = controls.reduce(
    (rows, field) =>
      rows.flatMap((row) =>
        (field.options?.map((o) => o.value) ?? [false, true]).map((value) => ({
          ...row,
          [field.id]: value,
        })),
      ),
    [{}],
  );
  const reads = new Set();
  const enabled = assignments.filter((assignment) =>
    predicate({
      kind: def.kind,
      values: new Proxy(
        { ...defaults, ...assignment },
        {
          get(target, key) {
            reads.add(key);
            return target[key];
          },
        },
      ),
    }),
  );
  if (enabled.length === assignments.length) return undefined;
  return [
    ...new Map(
      enabled.map((row) => {
        const condition = Object.fromEntries(
          controls.filter((c) => reads.has(c.id)).map((c) => [c.id, row[c.id]]),
        );
        return [JSON.stringify(condition), condition];
      }),
    ).values(),
  ];
}
function fields(def) {
  return def.fields.map((field) => {
    const { id, valueType, min, max, integer, unit } = field;
    return {
      id,
      valueType,
      min,
      max,
      integer,
      unit,
      options: field.options?.map((o) => o.value),
      activeWhen: conditions(def, (node) => active(node, field)),
    };
  });
}
const contract = {
  schemaVersion: 1,
  semanticVersion: "2026-10-22.2",
  definitions: catalog.map((def) => ({
    kind: def.kind,
    inputs: def.inputs.map(({ id, valueType, multiple, required }) => ({
      id,
      valueType,
      multiple,
      required,
      activeWhen: conditions(def, (node) => resonanceInputActive(node, id)),
      requiredWhen: conditions(
        def,
        (node) => required || resonanceRequiredPorts(node).includes(id),
      ),
    })),
    outputs: def.outputs.map(({ id, valueType, multiple, required }) => ({
      id,
      valueType,
      multiple,
      required,
    })),
    fields: fields(def),
  })),
};
const path = new URL(
  "../../core_enhanced/src/authoring/node_graph/catalog.json",
  import.meta.url,
);
const text = JSON.stringify(contract, null, 2) + "\n";
if (process.argv.includes("--check")) {
  if ((await readFile(path, "utf8")) !== text)
    throw new Error(
      "Core node contract differs from the editor catalog; regenerate it.",
    );
} else await writeFile(path, text);
