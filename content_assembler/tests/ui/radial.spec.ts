import { readFile } from "node:fs/promises";
import type { Page } from "@playwright/test";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectLayout,
  inspectAccessibility,
} from "./fixtures";
import { parseDocument } from "../../src/editor/serialization";

async function openRadial(page: Page) {
  const canvas = page.locator(".map-canvas");
  await canvas.focus();
  const box = (await canvas.boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.keyboard.down("q");
  await expect(
    page.getByRole("menu", { name: "빠른 편집 도구" }),
  ).toBeVisible();
}
async function choose(page: Page, label: string) {
  await openRadial(page);
  await page.getByRole("menuitem", { name: label, exact: true }).hover();
  await page.keyboard.up("q");
  await expect(page.getByRole("menu", { name: "빠른 편집 도구" })).toBeHidden();
}
async function cell(page: Page, key: string) {
  const box = (await page.locator(`[data-cell="${key}"]`).boundingBox())!;
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}
async function clickCell(page: Page, key: string) {
  const p = await cell(page, key);
  await page.mouse.click(p.x, p.y);
}
async function saved(page: Page) {
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "저장", exact: true }).click();
  return parseDocument(
    await readFile((await (await download).path())!, "utf8"),
  );
}

test("radial release selects without painting; cancel, edges, focus and temporary tools", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  const undo = page.getByRole("button", { name: "실행 취소", exact: true });
  const canvas = page.locator(".map-canvas");
  await openRadial(page);
  await page.getByRole("menuitem", { name: "장애물", exact: true }).hover();
  await expect(
    page.locator('.radial-item[data-tool="obstacle"]'),
  ).toHaveAttribute("data-active", "true");
  await capture(page, info, "radial-menu");
  await inspectLayout(page, info, "radial-menu");
  await inspectAccessibility(page, info, "radial-menu");
  await page.keyboard.press("b");
  await page.keyboard.up("q");
  await expect(
    page.getByRole("button", { name: "장애물 O", exact: true }),
  ).toHaveAttribute("aria-pressed", "true");
  await expect(undo).toBeDisabled();
  await clickCell(page, "3,3");
  await expect(page.locator('[data-cell="3,3"]')).toHaveClass(/obstacle/);

  // A neutral release and Escape preserve the active tool and document.
  await openRadial(page);
  await page.keyboard.up("q");
  await openRadial(page);
  await page.getByRole("menuitem", { name: "일반 바닥", exact: true }).hover();
  await page.keyboard.press("Escape");
  await page.keyboard.up("q");
  await expect(canvas).toBeFocused();
  await expect(page.locator(".current-tool")).toHaveText("장애물");

  await page.keyboard.down("d");
  await expect(page.locator(".current-tool")).toHaveText("지형 지우기 · 임시");
  await clickCell(page, "3,3");
  await page.keyboard.up("d");
  await expect(page.locator(".current-tool")).toHaveText("장애물");
  await expect(page.locator('[data-cell="3,3"]')).toHaveClass(/floor/);
  const before = await cell(page, "3,3");
  await page.keyboard.down("Space");
  await page.mouse.move(before.x, before.y);
  await page.mouse.down();
  await page.mouse.move(before.x + 35, before.y + 20);
  await page.mouse.up();
  await page.keyboard.up("Space");
  expect((await cell(page, "3,3")).x - before.x).toBeCloseTo(35, 0);
  await expect(page.locator(".current-tool")).toHaveText("장애물");
  await page.getByRole("button", { name: "화면 맞춤", exact: true }).click();

  const bounds = (await canvas.boundingBox())!;
  await canvas.focus();
  await page.mouse.move(bounds.x + 2, bounds.y + 2);
  await page.keyboard.down("q");
  await expect(
    page.getByRole("menu", { name: "빠른 편집 도구" }),
  ).toBeInViewport({ ratio: 1 });
  await capture(page, info, "radial-edge");
  await page.keyboard.up("q");
  await expect(page.locator(".current-tool")).toHaveText("장애물");
  await openRadial(page);
  await page.getByRole("menuitem", { name: "고지대", exact: true }).hover();
  await page.evaluate(() => window.dispatchEvent(new Event("blur")));
  await page.keyboard.up("q");
  await expect(page.getByRole("menu", { name: "빠른 편집 도구" })).toBeHidden();
  await expect(page.locator(".current-tool")).toHaveText("장애물");
  expect((await saved(page)).terrain).toEqual({});

  const name = page.getByLabel("맵 이름", { exact: true });
  await name.focus();
  await page.keyboard.press("q");
  await expect(page.getByRole("menu", { name: "빠른 편집 도구" })).toBeHidden();
  await expect(name).toHaveValue(/q/);
  await page.keyboard.press("Escape");
});

test("radial zones and routes create on first edit, choose targets and preserve one-edit undo", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await choose(page, "배치 구역");
  await expect(
    page.getByRole("button", { name: "실행 취소", exact: true }),
  ).toBeDisabled();
  await clickCell(page, "2,2");
  let doc = await saved(page);
  expect(doc.zones).toHaveLength(1);
  expect(doc.zones[0].cells).toEqual(["2,2"]);

  await choose(page, "등장 구역");
  const at = await cell(page, "4,4");
  await page.mouse.move(at.x, at.y);
  await page.mouse.down();
  await page.keyboard.press("q");
  await expect(page.getByRole("menu", { name: "빠른 편집 도구" })).toBeHidden();
  await page.keyboard.press("Escape");
  await page.mouse.up();
  expect((await saved(page)).zones).toHaveLength(1);
  await clickCell(page, "4,4");

  await choose(page, "경로");
  await clickCell(page, "4,4");
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  expect((await saved(page)).routes).toHaveLength(0);
  await page.getByRole("button", { name: "다시 실행", exact: true }).click();
  // Redo restores the route; re-select it after undo cleared the removed selection.
  await choose(page, "경로");
  await clickCell(page, "6,4");
  doc = await saved(page);
  expect(doc.zones.map((zone) => zone.kind)).toEqual(["deploy", "spawn"]);
  expect(doc.routes).toHaveLength(1);
  expect(doc.routes[0].points.map(({ x, y }) => ({ x, y }))).toEqual([
    { x: 4.5, y: 4.5 },
    { x: 6.5, y: 4.5 },
  ]);
  await choose(page, "배치 구역");
  await clickCell(page, "2,3");
  await page
    .getByRole("button", { name: "+ 새 배치 구역", exact: true })
    .click();
  expect((await saved(page)).zones).toHaveLength(2);
  await clickCell(page, "8,5");
  doc = await saved(page);
  expect(doc.zones).toHaveLength(3);
  expect(doc.zones[0].cells).toEqual(["2,2", "2,3"]);
  await page
    .getByLabel("편집할 배치 구역", { exact: true })
    .selectOption(doc.zones[0].id);
  await capture(page, info, "radial-targets");
  await clickCell(page, "2,4");
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  expect((await saved(page)).zones).toHaveLength(2);
  await page.getByLabel("구역 잠금", { exact: true }).check();
  await choose(page, "등장 구역");
  await clickCell(page, "5,5");
  await expect(
    page.getByLabel("편집할 등장 구역", { exact: true }),
  ).toBeDisabled();
  expect((await saved(page)).zones[1].cells).toEqual(["4,4"]);
  await page.getByLabel("구역 잠금", { exact: true }).uncheck();
  await page.getByLabel("구역 표시", { exact: true }).uncheck();
  await choose(page, "등장 구역");
  await clickCell(page, "5,5");
  expect((await saved(page)).zones[1].cells).toEqual(["4,4"]);
});

test("radial toolbar supports keyboard selection and focus return", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  const trigger = page.getByRole("button", {
    name: "빠른 도구 메뉴 (Q)",
    exact: true,
  });
  await trigger.focus();
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("menu", { name: "빠른 편집 도구" }),
  ).toBeFocused();
  await page.keyboard.press("Home");
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("ArrowRight");
  await expect(
    page.getByRole("menuitem", { name: "고지대", exact: true }),
  ).toBeFocused();
  await capture(page, info, "radial-keyboard");
  await page.keyboard.press("Enter");
  await expect(trigger).toBeFocused();
  await expect(
    page.getByRole("button", { name: "고지대 / 플랫폼 P", exact: true }),
  ).toHaveAttribute("aria-pressed", "true");
  await trigger.click();
  await page.keyboard.press("Escape");
  await expect(trigger).toBeFocused();
  await expect(page.getByRole("menu", { name: "빠른 편집 도구" })).toBeHidden();
  const at = await cell(page, "3,3");
  await page.mouse.move(at.x, at.y);
  await trigger.focus();
  await page.keyboard.press("Space");
  await expect(
    page.getByRole("menu", { name: "빠른 편집 도구" }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(trigger).toBeFocused();
});
