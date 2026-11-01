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
import { fixture } from "../model/fixtures";
import { parseDocument } from "../../src/editor/serialization";

async function openFixture(page: Page, doc = fixture()) {
  await page.getByLabel("전투 맵 JSON 열기").setInputFiles({
    name: "map.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(doc)),
  });
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "계속", exact: true })
    .click();
  await expect(page.getByRole("status", { name: "문서 상태" })).toContainText(
    "파일을 열었습니다",
  );
}
async function at(page: Page, key: string) {
  const box = (await page.locator(`[data-cell="${key}"]`).boundingBox())!;
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}
async function click(
  page: Page,
  key: string,
  button: "left" | "right" = "left",
) {
  const p = await at(page, key);
  await page.mouse.click(p.x, p.y, { button });
}
async function startRectangle(
  page: Page,
  from: string,
  to: string,
  button: "left" | "right" = "left",
) {
  const a = await at(page, from),
    b = await at(page, to);
  await page.keyboard.down("Shift");
  await page.mouse.move(a.x, a.y);
  await page.mouse.down({ button });
  await page.mouse.move(b.x, b.y, { steps: 3 });
  await page.keyboard.up("Shift");
}
async function saved(page: Page) {
  const pending = page.waitForEvent("download");
  await page.getByRole("button", { name: "저장", exact: true }).click();
  return parseDocument(await readFile((await (await pending).path())!, "utf8"));
}

test("terrain borders stay continuous around isolated, joined and stepped cells without blocking paint", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  const doc = fixture();
  doc.zones = [];
  doc.routes = [];
  doc.points = [];
  doc.waves = [];
  doc.terrain = { "4,1": "obstacle", "4,4": "obstacle" };
  for (const key of [
    "1,1",
    "2,1",
    "3,1",
    "1,2",
    "3,2",
    "4,2",
    "1,3",
    "2,3",
    "3,3",
    "4,3",
    "6,1",
    "5,4",
    "6,4",
    "7,4",
  ])
    doc.terrain[key] = "platform";
  await openFixture(page, doc);
  await capture(page, info, "terrain-borders");
  await inspectLayout(page, info, "terrain-borders");
  await inspectAccessibility(page, info, "terrain-borders");
  await page.getByRole("button", { name: "확대", exact: true }).click();
  await capture(page, info, "terrain-borders-zoomed");
  await page
    .getByRole("button", { name: "고지대 / 플랫폼 P", exact: true })
    .click();
  const box = (await page.locator('[data-cell="1,1"]').boundingBox())!;
  // The outline must not capture a click just inside the adjoining cell.
  await page.mouse.click(box.x + box.width + 0.5, box.y + box.height / 2, {
    button: "right",
  });
  let result = await saved(page);
  expect(result.terrain["2,1"]).toBeUndefined();
  expect(result.terrain["1,1"]).toBe("platform");
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  expect((await saved(page)).terrain).toEqual(doc.terrain);
  await page.getByLabel("지형 표시", { exact: true }).uncheck();
  await expect(page.locator(".terrain-borders")).toHaveCount(0);
  await page.getByLabel("지형 표시", { exact: true }).check();
  await page
    .getByRole("button", { name: "유효 영역 제거 X", exact: true })
    .click();
  await click(page, "6,1");
  result = await saved(page);
  expect(result.valid).not.toContain("6,1");
  expect(result.terrain["6,1"]).toBe("platform");
  await expect(page.locator('[data-cell="6,1"]')).toHaveClass(/void-cell/);
  await page.getByRole("button", { name: "맵 정보", exact: true }).click();
  await capture(page, info, "terrain-borders-hole");
});

test("shift rectangles preview and commit once; right drag removes terrain only; cancel and locks", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await openFixture(page);
  await page.getByRole("button", { name: "장애물 O", exact: true }).click();
  await startRectangle(page, "1,1", "3,2");
  await expect(page.locator(".paint-bounds")).toBeVisible();
  await expect(
    page.getByRole("status", { name: "범위 편집 미리보기" }),
  ).toContainText("6셀");
  await expect(page.locator('[data-cell="1,1"]')).toHaveClass(/floor/);
  await capture(page, info, "rectangle-apply");
  await inspectLayout(page, info, "rectangle-apply");
  await inspectAccessibility(page, info, "rectangle-apply");
  // Shift has already been released: continue the same rectangle to nine cells.
  const end = await at(page, "3,3");
  await page.mouse.move(end.x, end.y);
  await expect(
    page.getByRole("status", { name: "범위 편집 미리보기" }),
  ).toContainText("9셀");
  await page.mouse.up();
  let doc = await saved(page);
  expect(doc.terrain["1,3"]).toBe("obstacle");
  expect(doc.terrain["2,2"]).toBe("obstacle");
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  expect((await saved(page)).terrain).toEqual(fixture().terrain);
  await page.getByRole("button", { name: "다시 실행", exact: true }).click();
  await startRectangle(page, "3,3", "1,1", "right");
  await expect(page.locator(".paint-bounds")).toHaveCSS("fill", "none");
  await expect(page.locator(".paint-bounds")).toHaveClass(/remove-preview/);
  await capture(page, info, "rectangle-remove");
  await page.mouse.up({ button: "right" });
  doc = await saved(page);
  expect(doc.terrain).toEqual(fixture().terrain);
  expect(doc.valid).toEqual(fixture().valid);
  expect(doc.zones).toEqual(fixture().zones);
  expect(doc.routes).toEqual(fixture().routes);
  await startRectangle(page, "1,1", "3,2");
  await page.keyboard.press("Escape");
  await page.mouse.up();
  expect((await saved(page)).terrain).toEqual(fixture().terrain);
  await page.getByLabel("지형 잠금", { exact: true }).check();
  await click(page, "6,2", "right");
  expect((await saved(page)).terrain["6,2"]).toBe("platform");
});

test("zone replacement, right removal, kind changes and targeted route deletion", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await openFixture(page);
  await page
    .getByRole("button", { name: "아군 배치 구역", exact: true })
    .click();
  await page
    .getByRole("button", { name: "구역 칠하기 Z", exact: true })
    .click();
  await startRectangle(page, "0,2", "0,3");
  await page.mouse.up();
  let doc = await saved(page);
  expect(doc.zones[1].cells).toEqual([]);
  expect(doc.zones[0].cells).toEqual(["6,2", "6,3", "0,2", "0,3"]);
  expect(doc.waves[0].zoneId).toBe("zone-spawn");
  await capture(page, info, "exclusive-zones");
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  expect((await saved(page)).zones).toEqual(fixture().zones);
  await page.getByRole("button", { name: "다시 실행", exact: true }).click();
  await startRectangle(page, "0,2", "0,3", "right");
  await page.mouse.up({ button: "right" });
  expect((await saved(page)).zones[0].cells).toEqual(["6,2", "6,3"]);
  await page.getByRole("button", { name: "+ 배치 구역", exact: true }).click();
  await click(page, "6,2");
  await page.getByLabel("구역 종류", { exact: true }).selectOption("spawn");
  doc = await saved(page);
  expect(doc.zones[0].cells).toEqual(["6,3"]);
  expect(doc.zones[2].kind).toBe("spawn");
  expect(doc.zones[2].cells).toEqual(["6,2"]);
  await page
    .getByRole("button", { name: "경로 점 추가 R", exact: true })
    .click();
  // Right-clicking empty terrain must not add a route or delete any zone.
  await click(page, "1,1", "right");
  expect((await saved(page)).routes).toEqual(fixture().routes);
  const point = page.locator('[data-point="route-point-b"] circle');
  await point.click({ button: "right" });
  doc = await saved(page);
  expect(doc.routes[0].points.map((p) => p.id)).toEqual(["route-point-a"]);
  expect(doc.zones[2].cells).toEqual(["6,2"]);
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  expect((await saved(page)).routes).toEqual(fixture().routes);
  await page
    .getByRole("button", { name: "유효 영역 추가 A", exact: true })
    .click();
  await click(page, "6,2", "right");
  doc = await saved(page);
  expect(doc.valid).not.toContain("6,2");
  expect(doc.terrain["6,2"]).toBe("platform");
  expect(doc.zones[2].cells).toEqual(["6,2"]);
});
