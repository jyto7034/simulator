import { readFileSync } from "node:fs";
import { test, expect } from "@playwright/test";
import { example } from "../../src/nodes/core/examples";
import { definition } from "../../src/nodes/core/catalog";
import { p4FieldActive } from "../../src/nodes/core/p4-fields";
import { GraphSession } from "../../src/nodes/core/session";
import { parseDocument } from "../../src/nodes/core/validation";
import { inputSources } from "../../src/nodes/core/connections";
import type { GraphNode } from "../../src/nodes/types";

const fixtures = [
  ["knight-of-despair", 18],
  ["knight-of-despair-bloom", 18],
  ["little-red", 34],
  ["king-of-greed", 18],
] as const;
type Name = (typeof fixtures)[number][0];
const codes = (s: GraphSession) =>
  s.getSnapshot().diagnostics.map((d) => d.code);
const nodes = (s: GraphSession, kind: string) =>
  s.getSnapshot().doc.nodes.filter((n) => n.kind === `skill-${kind}`);
const find = (s: GraphSession, kind: string) => nodes(s, kind)[0]!;
const title = (s: GraphSession, label: string) =>
  s.getSnapshot().doc.nodes.find((n) => n.title === label)!;
const source = (s: GraphSession, n: GraphNode, port: string) =>
  inputSources(s.getSnapshot().doc)(n, port)!;
const add = (s: GraphSession, kind: string) => {
  const ids = new Set(s.getSnapshot().doc.nodes.map((n) => n.id));
  s.edit({ type: "add", kind: `skill-${kind}`, position: { x: 0, y: 0 } });
  return s.getSnapshot().doc.nodes.find((n) => !ids.has(n.id))!;
};
const set = (s: GraphSession, n: GraphNode, values: GraphNode["values"]) =>
  s.edit({ type: "setNode", id: n.id, values });
const connect = (
  s: GraphSession,
  a: GraphNode,
  ap: string,
  b: GraphNode,
  bp: string,
) =>
  s.edit({
    type: "connect",
    source: { node: a.id, port: ap },
    target: { node: b.id, port: bp },
  });
function assigned(name: Name = "knight-of-despair", input = example(name)) {
  const doc = structuredClone(input);
  for (const n of doc.nodes) {
    for (const field of definition(n.kind).fields)
      if (
        field.valueType === "optional-number" &&
        p4FieldActive(n, field) &&
        n.values[field.id] === ""
      )
        n.values[field.id] = 10;
    if (n.kind === "skill-modifier-projectile-explosion")
      n.values.air = "include";
    if (n.kind === "skill-explosion-rule") n.values.damageType = "magic";
    if (n.kind === "skill-range" && n.values.pattern === "") {
      n.values.pattern = "@X";
      n.values.includeAnchor = true;
    }
  }
  return new GraphSession(doc);
}
function pasteOther(s: GraphSession, name: Name) {
  const other = assigned(name),
    before = new Set(s.getSnapshot().doc.nodes.map((n) => n.id));
  s.edit({
    type: "paste",
    fragment: other.copy(other.getSnapshot().doc.nodes.map((n) => n.id)),
    position: { x: 0, y: 0 },
  });
  return s.getSnapshot().doc.nodes.filter((n) => !before.has(n.id));
}

for (const [name, unset] of fixtures)
  test(`${name} actual graph and factory retain only explicit balance gaps and save/reload`, () => {
    for (const doc of [
      example(name),
      parseDocument(readFileSync(`content/${name}.node-graph.json`, "utf8")),
    ]) {
      const s = new GraphSession(doc);
      expect(codes(s).sort()).toEqual(
        [
          ...Array(unset).fill("skill-value-unset"),
          ...Array(name === "little-red" ? 3 : 0).fill("skill-policy-unset"),
        ].sort(),
      );
      expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
      expect(codes(assigned(name, doc))).toEqual([]);
    }
  });

test("knight keeps one blessing, a battle shield record and a deferred HP-zero transition", () => {
  const s = assigned(),
    relation = find(s, "ally-relation"),
    mode = find(s, "battle-mode"),
    bank = find(s, "stock");
  expect(bank.values).toEqual({
    initial: 0,
    lifetime: "battle",
    limit: "bounded",
    capacity: 4,
  });
  expect(source(s, relation, "blocked").id).toBe(mode.id);
  const shield = find(s, "relation-emergency-shield");
  expect(shield.values).toEqual({ threshold: 10, shield: 50 });
  expect(source(s, shield, "relation").id).toBe(relation.id);
  expect(source(s, shield, "duration").values.value).toBe(15);
  const death = find(s, "relation-hp-zero"),
    defer = find(s, "defer-mode");
  expect(source(s, death, "relation").id).toBe(relation.id);
  expect(source(s, defer, "event").id).toBe(death.id);
  expect(source(s, defer, "mode").id).toBe(mode.id);
  expect(source(s, find(s, "relation-clear"), "relation").id).toBe(relation.id);
  const self = find(s, "self-stock-damage");
  expect(self.values).toEqual({ percent: 5, damage: "fixed-bypass" });
  expect(source(s, source(s, self, "count"), "bank").id).toBe(bank.id);
});

test("knight normal plan pays four ordered stages and applies paid-count wounds only after each entire stage", () => {
  const s = assigned(),
    program = find(s, "mode-program"),
    normal = source(s, program, "normal");
  expect(normal.values.start).toBe("next-basic");
  const root = source(s, normal, "plan");
  expect(source(s, root, "first").title).toBe("검 없이도 첫 찌르기");
  let tail = source(s, root, "next");
  const stages: GraphNode[] = [];
  for (let i = 0; i < 3; i++) {
    stages.push(source(s, tail, "first"));
    tail = source(s, tail, "next");
  }
  stages.push(tail);
  expect(stages.map((n) => n.title.split(" · ")[0])).toEqual([
    "아탕호",
    "에스토카다",
    "데스비오",
    "엑시큐션",
  ]);
  const quantity = source(s, stages[0], "count");
  expect(quantity.kind).toBe("skill-count-quotient");
  expect(quantity.values).toEqual({ divisor: 4, minimum: 1 });
  expect(source(s, quantity, "count").kind).toBe("skill-count-stock");
  for (const step of stages) {
    expect(step.kind).toBe("skill-spend-plan");
    expect(step.values.spend).toBe("amount");
    expect(source(s, step, "count").id).toBe(quantity.id);
    const wound = source(s, step, "after");
    expect(wound.values).toEqual({ effect: "vulnerability", percent: 5 });
    expect(source(s, wound, "count").kind).toBe("skill-count-paid");
    expect(source(s, wound, "duration").values.value).toBe(5);
  }
  const normalRepeat = title(s, "일반 엑시큐션 · n회"),
    despair = source(s, program, "active"),
    repeat = source(s, despair, "plan");
  expect(despair.values.start).toBe("immediate");
  expect(repeat.kind).toBe("skill-repeat-plan");
  expect(source(s, repeat, "time").id).toBe(source(s, normalRepeat, "time").id);
  const total = source(s, repeat, "count");
  expect(total.kind).toBe("skill-count-sum");
  expect(source(s, total, "first").id).toBe(
    source(s, normalRepeat, "count").id,
  );
  expect(source(s, total, "second").kind).toBe("skill-count-stock");
  const hit = source(s, repeat, "body");
  expect(source(s, hit, "ramp").values.percent).toBe(10);
  expect(source(s, hit, "after")).toBeUndefined();
});

test("knight bloom spends at big-sword release and keeps two independently owned despair clocks", () => {
  const s = assigned("knight-of-despair-bloom"),
    program = find(s, "mode-program");
  const normal = source(s, program, "normal"),
    root = source(s, normal, "plan");
  expect(source(s, root, "first").kind).toBe("skill-wait-plan");
  const spend = source(s, root, "next"),
    body = source(s, spend, "body");
  expect(spend.values.spend).toBe("all");
  const big = source(s, body, "first"),
    repeat = source(s, body, "next"),
    small = source(s, repeat, "body");
  expect(source(s, repeat, "count").kind).toBe("skill-count-paid");
  expect(repeat.values).toEqual({
    timing: "interval",
    first: "after-interval",
  });
  expect(source(s, big, "range").id).toBe(source(s, small, "range").id);
  expect(source(s, source(s, big, "after"), "duration").values.value).toBe(2);
  expect(source(s, small, "after")).toBeUndefined();
  const despair = source(s, program, "active"),
    timed = source(s, despair, "plan");
  expect(despair.values.interruption).toBe("silence");
  expect(source(s, timed, "interval").values.value).toBe(0.5);
  expect(timed.values.first).toBe("after-interval");
  expect(source(s, timed, "duration").kind).toBe("skill-count-duration");
  const self = find(s, "action-periodic-self-damage");
  expect(source(s, self, "entry").id).toBe(despair.id);
  expect(source(s, self, "interval").values.value).toBe(1);
  expect(self.values.percent).toBe(10);
  expect(source(s, find(s, "action-stop-input"), "run").id).toBe(
    find(s, "run-action").id,
  );
});

test("resource rewards cannot change banks, duplicate a right or reset a battle reward into deployment storage", () => {
  const s = assigned("little-red"),
    killed = find(s, "cohort-killed"),
    credit = nodes(s, "resource-credit").find(
      (p) => source(s, p, "grant")?.id === killed.id,
    )!;
  const wrong = source(s, find(s, "stock-periodic"), "bank");
  connect(s, wrong, "bank", credit, "bank");
  expect(codes(s)).toContain("resource-grant-owner");
  s.travel(false);
  const duplicate = add(s, "resource-credit");
  set(s, duplicate, { amount: 1 });
  connect(s, credit, "out", duplicate, "in");
  connect(s, killed, "grant", duplicate, "grant");
  connect(s, source(s, killed, "bank"), "bank", duplicate, "bank");
  expect(codes(s)).toContain("resource-grant-consumer");
  const fresh = assigned("little-red");
  set(fresh, source(fresh, find(fresh, "cohort-killed"), "bank"), {
    lifetime: "deployment",
  });
  expect(codes(fresh)).toContain("cohort-reward-lifetime");
  expect(codes(fresh)).toContain("plan-stock-lifetime");
});

test("paid-count formulas reject use outside a paid stage and costs from a different bank", () => {
  const s = assigned(),
    quotient = find(s, "count-quotient");
  connect(s, find(s, "count-paid"), "count", quotient, "count");
  expect(codes(s)).toContain("action-paid-scope");
  s.travel(false);
  const bank = add(s, "stock");
  set(s, bank, {
    initial: 0,
    lifetime: "battle",
    limit: "bounded",
    capacity: 4,
  });
  connect(s, bank, "bank", find(s, "count-stock"), "bank");
  expect(codes(s)).toContain("action-cost-bank");
  const bloom = assigned("knight-of-despair-bloom");
  connect(
    bloom,
    find(bloom, "count-paid"),
    "count",
    find(bloom, "count-duration"),
    "count",
  );
  expect(codes(bloom)).toContain("action-paid-scope");
});

test("relation transitions require the matching HP-zero success and preserve one shield record", () => {
  const s = assigned();
  connect(s, find(s, "mode-entered"), "out", find(s, "defer-mode"), "in");
  expect(codes(s)).toContain("p4-source-scope");
  s.travel(false);
  const foreign = add(s, "battle-mode");
  connect(s, foreign, "mode", find(s, "defer-mode"), "mode");
  expect(codes(s)).toContain("relation-mode-owner");
  const second = assigned(),
    shield = find(second, "relation-emergency-shield"),
    duplicate = add(second, "relation-emergency-shield");
  set(second, duplicate, { threshold: 10, shield: 50 });
  connect(
    second,
    find(second, "ally-relation"),
    "relation",
    duplicate,
    "relation",
  );
  connect(
    second,
    source(second, shield, "duration"),
    "value",
    duplicate,
    "duration",
  );
  expect(codes(second)).toContain("relation-shield-record");
});

test("action execution rejects different admitted programs, closed uses and wrong conditional self damage", () => {
  const s = assigned(),
    run = find(s, "run-action"),
    program = find(s, "mode-program");
  connect(s, source(s, program, "normal"), "program", run, "program");
  expect(codes(s)).toContain("p4-program-scope");
  s.travel(false);
  const second = add(s, "run-action");
  connect(s, run, "completed", second, "in");
  connect(s, source(s, run, "use"), "use", second, "use");
  connect(s, program, "program", second, "program");
  expect(codes(s)).toContain("use-lifetime-owner");
  expect(codes(s)).toContain("use-lifetime-closed");
  const bloom = assigned("knight-of-despair-bloom");
  connect(
    bloom,
    source(bloom, find(bloom, "mode-program"), "normal"),
    "program",
    find(bloom, "action-periodic-self-damage"),
    "entry",
  );
  expect(codes(bloom)).toContain("action-self-period-scope");
});

test("instant repeated bodies, times and primary/tile selection remain explicit", () => {
  const s = assigned(),
    repeat = find(s, "repeat-plan");
  connect(s, find(s, "stance-plan"), "plan", repeat, "body");
  expect(codes(s)).toContain("p4-source-scope");
  s.travel(false);
  set(s, source(s, repeat, "time"), { value: 0 });
  expect(codes(s)).toContain("p4-positive-time");
  s.travel(false);
  const before = s.getSnapshot();
  expect(() => set(s, find(s, "count-positive"), { value: 0 })).toThrow();
  expect(s.getSnapshot()).toBe(before);
  const bloom = assigned("knight-of-despair-bloom");
  set(bloom, find(bloom, "target-selection"), { mode: "self" });
  expect(codes(bloom)).toContain("action-request-selection");
});

test("red authors actual hostile wolves, source-family weakness and an atomic two-mode volley", () => {
  const s = assigned("little-red"),
    spawner = find(s, "hostile-spawner"),
    weakness = source(s, spawner, "profile");
  expect(weakness.values).toEqual({ family: "little-red", percent: 50 });
  expect(source(s, weakness, "base").kind).toBe("skill-hostile-profile");
  const memory = find(s, "layout-memory"),
    normal = source(s, memory, "normal"),
    enhanced = source(s, memory, "enhanced");
  expect(normal.id).not.toBe(enhanced.id);
  expect(source(s, find(s, "stock-periodic"), "bank").id).toBe(normal.id);
  expect(source(s, find(s, "cohort-killed"), "bank").id).toBe(enhanced.id);
  const commit = find(s, "plan-commit"),
    merge = source(s, commit, "plan"),
    prepare = find(s, "plan-prepare");
  expect(merge.kind).toBe("skill-plan-merge");
  expect(source(s, source(s, commit, "permit"), "request").id).toBe(prepare.id);
  expect(nodes(s, "resonance-admit")).toHaveLength(0);
  const p = source(s, commit, "normal"),
    strong = source(s, commit, "enhanced");
  expect([p.values.delivery, strong.values.delivery]).toEqual([
    "directional",
    "directional",
  ]);
  expect(source(s, source(s, strong, "power"), "base").id).toBe(
    source(s, p, "power").id,
  );
  expect(source(s, strong, "after").values.effect).toBe("root");
  expect(source(s, source(s, strong, "after"), "duration").values.value).toBe(
    2,
  );
});

test("preparation cancellation and the opposite aim branch cannot supply a successful plan", () => {
  const s = assigned("little-red"),
    prepare = find(s, "plan-prepare"),
    layout = find(s, "plan-layout");
  connect(s, prepare, "cancelled", layout, "in");
  expect(codes(s)).toContain("p4-source-scope");
  s.travel(false);
  connect(s, prepare, "disconnected", layout, "in");
  expect(codes(s)).toContain("p4-source-scope");
  s.travel(false);
  connect(
    s,
    find(s, "plan-aim-mode"),
    "focused",
    find(s, "plan-precise-aim"),
    "in",
  );
  expect(codes(s)).toContain("plan-aim-branch");
});

test("merging or committing another complete preparation is rejected after valid independent graphs", () => {
  const s = assigned("little-red"),
    merge = find(s, "plan-merge"),
    commit = find(s, "plan-commit"),
    other = pasteOther(s, "little-red");
  expect(codes(s)).toEqual([]);
  connect(
    s,
    other.find((n) => n.kind === "skill-plan-precise-aim")!,
    "plan",
    merge,
    "precise",
  );
  expect(codes(s)).toContain("plan-merge-owner");
  expect(codes(s)).toContain("plan-merge-flow");
  s.travel(false);
  connect(
    s,
    other.find((n) => n.kind === "skill-plan-merge")!,
    "plan",
    commit,
    "plan",
  );
  expect(codes(s)).toContain("plan-commit-scope");
});

test("red cannot pay before a batch commit, double-commit or collapse both stock kinds", () => {
  const s = assigned("little-red"),
    qualify = find(s, "resonance-qualify"),
    early = add(s, "resonance-admit");
  connect(s, qualify, "out", early, "in");
  connect(s, qualify, "permit", early, "permit");
  expect(codes(s)).toContain("plan-early-admission");
  const fresh = assigned("little-red"),
    commit = find(fresh, "plan-commit"),
    duplicate = add(fresh, "plan-commit");
  connect(fresh, commit, "out", duplicate, "in");
  for (const [port, output] of [
    ["permit", "permit"],
    ["plan", "plan"],
    ["normal", "profile"],
    ["enhanced", "profile"],
  ])
    connect(fresh, source(fresh, commit, port), output, duplicate, port);
  expect(codes(fresh)).toContain("plan-commit-duplicate");
  const memory = find(fresh, "layout-memory");
  connect(fresh, source(fresh, memory, "normal"), "bank", memory, "enhanced");
  expect(codes(fresh)).toContain("plan-stock-kinds");
});

test("directional projectiles reject homing, zero speed and paid-count effects without a stage", () => {
  const s = assigned("little-red"),
    profile = find(s, "projectile-profile");
  set(s, profile, { tracking: "homing" });
  expect(codes(s)).toContain("projectile-direction-tracking");
  s.travel(false);
  set(s, profile, { speed: 0 });
  expect(codes(s)).toContain("p4-positive-value");
  s.travel(false);
  const status = title(s, "강화탄 적중 · 2초 속박"),
    paid = add(s, "count-paid");
  set(s, status, { effect: "vulnerability", percent: 5 });
  connect(s, paid, "count", status, "count");
  expect(codes(s)).toContain("action-paid-scope");
});

test("greed separates the loan ledger, preparation costs, fixed volley and owner-specific recovery", () => {
  const s = assigned("king-of-greed"),
    ledger = find(s, "credit-ledger"),
    program = find(s, "portal-program"),
    recovery = find(s, "damage-recovery");
  expect(ledger.values).toEqual({ grant: 10, creditLimit: 20 });
  expect(source(s, program, "bank").id).toBe(ledger.id);
  expect(source(s, recovery, "bank").id).toBe(ledger.id);
  const selector = find(s, "enemy-selector");
  expect(selector.values.order).toBe("unmarked-route");
  expect(source(s, selector, "mark").id).toBe(source(s, recovery, "mark").id);
  const release = find(s, "portal-release-request"),
    freeze = find(s, "portal-freeze"),
    serial = find(s, "serial-projectiles");
  expect(source(s, release, "charge").id).toBe(find(s, "run-portals").id);
  expect(source(s, freeze, "right").id).toBe(release.id);
  expect(source(s, serial, "volley").id).toBe(freeze.id);
  expect(source(s, serial, "profile").values.delivery).toBe("impact");
  expect(find(s, "resource-credit").values.amount).toBe(1);
  expect(nodes(s, "self-stock-damage")).toHaveLength(0);
  expect(nodes(s, "action-periodic-self-damage")).toHaveLength(0);
});

test("unfrozen or twice-consumed volleys cannot launch and termination cannot reopen the original use", () => {
  const s = assigned("king-of-greed"),
    serial = find(s, "serial-projectiles");
  connect(s, find(s, "portal-release-request"), "out", serial, "in");
  expect(codes(s)).toContain("p4-source-scope");
  s.travel(false);
  const duplicate = add(s, "serial-projectiles");
  connect(s, serial, "completed", duplicate, "in");
  for (const [port, output] of [
    ["volley", "volley"],
    ["profile", "profile"],
    ["selector", "selector"],
  ])
    connect(s, source(s, serial, port), output, duplicate, port);
  expect(codes(s)).toContain("portal-volley-consumer");
  const fresh = assigned("king-of-greed"),
    charge = find(fresh, "run-portals"),
    next = add(fresh, "run-portals");
  connect(fresh, find(fresh, "serial-projectiles"), "cancelled", next, "in");
  connect(fresh, source(fresh, charge, "use"), "use", next, "use");
  connect(fresh, source(fresh, charge, "program"), "program", next, "program");
  expect(codes(fresh)).toContain("use-lifetime-closed");
});

test("recovery selection cannot use a different ledger and negative balances are not stock snapshots", () => {
  const s = assigned("king-of-greed"),
    other = add(s, "credit-ledger");
  set(s, other, { grant: 10, creditLimit: 20 });
  connect(s, other, "bank", find(s, "damage-recovery"), "bank");
  expect(codes(s)).toContain("portal-recovery-owner");
  expect(codes(s)).toContain("resource-grant-owner");
  const snapshot = add(s, "count-stock");
  connect(s, other, "bank", snapshot, "bank");
  expect(codes(s)).toContain("p4-source-scope");
  const policy = find(s, "portal-policy");
  set(s, source(s, policy, "last"), { value: 11 });
  expect(codes(s)).toContain("portal-acceleration");
});

test("P4 port types, plan branches, complete-copy undo and JSON roundtrips preserve ownership", () => {
  for (const [name] of fixtures) {
    const s = assigned(name),
      seen = new Set<string>();
    for (const edge of [...s.getSnapshot().doc.edges]) {
      const from = s
        .getSnapshot()
        .doc.nodes.find((n) => n.id === edge.source.node)!;
      const type = definition(from.kind).outputs.find(
        (p) => p.id === edge.source.port,
      )!.valueType;
      if (seen.has(type)) continue;
      seen.add(type);
      s.edit({ type: "reroute", edgeId: edge.id, position: { x: 0, y: 0 } });
    }
    expect(codes(s)).toEqual([]);
    const copy = new GraphSession();
    copy.edit({
      type: "paste",
      fragment: s.copy(s.getSnapshot().doc.nodes.map((n) => n.id)),
      position: { x: 0, y: 0 },
    });
    expect(codes(copy)).toEqual([]);
    expect(parseDocument(copy.export().json)).toEqual(copy.getSnapshot().doc);
    const before = copy.getSnapshot().doc,
      n = find(copy, name === "king-of-greed" ? "credit-ledger" : "stock");
    set(
      copy,
      n,
      name === "king-of-greed" ? { creditLimit: 25 } : { capacity: 7 },
    );
    copy.travel(false);
    expect(copy.getSnapshot().doc).toEqual(before);
    copy.travel(true);
    expect(parseDocument(copy.export().json)).toEqual(copy.getSnapshot().doc);
  }
});

test("impact projectile collision width is editable, required and preserved through save and undo", () => {
  const s = assigned("king-of-greed");
  const profile = find(s, "projectile-profile");
  expect(p4FieldActive(profile, { id: "width" })).toBe(true);
  set(s, profile, { width: 0.5 });
  expect(
    find(new GraphSession(parseDocument(s.export().json)), "projectile-profile")
      .values.width,
  ).toBe(0.5);
  set(s, profile, { width: 0 });
  expect(codes(s)).toContain("p4-positive-value");
  s.travel(false);
  expect(find(s, "projectile-profile").values.width).toBe(0.5);
});
