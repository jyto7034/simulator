import { test, expect } from "@playwright/test";
import { example } from "../../src/nodes/core/examples";
import { diagnostics } from "../../src/nodes/core/validation";
import { inputSources } from "../../src/nodes/core/connections";
const codes = (doc: ReturnType<typeof example>) =>
  diagnostics(doc).map((d) => d.code);

test("both weapon stages share one identity and only mutually exclusive subscriptions may coexist", () => {
  for (const file of [
    "funeral-of-dying-butterflies",
    "funeral-of-dying-butterflies-bloom",
    "funeral-of-dying-butterflies-orb",
  ]) {
    const doc = example(file),
      source = inputSources(doc);
    const hits = doc.nodes.filter((n) => n.kind === "skill-mark-weapon-hit");
    expect(hits.map((n) => n.values.binding).sort()).toEqual([
      "weapon-base",
      "weapon-upgrade",
    ]);
    expect(new Set(hits.map((n) => source(n, "marks")!.id)).size).toBe(1);
    expect(new Set(hits.map((n) => source(n, "evolution")!.id)).size).toBe(1);
    expect(
      doc.nodes.find((n) => n.kind === "skill-weapon-mark-owner")!.values.key,
    ).toBe("butterflies");
    expect(codes(doc).every((c) => c === "skill-value-unset")).toBe(true);
    hits[1].values.binding = hits[0].values.binding;
    expect(codes(doc)).toContain("mark-subscription-duplicate");
  }
});

test("foreign weapon evolution, missing ownership and presence stores cannot borrow weapon identity", () => {
  const doc = example("funeral-of-dying-butterflies-bloom");
  const hit = doc.nodes.find((n) => n.kind === "skill-mark-weapon-hit")!;
  hit.values.weapon = "other";
  expect(codes(doc)).toContain("weapon-evolution-owner");
  const owner = doc.nodes.find((n) => n.kind === "skill-weapon-mark-owner")!;
  doc.edges = doc.edges.filter(
    (e) => e.target.node !== owner.id || e.target.port !== "evolution",
  );
  expect(codes(doc)).toContain("mark-weapon-owner");
  doc.nodes.find((n) => n.kind === "skill-target-marks")!.values.mode =
    "presence";
  expect(codes(doc)).toContain("mark-weapon-storage");
});

test("growth multipliers change three new grants and do not target the damage or stock", () => {
  const doc = example("mountain-of-smiling-bodies"),
    source = inputSources(doc);
  const growth = doc.nodes.find((n) => n.kind === "skill-weapon-kill-growth")!;
  const amounts = ["normal", "elite", "abnormality"].map((p) =>
    source(growth, p)!,
  );
  const modifiers = doc.nodes.filter((n) => n.kind === "skill-modifier-number");
  expect(modifiers).toHaveLength(3);
  expect(new Set(modifiers.map((n) => source(n, "target")!.id))).toEqual(
    new Set(amounts.map((n) => n.id)),
  );
  expect(modifiers.every((n) => n.values.factor === 2)).toBe(true);
  expect(growth.values.damage).toBe("");
  expect(growth.values.secondaryPercent).toBe("");
  amounts[0].values.value = 1.5;
  expect(codes(doc)).toContain("weapon-growth-amount");
});
