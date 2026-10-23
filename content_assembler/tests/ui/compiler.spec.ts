import { readFile } from "node:fs/promises";
import type { Page } from "@playwright/test";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
  inspectLayout,
} from "./fixtures";
import { observeNodes } from "./node-observer";
import type { GraphDocument } from "../../src/nodes/types";
import { canonicalize } from "../../src/nodes/core/model";

async function importGraph(page: Page, doc: GraphDocument) {
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "compile.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(doc)),
  });
  const confirm = page
    .getByRole("dialog")
    .getByRole("button", { name: "교체", exact: true });
  await expect
    .poll(
      async () =>
        (await confirm.isVisible()) ||
        (await page.locator(".ng-document-title strong").textContent()) ===
          doc.name,
    )
    .toBe(true);
  if (await confirm.isVisible()) await confirm.click();
}
async function fixture(name: string): Promise<GraphDocument> {
  const doc = JSON.parse(
    await readFile(
      `docs/examples/compiler-preflight/${name}.node-graph.json`,
      "utf8",
    ),
  );
  canonicalize(doc);
  return doc;
}
async function nodes(page: Page) {
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "노드", exact: true })
    .click();
}
async function begin(page: Page) {
  await page.getByRole("button", { name: "컴파일", exact: true }).click();
  await page
    .getByRole("button", { name: "현재 문서 컴파일", exact: true })
    .click();
}

test("Core compiler: real RON success preserves authoring and exports exact artifact", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc = await fixture("G02");
  await importGraph(page, doc);
  await expect.poll(() => model.state()?.doc).toEqual(doc);
  const before = model.state();
  const response = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const core = await (await response).json();
  expect(core.status).toBe("compiled");
  await expect(
    page.getByText("컴파일 성공 · RON 재로드 검증 완료", { exact: true }),
  ).toBeVisible();
  expect(model.state()).toEqual(before);
  await capture(page, info, "compile-success");
  await inspectLayout(page, info, "compile-success", { modal: true });
  await inspectAccessibility(page, info, "compile-success");
  const pending = page.waitForEvent("download");
  await page.getByRole("button", { name: "RON 다운로드", exact: true }).click();
  const download = await pending;
  expect(download.suggestedFilename()).toBe("G02.skill-graph.ron");
  expect(await readFile((await download.path())!, "utf8")).toBe(core.ron);
  expect(model.state()).toEqual(before);
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("button", { name: "컴파일", exact: true }),
  ).toBeFocused();
});

test("Core compiler: real self-state graph edits survive RON compilation and undo", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/schadenfreude.node-graph.json", "utf8"),
  );
  // Open balance values belong only to this test copy.
  for (const node of doc.nodes) {
    if (node.kind === "skill-resonance-amount") node.values.value = 100;
    if (node.kind === "skill-seconds") node.values.value = 1;
    if (node.kind === "skill-attribute-effect") node.values.amount = 50;
  }
  canonicalize(doc);
  await importGraph(page, doc);
  await expect.poll(() => model.state()?.doc).toEqual(doc);
  const attack = doc.nodes.find(
    (n) =>
      n.kind === "skill-attribute-effect" && n.values.attribute === "attack",
  )!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(attack.id);
  const edit = page
    .getByTestId(`rf__node-${attack.id}`)
    .getByRole("button", { name: `${attack.title} 증가량 수정`, exact: true });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel("증가량", { exact: true })
    .fill("75");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === attack.id)?.values.amount,
    )
    .toBe(75);
  await capture(page, info, "self-state-edited");
  await inspectLayout(page, info, "self-state-edited");
  await inspectAccessibility(page, info, "self-state-edited");
  const beforeCompile = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const response = await (await pending).json();
  expect(response.status).toBe("compiled");
  expect(response.ron).toContain("ResonanceState");
  expect(response.ron).toContain("attack_percent: 75");
  expect(response.ron).toContain("duration_ms: 1000");
  await expect(
    page.getByText("컴파일 성공 · RON 재로드 검증 완료", { exact: true }),
  ).toBeVisible();
  expect(model.state()).toEqual(beforeCompile);
  await capture(page, info, "self-state-compiled");
  await inspectLayout(page, info, "self-state-compiled", { modal: true });
  await inspectAccessibility(page, info, "self-state-compiled");
  // axe marks Radix's background focus guards as incomplete; exercise real Tab containment.
  await page
    .getByRole("button", { name: "현재 문서 컴파일", exact: true })
    .focus();
  for (const key of ["Tab", "Shift+Tab"]) {
    for (let i = 0; i < 6; i++) {
      await page.keyboard.press(key);
      expect(
        await page.evaluate(() =>
          Boolean(document.activeElement?.closest('[role="dialog"]')),
        ),
      ).toBe(true);
    }
  }
  await page.keyboard.press("Escape");
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const again = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const restored = await (await again).json();
  expect(restored.status).toBe("compiled");
  expect(restored.ron).toContain("attack_percent: 50");
  expect(restored.source_digest).not.toBe(response.source_digest);
});

test("Core compiler: reactive heal count edit survives RON and undo", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/queen-bee.node-graph.json", "utf8"),
  );
  for (const node of doc.nodes) {
    if (node.kind === "skill-resonance-amount") node.values.value = 100;
    if (node.kind === "skill-seconds") node.values.value = 2;
    if (node.kind === "skill-range") {
      node.values.pattern =
        "XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX";
      node.values.includeAnchor = true;
    }
    if (node.kind === "skill-reactive-heal-plan") {
      node.values.charges = 2;
      node.values.percent = 20;
    }
  }
  canonicalize(doc);
  await importGraph(page, doc);
  await expect.poll(() => model.state()?.doc).toEqual(doc);
  const plan = doc.nodes.find((n) => n.kind === "skill-reactive-heal-plan")!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(plan.id);
  const edit = page.getByTestId(`rf__node-${plan.id}`).getByRole("button", {
    name: `${plan.title} 최초 회복 횟수 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel("최초 회복 횟수", { exact: true })
    .fill("3");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === plan.id)?.values.charges,
    )
    .toBe(3);
  await capture(page, info, "reactive-heal-edited");
  await inspectLayout(page, info, "reactive-heal-edited");
  await inspectAccessibility(page, info, "reactive-heal-edited");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  for (const value of [
    "ResonanceReactiveHeal",
    "charges: 3",
    "max_health_percent: 20",
    "duration_ms: 2000",
  ])
    expect(result.ron).toContain(value);
  expect(model.state()).toEqual(before);
  await capture(page, info, "reactive-heal-compiled");
  await inspectLayout(page, info, "reactive-heal-compiled", { modal: true });
  await inspectAccessibility(page, info, "reactive-heal-compiled");
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("button", { name: "컴파일", exact: true }),
  ).toBeFocused();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const again = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const restored = await (await again).json();
  expect(restored.status).toBe("compiled");
  expect(restored.ron).toContain("charges: 2");
  expect(restored.source_digest).not.toBe(result.source_digest);
});

test("Core compiler: delayed area timing edit preserves independent impact RON", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/clouded-monk.node-graph.json", "utf8"),
  );
  for (const node of doc.nodes) {
    if (node.kind === "skill-resonance-amount") node.values.value = 100;
    if (node.kind === "skill-seconds")
      node.values.value = node.title.includes("준비") ? 0.1 : 0.2;
    if (node.kind === "skill-range") {
      node.values.pattern = node.title.includes("착탄")
        ? "XXX\nX@X\nXXX"
        : "XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX";
      node.values.includeAnchor = true;
    }
    if (node.kind === "skill-weapon-power") node.values.coefficient = 1.5;
    if (node.kind === "skill-self-heal-plan") node.values.percent = 20;
  }
  canonicalize(doc);
  await importGraph(page, doc);
  await expect.poll(() => model.state()?.doc).toEqual(doc);
  const delay = doc.nodes.find(
    (n) => n.kind === "skill-seconds" && n.title.includes("도착"),
  )!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(delay.id);
  const edit = page.getByTestId(`rf__node-${delay.id}`).getByRole("button", {
    name: `${delay.title} 시간 (초) 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel("시간 (초)", { exact: true })
    .fill("0.3");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === delay.id)?.values.value,
    )
    .toBe(0.3);
  await capture(page, info, "delayed-area-edited");
  await inspectLayout(page, info, "delayed-area-edited");
  await inspectAccessibility(page, info, "delayed-area-edited");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  for (const text of [
    "ResonanceDelayedArea",
    "delivery_delay_ms: 300",
    "prepare_ms: 100",
    "coefficient_numerator: 3",
    "coefficient_denominator: 2",
  ])
    expect(result.ron).toContain(text);
  expect(model.state()).toEqual(before);
  await capture(page, info, "delayed-area-compiled");
  await inspectLayout(page, info, "delayed-area-compiled", { modal: true });
  await inspectAccessibility(page, info, "delayed-area-compiled");
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("button", { name: "컴파일", exact: true }),
  ).toBeFocused();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const again = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const restored = await (await again).json();
  expect(restored.status).toBe("compiled");
  expect(restored.ron).toContain("delivery_delay_ms: 200");
  expect(restored.source_digest).not.toBe(result.source_digest);
});

test("Core compiler: scheduled opportunity count edit preserves finite child RON", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/rudolta.node-graph.json", "utf8"),
  );
  for (const node of doc.nodes) {
    if (node.kind === "skill-resonance-amount") node.values.value = 100;
    if (node.kind === "skill-seconds")
      node.values.value = node.title.includes("간격") ? 0.1 : 0.2;
    if (node.kind === "skill-range") {
      node.values.pattern = node.title.includes("착탄")
        ? "XXX\nX@X\nXXX"
        : "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX";
      node.values.includeAnchor = true;
    }
    if (node.kind === "skill-weapon-power") node.values.coefficient = 1.5;
    if (node.kind === "skill-count-positive") node.values.value = 3;
  }
  canonicalize(doc);
  await importGraph(page, doc);
  await expect.poll(() => model.state()?.doc).toEqual(doc);
  const delay = doc.nodes.find((n) => n.kind === "skill-count-positive")!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(delay.id);
  const edit = page.getByTestId(`rf__node-${delay.id}`).getByRole("button", {
    name: `${delay.title} 개수 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page.getByRole("dialog").getByLabel("개수", { exact: true }).fill("4");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === delay.id)?.values.value,
    )
    .toBe(4);
  await capture(page, info, "scheduled-area-edited");
  await inspectLayout(page, info, "scheduled-area-edited");
  await inspectAccessibility(page, info, "scheduled-area-edited");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  for (const text of [
    "ResonanceScheduledArea",
    "opportunities: 4",
    "interval_ms: 100",
    "coefficient_numerator: 3",
    "coefficient_denominator: 2",
  ])
    expect(result.ron).toContain(text);
  expect(model.state()).toEqual(before);
  await capture(page, info, "scheduled-area-compiled");
  await inspectLayout(page, info, "scheduled-area-compiled", { modal: true });
  await inspectAccessibility(page, info, "scheduled-area-compiled");
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("button", { name: "컴파일", exact: true }),
  ).toBeFocused();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const again = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const restored = await (await again).json();
  expect(restored.status).toBe("compiled");
  expect(restored.ron).toContain("opportunities: 3");
  expect(restored.source_digest).not.toBe(result.source_digest);
});

test("Core compiler: prepared basic multiplier edit preserves release preparation RON", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile(
      "content/scarecrow-searching-for-wisdom.node-graph.json",
      "utf8",
    ),
  );
  for (const node of doc.nodes) {
    if (node.kind === "skill-resonance-amount") node.values.value = 100;
    if (node.kind === "skill-seconds") node.values.value = 2;
    if (node.kind === "skill-attribute-effect") node.values.amount = 25;
    if (node.kind === "skill-next-basic-plan") node.values.multiplier = 2.5;
  }
  canonicalize(doc);
  await importGraph(page, doc);
  await expect.poll(() => model.state()?.doc).toEqual(doc);
  const plan = doc.nodes.find((n) => n.kind === "skill-next-basic-plan")!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(plan.id);
  const edit = page.getByTestId(`rf__node-${plan.id}`).getByRole("button", {
    name: `${plan.title} 기본 공격 피해 배율 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel("기본 공격 피해 배율", { exact: true })
    .fill("3.5");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === plan.id)?.values
          .multiplier,
    )
    .toBe(3.5);
  await capture(page, info, "prepared-basic-edited");
  await inspectLayout(page, info, "prepared-basic-edited");
  await inspectAccessibility(page, info, "prepared-basic-edited");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  for (const text of [
    "ResonancePreparedBasic",
    "numerator: 7",
    "denominator: 2",
    "attack_percent: 25",
    "duration_ms: 2000",
  ])
    expect(result.ron).toContain(text);
  expect(model.state()).toEqual(before);
  await capture(page, info, "prepared-basic-compiled");
  await inspectLayout(page, info, "prepared-basic-compiled", { modal: true });
  await inspectAccessibility(page, info, "prepared-basic-compiled");
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("button", { name: "컴파일", exact: true }),
  ).toBeFocused();
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const again = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const restored = await (await again).json();
  expect(restored.status).toBe("compiled");
  expect(restored.ron).toContain("numerator: 5");
  expect(restored.source_digest).not.toBe(result.source_digest);
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  await expect.poll(() => model.state().doc).toEqual(saved);
  expect(model.state().dirty).toBe(false);
  const reopen = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const reopened = await (await reopen).json();
  expect(reopened.status).toBe("compiled");
  expect(reopened.ron).toContain("numerator: 7");
});

test("Core compiler: parry charge edit preserves passive ownership and finite window RON", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/dream-of-black-swan.node-graph.json", "utf8"),
  );
  for (const node of doc.nodes) {
    if (node.kind === "skill-resonance-amount") node.values.value = 100;
    if (node.kind === "skill-seconds" && node.values.value === "")
      node.values.value = 0.03;
    if (node.kind === "skill-passive-guard") node.values.percent = 30;
    if (node.kind === "skill-direct-reflection") {
      node.values.chance = 100;
      node.values.percent = 50;
    }
    if (node.kind === "skill-weapon-power") node.values.coefficient = 2.5;
    if (node.kind === "skill-guard-charge")
      Object.assign(node.values, { capacity: 1000, factor: 2, splash: 0.5 });
    if (node.kind === "skill-range")
      Object.assign(node.values, {
        pattern: "XXX\nX@X\nXXX",
        includeAnchor: true,
      });
  }
  canonicalize(doc);
  await importGraph(page, doc);
  await expect.poll(() => model.state()?.doc).toEqual(doc);
  const passive = doc.nodes.find((n) => n.kind === "skill-passive-guard")!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(passive.id);
  await capture(page, info, "parry-passive-source");
  await inspectLayout(page, info, "parry-passive-source");
  await inspectAccessibility(page, info, "parry-passive-source");
  const plan = doc.nodes.find((n) => n.kind === "skill-guard-charge")!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(plan.id);
  const edit = page.getByTestId(`rf__node-${plan.id}`).getByRole("button", {
    name: `${plan.title} 축적 상한 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel("축적 상한", { exact: true })
    .fill("400");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === plan.id)?.values.capacity,
    )
    .toBe(400);
  await capture(page, info, "parry-charge-edited");
  await inspectLayout(page, info, "parry-charge-edited");
  await inspectAccessibility(page, info, "parry-charge-edited");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  for (const text of [
    "ResonanceParry",
    "reduction_percent: 30",
    "capacity: 400",
    "reflection_percent: 50",
    "coefficient_numerator: 5",
    "invulnerable_ms: 500",
  ])
    expect(result.ron).toContain(text);
  expect(model.state()).toEqual(before);
  await capture(page, info, "parry-charge-compiled");
  await inspectLayout(page, info, "parry-charge-compiled", { modal: true });
  await inspectAccessibility(page, info, "parry-charge-compiled");
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("button", { name: "컴파일", exact: true }),
  ).toBeFocused();
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const again = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const restored = await (await again).json();
  expect(restored.status).toBe("compiled");
  expect(restored.ron).toContain("capacity: 1000");
  expect(restored.source_digest).not.toBe(result.source_digest);
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  await expect.poll(() => model.state().doc).toEqual(saved);
  expect(model.state().dirty).toBe(false);
  const reopen = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const reopened = await (await reopen).json();
  expect(reopened.status).toBe("compiled");
  expect(reopened.ron).toBe(result.ron);
});

for (const channel of [false, true]) {
  test(`Core compiler: conditional support ${channel ? "channel" : "shield"} edit survives RON and reload`, async ({
    page,
  }, info) => {
    const model = await observeNodes(page);
    await openWorkspace(page, info);
    await nodes(page);
    const name = channel ? "nameless-fetus" : "porccubus";
    const doc: GraphDocument = JSON.parse(
      await readFile(`content/${name}.node-graph.json`, "utf8"),
    );
    const modifier = doc.nodes.find(
      (n) => n.kind === "skill-modifier-single-support",
    )!;
    const selectionRange = doc.edges.find(
      (e) => e.target.node === modifier.id && e.target.port === "range",
    )!.source.node;
    for (const node of doc.nodes) {
      if (node.kind === "skill-resonance-amount") node.values.value = 100;
      if (node.kind === "skill-seconds") node.values.value = 0.2;
      if (node.kind === "skill-personal-shield-effect")
        node.values.percent = 20;
      if (node.kind === "skill-modifier-single-support" && !channel)
        node.values.factor = 2.5;
      if (node.kind === "skill-range") {
        node.values.includeAnchor = true;
        node.values.pattern = node.title.startsWith("울림통 · 반경")
          ? "XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX"
          : node.id === selectionRange
            ? "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX"
            : "XXX\nX@X\nXXX";
      }
    }
    canonicalize(doc);
    await importGraph(page, doc);
    await expect.poll(() => model.state()?.doc).toEqual(doc);
    if (channel) {
      for (const [kind, name] of [
        ["skill-modifier-range", "support-aura-footprint"],
        ["skill-modifier-single-support", "support-single-reach"],
      ]) {
        const node = doc.nodes.find((n) => n.kind === kind)!;
        await page.getByLabel("노드 또는 구간으로 이동").selectOption(node.id);
        await capture(page, info, name);
        await inspectLayout(page, info, name);
        await inspectAccessibility(page, info, name);
      }
    }
    const plan = channel
      ? doc.nodes.find((n) => n.kind === "skill-modifier-duration")!
      : modifier;
    const field = channel ? "percent" : "factor";
    const label = channel ? "시간 변화율 (%)" : "개인 보호막 배율";
    const changed = channel ? 50 : 3;
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(plan.id);
    const edit = page.getByTestId(`rf__node-${plan.id}`).getByRole("button", {
      name: `${plan.title} ${label} 수정`,
      exact: true,
    });
    await edit.focus();
    await page.keyboard.press("Enter");
    await page
      .getByRole("dialog")
      .getByLabel(label, { exact: true })
      .fill(String(changed));
    await page.getByRole("button", { name: "값 적용", exact: true }).click();
    await expect(edit).toBeFocused();
    await expect
      .poll(
        () =>
          model.state().doc.nodes.find((n) => n.id === plan.id)?.values[field],
      )
      .toBe(changed);
    const prefix = channel ? "support-channel" : "support-shield";
    await capture(page, info, `${prefix}-edited`);
    await inspectLayout(page, info, `${prefix}-edited`);
    await inspectAccessibility(page, info, `${prefix}-edited`);
    const before = model.state();
    const pending = page.waitForResponse((r) =>
      r.url().endsWith("/__content/compile"),
    );
    await begin(page);
    const result = await (await pending).json();
    expect(result.status).toBe("compiled");
    for (const value of [
      "ResonanceSupport",
      "selection_range",
      "single-target-conversion",
      channel ? "percent: 50" : "numerator: 3",
      "duration_ms: 200",
    ])
      expect(result.ron).toContain(value);
    if (channel) {
      expect(result.ron).toContain("SupportRange");
      expect(result.ron).toContain('relic_definition: "resonator"');
      expect(result.ron).toContain("damage_percent: 40");
    }
    expect(model.state()).toEqual(before);
    await capture(page, info, `${prefix}-compiled`);
    await inspectLayout(page, info, `${prefix}-compiled`, { modal: true });
    await inspectAccessibility(page, info, `${prefix}-compiled`);
    await page.keyboard.press("Escape");
    await expect(
      page.getByRole("button", { name: "컴파일", exact: true }),
    ).toBeFocused();
    const exported = page.waitForEvent("download");
    await page
      .getByRole("button", { name: "JSON 내보내기", exact: true })
      .click();
    const saved = JSON.parse(
      await readFile((await (await exported).path())!, "utf8"),
    );
    expect(saved).toEqual(before.doc);
    await page
      .getByRole("button", { name: "다운로드 확인", exact: true })
      .click();
    await page
      .getByRole("button", { name: "노드 실행 취소", exact: true })
      .click();
    await expect.poll(() => model.state().doc).toEqual(doc);
    await importGraph(page, saved);
    await expect.poll(() => model.state().doc).toEqual(saved);
    expect(model.state().dirty).toBe(false);
    const reopen = page.waitForResponse((r) =>
      r.url().endsWith("/__content/compile"),
    );
    await begin(page);
    const reopened = await (await reopen).json();
    expect(reopened.status).toBe("compiled");
    expect(reopened.ron).toContain(channel ? "percent: 50" : "numerator: 3");
  });
}

test("Core compiler: unresolved values and unsupported operations navigate to original nodes", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc = await fixture("G03");
  doc.name = "긴 한글 이름 · 아직 결정하지 않은 공명 최대량의 원본 위치 확인";
  doc.nodes.find((n) => n.id === "capacity")!.title =
    "제작자가 아직 확정하지 않은 최대 공명량";
  await importGraph(page, doc);
  await expect.poll(() => model.state()?.doc).toEqual(doc);
  await begin(page);
  await expect(
    page.getByText("컴파일 차단 · 1개 확인 필요", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "RON 다운로드", exact: true }),
  ).toHaveCount(0);
  await capture(page, info, "compile-unresolved");
  await inspectLayout(page, info, "compile-unresolved", { modal: true });
  await inspectAccessibility(page, info, "compile-unresolved");
  await page.getByRole("button", { name: /값을 입력해야 합니다/ }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  const card = page.getByTestId("rf__node-capacity");
  await expect(card).toBeInViewport();
  expect(model.state().doc).toEqual(doc);
  const unsupported: GraphDocument = JSON.parse(
    await readFile("content/queen-of-hatred.node-graph.json", "utf8"),
  );
  await importGraph(page, unsupported);
  await expect.poll(() => model.state()?.doc).toEqual(unsupported);
  await begin(page);
  await expect(page.getByText(/컴파일 차단 · \d+개 확인 필요/)).toBeVisible();
  expect(
    await page.locator(".ng-compiler-diagnostics button").count(),
  ).toBeGreaterThan(20);
  await capture(page, info, "compile-many-diagnostics");
  await inspectLayout(page, info, "compile-many-diagnostics", { modal: true });
  await inspectAccessibility(page, info, "compile-many-diagnostics");
});

test("Core compiler: revision changes and mismatched responses cannot install success", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc = await fixture("G02");
  await importGraph(page, doc);
  await expect.poll(() => model.state()?.doc).toEqual(doc);
  let release!: () => void;
  const gate = new Promise<void>((resolve) => {
    release = resolve;
  });
  let started = false;
  await page.route(
    "**/__content/compile",
    async (route) => {
      const response = await route.fetch();
      started = true;
      await gate;
      await route.fulfill({ response });
    },
    { times: 1 },
  );
  await begin(page);
  await expect.poll(() => started).toBe(true);
  await capture(page, info, "compile-loading");
  await inspectAccessibility(page, info, "compile-loading");
  await page.keyboard.press("Escape");
  await page.getByLabel("노드 또는 구간으로 이동").selectOption("amount");
  await page
    .getByTestId("rf__node-amount")
    .getByRole("button", { name: /수정$/ })
    .first()
    .click();
  await page.getByRole("dialog").getByRole("textbox").fill("30");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === "amount")?.values.value,
    )
    .toBe(30);
  release();
  await page.getByRole("button", { name: "컴파일", exact: true }).click();
  await expect(
    page.getByText(/문서가 변경되어 이전 컴파일 결과를 적용하지 않았습니다/),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "RON 다운로드", exact: true }),
  ).toHaveCount(0);
  await capture(page, info, "compile-stale");
  await inspectLayout(page, info, "compile-stale", { modal: true });
  await inspectAccessibility(page, info, "compile-stale");
  await page.route(
    "**/__content/compile",
    async (route) => {
      const response = await route.fetch();
      const body = await response.json();
      body.request_id += 1;
      await route.fulfill({ json: body });
    },
    { times: 1 },
  );
  await page
    .getByRole("button", { name: "현재 문서 컴파일", exact: true })
    .click();
  await expect(
    page.getByText(
      "컴파일 응답이 요청한 문서 또는 실행 대상과 일치하지 않습니다.",
      { exact: true },
    ),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "RON 다운로드", exact: true }),
  ).toHaveCount(0);
  await page
    .getByRole("button", { name: "현재 문서 컴파일", exact: true })
    .click();
  await expect(
    page.getByText("컴파일 성공 · RON 재로드 검증 완료", { exact: true }),
  ).toBeVisible();
  expect(model.state().undoCount).toBe(1);
  await page.keyboard.press("Escape");
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  await page.getByRole("button", { name: "컴파일", exact: true }).click();
  await expect(
    page.getByText("편집 후 다시 컴파일해야 합니다.", { exact: true }),
  ).toBeVisible();
});

for (const heal of [false, true]) {
  test(`Core compiler: independent ground ${heal ? "heal" : "damage"} edit survives RON and reload`, async ({
    page,
  }, info) => {
    const model = await observeNodes(page);
    await openWorkspace(page, info);
    await nodes(page);
    const name = heal ? "ground-heal" : "ground-damage";
    const doc: GraphDocument = JSON.parse(
      await readFile(`content/${name}.node-graph.json`, "utf8"),
    );
    const periodic = doc.nodes.find((n) => n.kind === "skill-ground-periodic")!;
    const interval = doc.edges.find(
      (e) => e.target.node === periodic.id && e.target.port === "interval",
    )!.source.node;
    const placement = doc.nodes.find(
      (n) => n.kind === "skill-range-placement",
    )!;
    const footprint = doc.edges.find(
      (e) => e.target.node === placement.id && e.target.port === "range",
    )!.source.node;
    for (const node of doc.nodes) {
      if (node.kind === "skill-resonance-amount") node.values.value = 100;
      if (node.kind === "skill-seconds")
        node.values.value = node.id === interval ? 0.1 : 0.3;
      if (node.kind === "skill-ground-periodic")
        node.values.first = "immediate";
      if (node.kind === "skill-ground-damage") node.values.damageType = "true";
      if (["skill-damage-amount", "skill-healing-amount"].includes(node.kind))
        node.values.value = 50;
      if (node.kind === "skill-range") {
        node.values.includeAnchor = true;
        node.values.pattern =
          node.id === footprint ? "@X" : "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX";
      }
    }
    canonicalize(doc);
    await importGraph(page, doc);
    await expect.poll(() => model.state()?.doc).toEqual(doc);
    const power = doc.nodes.find(
      (n) => n.kind === (heal ? "skill-healing-amount" : "skill-damage-amount"),
    )!;
    const label = heal ? "회복 요청량" : "피해 위력";
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(power.id);
    const edit = page.getByTestId(`rf__node-${power.id}`).getByRole("button", {
      name: `${power.title} ${label} 수정`,
      exact: true,
    });
    await edit.focus();
    await page.keyboard.press("Enter");
    await page
      .getByRole("dialog")
      .getByLabel(label, { exact: true })
      .fill("75");
    await page.getByRole("button", { name: "값 적용", exact: true }).click();
    await expect(edit).toBeFocused();
    await expect
      .poll(
        () =>
          model.state().doc.nodes.find((n) => n.id === power.id)?.values.value,
      )
      .toBe(75);
    await capture(page, info, `${name}-edited`);
    await inspectLayout(page, info, `${name}-edited`);
    await inspectAccessibility(page, info, `${name}-edited`);
    const before = model.state();
    const pending = page.waitForResponse((r) =>
      r.url().endsWith("/__content/compile"),
    );
    await begin(page);
    const result = await (await pending).json();
    expect(result.status).toBe("compiled");
    for (const value of [
      "ResonanceGround",
      "amount: 75",
      "interval_ms: 100",
      "duration_ms: 300",
      "@X",
    ])
      expect(result.ron).toContain(value);
    expect(model.state()).toEqual(before);
    await capture(page, info, `${name}-compiled`);
    await inspectLayout(page, info, `${name}-compiled`, { modal: true });
    await inspectAccessibility(page, info, `${name}-compiled`);
    await page.keyboard.press("Escape");
    await expect(
      page.getByRole("button", { name: "컴파일", exact: true }),
    ).toBeFocused();
    const exported = page.waitForEvent("download");
    await page
      .getByRole("button", { name: "JSON 내보내기", exact: true })
      .click();
    const saved = JSON.parse(
      await readFile((await (await exported).path())!, "utf8"),
    );
    expect(saved).toEqual(before.doc);
    await page
      .getByRole("button", { name: "다운로드 확인", exact: true })
      .click();
    await page
      .getByRole("button", { name: "노드 실행 취소", exact: true })
      .click();
    await expect.poll(() => model.state().doc).toEqual(doc);
    const undone = page.waitForResponse((r) =>
      r.url().endsWith("/__content/compile"),
    );
    await begin(page);
    expect((await (await undone).json()).ron).toContain("amount: 50");
    await page.keyboard.press("Escape");
    await importGraph(page, saved);
    await expect.poll(() => model.state().doc).toEqual(saved);
    expect(model.state().dirty).toBe(false);
    const reopened = page.waitForResponse((r) =>
      r.url().endsWith("/__content/compile"),
    );
    await begin(page);
    expect((await (await reopened).json()).ron).toContain("amount: 75");
  });
}

test("Core compiler: ground status edit preserves lifetime, overlap and dwell through RON", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/ground-status.node-graph.json", "utf8"),
  );
  const dwell = doc.nodes.find((n) => n.kind === "skill-ground-dwell")!;
  const dwellDuration = doc.edges.find(
    (e) => e.target.node === dwell.id && e.target.port === "duration",
  )!.source.node;
  for (const node of doc.nodes) {
    if (node.kind === "skill-resonance-amount") node.values.value = 100;
    if (node.kind === "skill-seconds")
      node.values.value = node.id === dwellDuration ? 0.1 : 0.3;
    if (node.kind === "skill-range") {
      node.values.includeAnchor = true;
      node.values.pattern = "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX";
    }
    if (node.kind === "skill-ground-status") {
      node.values.lifetime = "membership";
      if (node.values.effect === "slow") {
        node.values.percent = 20;
        node.values.overlap = "sum";
      }
    }
  }
  canonicalize(doc);
  await importGraph(page, doc);
  const slow = doc.nodes.find(
    (n) => n.kind === "skill-ground-status" && n.values.effect === "slow",
  )!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(slow.id);
  const edit = page.getByTestId(`rf__node-${slow.id}`).getByRole("button", {
    name: `${slow.title} 효과 크기 (%) 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel("효과 크기 (%)", { exact: true })
    .fill("35");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === slow.id)?.values.percent,
    )
    .toBe(35);
  await capture(page, info, "ground-status-edited");
  await inspectLayout(page, info, "ground-status-edited");
  await inspectAccessibility(page, info, "ground-status-edited");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  for (const value of [
    "ResonanceGroundStatus",
    "percent: 35",
    "duration_ms: 100",
    "UntilExit",
    "Sum",
    "Stun",
  ])
    expect(result.ron).toContain(value);
  expect(model.state()).toEqual(before);
  await capture(page, info, "ground-status-compiled");
  await inspectLayout(page, info, "ground-status-compiled", { modal: true });
  await inspectAccessibility(page, info, "ground-status-compiled");
  await page.keyboard.press("Escape");
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const undone = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await undone).json()).ron).toContain("percent: 20");
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  await expect.poll(() => model.state().doc).toEqual(saved);
  expect(model.state().dirty).toBe(false);
  const reopened = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await reopened).json()).ron).toContain("percent: 35");
});

test("Core compiler: finite strikes edit preserves target HP scaling and equipped modifiers", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/warm-hearted-woodsman.node-graph.json", "utf8"),
  );
  const stun = doc.nodes.find((n) => n.kind === "skill-hit-status")!;
  const stunDuration = doc.edges.find(
    (e) => e.target.node === stun.id && e.target.port === "duration",
  )!.source.node;
  for (const node of doc.nodes) {
    if (node.kind === "skill-resonance-amount") node.values.value = 100;
    if (node.kind === "skill-seconds" && node.values.value === "")
      node.values.value = node.id === stunDuration ? 0.3 : 0.1;
    if (node.kind === "skill-range") {
      node.values.includeAnchor = true;
      node.values.pattern = "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX";
    }
    if (node.kind === "skill-weapon-power") node.values.coefficient = 1;
    if (node.kind === "skill-missing-hp-power") {
      node.values.percent = 100;
      node.values.cap = 100;
    }
    if (node.kind === "skill-hit-pull") node.values.tiles = 1;
    if (node.kind === "skill-modifier-control-vulnerability")
      node.values.percent = 20;
  }
  canonicalize(doc);
  await importGraph(page, doc);
  const power = doc.nodes.find((n) => n.kind === "skill-missing-hp-power")!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(power.id);
  const edit = page.getByTestId(`rf__node-${power.id}`).getByRole("button", {
    name: `${power.title} HP 0% 기준 증가율 (%) 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel("HP 0% 기준 증가율 (%)", { exact: true })
    .fill("150");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === power.id)?.values.percent,
    )
    .toBe(150);
  await capture(page, info, "finite-strikes-edited");
  await inspectLayout(page, info, "finite-strikes-edited");
  await inspectAccessibility(page, info, "finite-strikes-edited");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  for (const value of [
    "ResonanceStrike",
    "missing_hp",
    "150",
    "deep-wedge",
    "long-breath",
    "ControlVulnerability",
    "ProtectInterruption",
  ])
    expect(result.ron).toContain(value);
  expect(model.state()).toEqual(before);
  await capture(page, info, "finite-strikes-compiled");
  await inspectLayout(page, info, "finite-strikes-compiled", { modal: true });
  await inspectAccessibility(page, info, "finite-strikes-compiled");
  await page.keyboard.press("Escape");
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const undone = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await undone).json()).status).toBe("compiled");
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  await expect.poll(() => model.state().doc).toEqual(saved);
  expect(model.state().dirty).toBe(false);
  const reopened = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await reopened).json()).ron).toBe(result.ron);
});

test("Core compiler: approach taunt preserves independent weapon growth through edit undo and reopen", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile(
      "content/mountain-of-smiling-bodies.node-graph.json",
      "utf8",
    ),
  );
  const growth = doc.nodes.find((n) => n.kind === "skill-weapon-kill-growth")!;
  const rangeModifier = doc.nodes.find(
    (n) => n.kind === "skill-modifier-range",
  )!;
  const replacement = doc.edges.find(
    (e) =>
      e.target.node === rangeModifier.id && e.target.port === "replacement",
  )!.source.node;
  for (const node of doc.nodes) {
    if (node.kind === "skill-resonance-amount") node.values.value = 100;
    if (node.kind === "skill-seconds") node.values.value = 1.5;
    if (node.kind === "skill-range") {
      node.values.includeAnchor = true;
      node.values.pattern =
        node.id === replacement
          ? "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX"
          : "XXX\nX@X\nXXX";
    }
    if (node.kind === "skill-target-speed") node.values.percent = 100;
    if (node.kind === "skill-modifier-taunt-damage")
      node.values.percent = node.values.mode === "taken" ? 20 : 30;
    if (node.kind === "skill-weapon-kill-growth") {
      node.values.damage = 10;
      node.values.secondaryPercent = 40;
    }
    if (node.kind === "skill-integer-amount") {
      const port = doc.edges.find(
        (e) => e.source.node === node.id && e.target.node === growth.id,
      )!.target.port;
      node.values.value = port === "normal" ? 2 : port === "elite" ? 5 : 9;
    }
  }
  canonicalize(doc);
  await importGraph(page, doc);
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(growth.id);
  const edit = page.getByTestId(`rf__node-${growth.id}`).getByRole("button", {
    name: `${growth.title} 스택당 주 대상 추가 피해 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel("스택당 주 대상 추가 피해", { exact: true })
    .fill("12");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === growth.id)?.values.damage,
    )
    .toBe(12);
  await capture(page, info, "taunt-weapon-growth-edited");
  await inspectLayout(page, info, "taunt-weapon-growth-edited");
  await inspectAccessibility(page, info, "taunt-weapon-growth-edited");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  for (const value of [
    "ResonanceTaunt",
    "weapon_growth",
    "laughter",
    "damage_per_stack: 12",
    "double-harvest",
    "TauntDamage",
    "long-breath",
  ])
    expect(result.ron).toContain(value);
  expect(model.state()).toEqual(before);
  await capture(page, info, "taunt-weapon-growth-compiled");
  await inspectLayout(page, info, "taunt-weapon-growth-compiled", {
    modal: true,
  });
  await inspectAccessibility(page, info, "taunt-weapon-growth-compiled");
  await page.keyboard.press("Escape");
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const undone = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await undone).json()).status).toBe("compiled");
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  await expect.poll(() => model.state().doc).toEqual(saved);
  expect(model.state().dirty).toBe(false);
  const reopened = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await reopened).json()).ron).toBe(result.ron);
});

test("Core compiler: channel ramp preserves ownership through edit undo and reopen", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/burrowing-heaven.node-graph.json", "utf8"),
  );
  const channel = doc.nodes.find((n) => n.kind === "skill-channel-begin")!;
  const ramp = doc.nodes.find((n) => n.kind === "skill-ground-ramping-slow")!;
  const modifier = doc.nodes.find((n) => n.kind === "skill-modifier-range")!;
  const source = (node: string, port: string) =>
    doc.edges.find((e) => e.target.node === node && e.target.port === port)!
      .source.node;
  const lifetime = source(channel.id, "duration");
  const progress = source(ramp.id, "duration");
  const replacement = source(modifier.id, "replacement");
  for (const n of doc.nodes) {
    if (n.kind === "skill-resonance-amount") n.values.value = 100;
    if (n.kind === "skill-seconds" && n.id === lifetime) n.values.value = 0.6;
    if (n.kind === "skill-seconds" && n.id === progress) n.values.value = 0.2;
    if (n.kind === "skill-range") {
      n.values.includeAnchor = true;
      n.values.pattern =
        n.id === replacement
          ? "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX"
          : "XXX\nX@X\nXXX";
    }
    if (n.kind === "skill-ground-ramping-slow") n.values.maxSlow = 60;
    if (n.kind === "skill-modifier-ramp") n.values.factor = 2;
    if (n.kind === "skill-modifier-control-vulnerability")
      n.values.percent = 30;
  }
  canonicalize(doc);
  await importGraph(page, doc);
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(ramp.id);
  const edit = page.getByTestId(`rf__node-${ramp.id}`).getByRole("button", {
    name: `${ramp.title} 최대 둔화율 (%) 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel("최대 둔화율 (%)", { exact: true })
    .fill("80");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === ramp.id)?.values.maxSlow,
    )
    .toBe(80);
  await capture(page, info, "channel-ramp-edited");
  await inspectLayout(page, info, "channel-ramp-edited");
  await inspectAccessibility(page, info, "channel-ramp-edited");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  for (const text of [
    "ResonanceChannelGround",
    "RampingSlow",
    "max_percent: 80",
    "tightening-shackles",
    "ControlVulnerability",
    "long-breath",
  ])
    expect(result.ron).toContain(text);
  expect(model.state()).toEqual(before);
  await capture(page, info, "channel-ramp-compiled");
  await inspectLayout(page, info, "channel-ramp-compiled", { modal: true });
  await inspectAccessibility(page, info, "channel-ramp-compiled");
  await page.keyboard.press("Escape");
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const undone = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const undoResult = await (await undone).json();
  expect(undoResult.status).toBe("compiled");
  expect(undoResult.ron).toContain("max_percent: 60");
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  await expect.poll(() => model.state().doc).toEqual(saved);
  const reopened = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await reopened).json()).ron).toBe(result.ron);
});

test("Core compiler: presence power stays independent from periodic damage through edit undo and reopen", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/melting-love.node-graph.json", "utf8"),
  );
  for (const n of doc.nodes) {
    if (n.kind === "skill-resonance-amount") n.values.value = 100;
    if (n.kind === "skill-seconds") n.values.value = 1;
    if (n.kind === "skill-population-power")
      Object.assign(n.values, { growth: 10, cap: 100, decay: 2 });
    if (n.kind === "skill-mark-spread")
      Object.assign(n.values, { radius: 2, limit: 1 });
    if (n.kind === "skill-mark-weapon-hit") n.values.chance = 100;
    if (n.kind === "skill-modifier-duration") n.values.percent = -50;
    if (n.kind === "skill-modifier-explosion-power") n.values.percent = 20;
    if (n.kind === "skill-modifier-population" && n.values.mode === "growth")
      n.values.percent = 50;
  }
  canonicalize(doc);
  await importGraph(page, doc);
  const power = doc.nodes.find((n) => n.kind === "skill-population-power")!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(power.id);
  const edit = page.getByTestId(`rf__node-${power.id}`).getByRole("button", {
    name: `${power.title} 숙주당 초당 성장 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel("숙주당 초당 성장", { exact: true })
    .fill("12");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === power.id)?.values.growth,
    )
    .toBe(12);
  await capture(page, info, "presence-power-edited");
  await inspectLayout(page, info, "presence-power-edited");
  await inspectAccessibility(page, info, "presence-power-edited");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  for (const text of [
    "ResonancePresenceBurst",
    "growth_per_second",
    "numerator: 12",
    "periodic_interval_ms: 1000",
    "host_multiplier",
    "PreservePower",
    "overlapping-explosions",
  ])
    expect(result.ron).toContain(text);
  expect(model.state()).toEqual(before);
  await capture(page, info, "presence-power-compiled");
  await inspectLayout(page, info, "presence-power-compiled", { modal: true });
  await inspectAccessibility(page, info, "presence-power-compiled");
  await page.keyboard.press("Escape");
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const undone = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const undoResult = await (await undone).json();
  expect(undoResult.status).toBe("compiled");
  expect(undoResult.ron).toContain("numerator: 10");
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  await expect.poll(() => model.state().doc).toEqual(saved);
  const reopened = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await reopened).json()).ron).toBe(result.ron);
});

test("Core compiler: butterfly orb keeps its actual equipment condition through edit undo and reopen", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile(
      "content/funeral-of-dying-butterflies-orb.node-graph.json",
      "utf8",
    ),
  );
  for (const n of doc.nodes) {
    if (n.kind === "skill-resonance-amount") n.values.value = 100;
    if (n.kind === "skill-damage-amount") n.values.value = 10;
    if (n.kind === "skill-seconds")
      n.values.value = n.title.includes("연출") ? 0.5 : 0.1;
    if (
      n.kind === "skill-mark-weapon-hit" &&
      n.values.binding === "weapon-base"
    )
      n.values.amount = 2;
    if (n.kind === "skill-stack-grade")
      Object.assign(n.values, { divisor: 3, rankBonus: 50 });
    if (n.kind === "skill-modifier-explosion-power") n.values.percent = 50;
    if (n.kind === "skill-burst-conversion")
      Object.assign(n.values, { threshold: 6, perStack: 2 });
    if (n.kind === "skill-range" && n.title.includes("구체"))
      Object.assign(n.values, {
        pattern: "XXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX",
        includeAnchor: true,
      });
  }
  canonicalize(doc);
  await importGraph(page, doc);
  const conversion = doc.nodes.find(
    (n) => n.kind === "skill-burst-conversion",
  )!;
  const condition = doc.nodes.find(
    (n) =>
      n.kind === "skill-equipment-condition" &&
      n.values.definition === "funeral-butterfly-orb",
  )!;
  expect(
    doc.edges.some(
      (e) =>
        e.source.node === condition.id &&
        e.target.node === conversion.id &&
        e.target.port === "condition",
    ),
  ).toBe(true);
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(conversion.id);
  const card = page.getByTestId(`rf__node-${conversion.id}`);
  const presentation = doc.nodes.find(
    (n) => n.kind === "skill-seconds" && n.title.includes("구체"),
  )!;
  const range = doc.nodes.find(
    (n) => n.kind === "skill-range" && n.title.includes("구체"),
  )!;
  for (const [a, b] of [
    [condition, presentation],
    [presentation, range],
  ]) {
    const first = await page.getByTestId(`rf__node-${a.id}`).boundingBox();
    const second = await page.getByTestId(`rf__node-${b.id}`).boundingBox();
    expect(first).not.toBeNull();
    expect(second).not.toBeNull();
    expect(
      first!.y + first!.height <= second!.y ||
        second!.y + second!.height <= first!.y ||
        first!.x + first!.width <= second!.x ||
        second!.x + second!.width <= first!.x,
    ).toBe(true);
  }
  await expect(card).toContainText("대체 폭발의 장착 조건");
  const edit = card.getByRole("button", {
    name: `${conversion.title} 총 스택 임계치 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel("총 스택 임계치", { exact: true })
    .fill("9");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === conversion.id)?.values
          .threshold,
    )
    .toBe(9);
  await capture(page, info, "butterfly-orb-edited");
  await inspectLayout(page, info, "butterfly-orb-edited");
  await inspectAccessibility(page, info, "butterfly-orb-edited");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  for (const text of [
    "ResonanceTileBurst",
    "threshold: 9",
    "funeral-butterfly-orb",
    "presentation_ms: 500",
    "evolution: (",
    "paired-pistols",
    "minigun",
  ])
    expect(result.ron).toContain(text);
  expect(model.state()).toEqual(before);
  await capture(page, info, "butterfly-orb-compiled");
  await inspectLayout(page, info, "butterfly-orb-compiled", { modal: true });
  await inspectAccessibility(page, info, "butterfly-orb-compiled");
  await page.keyboard.press("Escape");
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  await importGraph(page, saved);
  await expect.poll(() => model.state().doc).toEqual(saved);
  const reopened = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await reopened).json()).ron).toBe(result.ron);
});

test("Core compiler: counter menu phases preserve their clock through edit undo and reopen", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/queen-of-hatred-bloom.node-graph.json", "utf8"),
  );
  for (const n of doc.nodes) {
    if (n.kind === "skill-resonance-amount") n.values.value = 100;
    if (n.kind === "skill-range")
      Object.assign(n.values, {
        pattern: "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX",
        includeAnchor: true,
      });
    if (n.kind === "skill-seconds" && typeof n.values.value !== "number")
      n.values.value = n.title.includes("히스테릭") ? 1 : 0.3;
    if (n.kind === "skill-ground-pull") n.values.tiles = 0.5;
    if (n.kind === "skill-damage-amount") n.values.value = 10;
    if (n.kind === "skill-healing-amount") n.values.value = 8;
    if (["skill-count-damage", "skill-count-healing"].includes(n.kind))
      n.values.percent = 25;
    if (n.kind === "skill-modifier-duration") n.values.percent = -20;
    if (n.kind === "skill-modifier-healing") n.values.percent = 25;
    if (n.kind === "skill-attribute-effect")
      Object.assign(n.values, {
        attribute: "attack",
        operation: "percent",
        amount: 10,
      });
  }
  canonicalize(doc);
  await importGraph(page, doc);
  const timer = doc.nodes.find(
    (n) => n.kind === "skill-seconds" && n.values.value === 15,
  )!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(timer.id);
  const edit = page.getByTestId(`rf__node-${timer.id}`).getByRole("button", {
    name: `${timer.title} 시간 (초) 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel("시간 (초)", { exact: true })
    .fill("12");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === timer.id)?.values.value,
    )
    .toBe(12);
  await capture(page, info, "counter-clock-edited");
  await inspectLayout(page, info, "counter-clock-edited");
  await inspectAccessibility(page, info, "counter-clock-edited");
  const receipt = doc.nodes.find((n) => n.kind === "skill-counter-receipt")!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(receipt.id);
  await expect(page.getByTestId(`rf__node-${receipt.id}`)).toContainText(
    "소비",
  );
  await capture(page, info, "counter-menu-receipt");
  await inspectLayout(page, info, "counter-menu-receipt");
  await inspectAccessibility(page, info, "counter-menu-receipt");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  for (const text of [
    "ResonanceCounterChannel",
    "interval_ms: 12000",
    "capacity: 6",
    "Consumed",
    "SelectedTile",
    "duration_ms: 3000",
    "EndAll",
    "damage_and_healing_bonus",
    "decrease: true",
  ])
    expect(result.ron).toContain(text);
  expect(model.state()).toEqual(before);
  await capture(page, info, "counter-menu-compiled");
  await inspectLayout(page, info, "counter-menu-compiled", { modal: true });
  await inspectAccessibility(page, info, "counter-menu-compiled");
  await page.keyboard.press("Escape");
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const undone = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const undoResult = await (await undone).json();
  expect(undoResult.status).toBe("compiled");
  expect(undoResult.ron).toContain("interval_ms: 15000");
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  await expect.poll(() => model.state().doc).toEqual(saved);
  const reopened = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await reopened).json()).ron).toBe(result.ron);
});

test("Core compiler: attack overlay preserves reinput and equipped piercing through edit undo and reopen", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/hello-goodbye.node-graph.json", "utf8"),
  );
  for (const n of doc.nodes) {
    if (n.kind === "skill-resonance-amount") n.values.value = 100;
    if (n.kind === "skill-seconds") n.values.value = 2;
    if (n.kind === "skill-attack-profile")
      Object.assign(n.values, { length: 6.5, width: 0.5, power: 40 });
    if (n.kind === "skill-time-power")
      Object.assign(n.values, {
        base: 10,
        rate: 20,
        cap: 45,
        curve: "capped-linear",
      });
    if (n.kind === "skill-area")
      Object.assign(n.values, {
        pattern: "XXX\nXXX\nX@X",
        includeAnchor: true,
      });
    if (n.kind === "skill-modifier-piercing")
      Object.assign(n.values, { step: 25, cap: 50, curve: "capped-linear" });
  }
  canonicalize(doc);
  await importGraph(page, doc);
  const lookup = doc.nodes.find((n) => n.kind === "skill-lookup")!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(lookup.id);
  await capture(page, info, "overlay-input-branches");
  await inspectLayout(page, info, "overlay-input-branches");
  await inspectAccessibility(page, info, "overlay-input-branches");
  const rule = doc.nodes.find((n) => n.kind === "skill-modifier-piercing")!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(rule.id);
  const edit = page.getByTestId(`rf__node-${rule.id}`).getByRole("button", {
    name: `${rule.title} 앞선 관통당 증가 (%p) 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel("앞선 관통당 증가 (%p)", { exact: true })
    .fill("35");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await expect
    .poll(
      () => model.state().doc.nodes.find((n) => n.id === rule.id)?.values.step,
    )
    .toBe(35);
  await capture(page, info, "overlay-piercing-edited");
  await inspectLayout(page, info, "overlay-piercing-edited");
  await inspectAccessibility(page, info, "overlay-piercing-edited");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  for (const text of [
    "ResonanceAttackOverlay",
    "duration_ms: 2000",
    "length_units: 6500000",
    "increment_percent: 35",
    "piercing-damage-increase",
    "per_second: 20",
    "Physical",
  ])
    expect(result.ron).toContain(text);
  expect(model.state()).toEqual(before);
  await capture(page, info, "overlay-compiled");
  await inspectLayout(page, info, "overlay-compiled", { modal: true });
  await inspectAccessibility(page, info, "overlay-compiled");
  await page.keyboard.press("Escape");
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const undone = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const undoResult = await (await undone).json();
  expect(undoResult.status).toBe("compiled");
  expect(undoResult.ron).toContain("increment_percent: 25");
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  await expect.poll(() => model.state().doc).toEqual(saved);
  const reopened = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await reopened).json()).ron).toBe(result.ron);
});

test("Core compiler: relation action costs and timed damage survive edit undo and reopen", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  for (const bloom of [false, true]) {
    const doc: GraphDocument = JSON.parse(
      await readFile(
        `content/knight-of-despair${bloom ? "-bloom" : ""}.node-graph.json`,
        "utf8",
      ),
    );
    for (const n of doc.nodes) {
      const v = n.values;
      if (n.kind === "skill-resonance-amount") v.value = 100;
      if (
        n.kind === "skill-range" &&
        (!v.pattern || String(v.pattern).trim() === "")
      )
        Object.assign(v, {
          pattern: "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX",
          includeAnchor: true,
        });
      if (n.kind === "skill-seconds" && typeof v.value !== "number")
        v.value = n.title.includes("검 생성")
          ? 1
          : n.title.includes("총시간")
            ? 0.6
            : n.title.includes("기본 시간")
              ? 2
              : n.title.includes("추가 시간")
                ? 0.5
                : 0.1;
      if (n.kind === "skill-damage-amount") v.value = 100;
      if (n.kind === "skill-count-positive") v.value = 3;
      if (n.kind === "skill-relation-reduction") v.percent = 25;
      if (n.kind === "skill-modifier-duration") v.percent = -20;
      if (n.kind === "skill-modifier-relation-shield") v.percent = 50;
      if (n.kind === "skill-modifier-control-vulnerability") v.percent = 10;
    }
    canonicalize(doc);
    await importGraph(page, doc);
    if (!bloom) {
      const relation = doc.nodes.find((n) => n.kind === "skill-ally-relation")!;
      await page
        .getByLabel("노드 또는 구간으로 이동")
        .selectOption(relation.id);
      await capture(page, info, "relation-menu-lifetime");
      await inspectLayout(page, info, "relation-menu-lifetime");
      await inspectAccessibility(page, info, "relation-menu-lifetime");
    }
    const node = doc.nodes.find(
      (n) =>
        n.kind ===
        (bloom ? "skill-action-periodic-self-damage" : "skill-count-quotient"),
    )!;
    const label = bloom ? "최대 HP 피해 (%)" : "나눌 수";
    const field = bloom ? "percent" : "divisor";
    const value = bloom ? 12 : 5;
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(node.id);
    const edit = page.getByTestId(`rf__node-${node.id}`).getByRole("button", {
      name: `${node.title} ${label} 수정`,
      exact: true,
    });
    await edit.focus();
    await page.keyboard.press("Enter");
    await page
      .getByRole("dialog")
      .getByLabel(label, { exact: true })
      .fill(String(value));
    await page.getByRole("button", { name: "값 적용", exact: true }).click();
    await expect(edit).toBeFocused();
    await expect
      .poll(
        () =>
          model.state().doc.nodes.find((n) => n.id === node.id)?.values[field],
      )
      .toBe(value);
    const name = bloom ? "relation-timed-self-damage" : "relation-stage-cost";
    await capture(page, info, name);
    await inspectLayout(page, info, name);
    await inspectAccessibility(page, info, name);
    const before = model.state();
    const pending = page.waitForResponse((r) =>
      r.url().endsWith("/__content/compile"),
    );
    await begin(page);
    const result = await (await pending).json();
    expect(result.status).toBe("compiled");
    expect(result.ron).toContain("ResonanceRelationAction");
    expect(result.ron).toContain(bloom ? "numerator: 12" : "divisor: 5");
    expect(model.state()).toEqual(before);
    if (!bloom) {
      await capture(page, info, "relation-compiled");
      await inspectLayout(page, info, "relation-compiled", { modal: true });
      await inspectAccessibility(page, info, "relation-compiled");
    }
    await page.keyboard.press("Escape");
    const exported = page.waitForEvent("download");
    await page
      .getByRole("button", { name: "JSON 내보내기", exact: true })
      .click();
    const saved = JSON.parse(
      await readFile((await (await exported).path())!, "utf8"),
    );
    expect(saved).toEqual(before.doc);
    await page
      .getByRole("button", { name: "다운로드 확인", exact: true })
      .click();
    await page
      .getByRole("button", { name: "노드 실행 취소", exact: true })
      .click();
    await expect.poll(() => model.state().doc).toEqual(doc);
    const undone = page.waitForResponse((r) =>
      r.url().endsWith("/__content/compile"),
    );
    await begin(page);
    const undoResult = await (await undone).json();
    expect(undoResult.status).toBe("compiled");
    expect(undoResult.ron).not.toBe(result.ron);
    await page.keyboard.press("Escape");
    await importGraph(page, saved);
    const reopened = page.waitForResponse((r) =>
      r.url().endsWith("/__content/compile"),
    );
    await begin(page);
    expect((await (await reopened).json()).ron).toBe(result.ron);
    await page.keyboard.press("Escape");
  }
});

test("Core compiler: input plan volley keeps branch intent through edit undo and reopen", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/little-red.node-graph.json", "utf8"),
  );
  for (const n of doc.nodes) {
    const v = n.values;
    if (n.kind === "skill-resonance-amount") v.value = 100;
    if (n.kind === "skill-stock") v.capacity = 8;
    if (n.kind === "skill-range")
      Object.assign(v, {
        pattern: "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX",
        includeAnchor: true,
      });
    if (n.kind === "skill-seconds" && typeof v.value !== "number")
      v.value = n.title.includes("출현") ? 5 : n.title.includes("비행") ? 3 : 1;
    if (n.kind === "skill-damage-amount") v.value = 100;
    if (n.kind === "skill-damage-bonus") v.bonus = 50;
    if (n.kind === "skill-hostile-profile")
      Object.assign(v, {
        hp: 300,
        power: 20,
        speed: 1,
        physicalDefense: 0,
        magicDefense: 0,
        leak: 1,
      });
    if (n.kind === "skill-projectile-profile")
      Object.assign(v, { speed: 5, range: 10, width: 0.5, hits: 3 });
    if (n.kind === "skill-modifier-projectile-wound") v.cap = 5;
    if (n.kind === "skill-modifier-projectile-push") v.tiles = 1;
    if (n.kind === "skill-modifier-projectile-explosion") v.air = "include";
    if (n.kind === "skill-explosion-rule") v.damageType = "magic";
    if (n.kind === "skill-modifier-control-vulnerability") v.percent = 10;
  }
  canonicalize(doc);
  await importGraph(page, doc);
  await expect.poll(() => model.state().doc).toEqual(doc);
  for (const [kind, name] of [
    ["skill-plan-aim-mode", "volley-aim-branches"],
    ["skill-plan-commit", "volley-atomic-commit"],
  ]) {
    const node = doc.nodes.find((n) => n.kind === kind)!;
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(node.id);
    await capture(page, info, name);
    await inspectLayout(page, info, name);
    await inspectAccessibility(page, info, name);
  }
  const node = doc.nodes.find((n) => n.kind === "skill-projectile-profile")!;
  const label = "비행 속도 (타일/초)";
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(node.id);
  const edit = page
    .getByTestId(`rf__node-${node.id}`)
    .getByRole("button", { name: `${node.title} ${label} 수정`, exact: true });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page.getByRole("dialog").getByLabel(label, { exact: true }).fill("7");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await capture(page, info, "volley-flight-speed");
  await inspectLayout(page, info, "volley-flight-speed");
  await inspectAccessibility(page, info, "volley-flight-speed");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  expect(result.ron).toContain("ResonanceInputPlan");
  expect(result.ron).toContain("numerator: 7");
  expect(model.state()).toEqual(before);
  await capture(page, info, "volley-compiled");
  await inspectLayout(page, info, "volley-compiled", { modal: true });
  await inspectAccessibility(page, info, "volley-compiled");
  await page.keyboard.press("Escape");
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const undone = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await undone).json()).ron).not.toBe(result.ron);
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  const reopened = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await reopened).json()).ron).toBe(result.ron);
});

test("Core compiler: prepared volley collision width preserves serial release through edit undo and reopen", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/king-of-greed.node-graph.json", "utf8"),
  );
  for (const n of doc.nodes) {
    const v = n.values;
    if (n.kind === "skill-resonance-amount") v.value = 100;
    if (n.kind === "skill-portal-policy") v.maximum = 3;
    if (n.kind === "skill-range")
      Object.assign(v, {
        pattern: n.title.includes("선택")
          ? "XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX"
          : "@",
        includeAnchor: true,
      });
    if (n.kind === "skill-seconds" && typeof v.value !== "number")
      v.value = n.title.includes("회수")
        ? 5
        : n.title.includes("첫 포탈")
          ? 1
          : n.title.includes("마지막 포탈")
            ? 0.5
            : n.title.includes("발사 기한")
              ? 10
              : n.title.includes("유지 유예")
                ? 2
                : n.title.includes("비행")
                  ? 5
                  : n.title.includes("사출 간격")
                    ? 0.25
                    : 1;
    if (n.kind === "skill-damage-amount") v.value = 100;
    if (n.kind === "skill-projectile-profile")
      Object.assign(v, { speed: 5, range: 20, width: 0.5 });
  }
  canonicalize(doc);
  await importGraph(page, doc);
  await expect.poll(() => model.state().doc).toEqual(doc);
  for (const [kind, name] of [
    ["skill-portal-policy", "prepared-volley-policy"],
    ["skill-serial-projectiles", "prepared-volley-impact-barrier"],
  ]) {
    const node = doc.nodes.find((n) => n.kind === kind)!;
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(node.id);
    await capture(page, info, name);
    await inspectLayout(page, info, name);
    await inspectAccessibility(page, info, name);
  }
  const node = doc.nodes.find((n) => n.kind === "skill-projectile-profile")!;
  const label = "충돌 폭 (타일)";
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(node.id);
  const edit = page
    .getByTestId(`rf__node-${node.id}`)
    .getByRole("button", { name: `${node.title} ${label} 수정`, exact: true });
  const beforeFocus = model.state();
  await edit.focus();
  await expect
    .poll(async () => {
      const canvas = await page.getByTestId("node-canvas").boundingBox();
      const row = await edit.locator("..").boundingBox();
      return Boolean(
        canvas &&
        row &&
        row.y >= canvas.y &&
        row.y + row.height <= canvas.y + canvas.height &&
        row.x >= canvas.x &&
        row.x + row.width <= canvas.x + canvas.width,
      );
    })
    .toBe(true);
  expect(model.state()).toEqual(beforeFocus);
  await page.keyboard.press("Enter");
  await page
    .getByRole("dialog")
    .getByLabel(label, { exact: true })
    .fill("0.75");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await capture(page, info, "prepared-volley-collision-width");
  await inspectLayout(page, info, "prepared-volley-collision-width");
  await inspectAccessibility(page, info, "prepared-volley-collision-width");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  expect(result.ron).toContain("ResonancePreparedVolley");
  expect(result.ron).toContain("numerator: 3");
  expect(model.state()).toEqual(before);
  await capture(page, info, "prepared-volley-compiled");
  await inspectLayout(page, info, "prepared-volley-compiled", { modal: true });
  await inspectAccessibility(page, info, "prepared-volley-compiled");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "현재 문서 컴파일", exact: true })
    .focus();
  for (const key of ["Tab", "Tab", "Shift+Tab"]) {
    await page.keyboard.press(key);
    expect(
      await page
        .getByRole("dialog")
        .evaluate((dialog) => dialog.contains(document.activeElement)),
    ).toBe(true);
  }
  await page.keyboard.press("Escape");
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const undone = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await undone).json()).ron).not.toBe(result.ron);
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  const reopened = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await reopened).json()).ron).toBe(result.ron);
});

test("Core compiler: paired areas transfer preserves atomic selection and settlement through edit undo and reopen", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/yin-yang.node-graph.json", "utf8"),
  );
  for (const n of doc.nodes) {
    const v = n.values;
    if (n.kind === "skill-resonance-amount") v.value = 100;
    if (n.kind === "skill-range")
      Object.assign(v, {
        pattern: n.title.includes("선택")
          ? "XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX"
          : n.title.includes("확대")
            ? "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX"
            : "XXX\nX@X\nXXX",
        includeAnchor: true,
      });
    if (n.kind === "skill-seconds")
      v.value = n.title.includes("공통 수명")
        ? 3
        : n.title.includes("온기")
          ? 2
          : 1;
    if (n.kind === "skill-healing-amount") v.value = 100;
    if (n.kind === "skill-pair-transfer") v.percent = 50;
    if (n.kind === "skill-attack-speed-buff") v.points = 20;
    if (["skill-modifier-buff", "skill-modifier-healing"].includes(n.kind))
      v.percent = 50;
    if (n.kind === "skill-attribute-effect")
      Object.assign(v, {
        attribute: "attack",
        operation: "percent",
        amount: 10,
      });
  }
  canonicalize(doc);
  await importGraph(page, doc);
  await expect.poll(() => model.state().doc).toEqual(doc);
  for (const [kind, name] of [
    ["skill-paired-areas", "paired-areas-install"],
    ["skill-pair-base-heal", "paired-areas-base-settlement"],
  ]) {
    const node = doc.nodes.find((n) => n.kind === kind)!;
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(node.id);
    await capture(page, info, name);
    await inspectLayout(page, info, name);
    await inspectAccessibility(page, info, name);
  }
  const node = doc.nodes.find((n) => n.kind === "skill-pair-transfer")!;
  const label = "초과분 전달률 (%)";
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(node.id);
  const edit = page
    .getByTestId(`rf__node-${node.id}`)
    .getByRole("button", { name: `${node.title} ${label} 수정`, exact: true });
  const beforeFocus = model.state();
  await edit.focus();
  await expect
    .poll(async () => {
      const canvas = await page.getByTestId("node-canvas").boundingBox();
      const row = await edit.locator("..").boundingBox();
      return Boolean(
        canvas &&
        row &&
        row.y >= canvas.y &&
        row.y + row.height <= canvas.y + canvas.height &&
        row.x >= canvas.x &&
        row.x + row.width <= canvas.x + canvas.width,
      );
    })
    .toBe(true);
  expect(model.state()).toEqual(beforeFocus);
  await page.keyboard.press("Enter");
  await page.getByRole("dialog").getByLabel(label, { exact: true }).fill("25");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await capture(page, info, "paired-areas-transfer");
  await inspectLayout(page, info, "paired-areas-transfer");
  await inspectAccessibility(page, info, "paired-areas-transfer");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  expect(result.ron).toContain("ResonancePairedAreas");
  expect(result.ron).toMatch(/transfer_percent:\s*\(\s*numerator: 25,/);
  expect(model.state()).toEqual(before);
  await capture(page, info, "paired-areas-compiled");
  await inspectLayout(page, info, "paired-areas-compiled", { modal: true });
  await inspectAccessibility(page, info, "paired-areas-compiled");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "현재 문서 컴파일", exact: true })
    .focus();
  for (const key of ["Tab", "Tab", "Shift+Tab"]) {
    await page.keyboard.press(key);
    expect(
      await page
        .getByRole("dialog")
        .evaluate((dialog) => dialog.contains(document.activeElement)),
    ).toBe(true);
  }
  await page.keyboard.press("Escape");
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const undone = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await undone).json()).ron).not.toBe(result.ron);
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  const reopened = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await reopened).json()).ron).toBe(result.ron);
});

test("Core compiler: fusion placement preserves count scaling and both forms through edit undo and reopen", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/laetitia.node-graph.json", "utf8"),
  );
  const resolve = (graph: GraphDocument) => {
    for (const n of graph.nodes) {
      const v = n.values;
      if (n.kind === "skill-resonance-amount") v.value = 100;
      if (n.kind === "skill-range")
        Object.assign(v, {
          pattern: n.title.includes("선택")
            ? "XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX"
            : n.title.includes("설치 위치")
              ? ".X.\nX@X\n.X."
              : n.title.includes("작은 친구")
                ? "@"
                : n.title.includes("확대")
                  ? "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX"
                  : "XXX\nX@X\nXXX",
          includeAnchor: true,
        });
      if (n.kind === "skill-seconds" && n.title.includes("대기")) v.value = 5;
      if (n.kind === "skill-seconds" && n.title.includes("도약")) v.value = 0.1;
      if (n.kind === "skill-weapon-power") v.coefficient = 1;
      if (n.kind === "skill-damage-bonus") v.bonus = 100;
      if (
        [
          "skill-fusion-count-power",
          "skill-modifier-mobile-power",
          "skill-modifier-mobile-vulnerability",
        ].includes(n.kind)
      )
        v.percent = 50;
      if (n.kind === "skill-fusion-family")
        Object.assign(v, { maxUnits: 20, maxTiles: 8 });
      if (n.kind === "skill-mobile-effect") v.speed = 10;
    }
    canonicalize(graph);
  };
  resolve(doc);
  canonicalize(doc);
  await importGraph(page, doc);
  await expect.poll(() => model.state().doc).toEqual(doc);
  for (const [kind, name] of [
    ["skill-place-fusion-plan", "fusion-placement"],
    ["skill-fusion-family", "fusion-family"],
  ]) {
    const node = doc.nodes.find((n) => n.kind === kind)!;
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(node.id);
    await capture(page, info, name);
    await inspectLayout(page, info, name);
    await inspectAccessibility(page, info, name);
  }
  const node = doc.nodes.find((n) => n.kind === "skill-fusion-count-power")!;
  const label = "추가 단위당 증가율 (%)";
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(node.id);
  const edit = page
    .getByTestId(`rf__node-${node.id}`)
    .getByRole("button", { name: `${node.title} ${label} 수정`, exact: true });
  const beforeFocus = model.state();
  await edit.focus();
  await expect
    .poll(async () => {
      const canvas = await page.getByTestId("node-canvas").boundingBox();
      const row = await edit.locator("..").boundingBox();
      return Boolean(
        canvas &&
        row &&
        row.y >= canvas.y &&
        row.y + row.height <= canvas.y + canvas.height &&
        row.x >= canvas.x &&
        row.x + row.width <= canvas.x + canvas.width,
      );
    })
    .toBe(true);
  expect(model.state()).toEqual(beforeFocus);
  await page.keyboard.press("Enter");
  await page.getByRole("dialog").getByLabel(label, { exact: true }).fill("25");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await capture(page, info, "fusion-count-scaling");
  await inspectLayout(page, info, "fusion-count-scaling");
  await inspectAccessibility(page, info, "fusion-count-scaling");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  expect(result.ron).toContain("ResonanceFusionPlacement");
  expect(result.ron).toMatch(/baseline: 2,\s*percent:\s*\(\s*numerator: 25,/);
  expect(model.state()).toEqual(before);
  await capture(page, info, "fusion-compiled");
  await inspectLayout(page, info, "fusion-compiled", { modal: true });
  await inspectAccessibility(page, info, "fusion-compiled");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "현재 문서 컴파일", exact: true })
    .focus();
  for (const key of ["Tab", "Tab", "Shift+Tab"]) {
    await page.keyboard.press(key);
    expect(
      await page
        .getByRole("dialog")
        .evaluate((dialog) => dialog.contains(document.activeElement)),
    ).toBe(true);
  }
  await page.keyboard.press("Escape");
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const undone = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await undone).json()).ron).not.toBe(result.ron);
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  const reopened = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await reopened).json()).ron).toBe(result.ron);
  await page.keyboard.press("Escape");
  const bloom: GraphDocument = JSON.parse(
    await readFile("content/laetitia-bloom.node-graph.json", "utf8"),
  );
  resolve(bloom);
  await importGraph(page, bloom);
  const placement = bloom.nodes.find(
    (n) => n.kind === "skill-place-fusion-plan",
  )!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(placement.id);
  await capture(page, info, "fusion-bloom-placement");
  await inspectLayout(page, info, "fusion-bloom-placement");
  await inspectAccessibility(page, info, "fusion-bloom-placement");
  const blooming = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const bloomResult = await (await blooming).json();
  expect(bloomResult.status).toBe("compiled");
  expect(bloomResult.ron).toMatch(/primary: 2,\s*adjacent: 1,/);
});

test("Core compiler: status strike preserves pre-hit transition and chase settings through edit undo and reopen", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await nodes(page);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/snow-queen.node-graph.json", "utf8"),
  );
  const resolve = (graph: GraphDocument) => {
    for (const n of graph.nodes) {
      const v = n.values;
      if (n.kind === "skill-resonance-amount") v.value = 100;
      if (n.kind === "skill-count-positive") v.value = 8;
      if (n.kind === "skill-range")
        Object.assign(v, {
          pattern: n.title.includes("T자")
            ? "XXX\n.X.\n.@."
            : n.title.includes("탐색")
              ? "XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX"
              : n.title.includes("확대")
                ? "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX"
                : "@",
          includeAnchor: !n.title.includes("T자"),
        });
      if (n.kind === "skill-seconds" && n.title.includes("지연")) v.value = 0.2;
      if (n.kind === "skill-weapon-power") v.coefficient = 1;
      if (n.kind === "skill-mobile-effect") v.speed = 5;
      if (
        [
          "skill-modifier-control-vulnerability",
          "skill-modifier-mobile-power",
          "skill-modifier-mobile-vulnerability",
        ].includes(n.kind)
      )
        v.percent = 50;
    }
    canonicalize(graph);
  };
  resolve(doc);
  canonicalize(doc);
  await importGraph(page, doc);
  await expect.poll(() => model.state().doc).toEqual(doc);
  for (const [kind, name] of [
    ["skill-convert-status-plan", "status-transition"],
    ["skill-mobile-effect", "status-mobile"],
  ]) {
    const node = doc.nodes.find((n) => n.kind === kind)!;
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(node.id);
    await capture(page, info, name);
    await inspectLayout(page, info, name);
    await inspectAccessibility(page, info, name);
  }
  const node = doc.nodes.find(
    (n) => n.kind === "skill-reactive-status" && n.values.effect === "slow",
  )!;
  const label = "이동 둔화 (%)";
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(node.id);
  const edit = page
    .getByTestId(`rf__node-${node.id}`)
    .getByRole("button", { name: `${node.title} ${label} 수정`, exact: true });
  const beforeFocus = model.state();
  await edit.focus();
  await expect
    .poll(async () => {
      const canvas = await page.getByTestId("node-canvas").boundingBox();
      const row = await edit.locator("..").boundingBox();
      return Boolean(
        canvas &&
        row &&
        row.y >= canvas.y &&
        row.y + row.height <= canvas.y + canvas.height &&
        row.x >= canvas.x &&
        row.x + row.width <= canvas.x + canvas.width,
      );
    })
    .toBe(true);
  expect(model.state()).toEqual(beforeFocus);
  await page.keyboard.press("Enter");
  await page.getByRole("dialog").getByLabel(label, { exact: true }).fill("20");
  await page.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(edit).toBeFocused();
  await capture(page, info, "status-slow-edit");
  await inspectLayout(page, info, "status-slow-edit");
  await inspectAccessibility(page, info, "status-slow-edit");
  const before = model.state();
  const pending = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  const result = await (await pending).json();
  expect(result.status).toBe("compiled");
  expect(result.ron).toContain("ResonanceStatusStrike");
  expect(result.ron).toMatch(/Slow\(\s*percent: 20,/);
  expect(result.ron).toContain("ChaseDelay");
  expect(model.state()).toEqual(before);
  await capture(page, info, "status-compiled");
  await inspectLayout(page, info, "status-compiled", { modal: true });
  await inspectAccessibility(page, info, "status-compiled");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "현재 문서 컴파일", exact: true })
    .focus();
  for (const key of ["Tab", "Tab", "Shift+Tab"]) {
    await page.keyboard.press(key);
    expect(
      await page
        .getByRole("dialog")
        .evaluate((dialog) => dialog.contains(document.activeElement)),
    ).toBe(true);
  }
  await page.keyboard.press("Escape");
  const exported = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const saved = JSON.parse(
    await readFile((await (await exported).path())!, "utf8"),
  );
  expect(saved).toEqual(before.doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const undone = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await undone).json()).ron).not.toBe(result.ron);
  await page.keyboard.press("Escape");
  await importGraph(page, saved);
  const reopened = page.waitForResponse((r) =>
    r.url().endsWith("/__content/compile"),
  );
  await begin(page);
  expect((await (await reopened).json()).ron).toBe(result.ron);
  await page.keyboard.press("Escape");
});
