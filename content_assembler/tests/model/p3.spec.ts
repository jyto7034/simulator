import { readFileSync } from "node:fs";
import { test, expect } from "@playwright/test";
import { queenOfHatred } from "../../src/nodes/core/queen-of-hatred";
import { yinYang } from "../../src/nodes/core/yin-yang";
import { definition } from "../../src/nodes/core/catalog";
import { GraphSession } from "../../src/nodes/core/session";
import { parseDocument } from "../../src/nodes/core/validation";
import { inputSources } from "../../src/nodes/core/connections";
import type { GraphDocument, GraphNode } from "../../src/nodes/types";

const nodes = (s: GraphSession, kind: string) =>
  s.getSnapshot().doc.nodes.filter((n) => n.kind === `skill-${kind}`);
const find = (s: GraphSession, kind: string) => nodes(s, kind)[0]!;
const requestFor = (s: GraphSession, use: GraphNode) => {
  const source = inputSources(s.getSnapshot().doc);
  const permit = source(use, "permit");
  return permit && source(permit, "request");
};
const channelFor = (s: GraphSession, entry: "beam" | "black-hole" | "auto") => {
  const source = inputSources(s.getSnapshot().doc);
  return nodes(s, "sequence-channel").find((n) => {
    const use = source(n, "use");
    const request = use && requestFor(s, use);
    return entry === "auto"
      ? request?.kind === "skill-counter-auto-request"
      : request?.values.entry === entry;
  })!;
};
const termsFor = (s: GraphSession, entry: "beam" | "black-hole" | "auto") => {
  const source = inputSources(s.getSnapshot().doc);
  const use = source(channelFor(s, entry), "use")!;
  return source(source(use, "permit")!, "terms")!;
};
const codes = (s: GraphSession) =>
  s.getSnapshot().diagnostics.map((d) => d.code);
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
const add = (s: GraphSession, kind: string) => {
  const previous = new Set(s.getSnapshot().doc.nodes.map((n) => n.id));
  s.edit({ type: "add", kind: `skill-${kind}`, position: { x: 0, y: 0 } });
  return s.getSnapshot().doc.nodes.find((n) => !previous.has(n.id))!;
};
const set = (s: GraphSession, n: GraphNode, values: GraphNode["values"]) =>
  s.edit({ type: "setNode", id: n.id, values });
function assigned(doc = queenOfHatred(true)) {
  // Explicit test-only values; original content remains unfinished balance.
  for (const n of doc.nodes) {
    for (const f of definition(n.kind).fields)
      if (f.valueType === "optional-number" && n.values[f.id] === "")
        n.values[f.id] = 10;
    if (n.kind === "skill-attribute-effect") {
      n.values.attribute = "attack";
      n.values.operation = "percent";
    }
    if (n.kind === "skill-range") {
      n.values.pattern = "@X";
      n.values.includeAnchor = true;
    }
  }
  return new GraphSession(doc);
}

for (const [name, factory, unassigned] of [
  ["queen-of-hatred", () => queenOfHatred(), 29],
  ["queen-of-hatred-bloom", () => queenOfHatred(true), 41],
  ["yin-yang", yinYang, 15],
] as const) {
  test(`${name} real file and factory preserve unfinished balance, typed ownership and save/reload`, () => {
    for (const doc of [
      factory(),
      parseDocument(readFileSync(`content/${name}.node-graph.json`, "utf8")),
    ]) {
      const s = new GraphSession(doc);
      expect(codes(s).sort()).toEqual(
        [
          ...Array(unassigned).fill("skill-value-unset"),
          "skill-policy-unset",
          "skill-policy-unset",
        ].sort(),
      );
      expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
      expect(codes(assigned(doc))).toEqual([]);
    }
  });
}

test("queen variants express independent manual entries, atomic heart spending, automatic zero cost and natural-only award", () => {
  for (const bloom of [false, true]) {
    const s = assigned(queenOfHatred(bloom)),
      source = inputSources(s.getSnapshot().doc);
    expect(find(s, "battle-counter").values.capacity).toBe(6);
    expect(source(find(s, "counter-periodic"), "interval")?.values.value).toBe(
      15,
    );
    expect(
      nodes(s, "menu-request")
        .map((n) => n.values.entry)
        .sort(),
    ).toEqual(["beam", "black-hole"]);
    const terms = [
      termsFor(s, "beam"),
      termsFor(s, "black-hole"),
      termsFor(s, "auto"),
    ];
    expect(terms.map((n) => n.values)).toEqual([
      { minimum: 0, maximum: 5, spend: "none" },
      { minimum: bloom ? 0 : 3, maximum: 5, spend: "admission-all" },
      { minimum: 6, maximum: 6, spend: "end-all" },
    ]);
    const owners = nodes(s, "sequence-channel");
    expect(owners).toHaveLength(3);
    const autoUse = source(channelFor(s, "auto"), "use")!,
      permit = source(autoUse, "permit")!;
    expect(source(permit, "request")?.kind).toBe("skill-counter-auto-request");
    expect(source(permit, "cost")?.values.mode).toBe("zero");
    expect(source(permit, "requirement")?.values.mode).toBe("none");
    const autoAreas = nodes(s, "phase-area").filter(
      (n) => source(n, "channel")?.id === channelFor(s, "auto").id,
    );
    expect(autoAreas).toHaveLength(2);
    const autoQueries = nodes(s, "ground-targets").filter((n) =>
      autoAreas.some((a) => source(n, "area")?.id === a.id),
    );
    const autoHeals = nodes(s, "ground-heal").filter((n) =>
      autoQueries.some((q) => source(n, "member")?.id === q.id),
    );
    expect(autoHeals).toHaveLength(0);
    expect(nodes(s, "completion-award")).toHaveLength(1);
    expect(find(s, "completion-award").values).toEqual({ base: 1, bonus: 1 });
    expect(nodes(s, "ground-pull")).toHaveLength(2);
    expect(nodes(s, "channel-phase")).toHaveLength(bloom ? 5 : 4);
  }
});

test("automatic requests reject paid casts, cap-open manual input, foreign memory and manual cancellation", () => {
  const s = assigned(),
    source = inputSources(s.getSnapshot().doc);
  const auto = find(s, "counter-auto-request"),
    qualify = nodes(s, "resonance-qualify").find(
      (n) => source(n, "request")?.id === auto.id,
    )!;
  set(s, source(qualify, "cost")!, { mode: "all" });
  expect(codes(s)).toContain("counter-auto-terms");
  s.travel(false);
  set(s, termsFor(s, "beam"), { maximum: 6 });
  expect(codes(s)).toContain("counter-auto-priority");
  s.travel(false);
  const foreign = add(s, "battle-counter");
  set(s, foreign, { capacity: 6 });
  connect(s, foreign, "counter", find(s, "aim-memory"), "counter");
  expect(codes(s)).toContain("counter-owner");
  s.travel(false);
  const cancel = add(s, "channel-cancel-request");
  connect(s, channelFor(s, "auto"), "channel", cancel, "channel");
  expect(codes(s)).toContain("phase-auto-cancel");
});

test("counter receipt needs atomic all-spend success and matching cast; result quantities never substitute for receipts", () => {
  const s = assigned(),
    receipt = find(s, "counter-receipt"),
    source = inputSources(s.getSnapshot().doc),
    use = source(receipt, "receipt")!;
  connect(s, use, "rejected", receipt, "in");
  expect(codes(s)).toContain("counter-receipt-scope");
  s.travel(false);
  const terms = source(source(use, "permit")!, "terms")!;
  set(s, terms, { spend: "none" });
  expect(codes(s)).toContain("counter-receipt-scope");
  s.travel(false);
  connect(
    s,
    inputSources(s.getSnapshot().doc)(channelFor(s, "beam"), "use")!,
    "use",
    channelFor(s, "black-hole"),
    "use",
  );
  expect(codes(s)).toContain("phase-count-scope");
  s.travel(false);
  const snapshot = s.getSnapshot();
  expect(() =>
    connect(s, find(s, "ground-heal"), "result", receipt, "receipt"),
  ).toThrow();
  expect(s.getSnapshot()).toBe(snapshot);
});

test("one natural completion cannot reward cancellation, another cast result or duplicate gains", () => {
  const s = assigned(),
    owner = channelFor(s, "beam"),
    award = find(s, "completion-award");
  connect(s, owner, "cancelled", award, "in");
  expect(codes(s)).toContain("phase-natural-completion");
  s.travel(false);
  const source = inputSources(s.getSnapshot().doc);
  const foreignDamage = nodes(s, "ground-damage").find((n) => {
    const area = source(source(n, "member")!, "area")!;
    return source(area, "channel")?.id !== owner.id;
  })!;
  connect(s, foreignDamage, "result", find(s, "cast-outcomes"), "damage");
  expect(codes(s)).toContain("phase-result-owner");
  s.travel(false);
  const duplicate = add(s, "counter-gain");
  const reward = nodes(s, "counter-gain").find(
    (n) => inputSources(s.getSnapshot().doc)(n, "amount")?.id === award.id,
  )!;
  connect(s, reward, "out", duplicate, "in");
  connect(s, award, "amount", duplicate, "amount");
  connect(s, find(s, "battle-counter"), "counter", duplicate, "counter");
  expect(codes(s)).toContain("counter-gain-duplicate");
});

test("resource declarations keep integer bounds, positive timers, unique menu IDs and one owner", () => {
  const s = assigned(),
    terms = termsFor(s, "beam");
  set(s, terms, { minimum: 6, maximum: 5 });
  expect(codes(s)).toContain("counter-range");
  s.travel(false);
  const timer = inputSources(s.getSnapshot().doc)(
    find(s, "counter-periodic"),
    "interval",
  )!;
  set(s, timer, { value: 0 });
  expect(codes(s)).toContain("counter-period");
  s.travel(false);
  set(
    s,
    nodes(s, "menu-request").find((n) => n.values.entry === "black-hole")!,
    { entry: "beam" },
  );
  expect(codes(s)).toContain("counter-menu-entry");
  s.travel(false);
  const before = s.getSnapshot();
  expect(() => set(s, terms, { maximum: 1.5 })).toThrow();
  expect(s.getSnapshot()).toBe(before);
  const file: GraphDocument = structuredClone(before.doc);
  file.nodes.find((n) => n.id === terms.id)!.values.maximum = -1;
  expect(() => parseDocument(JSON.stringify(file))).toThrow();
});

test("scaled range tiers cover every admitted count and cannot shrink or repeat", () => {
  const s = assigned(),
    tiers = nodes(s, "count-range-tier").sort(
      (a, b) => Number(a.values.count) - Number(b.values.count),
    );
  set(s, tiers[1], { count: 0 });
  expect(codes(s)).toContain("phase-range-coverage");
  s.travel(false);
  const range = inputSources(s.getSnapshot().doc)(tiers[1], "range")!;
  set(s, range, { pattern: "@", includeAnchor: true });
  expect(codes(s)).toContain("phase-range-shrinks");
  s.travel(false);
  s.edit({ type: "delete", ids: [tiers[3].id] });
  expect(codes(s)).toContain("phase-range-coverage");
  s.travel(false);
  expect(codes(s)).toEqual([]);
});

test("phase membership and flow completion cannot reopen a closed use", () => {
  const s = assigned(),
    owner = channelFor(s, "beam"),
    area = nodes(s, "phase-area").find(
      (n) => inputSources(s.getSnapshot().doc)(n, "channel")?.id === owner.id,
    )!;
  const otherPhase = inputSources(s.getSnapshot().doc)(
    channelFor(s, "black-hole"),
    "first",
  )!;
  connect(s, otherPhase, "phase", area, "phase");
  expect(codes(s)).toContain("phase-owner");
  s.travel(false);
  const end = add(s, "resonance-end");
  connect(
    s,
    inputSources(s.getSnapshot().doc)(channelFor(s, "beam"), "use")!,
    "use",
    end,
    "use",
  );
  connect(s, owner, "completed", end, "in");
  expect(codes(s)).toContain("use-lifetime-end");
  expect(codes(s)).toContain("use-lifetime-closed");
  connect(s, end, "out", owner, "in");
  expect(codes(s)).toContain("use-lifetime-closed");
});

test("pull targets exclude allies and query completion carries no individual member", () => {
  const s = assigned(),
    pull = find(s, "ground-pull"),
    source = inputSources(s.getSnapshot().doc),
    query = source(pull, "member")!;
  connect(s, find(s, "ground-filter"), "filter", query, "filter");
  expect(codes(s)).toContain("phase-pull-filter");
  s.travel(false);
  connect(s, query, "completed", pull, "in");
  expect(codes(s)).toContain("ground-member-scope");
  s.travel(false);
  set(s, pull, { tiles: 0 });
  expect(codes(s)).toContain("phase-pull-distance");
});

test("yin-yang has one atomic pair, a shared pulse, base-result transfer and an independent intersection subscription", () => {
  const s = assigned(yinYang()),
    source = inputSources(s.getSnapshot().doc);
  const pair = find(s, "paired-areas"),
    pulse = find(s, "pair-pulse"),
    heal = find(s, "pair-base-heal"),
    transfer = find(s, "pair-transfer");
  expect(source(pair, "first")?.id).toBe(source(pair, "second")?.id);
  expect(source(pulse, "pair")?.id).toBe(pair.id);
  expect(source(heal, "pulse")?.id).toBe(pulse.id);
  expect(source(transfer, "pulse")?.id).toBe(pulse.id);
  expect(source(transfer, "result")?.id).toBe(heal.id);
  expect(source(find(s, "pair-intersection"), "pair")?.id).toBe(pair.id);
  expect(nodes(s, "resonance-end")).toHaveLength(0);
  expect(nodes(s, "ground-damage")).toHaveLength(0);
});

test("pair transfer rejects foreign pulses, missing completed base flow and duplicate budgets", () => {
  const s = assigned(yinYang()),
    pulse = find(s, "pair-pulse"),
    heal = find(s, "pair-base-heal"),
    transfer = find(s, "pair-transfer");
  const other = add(s, "pair-pulse");
  connect(s, other, "pulse", transfer, "pulse");
  expect(codes(s)).toContain("pair-result-scope");
  expect(codes(s)).toContain("pair-pulse-scope");
  s.travel(false);
  connect(s, pulse, "out", transfer, "in");
  expect(codes(s)).toContain("pair-result-scope");
  s.travel(false);
  const duplicate = add(s, "pair-transfer");
  connect(s, pulse, "pulse", duplicate, "pulse");
  connect(s, heal, "out", duplicate, "in");
  connect(s, heal, "result", duplicate, "result");
  expect(codes(s)).toContain("pair-settlement-duplicate");
  const before = s.getSnapshot();
  expect(() => connect(s, transfer, "result", duplicate, "result")).toThrow();
  expect(s.getSnapshot()).toBe(before);
});

test("pair admission requires two selections and expiry/interval/buff values remain explicit", () => {
  const s = assigned(yinYang()),
    pair = find(s, "paired-areas"),
    source = inputSources(s.getSnapshot().doc);
  const single = add(s, "target-selection");
  set(s, single, { mode: "self" });
  connect(s, single, "selection", find(s, "request"), "selection");
  expect(codes(s)).toContain("pair-use-scope");
  s.travel(false);
  for (const [node, values, code] of [
    [source(pair, "duration")!, { value: 0 }, "pair-duration"],
    [source(find(s, "pair-pulse"), "interval")!, { value: 0 }, "pair-interval"],
    [find(s, "attack-speed-buff"), { points: 0 }, "pair-buff-positive"],
  ] as const) {
    set(s, node, values);
    expect(codes(s)).toContain(code);
    s.travel(false);
  }
  const before = s.getSnapshot();
  expect(() => set(s, find(s, "pair-transfer"), { percent: 101 })).toThrow();
  expect(s.getSnapshot()).toBe(before);
});

test("P3 typed reroutes, complete graph copy, undo and file reload preserve all ownership", () => {
  for (const doc of [queenOfHatred(true), yinYang()]) {
    const s = assigned(doc);
    for (const edge of [...s.getSnapshot().doc.edges])
      s.edit({ type: "reroute", edgeId: edge.id, position: { x: 0, y: 0 } });
    expect(codes(s)).toEqual([]);
    expect(parseDocument(s.export().json)).toEqual(s.getSnapshot().doc);
    const copy = new GraphSession();
    copy.edit({
      type: "paste",
      fragment: s.copy(s.getSnapshot().doc.nodes.map((n) => n.id)),
      position: { x: 0, y: 0 },
    });
    expect(codes(copy)).toEqual([]);
    const before = copy.getSnapshot().doc;
    const field = doc.name.startsWith("음양")
      ? find(copy, "pair-transfer")
      : find(copy, "battle-counter");
    set(
      copy,
      field,
      doc.name.startsWith("음양") ? { percent: 30 } : { capacity: 7 },
    );
    copy.travel(false);
    expect(copy.getSnapshot().doc).toEqual(before);
    copy.travel(true);
    expect(parseDocument(copy.export().json)).toEqual(copy.getSnapshot().doc);
  }
});
