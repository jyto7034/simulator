import { readFile } from "node:fs/promises";
import type { Locator, Page, TestInfo } from "@playwright/test";
import { example } from "../../src/nodes/core/examples";
import type { GraphDocument } from "../../src/nodes/types";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectLayout,
  inspectAccessibility,
} from "./fixtures";

import { observeNodes } from "./node-observer";

async function openNodes(page: Page, info: TestInfo) {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "노드", exact: true })
    .click();
  await expect(page.getByTestId("node-canvas")).toBeVisible();
  await expect.poll(() => model.state()?.doc.nodes.length).toBe(0);
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "values.node-graph.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(example("values"))),
  });
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "교체", exact: true })
    .click();
  await expect.poll(() => model.state()?.doc.nodes.length).toBe(4);
  await expect(
    page.getByRole("button", { name: "저장", exact: true }),
  ).toBeEnabled();
  return model;
}
async function drag(
  page: Page,
  node: Locator,
  dx = 45,
  dy = 30,
  cancel = false,
  during?: () => void,
) {
  const box = (await node.locator(".ng-drag-handle").boundingBox())!;
  const x = box.x + box.width / 2;
  const y = box.y + box.height / 2;
  await page.mouse.move(x, y);
  await page.mouse.down();
  await page.mouse.move(x + dx, y + dy, { steps: 10 });
  during?.();
  if (cancel) await page.keyboard.press("Escape");
  await page.mouse.up();
}
async function save(page: Page, ack = true): Promise<GraphDocument> {
  const download = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const file = await download;
  const doc = JSON.parse(await readFile((await file.path())!, "utf8"));
  if (ack) {
    const confirm = page.getByRole("button", {
      name: "다운로드 확인",
      exact: true,
    });
    await confirm.click();
    // The next file
    // operation must wait for the saved baseline to reach the UI.
    await expect(confirm).toHaveCount(0);
  }
  return doc;
}
async function checkView(page: Page, info: TestInfo, name: string) {
  await capture(page, info, name);
  await inspectLayout(page, info, name);
  await inspectAccessibility(page, info, name);
}
async function dragBetween(
  page: Page,
  from: Locator,
  to: Locator,
  cancel = false,
) {
  const a = (await from.boundingBox())!;
  const b = (await to.boundingBox())!;
  await page.mouse.move(a.x + a.width / 2, a.y + a.height / 2);
  await page.mouse.down();
  await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2, { steps: 12 });
  if (cancel) await page.keyboard.press("Escape");
  await page.mouse.up();
}
async function selectEdge(page: Page, id: string) {
  const p = await page
    .getByTestId(`rf__edge-${id}`)
    .locator(".react-flow__edge-path")
    .evaluate((element) => {
      const path = element as SVGPathElement;
      const p = path
        .getPointAtLength(path.getTotalLength() / 2)
        .matrixTransform(path.getScreenCTM()!);
      return { x: p.x, y: p.y };
    });
  await page.mouse.click(p.x, p.y);
}

test("Local editing: dragging commits once, cancels cleanly, and restores positions and properties through history and files", async ({
  page,
}, info) => {
  const model = await openNodes(page, info);
  const original = await save(page);
  await expect.poll(() => model.state().dirty).toBe(false);
  const target = original.nodes.find((n) => n.kind === "number")!;
  const node = page.getByTestId(`rf__node-${target.id}`);
  await drag(page, node, 45, 30, false, () =>
    expect(model.edits).toHaveLength(0),
  );
  await expect.poll(() => model.state().undoCount).toBe(1);
  expect(model.edits).toHaveLength(1);
  const moved = model.state().doc;
  expect(moved.nodes.find((n) => n.id === target.id)!.position).not.toEqual(
    target.position,
  );
  await page.keyboard.press("Control+z");
  await expect.poll(() => model.state().doc).toEqual(original);
  await expect.poll(() => model.state().dirty).toBe(false);
  await page.keyboard.press("Control+Shift+z");
  await expect.poll(() => model.state().doc).toEqual(moved);
  const count = model.edits.length;
  await drag(page, node, 25, 20, true);
  expect(model.edits).toHaveLength(count);
  expect(model.state().doc).toEqual(moved);
  const other = model
    .state()
    .doc.nodes.find((n) => n.kind === "number" && n.id !== target.id)!;
  await node.locator(".ng-drag-handle").click();
  await page.keyboard.down("Control");
  await page
    .getByTestId(`rf__node-${other.id}`)
    .locator(".ng-drag-handle")
    .click();
  await page.keyboard.up("Control");
  await expect(page.getByLabel("노드 속성")).toContainText("2개 선택");
  await drag(page, node, 20, 15);
  await expect.poll(() => model.state().undoCount).toBe(2);
  expect(
    model.state().doc.nodes.find((n) => n.id === other.id)!.position,
  ).not.toEqual(other.position);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(moved);
  await node.locator(".ng-drag-handle").click();
  await page
    .getByLabel("노드 표시 이름", { exact: true })
    .fill("아주 긴 한글 이름으로 속성과 저장 경계를 확인하는 숫자 노드");
  await page.getByLabel("값", { exact: true }).fill("42.5");
  const saved = await save(page);
  expect(saved.nodes.find((n) => n.id === target.id)!.values.value).toBe(42.5);
  await expect.poll(() => model.state().dirty).toBe(false);
  await page.getByRole("button", { name: "전체 보기", exact: true }).click();
  await checkView(page, info, "nodes-edited");
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(moved);
  await expect.poll(() => model.state().dirty).toBe(true);
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "saved.node-graph.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(saved)),
  });
  await page.getByRole("button", { name: "교체", exact: true }).click();
  await expect.poll(() => model.state().doc).toEqual(saved);
  await expect.poll(() => model.state().canUndo).toBe(false);
  expect(model.state().dirty).toBe(false);
  const navigation = page.getByRole("navigation", { name: "작업 공간" });
  await navigation.getByRole("button", { name: "맵", exact: true }).click();
  await navigation.getByRole("button", { name: "노드", exact: true }).click();
  expect(await save(page)).toEqual(saved);
  const canvas = page.getByTestId("node-canvas");
  const box = (await canvas.boundingBox())!;
  const zoom = () =>
    canvas
      .locator(".react-flow__viewport")
      .evaluate(
        (element) =>
          new DOMMatrixReadOnly(getComputedStyle(element).transform).a,
      );
  const initialZoom = await zoom();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.wheel(0, 240);
  await expect.poll(zoom).toBeLessThan(initialZoom);
  const zoomedOut = await zoom();
  await page.mouse.wheel(0, -240);
  await expect.poll(zoom).toBeGreaterThan(zoomedOut);
  expect(model.state().doc).toEqual(saved);
  expect(model.state().dirty).toBe(false);
  expect(model.state().canUndo).toBe(false);
  await checkView(page, info, "nodes-wheel-zoom");
});

test("connections, frames, clipboard and deletion share document history", async ({
  page,
  context,
}, info) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  const model = await openNodes(page, info);
  const original = await save(page);
  const number = original.nodes.find((n) => n.kind === "number")!;
  const multiply = original.nodes.find((n) => n.kind === "multiply")!;
  await page
    .getByTestId(`rf__node-${number.id}`)
    .getByRole("button", { name: "숫자 숫자 출력 연결", exact: true })
    .click();
  await page
    .getByLabel("연결할 포트", { exact: true })
    .selectOption(JSON.stringify({ node: multiply.id, port: "b" }));
  await page.getByRole("button", { name: "연결", exact: true }).click();
  await expect.poll(() => model.state().doc.edges.length).toBe(4);
  await expect(page.locator(".react-flow__edge-path")).toHaveCount(4);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(original);
  await page
    .getByTestId(`rf__node-${number.id}`)
    .locator(".ng-drag-handle")
    .click();
  await page.keyboard.press("Control+a");
  await expect(page.getByLabel("노드 속성")).toContainText("4개 선택");
  await page.getByRole("button", { name: "묶기", exact: true }).click();
  await expect.poll(() => model.state().doc.frames.length).toBe(1);
  const grouped = model.state().doc;
  const frame = page.getByTestId(`rf__node-${grouped.frames[0].id}`);
  await page.getByRole("button", { name: "전체 보기", exact: true }).click();
  await drag(page, frame, 20, 15);
  await expect.poll(() => model.state().undoCount).toBe(2);
  for (const n of grouped.nodes)
    expect(
      model.state().doc.nodes.find((v) => v.id === n.id)!.position,
    ).not.toEqual(n.position);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(grouped);
  await frame.locator(".ng-drag-handle").click({ button: "right" });
  await expect(page.getByRole("menu")).toBeVisible();
  await checkView(page, info, "nodes-frame-menu");
  await page.getByRole("menuitem", { name: "복사", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "저장", exact: true }),
  ).toBeEnabled();
  await page.keyboard.press("Control+v");
  await expect.poll(() => model.state().doc.nodes.length).toBe(8);
  expect(model.state().doc.frames).toHaveLength(2);
  expect(model.state().doc.edges).toHaveLength(6);
  const pasted = model.state().doc;
  const added = pasted.nodes.filter(
    (n) => !grouped.nodes.some((v) => v.id === n.id),
  );
  expect(new Set(added.map((n) => n.frame)).size).toBe(1);
  expect(added[0].frame).not.toBe(grouped.frames[0].id);
  await page
    .getByRole("button", { name: "선택 노드 삭제", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(grouped);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(pasted);
  await page.getByRole("button", { name: "전체 보기", exact: true }).click();
  await checkView(page, info, "nodes-frames");
});

test("pointer connections and reconnections preserve old links on cancel and support reroutes", async ({
  page,
}, info) => {
  const model = await openNodes(page, info);
  const original = await save(page);
  const number = original.nodes.find((n) => n.kind === "number")!;
  const multiply = original.nodes.find((n) => n.kind === "multiply")!;
  const from = page
    .getByTestId(`rf__node-${number.id}`)
    .locator('.react-flow__handle.source[data-handleid="value"]');
  const to = page
    .getByTestId(`rf__node-${multiply.id}`)
    .locator('.react-flow__handle.target[data-handleid="b"]');
  await page
    .getByTestId(`rf__node-${multiply.id}`)
    .getByRole("button", { name: "곱하기 추가 입력 1개", exact: true })
    .click();
  await expect(to).toBeVisible();
  expect(model.state().doc).toEqual(original);
  expect(model.state().undoCount).toBe(0);
  await dragBetween(page, from, to);
  await expect.poll(() => model.state().doc.edges.length).toBe(4);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(original);
  const old = original.edges.find((e) => e.source.node === number.id)!;
  await selectEdge(page, old.id);
  await expect(page.getByLabel("노드 속성")).toContainText("연결 속성");
  const updater = page
    .getByTestId(`rf__edge-${old.id}`)
    .locator(".react-flow__edgeupdater-target");
  await dragBetween(page, updater, to, true);
  await expect.poll(() => model.state().doc).toEqual(original);
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await dragBetween(page, updater, to);
  await expect.poll(() => model.state().undoCount).toBe(1);
  expect(model.state().doc.edges).toHaveLength(3);
  expect(
    model
      .state()
      .doc.edges.some(
        (e) =>
          e.source.node === number.id &&
          e.target.node === multiply.id &&
          e.target.port === "b",
      ),
  ).toBe(true);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(original);
  await selectEdge(page, old.id);
  await page
    .getByRole("button", { name: "경유 노드 삽입", exact: true })
    .click();
  await expect.poll(() => model.state().doc.nodes.length).toBe(5);
  expect(model.state().doc.edges).toHaveLength(4);
  expect(model.state().doc.nodes.some((n) => n.kind === "reroute-number")).toBe(
    true,
  );
  await page.getByRole("button", { name: "전체 보기", exact: true }).click();
  const inserted = model
    .state()
    .doc.nodes.find((n) => n.kind === "reroute-number")!;
  const insertedBox = (await page
    .getByTestId(`rf__node-${inserted.id}`)
    .boundingBox())!;
  for (const node of original.nodes) {
    expect(model.state().doc.nodes.find((n) => n.id === node.id)).toEqual(node);
    const box = (await page.getByTestId(`rf__node-${node.id}`).boundingBox())!;
    expect(
      insertedBox.x >= box.x + box.width ||
        box.x >= insertedBox.x + insertedBox.width ||
        insertedBox.y >= box.y + box.height ||
        box.y >= insertedBox.y + insertedBox.height,
      "Automatically inserted reroute must not cover an existing card",
    ).toBe(true);
  }
  await checkView(page, info, "nodes-reroute");
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(original);
});

test("loading, empty graphs, invalid files and incomplete flow stay explicit and preserve the document", async ({
  page,
}, info) => {
  let release!: () => void;
  const gate = new Promise<void>((resolve) => {
    release = resolve;
  });
  await page.route("**/src/nodes/NodeEditor.tsx", async (route) => {
    await gate;
    await route.continue();
  });
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "노드", exact: true })
    .click();
  await expect(
    page.getByText("노드 화면을 불러오는 중…", { exact: true }),
  ).toBeVisible();
  await capture(page, info, "nodes-loading");
  release();
  await expect(page.getByTestId("node-canvas")).toBeVisible();
  const original = await save(page);
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "invalid.json",
    mimeType: "application/json",
    buffer: Buffer.from('{"format":"bad"}'),
  });
  await expect(page.getByRole("alert")).toContainText("읽지 못했습니다");
  expect(model.state().doc).toEqual(original);
  expect(model.state().dirty).toBe(false);
  await checkView(page, info, "nodes-invalid-file");
  await page.getByRole("button", { name: "새 그래프", exact: true }).click();
  await expect.poll(() => model.state().doc.nodes.length).toBe(0);
  await expect(
    page.getByText("첫 노드를 추가하세요", { exact: true }),
  ).toBeVisible();
  await checkView(page, info, "nodes-empty");
  await page.getByLabel("노드 검색", { exact: true }).fill("없는노드");
  await expect(
    page.getByText("일치하는 노드가 없습니다.", { exact: true }),
  ).toBeVisible();
  await page.getByLabel("노드 검색", { exact: true }).fill("");
  await page
    .getByLabel("노드 목록")
    .getByRole("button", { name: "조건 분기", exact: true })
    .click();
  await expect.poll(() => model.state().doc.nodes.length).toBe(1);
  await page.getByRole("button", { name: "진단 1", exact: true }).click();
  await expect(page.getByLabel("노드 작성 진단")).toContainText(
    "입력이 연결되지 않았습니다",
  );
  await page.getByRole("button", { name: "전체 보기", exact: true }).click();
  await checkView(page, info, "nodes-incomplete");
  const incomplete = await save(page);
  expect(incomplete.nodes[0].kind).toBe("branch");
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "flow.node-graph.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(example("flow"))),
  });
  await expect.poll(() => model.state().doc.nodes.length).toBe(5);
  const flow = model.state().doc;
  const bool = flow.nodes.find((n) => n.kind === "boolean")!;
  const action = flow.nodes.find((n) => n.kind === "action")!;
  const revision = model.state().revision;
  await dragBetween(
    page,
    page
      .getByTestId(`rf__node-${bool.id}`)
      .locator(".react-flow__handle.source"),
    page
      .getByTestId(`rf__node-${action.id}`)
      .locator('.react-flow__handle.target[data-handleid="in"]'),
  );
  await expect(page.getByRole("alert")).toContainText("같은 자료형");
  expect(model.state().doc).toEqual(flow);
  expect(model.state().revision).toBe(revision);
});

test("adding from a loose connection is atomic and property drafts survive invalid input", async ({
  page,
}, info) => {
  const model = await openNodes(page, info);
  const original = await save(page);
  const number = original.nodes.find((n) => n.kind === "number")!;
  const node = page.getByTestId(`rf__node-${number.id}`);
  const handle = node.locator(
    '.react-flow__handle.source[data-handleid="value"]',
  );
  // The port must be reachable by a real pointer, not covered by a sibling card.
  await expect
    .poll(() =>
      handle.evaluate((element) => {
        const rect = element.getBoundingClientRect();
        return (
          document.elementFromPoint(
            rect.x + rect.width / 2,
            rect.y + rect.height / 2,
          ) === element
        );
      }),
    )
    .toBe(true);
  await capture(page, info, "value-example-reachable-port");
  const a = (await handle.boundingBox())!;
  const canvas = (await page.getByTestId("node-canvas").boundingBox())!;
  await page.mouse.move(a.x + a.width / 2, a.y + a.height / 2);
  await page.mouse.down();
  await page.mouse.move(canvas.x + canvas.width - 30, canvas.y + 35, {
    steps: 12,
  });
  await page.mouse.up();
  await expect(page.getByRole("dialog")).toContainText("연결할 노드 추가");
  await page.getByLabel("추가할 노드 검색", { exact: true }).fill("참");
  await expect(page.getByRole("dialog")).toContainText(
    "일치하는 노드가 없습니다",
  );
  await page.getByLabel("추가할 노드 검색", { exact: true }).fill("곱하기");
  await capture(page, info, "nodes-add-dialog");
  await expect(page.getByRole("dialog")).toBeInViewport({ ratio: 1 });
  await inspectAccessibility(page, info, "nodes-add-dialog");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: /곱하기/ })
    .click();
  await expect.poll(() => model.state().doc.nodes.length).toBe(5);
  expect(model.state().doc.edges).toHaveLength(4);
  expect(model.state().undoCount).toBe(1);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(original);
  await node.locator(".ng-drag-handle").click();
  await page.getByLabel("값", { exact: true }).fill("숫자 아님");
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("유한한 숫자");
  expect(model.state().doc).toEqual(original);
  const other = original.nodes.find((n) => n.id !== number.id)!;
  await page
    .getByTestId(`rf__node-${other.id}`)
    .locator(".ng-drag-handle")
    .click();
  await expect(page.getByLabel("값", { exact: true })).toHaveValue("숫자 아님");
  await page.getByRole("button", { name: "취소", exact: true }).click();
  await expect(page.getByLabel("값", { exact: true })).toHaveValue(
    String(number.values.value),
  );
  expect(model.state().doc).toEqual(original);
  await node.locator(".ng-card").focus();
  await page.keyboard.press("Shift+F10");
  await expect(page.getByRole("menu")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(node.locator(".ng-card")).toBeFocused();
  // Exercise the documented in-app clipboard when the platform API is absent.
  await page.evaluate(() => {
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: undefined,
    });
  });
  await page.keyboard.press("Control+c");
  await expect(
    page.getByTestId("node-editor").getByRole("status"),
  ).toContainText("앱 안에 노드를 복사했습니다");
  await page.keyboard.press("Control+v");
  await expect.poll(() => model.state().doc.nodes.length).toBe(5);
  expect(model.state().undoCount).toBe(1);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(original);
});
