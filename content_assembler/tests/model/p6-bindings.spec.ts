import { test, expect } from "@playwright/test";
import { example } from "../../src/nodes/core/examples";
import { inputSources } from "../../src/nodes/core/connections";
import type { GraphDocument } from "../../src/nodes/types";
function targets(doc: GraphDocument, relic: string) {
  const source = inputSources(doc);
  return doc.nodes
    .filter((n) => source(n, "condition")?.values.definition === relic)
    .map((n) => ({
      modifier: n,
      target: source(n, "target") || source(n, "reactive") || source(n, "hit"),
      source,
    }));
}
test("long breath only changes the adopted durations, never independent vulnerability or release delay", () => {
  const snow = targets(example("snow-queen"), "long-breath");
  expect(snow.map((t) => t.target?.values.value).sort()).toEqual([2, 3]);
  const gifts = targets(example("laetitia"), "long-breath");
  expect(gifts.map((t) => t.target?.values.value)).toEqual([10]);
  const wood = targets(example("warm-hearted-woodsman"), "long-breath");
  expect(wood.map((t) => t.target?.title)).toEqual(["기절 지속 시간 · 미정"]);
  for (const row of [...snow, ...gifts, ...wood])
    expect(row.modifier.values.percent).toBe(25);
});
test("explosion modifiers do not amplify shared infection power or the direct T attack", () => {
  expect(
    targets(example("melting-love"), "explosion-vulnerability").map(
      (t) => t.target?.kind,
    ),
  ).toEqual(["skill-explosion-rule"]);
  const snow = targets(example("snow-queen"), "explosion-vulnerability");
  expect(snow.map((t) => t.target?.kind)).toEqual(["skill-mobile-effect"]);
  expect(snow[0].target?.values.motion).toBe("chase-delay");
  expect(targets(example("laetitia"), "explosion-vulnerability")).toHaveLength(
    3,
  );
});
test("imprint modifies the weapon's new grant only and summon vulnerability is not silently adopted by funeral", () => {
  const bloom = example("funeral-of-dying-butterflies-bloom");
  const grant = targets(bloom, "deep-imprint");
  expect(grant).toHaveLength(2);
  expect(grant[0].target?.kind).toBe("skill-mark-weapon-hit");
  expect(grant[0].modifier.values.factor).toBe(2);
  expect(targets(bloom, "summon-vulnerability")).toEqual([]);
  expect(
    targets(example("funeral-of-dying-butterflies"), "deep-imprint"),
  ).toHaveLength(2);
});

test("knight growth, new wounds and emergency shield remain distinct binding sites", () => {
  const d = example("knight-of-despair-bloom");
  expect(targets(d, "double-harvest").map((t) => t.target?.kind)).toEqual([
    "skill-relation-periodic",
  ]);
  expect(targets(d, "deep-imprint").map((t) => t.target?.kind)).toEqual([
    "skill-hit-status",
  ]);
  expect(targets(d, "layered-shell").map((t) => t.target?.kind)).toEqual([
    "skill-ally-relation",
  ]);
  expect(targets(d, "deep-wedge")).toHaveLength(1);
  expect(targets(d, "next")).toHaveLength(0);
  expect(targets(example("knight-of-despair"), "next")).toHaveLength(2);
});
test("recovery and wolf harvest do not bind the ledger or periodic normal ammunition", () => {
  const greed = targets(example("king-of-greed"), "double-harvest");
  expect(greed.map((t) => t.target?.kind)).toEqual(["skill-damage-recovery"]);
  const red = targets(example("little-red"), "double-harvest");
  expect(red.map((t) => t.target?.kind)).toEqual(["skill-cohort-killed"]);
  expect(
    targets(example("mountain-of-smiling-bodies"), "broken-balance"),
  ).toEqual([]);
});
test("all authored relic conditions select the normal variant and unfinished healing buffs stay explicit", () => {
  const d = example("queen-of-hatred-bloom");
  for (const n of d.nodes.filter((n) => n.kind === "skill-equipment-condition"))
    expect(n.values.variant).toBe("normal");
  const modifiers = targets(d, "lingering-warmth");
  expect(modifiers).toHaveLength(3);
  const states = modifiers.map((t) => t.source(t.modifier, "state")!);
  expect(new Set(states.map((n) => n.id)).size).toBe(1);
  const state = states[0],
    src = modifiers[0].source;
  expect(src(state, "duration")?.values.value).toBe("");
  expect(src(state, "effects")?.values.attribute).toBe("unresolved");
  expect(targets(d, "layered-shell")).toEqual([]);
});
