import { readFile } from "node:fs/promises";
import type { Locator, Page } from "@playwright/test";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectLayout,
  inspectAccessibility,
} from "./fixtures";
import { parseSkill } from "../../src/skill/validation";

async function position(node: Locator) {
  return node.evaluate((element) => {
    const matrix = new DOMMatrix(getComputedStyle(element).transform);
    return { x: matrix.m41, y: matrix.m42 };
  });
}
async function drag(page: Page, node: Locator) {
  const box = (await node.locator(".skill-node-content").boundingBox())!;
  const x = box.x + box.width / 2;
  const y = box.y + box.height / 2;
  await page.mouse.move(x, y);
  await page.mouse.down();
  await page.mouse.move(x + 60, y + 45, { steps: 12 });
  await page.mouse.up();
}
async function ready(page: Page) {
  await expect(page.getByTestId("skill-graph")).toHaveAttribute(
    "data-layout",
    "ready",
  );
}
async function save(page: Page) {
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "저장", exact: true }).click();
  const result = await download;
  const json = await readFile((await result.path())!, "utf8");
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  return parseSkill(json);
}

test("node dragging preserves document and manual positions until automatic alignment or document replacement", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  const navigation = page.getByRole("navigation", { name: "작업 공간" });
  await navigation.getByRole("button", { name: "스킬", exact: true }).click();
  await ready(page);
  const original = await save(page);
  const id = original.routines[0].blocks[0].id;
  const node = page.getByTestId(`rf__node-${id}`);
  const initial = await position(node);
  const other = page.getByTestId(
    `rf__node-${original.routines[2].blocks[1].id}`,
  );
  await other.locator(".skill-node-content").click();
  const otherInitial = await position(other);
  await drag(page, node);
  expect(await position(other)).toEqual(otherInitial);
  const moved = await position(node);
  expect(moved.x - initial.x).toBeGreaterThan(40);
  expect(moved.y - initial.y).toBeGreaterThan(30);
  await expect(node.locator(".skill-graph-node")).toHaveAttribute(
    "data-selected",
    "true",
  );
  await expect(
    page.getByRole("button", { name: "실행 취소", exact: true }),
  ).toBeDisabled();
  await expect(page.getByTestId("skill-dirty")).toContainText("파일 확인됨");
  await page.getByRole("button", { name: "전체 보기", exact: true }).click();
  await capture(page, info, "skill-node-dragged");
  await inspectLayout(page, info, "skill-node-dragged");
  await inspectAccessibility(page, info, "skill-node-dragged");

  // Other selection, view fitting, property edits and history must not snap it back.
  await page.getByRole("button", { name: "전체 보기", exact: true }).click();
  await page
    .getByRole("button", { name: "2. 유한 반복 선택", exact: true })
    .click();
  expect(await position(node)).toEqual(moved);
  await page.getByLabel("횟수 또는 유한 목록", { exact: true }).fill("5");
  await page.getByLabel("횟수 또는 유한 목록", { exact: true }).press("Enter");
  expect(await position(node)).toEqual(moved);
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  expect(await position(node)).toEqual(moved);
  await page
    .getByRole("button", { name: "2. 유한 반복 펼치기", exact: true })
    .click();
  await ready(page);
  expect(await position(node)).toEqual(moved);
  await page
    .getByRole("button", { name: "2. 유한 반복 접기", exact: true })
    .click();
  await ready(page);
  expect(await position(node)).toEqual(moved);
  await navigation.getByRole("button", { name: "맵", exact: true }).click();
  await navigation.getByRole("button", { name: "스킬", exact: true }).click();
  expect(await position(node)).toEqual(moved);
  expect(await save(page)).toEqual(original);

  await page.getByRole("button", { name: "자동 정렬", exact: true }).click();
  await expect.poll(() => position(node)).toEqual(initial);
  await expect(page.getByTestId("skill-dirty")).toContainText("파일 확인됨");
  await capture(page, info, "skill-node-realigned");
  await drag(page, node);
  expect(await position(node)).not.toEqual(initial);
  // Reopening the same IDs is a new document session, not a continuation of view state.
  await page.getByLabel("스킬 JSON 열기").setInputFiles({
    name: "same-document.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(original)),
  });
  await ready(page);
  await expect.poll(() => position(node)).toEqual(initial);
  expect(await save(page)).toEqual(original);
});
