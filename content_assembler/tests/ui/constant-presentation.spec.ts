import { readFile } from "node:fs/promises";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectLayout,
  inspectAccessibility,
} from "./fixtures";
import { observeNodes } from "./node-observer";
import type { GraphDocument } from "../../src/nodes/types";

test("private constants: fold, edit, reveal, duplicate with values, undo and reopen", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "노드", exact: true })
    .click();
  const doc: GraphDocument = JSON.parse(
    await readFile("content/king-of-greed.node-graph.json", "utf8"),
  );
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "greed.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(doc)),
  });
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "교체", exact: true })
    .click();
  const volley = doc.nodes.find(
    (n) => n.kind === "skill-modifier-continuous-volley",
  )!;
  const cost = doc.nodes.find(
    (n) =>
      n.id ===
      doc.edges.find(
        (e) => e.target.node === volley.id && e.target.port === "costInterval",
      )!.source.node,
  )!;
  const focus = async (id: string) => {
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(id);
    await expect(page.getByTestId(`rf__node-${id}`)).toBeInViewport();
  };
  const shot = async (label: string) => {
    await capture(page, info, label);
    await inspectLayout(page, info, label);
    await inspectAccessibility(page, info, label);
  };
  await focus(volley.id);
  await shot("constants-before");
  const fold = page.getByRole("button", { name: "고정값 접기", exact: true });
  await fold.click();
  await expect(fold).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByTestId(`rf__node-${cost.id}`)).toHaveCount(0);
  expect(model.state().doc).toEqual(doc);
  expect(model.state().undoCount).toBe(0);
  await shot("constants-after");
  const edit = page.getByTestId(`rf__node-${volley.id}`).getByRole("button", {
    name: `${volley.title} HP 비용 주기 연결값 수정`,
    exact: true,
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await page.getByRole("dialog").getByRole("textbox").fill("2");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "값 적용", exact: true })
    .click();
  await expect
    .poll(
      () => model.state().doc.nodes.find((n) => n.id === cost.id)?.values.value,
    )
    .toBe(2);
  await expect(edit).toBeFocused();
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  // Navigation reveals the original node; selecting the consumer folds it again.
  await focus(cost.id);
  await expect(page.getByTestId(`rf__node-${cost.id}`)).toBeVisible();
  await focus(volley.id);
  await expect(page.getByTestId(`rf__node-${cost.id}`)).toHaveCount(0);
  await page.getByRole("button", { name: "복제", exact: true }).click();
  await expect
    .poll(() => model.state().doc.nodes.length)
    .toBe(doc.nodes.length + 3);
  const after = model.state().doc,
    copy = after.nodes.find(
      (n) => n.kind === volley.kind && n.id !== volley.id,
    )!;
  const copiedCostId = after.edges.find(
    (e) => e.target.node === copy.id && e.target.port === "costInterval",
  )!.source.node;
  expect(after.nodes.find((n) => n.id === copiedCostId)?.values.value).toBe(1);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().doc).toEqual(doc);
  const pending = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const download = await pending;
  const saved = JSON.parse(await readFile((await download.path())!, "utf8"));
  expect(saved).toEqual(doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "reopened.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(saved)),
  });
  await expect.poll(() => model.state().doc).toEqual(doc);
  expect(model.state().dirty).toBe(false);
});
