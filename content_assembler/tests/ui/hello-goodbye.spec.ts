import { spatialExample } from "../../src/nodes/core/spatial-examples";
import { readFile } from "node:fs/promises";
import type { Page, TestInfo } from "@playwright/test";
import type { GraphDocument } from "../../src/nodes/types";
import { definition } from "../../src/nodes/core/catalog";
import { canonicalize } from "../../src/nodes/core/model";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectLayout,
  inspectAccessibility,
} from "./fixtures";
import { observeNodes } from "./node-observer";

async function openSkill(page: Page, info: TestInfo) {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "노드", exact: true })
    .click();
  await page.getByRole("button", { name: "그래프 목록", exact: true }).click();
  await page
    .getByRole("dialog", { name: "그래프 목록", exact: true })
    .getByRole("button", { name: "hello-goodbye.node-graph.json", exact: true })
    .click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "교체", exact: true })
    .click();
  await expect
    .poll(() => model.state()?.doc.name)
    .toBe("아무 것도 없는 · Hello / Goodbye");
  await expect(page.getByTestId("node-canvas")).toBeVisible();
  return model;
}
async function focus(page: Page, doc: GraphDocument, title: string) {
  const node = [...doc.nodes, ...doc.frames].find((n) => n.title === title);
  if (!node) throw new Error(`missing ${title}`);
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(node.id);
  await expect(page.getByTestId(`rf__node-${node.id}`)).toBeInViewport();
  return node.id;
}
async function inspect(page: Page, info: TestInfo, name: string) {
  await capture(page, info, name);
  await inspectLayout(page, info, name);
  await inspectAccessibility(page, info, name);
}
async function save(page: Page): Promise<GraphDocument> {
  const pending = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const file = await pending;
  const doc = JSON.parse(await readFile((await file.path())!, "utf8"));
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "다운로드 확인", exact: true }),
  ).toHaveCount(0);
  return doc;
}

test("selected nodes highlight only their incident edges and preserve edge selection and history", async ({
  page,
}, info) => {
  const model = await openSkill(page, info);
  const before = model.state();
  const overlay = before.doc.nodes.find(
    (n) => n.kind === "skill-resonance-install",
  )!;
  const state = before.doc.nodes.find((n) => n.kind === "skill-state")!;
  const highlighted = () => page.locator(".ng-edge-connected");
  const expected = (ids: string[]) =>
    before.doc.edges
      .filter((e) => ids.includes(e.source.node) || ids.includes(e.target.node))
      .map((e) => e.id)
      .sort();
  const actual = () =>
    highlighted().evaluateAll((edges) =>
      edges.map((e) => e.getAttribute("data-id")!).sort(),
    );
  await focus(page, before.doc, overlay.title);
  const card = page.getByTestId(`rf__node-${overlay.id}`).locator(".ng-card");
  await card.click({ position: { x: 20, y: 15 } });
  await expect.poll(actual).toEqual(expected([overlay.id]));
  await expect(page.locator(".ng-edge.selected")).toHaveCount(0);
  const linked = before.doc.edges.find(
    (e) => e.target.node === overlay.id && e.target.port === "state",
  )!;
  const linkedPath = page
    .getByTestId(`rf__edge-${linked.id}`)
    .locator(".react-flow__edge-path");
  await expect(linkedPath).toHaveCSS("stroke-width", "3px");
  const selectedColor = await card.evaluate(
    (e) => getComputedStyle(e).borderTopColor,
  );
  await expect(linkedPath).toHaveCSS("stroke", selectedColor);
  const cueEdge = before.doc.edges.find(
    (e) => e.source.node === overlay.id && e.source.port === "cue",
  )!;
  await expect(
    page
      .getByTestId(`rf__edge-${cueEdge.id}`)
      .locator(".react-flow__edge-path"),
  ).toHaveCSS("stroke-dasharray", "6px, 4px");
  const unrelated = before.doc.edges.find(
    (e) => !expected([overlay.id]).includes(e.id),
  )!;
  const unrelatedSource = before.doc.nodes.find(
    (n) => n.id === unrelated.source.node,
  )!;
  const unrelatedFlow =
    definition(unrelatedSource.kind).outputs.find(
      (p) => p.id === unrelated.source.port,
    )?.valueType === "flow";
  await expect(
    page
      .getByTestId(`rf__edge-${unrelated.id}`)
      .locator(".react-flow__edge-path"),
  ).toHaveCSS("stroke-width", unrelatedFlow ? "3px" : "2px");
  await inspect(page, info, "node-connected-edges");
  await focus(page, before.doc, state.title);
  await expect.poll(actual).toEqual(expected([state.id]));
  await page.getByRole("button", { name: "전체 보기", exact: true }).click();
  await card.locator(".ng-drag-handle").click({ modifiers: ["Control"] });
  await expect.poll(actual).toEqual(expected([state.id, overlay.id]));
  await expect(page.getByLabel("노드 속성")).toContainText("2개 선택");
  await inspect(page, info, "multiple-node-connected-edges");
  await focus(page, before.doc, overlay.title);
  await expect.poll(actual).toEqual(expected([overlay.id]));
  await expect
    .poll(() =>
      page
        .locator(".react-flow__viewport")
        .evaluate(
          (element) => new DOMMatrix(getComputedStyle(element).transform).a,
        ),
    )
    .toBe(1);
  const flowEdge = before.doc.edges.find(
    (e) => e.target.node === overlay.id && e.target.port === "in",
  )!;
  const edge = page.getByTestId(`rf__edge-${flowEdge.id}`);
  const edgePath = edge.locator(".react-flow__edge-path");
  const clearSelection = async () => {
    await page.getByRole("button", { name: "전체 보기", exact: true }).click();
    const point = await page.locator(".react-flow__pane").evaluate((pane) => {
      const rect = pane.getBoundingClientRect();
      for (let y = rect.top + 10; y < rect.bottom - 10; y += 20)
        for (let x = rect.left + 10; x < rect.right - 10; x += 20)
          if (document.elementFromPoint(x, y) === pane) return { x, y };
      throw new Error("No visible canvas background");
    });
    await page.mouse.click(point.x, point.y);
  };
  const point = await edgePath.evaluate((element) => {
    const path = element as SVGPathElement;
    const edge = path.closest("[data-testid]");
    // The middle of a long edge may be outside the focused viewport. Click
    // the actual exposed interaction path, never a node or the side panel.
    for (let part = 5; part <= 95; part++) {
      const point = path
        .getPointAtLength((path.getTotalLength() * part) / 100)
        .matrixTransform(path.getScreenCTM()!);
      if (
        document
          .elementFromPoint(point.x, point.y)
          ?.closest("[data-testid]") === edge
      )
        return { x: point.x, y: point.y };
    }
    throw new Error("No exposed part of the selected flow connection");
  });
  await page.mouse.click(point.x, point.y);
  await expect(edge).toHaveClass(/selected/);
  await expect(highlighted()).toHaveCount(0);
  await expect(page.getByLabel("노드 속성")).toContainText("연결 속성");
  await expect(edgePath).toHaveCSS("stroke-width", "3px");
  await clearSelection();
  await expect(page.locator(".ng-edge.selected")).toHaveCount(0);
  await expect(edgePath).toHaveCSS("stroke-width", "3px");
  await card.focus();
  await card.press("Enter");
  await expect.poll(actual).toEqual(expected([overlay.id]));
  await clearSelection();
  await expect(highlighted()).toHaveCount(0);
  expect(model.state().doc).toEqual(before.doc);
  expect(model.state().undoCount).toBe(before.undoCount);
  expect(model.state().dirty).toBe(before.dirty);
});

test("skill roles distinguish declarations, actions and values without editing the graph", async ({
  page,
}, info) => {
  const model = await openSkill(page, info);
  const before = model.state();
  const inspector = page.getByRole("complementary", { name: "노드 속성" });
  for (const [title, role, screenshot] of [
    ["Hello 상태 선언", "상태 선언", "hello-state-role"],
    ["Hello 적용", "실행 동작", "hello-action-role"],
    ["Hello 유지 시간", "설정값", "hello-value-role"],
  ]) {
    const id = await focus(page, before.doc, title);
    const card = page
      .getByTestId(`rf__node-${id}`)
      .getByRole("group", { name: `노드 ${title}`, exact: true });
    await expect(card.locator(".ng-role-label")).toHaveText(role);
    await expect(inspector.locator(".ng-panel-heading small")).toHaveText(role);
    await card.focus();
    await card.press("Enter");
    await expect(card).toBeFocused();
    if (title === "Hello 상태 선언") {
      await expect(inspector).toContainText(
        "상태 참조 연결은 실행 순서가 아닙니다.",
      );
      await expect(inspector.locator(".ng-related")).toContainText("상태 참조");
    }
    await inspect(page, info, screenshot);
  }
  for (const [kind, role] of [
    ["skill-time-power", "값 계산"],
    ["skill-cue", "연출"],
    ["skill-request", "▶ 시작 사건"],
  ]) {
    const node = before.doc.nodes.find((n) => n.kind === kind)!;
    await expect(
      page.getByTestId(`rf__node-${node.id}`).locator(".ng-role-label"),
    ).toHaveText(role);
  }
  expect(model.state().doc).toEqual(before.doc);
  expect(model.state().dirty).toBe(before.dirty);
  expect(model.state().undoCount).toBe(before.undoCount);
});

test("state summary navigates to settings, protects drafts and follows edits and missing connections", async ({
  page,
}, info) => {
  const model = await openSkill(page, info);
  const before = model.state();
  const summary = page.getByRole("region", { name: "연결된 상태 구성" });
  await focus(page, before.doc, "Hello 상태 선언");
  await expect(summary).toContainText("적용 동작 · 1");
  await expect(summary).toContainText("조회·관찰하는 노드 · 4");
  await expect(summary).toContainText("값 미정");
  const jump = summary.getByRole("button", {
    name: "Hello 유지 시간 노드로 이동",
  });
  await jump.focus();
  await page.keyboard.press("Tab");
  await page.keyboard.press("Shift+Tab");
  await expect(jump).toBeFocused();
  await inspect(page, info, "state-summary-navigation");
  await jump.press("Enter");
  await expect(
    page.getByLabel("노드 표시 이름", { exact: true }),
  ).toBeFocused();
  await expect(page.getByLabel("시간 (초)", { exact: true })).toBeVisible();
  expect(model.state().doc).toEqual(before.doc);
  expect(model.state().undoCount).toBe(before.undoCount);
  expect(model.state().dirty).toBe(before.dirty);
  await page.getByLabel("시간 (초)", { exact: true }).fill("12.5");
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await focus(page, model.state().doc, "Hello 상태 선언");
  await expect(summary).toContainText("12.5초");
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect(summary).toContainText("값 미정");
  await page
    .getByRole("button", { name: "노드 다시 실행", exact: true })
    .click();
  await expect(summary).toContainText("12.5초");
  await page.getByLabel("노드 표시 이름", { exact: true }).fill("미확정 이름");
  await expect(jump).toBeDisabled();
  await page.getByRole("button", { name: "취소", exact: true }).click();
  await expect(jump).toBeEnabled();
  await jump.click();
  await page
    .getByRole("button", { name: "선택 노드 삭제", exact: true })
    .click();
  await focus(page, model.state().doc, "Hello 상태 선언");
  await expect(summary).toContainText("연결 없음");
  await inspect(page, info, "state-summary-missing-setting");
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect(summary).toContainText("12.5초");
  await summary.getByRole("button", { name: "Hello 적용 노드로 이동" }).click();
  await page
    .getByRole("button", { name: "선택 노드 삭제", exact: true })
    .click();
  await focus(page, model.state().doc, "Hello 상태 선언");
  await expect(summary).toContainText("적용 동작 · 0");
  await expect(summary).toContainText("조회·관찰하는 노드 · 4");
  await inspect(page, info, "state-summary-missing-application");
});

test("state summary handles multiple applications, long names and an unconnected declaration", async ({
  page,
}, info) => {
  const model = await openSkill(page, info);
  const doc = structuredClone(model.state().doc);
  const state = doc.nodes.find((n) => n.kind === "skill-state")!;
  const overlay = doc.nodes.find((n) => n.kind === "skill-resonance-install")!;
  const other = {
    ...structuredClone(overlay),
    id: "summary-other-application",
    position: { x: overlay.position.x, y: overlay.position.y + 700 },
    title:
      "아주 긴 한글 이름을 가진 두 번째 상태 적용 동작의 연결과 줄바꿈 확인",
  };
  doc.nodes.push(other);
  doc.edges.push(
    ...doc.edges
      .filter((e) => e.target.node === overlay.id && e.target.port !== "in")
      .map((e) => ({
        ...structuredClone(e),
        id: `${e.id}-summary-copy`,
        target: { ...e.target, node: other.id },
      })),
  );
  doc.nodes.find((n) => n.kind === "skill-seconds")!.values.value = 0;
  const load = async () => {
    canonicalize(doc);
    await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
      name: "state-summary-fixture.node-graph.json",
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify(doc)),
    });
    await expect.poll(() => model.state().doc).toEqual(doc);
    await focus(page, doc, state.title);
  };
  await load();
  const summary = page.getByRole("region", { name: "연결된 상태 구성" });
  await expect(summary).toContainText("적용 동작 · 2");
  await expect(summary).toContainText("활성 소유 범위를 확인하세요");
  await expect(summary.getByText("0초", { exact: true })).toHaveCount(2);
  await summary
    .getByRole("button", { name: `${other.title} 노드로 이동` })
    .scrollIntoViewIfNeeded();
  await inspect(page, info, "state-summary-multiple");
  doc.edges = doc.edges.filter((e) => e.source.node !== state.id);
  await load();
  await expect(summary).toContainText("적용 동작 · 0");
  await expect(summary).toContainText("연결된 조회·관찰 노드가 없습니다");
  await expect(summary.getByRole("button")).toHaveCount(0);
  await inspect(page, info, "state-summary-empty");
});

test("Hello skill edits typed properties and state labels with undo and a real file roundtrip", async ({
  page,
}, info) => {
  const model = await openSkill(page, info);
  const original = model.state().doc;
  const firstUse = original.nodes.find((n) => n.kind === "skill-request")!;
  const entry = page.getByTestId(`rf__node-${firstUse.id}`);
  await expect(entry).toBeInViewport({ ratio: 1 });
  await expect(entry.getByText("▶ 시작 사건", { exact: true })).toBeVisible();
  await expect(
    page.locator('.ng-card[data-entry="true"] .ng-role-label'),
  ).toHaveCount(4);
  const zoom = () =>
    page
      .locator(".react-flow__viewport")
      .evaluate(
        (element) => new DOMMatrix(getComputedStyle(element).transform).a,
      );
  await expect.poll(zoom).toBeCloseTo(0.85);
  const beforeNavigation = model.state();
  await inspect(page, info, "hello-entry");
  await page.getByRole("button", { name: "전체 보기", exact: true }).click();
  await expect.poll(zoom).toBeLessThan(0.85);
  // An expanded graph must fit inside the canvas, not underneath its toolbar.
  await expect
    .poll(() =>
      page.getByTestId("node-canvas").evaluate((canvas) => {
        const bounds = canvas.getBoundingClientRect();
        return [...canvas.querySelectorAll(".react-flow__node")].every(
          (node) => {
            const box = node.getBoundingClientRect();
            return (
              box.left >= bounds.left &&
              box.right <= bounds.right &&
              box.top >= bounds.top &&
              box.bottom <= bounds.bottom
            );
          },
        );
      }),
    )
    .toBe(true);
  expect(model.state().doc).toEqual(beforeNavigation.doc);
  expect(model.state().dirty).toBe(beforeNavigation.dirty);
  expect(model.state().undoCount).toBe(beforeNavigation.undoCount);
  await inspect(page, info, "hello-overview");
  const duration = await focus(page, original, "Hello 유지 시간");
  const seconds = page.getByLabel("시간 (초)", { exact: true });
  await expect(seconds).toHaveValue("");
  await seconds.fill("12.5");
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === duration)?.values.value,
    )
    .toBe(12.5);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect(seconds).toHaveValue("");
  await page
    .getByRole("button", { name: "노드 다시 실행", exact: true })
    .click();
  await expect(seconds).toHaveValue("12.5");
  const state = await focus(page, model.state().doc, "Hello 상태 선언");
  await page
    .getByLabel("노드 표시 이름", { exact: true })
    .fill("Hello 상태 선언 — 아주 긴 한글 표시 이름과 상태 참조 보존 확인");
  const saved = await save(page); // Save must commit the pending inspector edit.
  expect(saved.nodes.find((n) => n.id === state)?.title).toContain(
    "상태 참조 보존",
  );
  expect(saved.edges).toEqual(original.edges);
  expect(saved.nodes.find((n) => n.id === duration)?.values.value).toBe(12.5);
  await expect.poll(() => model.state().dirty).toBe(false);
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "hello.node-graph.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(saved)),
  });
  await expect.poll(() => model.state().doc).toEqual(saved);
  await expect.poll(() => model.state().canUndo).toBe(false);
  await expect.poll(zoom).toBeCloseTo(0.85);
  await expect(page.getByTestId(`rf__node-${firstUse.id}`)).toBeInViewport({
    ratio: 1,
  });
  await focus(page, model.state().doc, "2 · 재입력과 Goodbye");
  await inspect(page, info, "goodbye-region");
  const power = await focus(page, model.state().doc, "Goodbye 위력");
  await page.getByLabel("위력 곡선", { exact: true }).selectOption("linear");
  await expect(page.getByLabel("노드 속성")).toContainText(
    "최종 위력을 한 번 올림",
  );
  const beforePower = model.state().doc;
  await page.getByLabel("기본 위력", { exact: true }).fill("100.5");
  await page.getByLabel("초당 위력 증가", { exact: true }).fill("20");
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await expect(
    page.getByText("기본 위력: 소수 없이 안전한 정수를 입력하세요.", {
      exact: true,
    }),
  ).toBeVisible();
  expect(model.state().doc).toEqual(beforePower);
  await page.getByLabel("기본 위력", { exact: true }).fill("100");
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await expect
    .poll(() => model.state().doc.nodes.find((n) => n.id === power)?.values)
    .toEqual({ curve: "linear", base: 100, rate: 20, cap: "" });
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(beforePower);
  await page
    .getByRole("button", { name: "노드 다시 실행", exact: true })
    .click();
  await expect(page.getByLabel("기본 위력", { exact: true })).toHaveValue(
    "100",
  );
  await inspect(page, info, "goodbye-power-editor");
  const powerSaved = await save(page);
  expect(powerSaved.nodes.find((n) => n.id === power)?.values).toEqual({
    curve: "linear",
    base: 100,
    rate: 20,
    cap: "",
  });
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "hello-integer-power.node-graph.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(powerSaved)),
  });
  await expect.poll(() => model.state().doc).toEqual(powerSaved);
  await focus(page, model.state().doc, "3 · 만료와 외부 종료");
  await inspect(page, info, "hello-endings");
});

test("presentation is separate from execution; state deletion exposes diagnostics and undo restores references", async ({
  page,
}, info) => {
  const model = await openSkill(page, info);
  const before = model.state().doc;
  const cue = await focus(page, before, "포신 변형 표시");
  await page
    .getByLabel("연출 내용", { exact: true })
    .fill(
      "팔의 포신 외형을 표시합니다.\n배치 위치와 실제 피해는 변경하지 않습니다.",
    );
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === cue)?.values.description,
    )
    .toContain("\n");
  expect(model.state().doc.nodes.filter((n) => n.kind !== "skill-cue")).toEqual(
    before.nodes.filter((n) => n.kind !== "skill-cue"),
  );
  expect(model.state().doc.edges).toEqual(before.edges);
  await inspect(page, info, "hello-presentation");
  await focus(page, model.state().doc, "Hello 적용");
  await page
    .getByRole("button", {
      name: "Hello 적용 변형 시작 출력 연결",
      exact: true,
    })
    .click();
  const dialog = page.getByRole("dialog", { name: "포트 연결" });
  const target = dialog.getByLabel("연결할 포트");
  const texts = await target.locator("option").allTextContents();
  expect(texts.some((s) => s.includes("Goodbye 일격"))).toBe(false);
  expect(texts.some((s) => s.includes("정상 모습 복귀"))).toBe(true);
  const restore = model
    .state()
    .doc.nodes.find((n) => n.title === "정상 모습 복귀")!;
  await target.selectOption(
    JSON.stringify({ node: restore.id, port: "event" }),
  );
  const edgeCount = model.state().doc.edges.length;
  await dialog.getByRole("button", { name: "연결", exact: true }).click();
  await expect.poll(() => model.state().doc.edges.length).toBe(edgeCount + 1);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc.edges.length).toBe(edgeCount);
  const restored = model.state().doc;
  await focus(page, restored, "Hello 상태 선언");
  await page
    .getByRole("button", { name: "선택 노드 삭제", exact: true })
    .click();
  await expect
    .poll(
      () =>
        model.state().diagnostics.filter((d) => d.portId === "state").length,
    )
    .toBe(5);
  await page.getByRole("button", { name: /^진단 \d+/ }).click();
  const diagnostics = page.getByRole("region", { name: "노드 작성 진단" });
  await diagnostics
    .getByRole("button", {
      name: "Hello 활성 조회 · 상태 참조 입력이 연결되지 않았습니다.",
      exact: true,
    })
    .click();
  await expect(page.getByLabel("노드 표시 이름", { exact: true })).toHaveValue(
    "Hello 활성 조회",
  );
  await inspect(page, info, "hello-reference-diagnostic");
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(restored);
  const saved = await save(page);
  expect(saved).toEqual(restored);
});

test("semantic input branches expose permissions and diagnose an opposite branch with undo", async ({
  page,
}, info) => {
  const model = await openSkill(page, info);
  const before = model.state();
  await focus(page, before.doc, "Hello 활성 조회");
  await expect(page.getByLabel("노드 속성")).toContainText(
    "요청의 직원·장착 스킬 출처와 상태 선언이 모두 같은 활성을 조회",
  );
  await inspect(page, info, "hello-active-lookup");
  await focus(page, before.doc, "Goodbye 재입력 허용");
  const policy = page.getByLabel("재입력 허용 정책", { exact: true });
  await expect(policy).toHaveValue("manual-followup");
  await expect(page.getByLabel("노드 속성")).toContainText(
    "해당 스킬 자신의 사용 잠금만 예외",
  );
  await policy.selectOption("unresolved");
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  const permission = before.doc.nodes.find(
    (n) => n.kind === "skill-permission",
  )!;
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === permission.id)?.values
          .permission,
    )
    .toBe("unresolved");
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect(policy).toHaveValue("manual-followup");
  await inspect(page, info, "goodbye-permission");
  await focus(page, before.doc, "Goodbye 마무리 권리");
  await expect(
    page.getByLabel("만료와 동시 입력", { exact: true }),
  ).toHaveValue("expiry-first");
  await expect(page.getByLabel("노드 속성")).toContainText(
    "처리 시각이 만료 시각과 같거나 늦으면 거절",
  );
  await inspect(page, info, "goodbye-claim");
  const doc = structuredClone(before.doc);
  const lookup = doc.nodes.find((n) => n.kind === "skill-lookup")!;
  const qualification = doc.nodes.find(
    (n) => n.kind === "skill-resonance-qualify",
  )!;
  // A saved draft can be structurally valid yet use the wrong branch's value.
  doc.edges = doc.edges.filter(
    (e) => !(e.source.node === lookup.id && e.source.port === "present"),
  );
  doc.edges.find(
    (e) => e.target.node === qualification.id && e.target.port === "in",
  )!.source.port = "present";
  canonicalize(doc);
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "wrong-branch.node-graph.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(doc)),
  });
  await expect
    .poll(() =>
      model
        .state()
        .diagnostics.some(
          (d) => d.code === "skill-path-scope" && d.nodeId === qualification.id,
        ),
    )
    .toBe(true);
  await page.getByRole("button", { name: /^진단 \d+/ }).click();
  await page
    .getByRole("region", { name: "노드 작성 진단" })
    .getByRole("button", {
      name: "Hello 첫 사용 자격 · 이 입력값을 만든 분기를 모든 실행 경로가 통과해야 합니다.",
      exact: true,
    })
    .click();
  await expect(page.getByLabel("노드 표시 이름", { exact: true })).toHaveValue(
    "Hello 첫 사용 자격",
  );
  await inspect(page, info, "hello-wrong-branch");
  expect((await save(page)).edges).toEqual(doc.edges);
});

test("tile skill range preserves drafts, history, file roundtrip and legacy reassignment", async ({
  page,
}, info) => {
  const model = await openSkill(page, info);
  const before = model.state();
  const id = await focus(page, before.doc, "Goodbye 전방 범위");
  const inspector = page.getByRole("complementary", { name: "노드 속성" });
  const tile = (x: number, y: number) =>
    inspector.getByRole("button", {
      name: `타일 (${x}, ${y})${x === 0 && y === 0 ? " 기준" : ""}`,
      exact: true,
    });
  const apply = () =>
    page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await expect(page.getByLabel("범위 형상", { exact: true })).toHaveCount(0);
  await expect(inspector).toContainText("범위 미정");
  await tile(-1, -1).click();
  await tile(1, -1).focus();
  await tile(1, -1).press("Space");
  await tile(0, 0).click();
  expect(model.state().doc).toEqual(before.doc);
  await page.getByRole("button", { name: "취소", exact: true }).click();
  await expect(tile(-1, -1)).toHaveAttribute("aria-pressed", "false");
  expect(model.state().undoCount).toBe(before.undoCount);
  await tile(-1, -1).click();
  await tile(1, -1).click();
  await tile(0, 0).click();
  await page.getByLabel("방향 고정", { exact: true }).selectOption("accepted");
  await apply();
  const values = {
    pattern: "X.X\n.@.",
    includeAnchor: true,
    facing: "accepted",
  };
  await expect
    .poll(() => model.state().doc.nodes.find((n) => n.id === id)?.values)
    .toEqual(values);
  const edited = model.state();
  expect(edited.undoCount).toBe(before.undoCount + 1);
  await page
    .getByRole("button", { name: "타일 보기 오른쪽", exact: true })
    .click();
  await expect(tile(6, 0)).toBeVisible();
  expect(model.state().doc).toEqual(edited.doc);
  expect(model.state().undoCount).toBe(edited.undoCount);
  await page.getByRole("button", { name: "기준점 보기", exact: true }).click();
  await inspector.locator(".ng-tile-editor").scrollIntoViewIfNeeded();
  await inspect(page, info, "tile-range-editor");
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(before.doc);
  await page
    .getByRole("button", { name: "노드 다시 실행", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(edited.doc);
  const saved = await save(page);
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "tile-range.node-graph.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(saved)),
  });
  await expect.poll(() => model.state().doc).toEqual(saved);
  await focus(page, saved, "Goodbye 전방 범위");
  await expect(tile(0, 0)).toHaveAttribute("aria-pressed", "true");
  const legacy = structuredClone(saved);
  const oldValues = {
    shape: "sector",
    length: 2.5,
    width: "",
    angle: 90,
    facing: "accepted",
  };
  legacy.nodes.find((n) => n.id === id)!.values = oldValues;
  canonicalize(legacy);
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "legacy-shape.node-graph.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(legacy)),
  });
  await expect.poll(() => model.state().doc).toEqual(legacy);
  await focus(page, legacy, "Goodbye 전방 범위");
  await expect(inspector).toContainText("이전 도형 설정입니다");
  expect(
    model.state().diagnostics.some((d) => d.code === "skill-area-legacy"),
  ).toBe(true);
  // An unrelated title edit must not silently replace a legacy range.
  await page
    .getByLabel("노드 표시 이름", { exact: true })
    .fill("이전 범위 검토");
  await apply();
  await expect
    .poll(() => model.state().doc.nodes.find((n) => n.id === id)?.values)
    .toEqual(oldValues);
  const preserved = model.state().doc;
  expect((await save(page)).nodes.find((n) => n.id === id)?.values).toEqual(
    oldValues,
  );
  await tile(0, -1).click();
  await inspector.locator(".ng-tile-editor").scrollIntoViewIfNeeded();
  await inspect(page, info, "legacy-range-reassignment");
  await apply();
  await expect
    .poll(() => model.state().doc.nodes.find((n) => n.id === id)?.values)
    .toEqual({ pattern: "X\n@", includeAnchor: false, facing: "accepted" });
  expect(
    model.state().diagnostics.some((d) => d.code === "skill-area-legacy"),
  ).toBe(false);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(preserved);
});

test("common skill ranges separate target reach, placement and fixed ground patterns", async ({
  page,
}, info) => {
  const model = await openSkill(page, info);
  const load = async (name: string) => {
    const doc: GraphDocument =
      name === "front-strike"
        ? spatialExample("front-strike")
        : JSON.parse(
            await readFile(`content/range-${name}.node-graph.json`, "utf8"),
          );
    await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
      name: `range-${name}.node-graph.json`,
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify(doc)),
    });
    await expect.poll(() => model.state().doc).toEqual(doc);
    return doc;
  };
  let doc = await load("selected-ground");
  const inspector = page.getByRole("complementary", { name: "노드 속성" });
  const id = await focus(page, doc, "스킬 효과 범위");
  await expect(inspector.locator(".ng-node-kind")).toHaveText("스킬 범위");
  await expect(page.getByLabel("방향 고정", { exact: true })).toHaveCount(0);
  const before = model.state();
  await inspector
    .getByRole("button", { name: "타일 (1, 0)", exact: true })
    .click();
  await inspector
    .getByRole("button", { name: "타일 (1, 1)", exact: true })
    .click();
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await expect
    .poll(
      () => model.state().doc.nodes.find((n) => n.id === id)?.values.pattern,
    )
    .toBe("@X\n.X");
  const reach = doc.nodes.find((n) => n.title === "대상 선택 범위")!;
  expect(model.state().doc.nodes.find((n) => n.id === reach.id)).toEqual(reach);
  await inspector.locator(".ng-tile-editor").scrollIntoViewIfNeeded();
  await inspect(page, info, "common-skill-range");
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(before.doc);
  await page
    .getByRole("button", { name: "노드 다시 실행", exact: true })
    .click();
  await focus(page, model.state().doc, "효과 범위 배치");
  await expect(page.getByLabel("범위 기준", { exact: true })).toHaveValue(
    "selected-tile",
  );
  await expect(
    page.getByLabel("시전자 방향 고정", { exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByLabel("시전자 위치 고정", { exact: true }),
  ).toHaveCount(0);
  await expect(inspector).toContainText("패턴을 회전하지 않습니다");
  await inspect(page, info, "ground-fixed-placement");
  const saved = await save(page);
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "fixed-ground.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(saved)),
  });
  await expect.poll(() => model.state().doc).toEqual(saved);
  await focus(page, saved, "입력의 대상 선택");
  await expect(page.getByLabel("대상 선택 방식", { exact: true })).toHaveValue(
    "player-tile",
  );
  await expect(inspector).toContainText("대상 선택 범위");
  await focus(page, saved, "지정 타일 장판 생성");
  await expect(inspector).toContainText(
    "생성 자체는 피해·회복·상태 효과를 발생시키지 않습니다",
  );
  doc = await load("front-strike");
  await focus(page, doc, "효과 범위 배치");
  await expect(page.getByLabel("범위 기준", { exact: true })).toHaveValue(
    "caster",
  );
  await expect(
    page.getByLabel("시전자 위치 고정", { exact: true }),
  ).toHaveValue("unresolved");
  await expect(
    page.getByLabel("시전자 방향 고정", { exact: true }),
  ).toHaveValue("unresolved");
  const placement = doc.nodes.find((n) => n.title === "효과 범위 배치")!;
  await expect
    .poll(() =>
      page.getByTestId(`rf__node-${placement.id}`).evaluate(
        (element, ids) => {
          const a = element.getBoundingClientRect();
          return ids.every((id) => {
            const other = document.querySelector(
              `[data-testid="rf__node-${id}"]`,
            );
            if (!other) return true;
            const b = other.getBoundingClientRect();
            return (
              a.right <= b.left ||
              b.right <= a.left ||
              a.bottom <= b.top ||
              b.bottom <= a.top
            );
          });
        },
        doc.nodes.filter((n) => n.id !== placement.id).map((n) => n.id),
      ),
    )
    .toBe(true);
  await inspect(page, info, "front-strike-placement");
});

test("ground effects preserve first pulse, status overlap and lifetime through editing and file reload", async ({
  page,
}, info) => {
  const model = await openSkill(page, info);
  const load = async (mode: "damage" | "heal" | "status") => {
    const doc: GraphDocument = JSON.parse(
      await readFile(`content/ground-${mode}.node-graph.json`, "utf8"),
    );
    await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
      name: `ground-${mode}.node-graph.json`,
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify(doc)),
    });
    await expect.poll(() => model.state().doc).toEqual(doc);
    return doc;
  };
  const inspector = page.getByRole("complementary", { name: "노드 속성" });
  const apply = () =>
    page.getByRole("button", { name: "변경 적용", exact: true }).click();
  let doc = await load("damage");
  const periodic = await focus(page, doc, "장판 주기");
  await expect(page.getByLabel("첫 발동", { exact: true })).toHaveValue(
    "unresolved",
  );
  const before = model.state().doc;
  await page.getByLabel("첫 발동", { exact: true }).selectOption("immediate");
  await apply();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === periodic)?.values.first,
    )
    .toBe("immediate");
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(before);
  await page
    .getByRole("button", { name: "노드 다시 실행", exact: true })
    .click();
  await expect(page.getByLabel("첫 발동", { exact: true })).toHaveValue(
    "immediate",
  );
  await expect(inspector).toContainText(
    "장판 종료 시각의 주기는 발생하지 않습니다",
  );
  await inspect(page, info, "ground-first-pulse");
  await page
    .getByLabel("첫 발동", { exact: true })
    .selectOption("after-interval");
  await apply();
  const damageSaved = await save(page);
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "ground-damage-reloaded.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(damageSaved)),
  });
  await expect.poll(() => model.state().doc).toEqual(damageSaved);
  await focus(page, damageSaved, "장판 주기");
  await expect(page.getByLabel("첫 발동", { exact: true })).toHaveValue(
    "after-interval",
  );
  doc = await load("status");
  await focus(page, doc, "장판 진입");
  await expect(inspector).toContainText("생성 당시 이미 안에 있던 대상도 진입");
  await focus(page, doc, "진입 둔화 정의");
  await page.getByLabel("효과 크기 (%)", { exact: true }).fill("20");
  await page.getByLabel("같은 상태 중첩", { exact: true }).selectOption("sum");
  await page.getByLabel("이탈 후 상태", { exact: true }).selectOption("area");
  await apply();
  await page
    .getByLabel("이탈 후 상태", { exact: true })
    .scrollIntoViewIfNeeded();
  await inspect(page, info, "ground-status-lifetime");
  await page.getByLabel("상태 효과", { exact: true }).selectOption("stun");
  await expect(page.getByLabel("효과 크기 (%)", { exact: true })).toHaveCount(
    0,
  );
  await expect(page.getByLabel("같은 상태 중첩", { exact: true })).toHaveCount(
    0,
  );
  await apply();
  await page.getByLabel("상태 효과", { exact: true }).selectOption("slow");
  await expect(page.getByLabel("효과 크기 (%)", { exact: true })).toHaveValue(
    "20",
  );
  await expect(page.getByLabel("같은 상태 중첩", { exact: true })).toHaveValue(
    "sum",
  );
  await apply();
  const statusSaved = await save(page);
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "ground-status-reloaded.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(statusSaved)),
  });
  await expect.poll(() => model.state().doc).toEqual(statusSaved);
  await focus(page, statusSaved, "진입 둔화 정의");
  await expect(page.getByLabel("이탈 후 상태", { exact: true })).toHaveValue(
    "area",
  );
  await focus(page, statusSaved, "이 장판 종료");
  await expect(inspector).toContainText("다른 장판의 상태는 유지");
  await inspect(page, info, "ground-end-cleanup");
  doc = await load("heal");
  await focus(page, doc, "대상별 회복");
  await expect(inspector).toContainText(
    "초과분을 비축하거나 사망자를 부활시키지 않습니다",
  );
  await expect(inspector).toContainText("회복 요청량");
});

test("burrowing heaven channels preserve cancellation, progressive slow and owned cleanup through editing and reload", async ({
  page,
}, info) => {
  const model = await openSkill(page, info);
  const doc: GraphDocument = JSON.parse(
    await readFile("content/burrowing-heaven.node-graph.json", "utf8"),
  );
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "burrowing-heaven.node-graph.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(doc)),
  });
  await expect.poll(() => model.state().doc).toEqual(doc);
  await expect
    .poll(() => model.state().diagnostics.map((d) => d.code))
    .toEqual(Array(8).fill("skill-value-unset"));
  const inspector = page.getByRole("complementary", { name: "노드 속성" });
  const apply = () =>
    page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await focus(page, doc, "집중 시작");
  await expect(
    page.getByLabel("집중 중 기본 공격", { exact: true }),
  ).toHaveValue("pause");
  await page.getByLabel("외부 중단", { exact: true }).selectOption("protected");
  await apply();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  await page
    .getByRole("button", { name: "노드 다시 실행", exact: true })
    .click();
  await expect(page.getByLabel("외부 중단", { exact: true })).toHaveValue(
    "protected",
  );
  await page.getByLabel("외부 중단", { exact: true }).selectOption("standard");
  await apply();
  await inspect(page, info, "channel-start-settings");
  await focus(page, doc, "체류에 따른 점진 둔화");
  await expect(inspector).toContainText("재진입은 0부터 시작");
  await expect(inspector).toContainText(
    "장판 종료가 같은 시각의 도달보다 우선",
  );
  await expect(page.getByLabel("최대 둔화율 (%)", { exact: true })).toHaveValue(
    "",
  );
  await page.getByLabel("최대 둔화율 (%)", { exact: true }).fill("60");
  await apply();
  await inspect(page, info, "channel-progressive-slow");
  await focus(page, doc, "최대 집중 시간");
  await page.getByLabel("시간 (초)", { exact: true }).fill("12.5");
  await apply();
  const saved = await save(page);
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "channel-reloaded.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(saved)),
  });
  await expect.poll(() => model.state().doc).toEqual(saved);
  await expect
    .poll(() => model.state().diagnostics.map((d) => d.code))
    .toEqual(Array(6).fill("skill-value-unset"));
  await focus(page, saved, "최대 집중 시간");
  await expect(page.getByLabel("시간 (초)", { exact: true })).toHaveValue(
    "12.5",
  );
  await focus(page, saved, "체류에 따른 점진 둔화");
  await expect(page.getByLabel("최대 둔화율 (%)", { exact: true })).toHaveValue(
    "60",
  );
  await focus(page, saved, "직접 취소 입력");
  await expect(inspector).toContainText("침묵·사용 잠금 중에도 허용");
  await focus(page, saved, "집중 종료와 정리");
  await expect(inspector).toContainText("다른 시전자·시전 회차");
  await expect(inspector).toContainText("사용 잠금 해제 뒤");
  await inspect(page, info, "channel-end-cleanup");
});

test("mountain taunt keeps grouped effects and per-target ownership through editing and reload", async ({
  page,
}, info) => {
  const model = await openSkill(page, info);
  const doc: GraphDocument = JSON.parse(
    await readFile(
      "content/mountain-of-smiling-bodies.node-graph.json",
      "utf8",
    ),
  );
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "mountain-of-smiling-bodies.node-graph.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(doc)),
  });
  await expect.poll(() => model.state().doc).toEqual(doc);
  await expect
    .poll(() => model.state().diagnostics.map((d) => d.code))
    .toEqual(Array(12).fill("skill-value-unset"));
  const inspector = page.getByRole("complementary", { name: "노드 속성" });
  const apply = () =>
    page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await focus(page, doc, "도발 대상 한 번 조회");
  await expect(inspector).toContainText("빈 범위에서도 한 번");
  await focus(page, doc, "도발·접근 제약의 공통 수명");
  await expect(
    page.getByLabel("시전자 사망·철수", { exact: true }),
  ).toHaveValue("end");
  await page
    .getByLabel("시전자 사망·철수", { exact: true })
    .selectOption("independent");
  await apply();
  await expect
    .poll(() => model.state().diagnostics.map((d) => d.code))
    .toContain("target-taunt-owner");
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  await inspect(page, info, "target-shared-lifetime");
  await focus(page, doc, "시전자만 저지 가능한 도발");
  await expect(inspector).toContainText("도발 중 새 저지는 시전자만 허용");
  await expect(inspector).toContainText("이전 상태를 복원하지 않습니다");
  await inspect(page, info, "target-taunt-policy");
  await focus(page, doc, "접근 이동 속도 증가");
  await page.getByLabel("이동 속도 증가율 (%)", { exact: true }).fill("35");
  await apply();
  await focus(page, doc, "도발 공통 지속 시간");
  await page.getByLabel("시간 (초)", { exact: true }).fill("8.5");
  await apply();
  const saved = await save(page);
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "mountain-reloaded.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(saved)),
  });
  await expect.poll(() => model.state().doc).toEqual(saved);
  await expect
    .poll(() => model.state().diagnostics.map((d) => d.code))
    .toEqual(Array(10).fill("skill-value-unset"));
  const state = saved.nodes.find((n) => n.kind === "skill-target-state")!;
  expect(
    saved.edges.filter(
      (e) => e.target.node === state.id && e.target.port === "effects",
    ),
  ).toHaveLength(3);
  await focus(page, saved, "접근 이동 속도 증가");
  await expect(
    page.getByLabel("이동 속도 증가율 (%)", { exact: true }),
  ).toHaveValue("35");
  await focus(page, saved, "도발 공통 지속 시간");
  await expect(page.getByLabel("시간 (초)", { exact: true })).toHaveValue(
    "8.5",
  );
  await focus(page, saved, "이번 대상의 세 효과 정리");
  await expect(inspector).toContainText("정리 후 한 번 발생");
  await expect(inspector).toContainText(
    "오래된 종료는 새 활성에 영향을 주지 않습니다",
  );
  await inspect(page, info, "target-owned-cleanup");
});

for (const scenario of [
  {
    file: "queen-of-hatred",
    unset: 29,
    editTitle: "완주 1개 + 피해·회복 모두 성공 1개",
    field: "완주 기본 개수",
    value: "2",
    editText: "같은 집중의 자연 완료 흐름에서만",
    editedUnset: 29,
    inspectTitle: "히스테릭 예약 · 현재 시전 정리 뒤 자동 시작",
    inspectText: "시작한 자동 집중은 수동 취소할 수 없습니다",
    firstCapture: "queen-natural-completion",
    secondCapture: "queen-automatic-request",
  },
  {
    file: "queen-of-hatred-bloom",
    unset: 41,
    editTitle: "수동 블랙홀 · 확정 소비 하트 비례 회복 · 증가율 미정",
    field: "개당 증가율 (%)",
    value: "15",
    editText: "같은 사용",
    editedUnset: 40,
    inspectTitle: "소비 하트별 흡인·회복 범위",
    inspectText: "접수 조건에서 가능한 모든 개수를 명시",
    firstCapture: "queen-bloom-paid-count",
    secondCapture: "queen-bloom-range-tiers",
  },
  {
    file: "yin-yang",
    unset: 13,
    editTitle: "HP 상한 초과분만 전달 · 재전달 없음",
    field: "초과분 전달률 (%)",
    value: "30",
    editText: "다시 전달하지 않습니다",
    editedUnset: 12,
    inspectTitle: "교집합 진입 부여 · 이탈 즉시 해제",
    inspectText: "실제 교집합 안의 아군",
    firstCapture: "yin-yang-overflow-transfer",
    secondCapture: "yin-yang-intersection-lifetime",
  },
]) {
  test(`P3 authoring ${scenario.file} edits and reloads owned results`, async ({
    page,
  }, info) => {
    const model = await openSkill(page, info);
    const doc: GraphDocument = JSON.parse(
      await readFile(`content/${scenario.file}.node-graph.json`, "utf8"),
    );
    await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
      name: `${scenario.file}.json`,
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify(doc)),
    });
    await expect.poll(() => model.state().doc).toEqual(doc);
    await expect
      .poll(() =>
        model
          .state()
          .diagnostics.map((d) => d.code)
          .sort(),
      )
      .toEqual(
        [
          ...Array(scenario.unset).fill("skill-value-unset"),
          "skill-policy-unset",
          "skill-policy-unset",
        ].sort(),
      );
    const inspector = page.getByRole("complementary", { name: "노드 속성" });
    const targetId = await focus(page, doc, scenario.editTitle);
    await expect(inspector).toContainText(scenario.editText);
    const field = page.getByLabel(scenario.field, { exact: true });
    await field.fill(scenario.value);
    await page.getByRole("button", { name: "변경 적용", exact: true }).click();
    await expect
      .poll(() =>
        model
          .state()
          .diagnostics.map((d) => d.code)
          .sort(),
      )
      .toEqual(
        [
          ...Array(scenario.editedUnset).fill("skill-value-unset"),
          "skill-policy-unset",
          "skill-policy-unset",
        ].sort(),
      );
    await page
      .getByRole("button", { name: "노드 실행 취소", exact: true })
      .click();
    await expect.poll(() => model.state().doc).toEqual(doc);
    await field.fill(scenario.value);
    await page.getByRole("button", { name: "변경 적용", exact: true }).click();
    await inspect(page, info, scenario.firstCapture);
    const saved = await save(page);
    expect(saved.edges).toEqual(doc.edges);
    expect(saved.nodes.find((n) => n.id === targetId)?.values).not.toEqual(
      doc.nodes.find((n) => n.id === targetId)?.values,
    );
    await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
      name: `${scenario.file}-reloaded.json`,
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify(saved)),
    });
    await expect.poll(() => model.state().doc).toEqual(saved);
    await focus(page, saved, scenario.editTitle);
    await expect(page.getByLabel(scenario.field, { exact: true })).toHaveValue(
      scenario.value,
    );
    await focus(page, saved, scenario.inspectTitle);
    await expect(inspector).toContainText(scenario.inspectText);
    await inspect(page, info, scenario.secondCapture);
  });
}

for (const scenario of [
  {
    file: "knight-of-despair",
    unset: 18,
    editedUnset: 18,
    editTitle: "단계당 c = max(1, floor(S/4))",
    field: "나눌 수",
    value: "5",
    editText: "매 소비 직전의 잔고로 다시 나누지 않습니다",
    inspectTitle: "다음 행동 직전 절망 진입",
    inspectText: "전환",
    firstCapture: "knight-fixed-stage-cost",
    secondCapture: "knight-deferred-mode",
  },
  {
    file: "knight-of-despair-bloom",
    unset: 18,
    editedUnset: 18,
    editTitle: "개화 절망만 · 매초 최대 HP 10% 피해",
    field: "최대 HP 피해 (%)",
    value: "12",
    editText: "보호막",
    inspectTitle: "개화 절망 · 침묵에도 즉시 중단",
    inspectText: "외부 중단",
    firstCapture: "knight-bloom-self-damage",
    secondCapture: "knight-bloom-interruption",
  },
  {
    file: "little-red",
    unset: 26,
    editedUnset: 25,
    editTitle: "아군을 공격하는 늑대",
    field: "최대 HP",
    value: "100",
    editText: "아군",
    inspectTitle: "소비·모든 탄환 동시 방출·전투 재개",
    inspectText: "동시",
    firstCapture: "red-hostile-wolf",
    secondCapture: "red-atomic-release",
  },
  {
    file: "king-of-greed",
    unset: 15,
    editedUnset: 15,
    editTitle: "기본 지급 10 · 자동 대여 한도 20",
    field: "미상환 대여 한도",
    value: "25",
    editText: "직전 전투",
    inspectTitle: "한 발 정산 완료까지 대기 → 다음 표적",
    inspectText: "회수까지 모두 완료한 뒤",
    firstCapture: "greed-debt-ledger",
    secondCapture: "greed-serial-settlement",
  },
]) {
  test(`P4 authoring ${scenario.file} edits and reloads owned results`, async ({
    page,
  }, info) => {
    const model = await openSkill(page, info);
    const doc: GraphDocument = JSON.parse(
      await readFile(`content/${scenario.file}.node-graph.json`, "utf8"),
    );
    await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
      name: `${scenario.file}.json`,
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify(doc)),
    });
    await expect.poll(() => model.state().doc).toEqual(doc);
    await expect
      .poll(() => model.state().diagnostics.map((d) => d.code))
      .toEqual(Array(scenario.unset).fill("skill-value-unset"));
    const inspector = page.getByRole("complementary", { name: "노드 속성" });
    const targetId = await focus(page, doc, scenario.editTitle);
    await expect(inspector).toContainText(scenario.editText);
    const field = page.getByLabel(scenario.field, { exact: true });
    await field.fill(scenario.value);
    await page.getByRole("button", { name: "변경 적용", exact: true }).click();
    await expect
      .poll(() => model.state().diagnostics.map((d) => d.code))
      .toEqual(Array(scenario.editedUnset).fill("skill-value-unset"));
    await page
      .getByRole("button", { name: "노드 실행 취소", exact: true })
      .click();
    await expect.poll(() => model.state().doc).toEqual(doc);
    await field.fill(scenario.value);
    await page.getByRole("button", { name: "변경 적용", exact: true }).click();
    await inspect(page, info, scenario.firstCapture);
    const saved = await save(page);
    expect(saved.edges).toEqual(doc.edges);
    expect(saved.nodes.find((n) => n.id === targetId)?.values).not.toEqual(
      doc.nodes.find((n) => n.id === targetId)?.values,
    );
    await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
      name: `${scenario.file}-reloaded.json`,
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify(saved)),
    });
    await expect.poll(() => model.state().doc).toEqual(saved);
    await focus(page, saved, scenario.editTitle);
    await expect(page.getByLabel(scenario.field, { exact: true })).toHaveValue(
      scenario.value,
    );
    await focus(page, saved, scenario.inspectTitle);
    await expect(inspector).toContainText(scenario.inspectText);
    await inspect(page, info, scenario.secondCapture);
  });
}
