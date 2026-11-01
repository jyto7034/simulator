import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import {
  parseSource,
  scalarPatch,
  modelLocation,
} from "../../src/composition/source";

const fixtures = new URL(
  "../../../core_enhanced/tests/fixtures/skill_composition/",
  import.meta.url,
);
const source = readFileSync(
  new URL("woodsman_monk.skill.yaml", fixtures),
  "utf8",
);

test("equipment attachments expose the real equipment identity and shared program reference", async () => {
  const { projection } = await import("../../src/composition/view");
  const input = readFileSync(
    new URL("equipment_attack.skill.yaml", fixtures),
    "utf8",
  );
  const parsed = parseSource(input);
  if (!parsed.ok) throw new Error("equipment fixture failed");
  const view = projection(parsed.value);
  const binding = view.definitions.find((d) => d.group === "장비 공격")!;
  expect(binding.name).toBe("r11_weapon");
  expect(view.references).toContainEqual({
    from: binding.id,
    to: "values:0",
    path: ["equipment_attacks", 0, "program"],
    kind: "Shared",
  });
  const path = ["equipment_attacks", 0, "program", "Shared"];
  const changed = scalarPatch(input, path, "another-program");
  expect(changed).toBe(
    input.replace("Shared: mass-growth", 'Shared: "another-program"'),
  );
  const location = modelLocation(parsed, "equipment_attacks[0].equipment")!;
  expect(input.slice(location.start, location.end)).toBe("r11_weapon");
});

test("cohort bounty links both owned resources and edits the actual threshold scalar", async () => {
  const { projection } = await import("../../src/composition/view");
  const input = readFileSync(
    new URL("cohort_bounty.skill.yaml", fixtures),
    "utf8",
  );
  const parsed = parseSource(input);
  if (!parsed.ok) throw new Error("bounty fixture failed");
  const view = projection(parsed.value);
  for (const [field, index] of [
    ["pending", 0],
    ["confirmed", 1],
  ] as const) {
    expect(view.references).toContainEqual({
      from: "values:0",
      to: `resources:${index}`,
      kind: "Resource",
      path: ["values", 0, "value", "AttackProgram", "cohort", "bounty", field],
    });
  }
  const path = [
    "values",
    0,
    "value",
    "AttackProgram",
    "cohort",
    "bounty",
    "tiers",
    0,
    "threshold",
  ];
  const changed = scalarPatch(input, path, 3);
  expect(changed).toBe(input.replace("threshold: 2", "threshold: 3"));
  const location = modelLocation(
    parsed,
    "values[0].value.AttackProgram.cohort.bounty.tiers[0].threshold",
  )!;
  expect(input.slice(location.start, location.end)).toBe("2");
});

test("YAML meaning and rejection match the same corpus consumed by Core", () => {
  const cases: { source: string; accepted: boolean; value?: unknown }[] =
    JSON.parse(readFileSync(new URL("yaml_cases.json", fixtures), "utf8"));
  for (const item of cases) {
    const parsed = parseSource(item.source);
    expect(parsed.ok, item.source).toBe(item.accepted);
    if (parsed.ok) expect(parsed.value, item.source).toEqual(item.value);
  }
  expect(parseSource("[".repeat(2048) + "1" + "]".repeat(2048)).ok).toBe(false);
});

test("Korean source, scalar edits, and enum-transparent diagnostic locations preserve the original", () => {
  const parsed = parseSource(source);
  expect(parsed.ok).toBe(true);
  if (!parsed.ok) throw new Error("fixture failed");
  expect(parsed.value).toEqual(
    JSON.parse(readFileSync(new URL("woodsman_monk.json", fixtures), "utf8")),
  );
  const path = ["skills", 1, "prepare_ms"];
  const span = parsed.spans[JSON.stringify(path)]!;
  const changed = scalarPatch(source, path, 150);
  expect(changed.slice(0, span.start)).toBe(source.slice(0, span.start));
  expect(changed.slice(span.start + 3)).toBe(source.slice(span.end));
  expect(source.slice(span.start, span.end)).toBe("100");
  const location = modelLocation(parsed, "skills[1].body.arguments[0].value");
  expect(source.slice(location!.start, location!.end)).toContain(
    "Shared: burst-power",
  );
  const definition = modelLocation(parsed, "definitions[0].body.effects[0]");
  expect(source.slice(definition!.start, definition!.end)).toContain("Damage:");
});

test("direct target-state diagnostics select the actual literal definition", () => {
  const state = { duration_ms: 0, profile: { definition: "resolve" } };
  const literal = {
    Literal: {
      Effects: [
        {
          id: "sustain",
          op: { Sustained: { definition: { TargetState: state } } },
        },
      ],
    },
  };
  for (const [body, prefix] of [
    [
      { Call: { arguments: [{ name: "effects", value: literal }] } },
      "arguments[0].value",
    ],
    [
      { Action: { effects: [{ Append: { effects: literal } }] } },
      "effects[0].effects",
    ],
  ] as const) {
    const input = JSON.stringify({ skills: [{ body }] });
    const parsed = parseSource(input);
    const span = modelLocation(
      parsed,
      `skills[0].body.${prefix}.Literal.Effects[0].op.Sustained.definition.TargetState`,
    );
    expect(span).not.toBeNull();
    expect(input.slice(span!.start, span!.end)).toBe(JSON.stringify(state));
  }
});

test("resource declarations, operation and expression links share editable source locations", async () => {
  const { projection } = await import("../../src/composition/view");
  const input = readFileSync(new URL("resources.skill.yaml", fixtures), "utf8");
  const parsed = parseSource(input);
  expect(parsed.ok).toBe(true);
  if (!parsed.ok) throw new Error("resource fixture failed");
  const view = projection(parsed.value);
  expect(view.definitions.filter((item) => item.group === "자원")).toHaveLength(
    3,
  );
  expect(
    view.references.filter((item) => item.kind === "Resource"),
  ).toHaveLength(6);
  const initial = modelLocation(parsed, "resources[0].initial")!;
  expect(input.slice(initial.start, initial.end)).toBe("7");
  const reference = modelLocation(
    parsed,
    "values[0].value.Effects[0].op.Resources.operations[1].resource",
  )!;
  expect(input.slice(reference.start, reference.end)).toBe("copy_meter");
  expect(scalarPatch(input, ["resources", 2, "cap"], 12)).toBe(
    input.replace("initial: 0, cap: 15", "initial: 0, cap: 12"),
  );
});

test("action duration references and diagnostics select the same editable source", async () => {
  const { projection } = await import("../../src/composition/view");
  const input = readFileSync(
    new URL("duration_modifiers.skill.yaml", fixtures),
    "utf8",
  );
  const parsed = parseSource(input);
  if (!parsed.ok) throw new Error("duration fixture failed");
  const view = projection(parsed.value);
  expect(view.references).toContainEqual({
    from: "skills:0",
    to: "values:1",
    path: ["skills", 0, "body", "Call", "arguments", 1, "value"],
    kind: "Shared",
  });
  const effect = modelLocation(
    parsed,
    "definitions[0].body.modifiers[0].change.Duration.effect",
  )!;
  expect(input.slice(effect.start, effect.end)).toBe("support");
  const percent = modelLocation(parsed, "values[1].value.Count")!;
  expect(input.slice(percent.start, percent.end)).toBe("25");
  expect(scalarPatch(input, ["values", 1, "value", "Count"], 50)).toBe(
    input.replace("Count: 25", "Count: 50"),
  );
});

test("action support radius references and diagnostics select the same editable source", async () => {
  const { projection } = await import("../../src/composition/view");
  const input = readFileSync(
    new URL("support_radius.skill.yaml", fixtures),
    "utf8",
  );
  const parsed = parseSource(input);
  if (!parsed.ok) throw new Error("radius fixture failed");
  const view = projection(parsed.value);
  expect(view.references).toContainEqual({
    from: "skills:0",
    to: "values:1",
    path: ["skills", 0, "body", "Call", "arguments", 1, "value"],
    kind: "Shared",
  });
  const effect = modelLocation(
    parsed,
    "definitions[0].body.modifiers[0].change.SupportRadius.effect",
  )!;
  expect(input.slice(effect.start, effect.end)).toBe("support");
  const percent = modelLocation(parsed, "values[1].value.Count")!;
  expect(input.slice(percent.start, percent.end)).toBe("25");
  expect(scalarPatch(input, ["values", 1, "value", "Count"], 50)).toBe(
    input.replace("Count: 25", "Count: 50"),
  );
});

test("finite control shared inputs preserve exact scalar locations and edits", async () => {
  const { projection } = await import("../../src/composition/view");
  const input = readFileSync(new URL("control.skill.yaml", fixtures), "utf8");
  const parsed = parseSource(input);
  if (!parsed.ok) throw new Error("control fixture failed");
  expect(projection(parsed.value).references).toContainEqual({
    from: "skills:0",
    to: "values:0",
    path: ["skills", 0, "body", "Call", "arguments", 0, "value"],
    kind: "Shared",
  });
  const path = [
    "values",
    0,
    "value",
    "Effects",
    0,
    "op",
    "Sustained",
    "definition",
    "Control",
    "duration_ms",
  ];
  const span = modelLocation(
    parsed,
    "values[0].value.Effects[0].op.Sustained.definition.Control.duration_ms",
  )!;
  expect(input.slice(span.start, span.end)).toBe("1501");
  expect(scalarPatch(input, path, 2001)).toBe(
    input.replace("duration_ms: 1501", "duration_ms: 2001"),
  );
});

test("victory forms expose both resources and preserve stage-local edits", async () => {
  const { projection } = await import("../../src/composition/view");
  const input = readFileSync(
    new URL("equipment_forms.skill.yaml", fixtures),
    "utf8",
  );
  const parsed = parseSource(input);
  if (!parsed.ok) throw new Error("forms fixture failed");
  const references = projection(parsed.value).references;
  for (const [field, index] of [
    ["pending", 0],
    ["confirmed", 1],
  ] as const) {
    expect(references).toContainEqual({
      from: "values:0",
      to: `resources:${index}`,
      path: ["values", 0, "value", "AttackProgram", "victory_forms", field],
      kind: "Resource",
    });
  }
  const path = [
    "values",
    0,
    "value",
    "AttackProgram",
    "victory_forms",
    "forms",
    1,
    "wins_required",
  ];
  const span = modelLocation(
    parsed,
    "values[0].value.AttackProgram.victory_forms.forms[1].wins_required",
  )!;
  expect(input.slice(span.start, span.end)).toBe("2");
  expect(scalarPatch(input, path, 3)).toBe(
    input.replace("wins_required: 2", "wins_required: 3"),
  );
});

test("death burst authored coefficient preserves source and shared reference", () => {
  const input = readFileSync(
    new URL("equipment_death_burst.skill.yaml", fixtures),
    "utf8",
  );
  const parsed = parseSource(input);
  if (!parsed.ok) throw new Error("burst fixture failed");
  const path = [
    "values",
    0,
    "value",
    "AttackProgram",
    "death_burst",
    "attack_percent",
  ];
  const span = modelLocation(
    parsed,
    "values[0].value.AttackProgram.death_burst.attack_percent",
  )!;
  expect(input.slice(span.start, span.end)).toBe("100");
  expect(scalarPatch(input, path, 150)).toBe(
    input.replace("attack_percent: 100", "attack_percent: 150"),
  );
});

test("last will coefficient edits preserve armor binding and exact source span", () => {
  const input = readFileSync(
    new URL("equipment_last_will.skill.yaml", fixtures),
    "utf8",
  );
  const parsed = parseSource(input);
  if (!parsed.ok) throw new Error("last will fixture failed");
  const path = [
    "values",
    0,
    "value",
    "DefenseProgram",
    "LastWill",
    "attack_percent",
  ];
  const span = modelLocation(
    parsed,
    "values[0].value.DefenseProgram.LastWill.attack_percent",
  )!;
  expect(input.slice(span.start, span.end)).toBe("300");
  expect(scalarPatch(input, path, 450)).toBe(
    input.replace("attack_percent: 300", "attack_percent: 450"),
  );
});

test("weapon stage edits keep the base shot and locate the upgraded scalar", () => {
  const input = readFileSync(
    new URL("weapon_stages.skill.yaml", fixtures),
    "utf8",
  );
  const parsed = parseSource(input);
  if (!parsed.ok) throw new Error("weapon stage fixture failed");
  const path = [
    "values",
    0,
    "value",
    "AttackProgram",
    "weapon_stages",
    "upgraded",
    "push_distance_units",
  ];
  const location = modelLocation(
    parsed,
    "values[0].value.AttackProgram.weapon_stages.upgraded.push_distance_units",
  )!;
  expect(input.slice(location.start, location.end)).toBe("100000");
  expect(scalarPatch(input, path, 200000)).toBe(
    input.replace("push_distance_units: 100000", "push_distance_units: 200000"),
  );
});

test("named predicates expose shared calls, typed inputs and exact editable source", async () => {
  const { projection } = await import("../../src/composition/view");
  const { CompositionSession } = await import("../../src/composition/session");
  const original = `version: 2
predicates:
  - name: active-window
    context: State
    parameters:
      - { name: cutoff, value_type: { Amount: Milliseconds } }
    body:
      Call:
        name: compare-time
        arguments:
          - { name: cutoff, value: { Parameter: cutoff } }
  - name: compare-time
    context: State
    parameters: []
    body: { Bool: { field: { Literal: { BoolField: armed } }, equals: true } }
values:
  - name: armor
    value:
      StateProgram:
        handlers:
          - body:
              - If:
                  condition:
                    Predicate:
                      name: active-window
                      arguments:
                        - name: cutoff
                          value: { Literal: { Amount: { moment: Live, root: { Const: { value: 5000, unit: Milliseconds } } } } }
`;
  const parsed = parseSource(original);
  if (!parsed.ok) throw new Error("predicate fixture syntax");
  const view = projection(parsed.value);
  expect(
    view.definitions.filter((d) => d.group === "공통 조건").map((d) => d.name),
  ).toEqual(["active-window", "compare-time"]);
  expect(view.references).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        from: "values:0",
        to: "predicates:0",
        kind: "Predicate",
      }),
      expect.objectContaining({
        from: "predicates:0",
        to: "predicates:1",
        kind: "Predicate",
      }),
    ]),
  );
  const path = [
    "values",
    0,
    "value",
    "StateProgram",
    "handlers",
    0,
    "body",
    0,
    "If",
    "condition",
    "Predicate",
    "arguments",
    0,
    "value",
    "Literal",
    "Amount",
    "root",
    "Const",
    "value",
  ];
  const session = new CompositionSession(original);
  session.scalar(path, 7000, session.getSnapshot());
  expect(session.getSnapshot().source).toBe(
    original.replace("value: 5000", "value: 7000"),
  );
  session.travel(false);
  expect(session.getSnapshot().source).toBe(original);
  const location = modelLocation(
    parsed,
    "predicates[0].body.Call.arguments[0].value",
  )!;
  expect(original.slice(location.start, location.end)).toBe(
    "{ Parameter: cutoff }",
  );
});

test("kill cast state keeps chance and cast fields editable with exact source locations", async () => {
  const { CompositionSession } = await import("../../src/composition/session");
  const { projection } = await import("../../src/composition/view");
  const original = readFileSync(
    new URL("kill_cast_state.skill.yaml", fixtures),
    "utf8",
  );
  const session = new CompositionSession(original);
  const base = ["values", 1, "value", "StateProgram", "handlers"];
  const probability = [
    ...base,
    0,
    "body",
    0,
    "Chance",
    "percent",
    "root",
    "Const",
    "value",
  ];
  const damage = [
    ...base,
    1,
    "body",
    0,
    "If",
    "then_ops",
    1,
    "AddCastDamage",
    "percent",
    "root",
    "Const",
    "value",
  ];
  session.scalar(probability, 47, session.getSnapshot());
  session.scalar(damage, 40, session.getSnapshot());
  const changed = original
    .replace("value: 100, unit: Scalar", "value: 47, unit: Scalar")
    .replace("value: 20, unit: Scalar", "value: 40, unit: Scalar");
  expect(session.getSnapshot().source).toBe(changed);
  session.travel(false);
  session.travel(false);
  expect(session.getSnapshot().source).toBe(original);
  session.travel(true);
  session.travel(true);
  const parsed = session.getSnapshot().parsed;
  if (!parsed.ok) throw new Error("valid changed state expected");
  expect(projection(parsed.value).references).toContainEqual({
    from: "equipment_states:0",
    to: "values:1",
    path: ["equipment_states", 0, "program"],
    kind: "Shared",
  });
  const location = modelLocation(
    parsed,
    "values[1].value.StateProgram.handlers[0].body[0].Chance.then_ops[0].Set.field",
  )!;
  expect(changed.slice(location.start, location.end)).toBe("hearts");
  const save = session.beginSave("빌린-심장.skill.yaml");
  session.finishSave(save, "saved-source");
  const reopened = new CompositionSession(save.source);
  expect(reopened.getSnapshot().source).toBe(changed);
  expect(reopened.getSnapshot().parsed.ok).toBe(true);
});
