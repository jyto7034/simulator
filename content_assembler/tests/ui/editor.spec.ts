import { readFile } from "node:fs/promises";
import type { Page, TestInfo } from "@playwright/test";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
  inspectLayout,
} from "./fixtures";
import { fixture } from "../model/fixtures";
import { parseDocument } from "../../src/editor/serialization";
import type { MapDocument } from "../../src/editor/model";
async function openFile(page: Page, doc: MapDocument) {
  await page.getByLabel("전투 맵 JSON 열기").setInputFiles({
    name: "fixture.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(doc)),
  });
  const dialog = page.getByRole("dialog");
  if (await dialog.isVisible())
    await dialog.getByRole("button", { name: "계속", exact: true }).click();
  await expect(page.getByRole("status", { name: "문서 상태" })).toContainText(
    "파일을 열었습니다",
  );
}
async function exported(page: Page): Promise<MapDocument> {
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "저장", exact: true }).click();
  const file = await download;
  return parseDocument(await readFile((await file.path())!, "utf8"));
}
async function position(page: Page, key: string, fx = 0.5, fy = 0.5) {
  const b = await page.locator(`[data-cell="${key}"]`).boundingBox();
  expect(b).not.toBeNull();
  return { x: b!.x + b!.width * fx, y: b!.y + b!.height * fy };
}
async function cellClick(page: Page, key: string, fx = 0.5, fy = 0.5) {
  const p = await position(page, key, fx, fy);
  await page.mouse.click(p.x, p.y);
}
async function fill(page: Page, label: string, value: string) {
  const el = page.getByLabel(label, { exact: true });
  await el.fill(value);
  await el.press("Enter");
}

test("file replacement rechecks a prepared open after confirmation and preserves edits for a superseded request", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await openFile(page, fixture());
  await page.getByRole("button", { name: "맵 정보", exact: true }).click();
  await fill(page, "맵 이름", "보존해야 할 현재 편집");
  const before = await exported(page);
  const input = page.getByLabel("전투 맵 JSON 열기");
  await input.setInputFiles({
    name: "older.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(fixture("empty"))),
  });
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  // Control only the external read boundary. A newer request starts while an
  // earlier prepared replacement is waiting for user confirmation.
  await page.evaluate(() => {
    const original = File.prototype.text;
    File.prototype.text = function () {
      if (this.name !== "newer.json") return original.call(this);
      return new Promise<string>((resolve, reject) => {
        window.addEventListener(
          "release-newer-open",
          () => {
            File.prototype.text = original;
            void original.call(this).then(resolve, reject);
          },
          { once: true },
        );
      });
    };
  });
  const newer = fixture("irregular");
  newer.name = "최신 열기 요청";
  await input.setInputFiles({
    name: "newer.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(newer)),
  });
  await dialog.getByRole("button", { name: "계속", exact: true }).click();
  await expect(dialog).toBeHidden();
  await expect(page.getByRole("status", { name: "문서 상태" })).toContainText(
    "교체하지 않았습니다",
  );
  await expect(page.getByLabel("맵 이름", { exact: true })).toHaveValue(
    before.name,
  );
  await expect(page.getByTestId("dirty")).toContainText("미저장");
  await expect(
    page.getByRole("button", { name: "실행 취소", exact: true }),
  ).toBeEnabled();
  await capture(page, info, "stale-open-preserved");
  await inspectLayout(page, info, "stale-open-preserved");
  await inspectAccessibility(page, info, "stale-open-preserved");
  expect(await exported(page)).toEqual(before);
  await page.evaluate(() =>
    window.dispatchEvent(new Event("release-newer-open")),
  );
  await expect(dialog).toBeVisible();
  await dialog.getByRole("button", { name: "계속", exact: true }).click();
  await expect(page.getByLabel("맵 이름", { exact: true })).toHaveValue(
    newer.name,
  );
  await expect(page.getByTestId("dirty")).toContainText("파일 확인됨");
  await expect(
    page.getByRole("button", { name: "실행 취소", exact: true }),
  ).toBeDisabled();
  expect(await exported(page)).toEqual(newer);
});

test("cell selection clears on persistent editing tool changes and idle Escape, preserving property edits and navigation", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  const selected = page.locator('[data-selected-cell="1,1"]');
  const canvas = page.getByRole("application");
  await cellClick(page, "1,1");
  await expect(selected).toHaveCount(1);
  await expect(
    page.getByRole("button", { name: "실행 취소", exact: true }),
  ).toBeDisabled();
  await page.getByLabel("셀 지형", { exact: true }).selectOption("platform");
  await expect(selected).toHaveCount(1);
  await expect(page.locator('[data-cell="1,1"]')).toHaveClass(/platform/);
  await page.getByRole("button", { name: "확대", exact: true }).click();
  await expect(selected).toHaveCount(1);
  await page.getByRole("button", { name: "화면 이동 H", exact: true }).click();
  const start = await position(page, "1,1");
  await page.mouse.move(start.x, start.y);
  await page.mouse.down();
  await page.mouse.move(start.x + 20, start.y + 10);
  await page.mouse.up();
  await expect(selected).toHaveCount(1);
  await page
    .getByRole("button", { name: "선택 / 점 이동 V", exact: true })
    .click();
  const before = await exported(page);
  await page
    .getByRole("button", { name: "고지대 / 플랫폼 P", exact: true })
    .click();
  await expect(selected).toHaveCount(0);
  await expect(page.getByLabel("셀 지형", { exact: true })).toHaveCount(0);
  expect(await exported(page)).toEqual(before);
  await cellClick(page, "2,1");
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  expect(await exported(page)).toEqual(before);
  await capture(page, info, "tool-change-clears-cell-selection");
  await inspectLayout(page, info, "tool-change-clears-cell-selection");
  await inspectAccessibility(page, info, "tool-change-clears-cell-selection");

  await page
    .getByRole("button", { name: "선택 / 점 이동 V", exact: true })
    .click();
  await cellClick(page, "1,1");
  await page.keyboard.press("o");
  await expect(selected).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "다시 실행", exact: true }),
  ).toBeEnabled();
  await page.keyboard.press("v");
  await cellClick(page, "1,1");
  await page.keyboard.down("q");
  await page.getByRole("menuitem", { name: "고지대", exact: true }).hover();
  await page.keyboard.press("Escape");
  await page.keyboard.up("q");
  await expect(selected).toHaveCount(1);
  await page.keyboard.down("q");
  await page.getByRole("menuitem", { name: "고지대", exact: true }).hover();
  await page.keyboard.up("q");
  await expect(selected).toHaveCount(0);
  await page.keyboard.press("v");
  await cellClick(page, "1,1");
  await page.getByLabel("셀 지형", { exact: true }).focus();
  await page.keyboard.press("Escape");
  await expect(selected).toHaveCount(1);
  await canvas.focus();
  await page.keyboard.down("Space");
  const panStart = await position(page, "1,1");
  await page.mouse.move(panStart.x, panStart.y);
  await page.mouse.down();
  await page.mouse.move(panStart.x + 10, panStart.y + 10);
  await page.keyboard.press("Escape");
  await page.mouse.up();
  await page.keyboard.up("Space");
  await expect(selected).toHaveCount(1);
  await page.keyboard.press("Escape");
  await expect(selected).toHaveCount(0);
  expect(await exported(page)).toEqual(before);
  await openFile(page, fixture());
  await page
    .getByRole("button", { name: "아군 배치 구역", exact: true })
    .click();
  await page
    .getByRole("button", { name: "구역 칠하기 Z", exact: true })
    .click();
  await expect(page.getByLabel("구역 종류", { exact: true })).toHaveValue(
    "deploy",
  );
  await page.getByRole("button", { name: "주 이동 경로", exact: true }).click();
  await page
    .getByRole("button", { name: "경로 점 추가 R", exact: true })
    .click();
  await expect(page.getByRole("group", { name: "경로 점 목록" })).toBeVisible();
  await canvas.focus();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("group", { name: "경로 점 목록" })).toBeVisible();
});

test("expanded map cells accept terrain and zones immediately with holes and resize history preserved", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  const original = fixture("irregular");
  await openFile(page, original);
  await page.getByRole("button", { name: "맵 정보", exact: true }).click();
  await fill(page, "가로 (셀)", "10");
  await expect(page.locator('[data-cell="9,5"]')).toHaveClass(/floor/);
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  expect(await exported(page)).toEqual(original);
  await page.getByRole("button", { name: "다시 실행", exact: true }).click();
  await fill(page, "세로 (셀)", "8");
  await expect(page.locator('[data-cell="9,7"]')).toHaveClass(/floor/);
  await expect(page.locator('[data-cell="3,3"]')).toHaveClass(/void/);
  await fill(page, "가로 (셀)", "129");
  await expect(page.getByLabel("가로 (셀)", { exact: true })).toHaveValue("10");
  await page.getByRole("button", { name: "장애물 O", exact: true }).click();
  await cellClick(page, "9,1");
  await page
    .getByRole("button", { name: "고지대 / 플랫폼 P", exact: true })
    .click();
  await cellClick(page, "1,7");
  await page.getByRole("button", { name: "+ 배치 구역", exact: true }).click();
  await cellClick(page, "9,7");
  const expanded = await exported(page);
  expect(expanded.terrain["9,1"]).toBe("obstacle");
  expect(expanded.terrain["1,7"]).toBe("platform");
  expect(expanded.zones.at(-1)!.cells).toEqual(["9,7"]);
  expect(expanded.valid).not.toContain("3,3");
  await page.getByRole("button", { name: "맵 정보", exact: true }).click();
  await capture(page, info, "expanded-map-painted");
  await inspectLayout(page, info, "expanded-map-painted");
  await inspectAccessibility(page, info, "expanded-map-painted");
  await fill(page, "세로 (셀)", "4");
  await expect(
    page.getByRole("dialog", { name: "맵 크기 축소" }),
  ).toBeVisible();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "취소", exact: true })
    .click();
  expect(await exported(page)).toEqual(expanded);
  await fill(page, "세로 (셀)", "4");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "계속", exact: true })
    .click();
  expect((await exported(page)).terrain).toEqual(expanded.terrain);
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  expect(await exported(page)).toEqual(expanded);
  await openFile(page, expanded);
  expect(await exported(page)).toEqual(expanded);
});

test("brush and rectangle, cancellation, terrain versus holes, locks and one-stroke history", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await openFile(page, fixture());
  await page.getByRole("button", { name: "장애물 O", exact: true }).click();
  const a = await position(page, "1,1");
  const b = await position(page, "3,1");
  await page.mouse.move(a.x, a.y);
  await page.mouse.down();
  await page.mouse.move(b.x, b.y, { steps: 4 });
  await page.keyboard.press("Escape");
  await page.mouse.up();
  expect((await exported(page)).terrain["1,1"]).toBeUndefined();
  await page.mouse.move(a.x, a.y);
  await page.mouse.down();
  await page.mouse.move(b.x, b.y, { steps: 4 });
  await page.mouse.up();
  let d = await exported(page);
  expect(d.terrain["1,1"]).toBe("obstacle");
  expect(d.terrain["3,1"]).toBe("obstacle");
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  d = await exported(page);
  expect(d.terrain["1,1"]).toBeUndefined();
  expect(d.terrain["3,1"]).toBeUndefined();
  await page.getByRole("button", { name: "다시 실행", exact: true }).click();
  await page
    .getByRole("button", { name: "지형 지우기 E", exact: true })
    .click();
  await cellClick(page, "1,1");
  d = await exported(page);
  expect(d.valid).toContain("1,1");
  expect(d.terrain["1,1"]).toBeUndefined();
  await page.getByLabel("범위 방식", { exact: true }).selectOption("rectangle");
  await page
    .getByRole("button", { name: "유효 영역 제거 X", exact: true })
    .click();
  const c = await position(page, "2,2");
  const e = await position(page, "3,3");
  await page.mouse.move(c.x, c.y);
  await page.mouse.down();
  await page.mouse.move(e.x, e.y);
  await page.mouse.up();
  d = await exported(page);
  for (const k of ["2,2", "3,2", "2,3", "3,3"])
    expect(d.valid).not.toContain(k);
  await page.getByLabel("지형 잠금", { exact: true }).check();
  await page.getByRole("button", { name: "장애물 O", exact: true }).click();
  await cellClick(page, "1,1");
  expect((await exported(page)).terrain["1,1"]).toBeUndefined();
  await page.getByLabel("지형 표시", { exact: true }).uncheck();
  expect(
    await page.locator('[data-cell="3,1"]').getAttribute("class"),
  ).toContain("floor");
  await capture(page, info, "edited-hole");
});

test("zones routes landmarks and snap persist after zoom, pan, insertion, drag and reopen", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await openFile(page, fixture());
  await page.getByRole("button", { name: "+ 등장 구역", exact: true }).click();
  await cellClick(page, "1,1");
  await fill(page, "이름", "북쪽 진입 구역");
  await page
    .getByRole("button", { name: "+ 경로 만들기", exact: true })
    .click();
  await cellClick(page, "1,1");
  await cellClick(page, "5,1");
  await page.getByLabel("경로·지점 입력 스냅").selectOption("4");
  await cellClick(page, "6,1", 0.24, 0.74);
  let d = await exported(page);
  const route = d.routes.at(-1)!;
  expect(route.points.map((p) => [p.x, p.y])).toEqual([
    [1.5, 1.5],
    [5.5, 1.5],
    [6.25, 1.75],
  ]);
  await page.getByLabel("경로·지점 입력 스냅").selectOption("8");
  expect((await exported(page)).routes.at(-1)!.points).toEqual(route.points);
  await page.getByRole("button", { name: "확대", exact: true }).click();
  const canvas = page.getByRole("application");
  const cb = await canvas.boundingBox();
  await page.mouse.move(cb!.x + 100, cb!.y + 70);
  await page.mouse.down({ button: "middle" });
  await page.mouse.move(cb!.x + 120, cb!.y + 80);
  await page.mouse.up({ button: "middle" });
  await page
    .getByRole("button", { name: "이름 있는 지점 T", exact: true })
    .click();
  await cellClick(page, "3,1", 0.5, 0.5);
  await fill(page, "이름", "출입 감지 지점");
  await page
    .getByRole("button", { name: "선택 / 점 이동 V", exact: true })
    .click();
  const handle = page.locator(`[data-point="${route.points[0].id}"] circle`);
  const hb = await handle.boundingBox();
  const target = await position(page, "2,1");
  await page.mouse.move(hb!.x + hb!.width / 2, hb!.y + hb!.height / 2);
  await page.mouse.down();
  await page.mouse.move(target.x, target.y);
  await page.keyboard.press("Escape");
  await page.mouse.up();
  expect((await exported(page)).routes.at(-1)!.points[0].x).toBe(1.5);
  await page.mouse.move(hb!.x + hb!.width / 2, hb!.y + hb!.height / 2);
  await page.mouse.down();
  await page.mouse.move(target.x, target.y);
  await page.mouse.up();
  expect((await exported(page)).routes.at(-1)!.points[0].x).toBe(2.5);
  await page
    .getByRole("button", { name: "경로 점 추가 R", exact: true })
    .click();
  await cellClick(page, "4,1");
  d = await exported(page);
  expect(d.routes.at(-1)!.points.map((p) => p.x)).toEqual([
    2.5, 4.5, 5.5, 6.25,
  ]);
  expect(d.points.at(-1)!.name).toBe("출입 감지 지점");
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await expect(page.getByTestId("dirty")).toContainText("파일 확인됨");
  await openFile(page, d);
  expect(await exported(page)).toEqual(d);
  await capture(page, info, "routes-and-zones");
});

test("route cell uniqueness across clicks, subcell snap, dragging and coordinate fields", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await openFile(page, fixture());
  await page.getByRole("button", { name: "주 이동 경로", exact: true }).click();
  await page
    .getByRole("button", { name: "경로 점 추가 R", exact: true })
    .click();
  await cellClick(page, "0,2");
  await cellClick(page, "0,2");
  await page.getByLabel("경로·지점 입력 스냅").selectOption("4");
  await cellClick(page, "0,2", 0.24, 0.74);
  await expect(page.getByRole("status", { name: "문서 상태" })).toContainText(
    "이미 점이 있습니다",
  );
  expect((await exported(page)).routes).toEqual(fixture().routes);
  await expect(
    page.getByRole("button", { name: "실행 취소", exact: true }),
  ).toBeDisabled();
  await cellClick(page, "1,2", 0.24, 0.74);
  const added = (await exported(page)).routes[0].points.at(-1)!;
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  await cellClick(page, "6,2", 0.74, 0.24);
  await expect(
    page.getByRole("button", { name: "다시 실행", exact: true }),
  ).toBeEnabled();
  await page.getByRole("button", { name: "다시 실행", exact: true }).click();
  await page
    .getByRole("button", { name: "선택 / 점 이동 V", exact: true })
    .click();
  const handle = page.locator(`[data-point="${added.id}"] circle`);
  const box = (await handle.boundingBox())!;
  const target = await position(page, "6,2", 0.24, 0.74);
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(target.x, target.y, { steps: 3 });
  await page.mouse.up();
  expect((await exported(page)).routes[0].points.at(-1)).toEqual(added);
  await fill(page, "X 좌표", "6.75");
  await expect(page.getByLabel("X 좌표", { exact: true })).toHaveValue("1.25");
  await expect(page.getByRole("status", { name: "문서 상태" })).toContainText(
    "이미 점이 있습니다",
  );
  await capture(page, info, "route-overlap-rejected");
  await inspectLayout(page, info, "route-overlap-rejected");
  await inspectAccessibility(page, info, "route-overlap-rejected");
  expect((await exported(page)).routes[0].points.at(-1)).toEqual(added);
  await fill(page, "X 좌표", "1.75");
  expect((await exported(page)).routes[0].points.at(-1)!.x).toBe(1.75);
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  expect((await exported(page)).routes[0].points.at(-1)).toEqual(added);
  await page
    .getByRole("button", { name: "+ 경로 만들기", exact: true })
    .click();
  await cellClick(page, "0,2");
  const result = await exported(page);
  expect(result.routes).toHaveLength(2);
  expect(result.routes[1].points).toHaveLength(1);
  expect(result.routes[1].points[0]).toMatchObject({ x: 0.5, y: 2.5 });
  await openFile(page, result);
  expect(await exported(page)).toEqual(result);
});

test("all condition forms, enemy composition, diagnostics and saved references", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await openFile(page, fixture());
  await page
    .getByRole("button", { name: "1. 첫 번째 웨이브", exact: true })
    .click();
  await fill(page, "적 1 콘텐츠 ID", "enemy.reviewed");
  await fill(page, "적 1 수량", "7");
  await fill(page, "적 1 간격 (초)", "1.75");
  await fill(page, "전투 시작부터 (초)", "2.5");
  let d = await exported(page);
  expect(d.waves[0].trigger).toEqual({ kind: "time", timeMs: 2500 });
  expect(d.waves[0].enemies[0]).toMatchObject({
    contentId: "enemy.reviewed",
    count: 7,
    intervalMs: 1750,
  });
  await page.getByLabel("기본 조건", { exact: true }).selectOption("clear");
  await page
    .getByLabel("선행 웨이브", { exact: true })
    .selectOption("wave-first");
  await page.getByLabel("대체 발동 시각 사용", { exact: true }).check();
  await fill(page, "대체 발동 시각 (초)", "45");
  await page.getByRole("tab", { name: /진단/ }).click();
  await expect(
    page.getByRole("button").filter({ hasText: "WAVE_SELF" }),
  ).toBeVisible();
  await page.getByRole("button").filter({ hasText: "WAVE_SELF" }).click();
  await expect(page.getByLabel("기본 조건", { exact: true })).toBeFocused();
  for (const kind of ["hp", "death", "reach"]) {
    await page.getByLabel("기본 조건", { exact: true }).selectOption(kind);
    await page
      .getByLabel("임시 개체 슬롯", { exact: true })
      .selectOption("slot-boss");
    if (kind === "hp") await fill(page, "HP 이하 (%)", "35");
    if (kind === "reach") {
      await page
        .getByLabel("도달 지점", { exact: true })
        .selectOption("landmark-a");
      await fill(page, "도달 반경 (셀)", "2");
    }
    d = await exported(page);
    expect(d.waves[0].trigger.kind).toBe(kind);
    expect(d.waves[0].fallbackMs).toBe(45000);
  }
  await page.getByRole("button", { name: "복제", exact: true }).click();
  d = await exported(page);
  expect(d.waves).toHaveLength(2);
  expect(d.waves[0].id).not.toBe(d.waves[1].id);
  expect(d.waves[1].trigger).toEqual({
    kind: "reach",
    slotId: "slot-boss",
    pointId: "landmark-a",
    radius: 2,
  });
  await expect(
    page.getByRole("button", { name: "아래로", exact: true }),
  ).toBeDisabled();
  await page.getByRole("button", { name: "위로", exact: true }).click();
  expect((await exported(page)).waves[0].id).toBe(d.waves[1].id);
  await capture(page, info, "wave-conditions");
  await inspectAccessibility(page, info, "wave-conditions");
});

test("invalid file and download failure preserve edits; form Escape, shortcuts and replacement cancel", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await openFile(page, fixture());
  await page.getByLabel("맵 이름", { exact: true }).fill("입력 버퍼 저장");
  let d = await exported(page);
  expect(d.name).toBe("입력 버퍼 저장");
  await page.getByLabel("맵 이름", { exact: true }).fill("취소할 값");
  await page.getByLabel("맵 이름", { exact: true }).press("Escape");
  await expect(page.getByLabel("맵 이름", { exact: true })).toHaveValue(
    "입력 버퍼 저장",
  );
  await page.getByLabel("맵 이름", { exact: true }).press("Delete");
  await expect(page.getByRole("dialog")).toBeHidden();
  await page.getByLabel("전투 맵 JSON 열기").setInputFiles({
    name: "broken.json",
    mimeType: "application/json",
    buffer: Buffer.from("{oops"),
  });
  await expect(page.getByRole("status", { name: "문서 상태" })).toContainText(
    "열기 실패",
  );
  d = await exported(page);
  expect(d.name).toBe("입력 버퍼 저장");
  await page.evaluate(() => {
    URL.createObjectURL = () => {
      throw new Error("fixture download failure");
    };
  });
  await page.getByRole("button", { name: "저장", exact: true }).click();
  await expect(page.getByRole("status", { name: "문서 상태" })).toContainText(
    "저장 실패",
  );
  await expect(page.getByTestId("dirty")).toContainText("미저장");
  await page.getByRole("button", { name: "새로 만들기", exact: true }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await capture(page, info, "unsaved-dialog");
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toBeHidden();
  await expect(page.getByLabel("맵 이름", { exact: true })).toHaveValue(
    "입력 버퍼 저장",
  );
  await expect(
    page.getByRole("button", { name: "새로 만들기", exact: true }),
  ).toBeFocused();
});

test("fixed irregular, many, empty and incomplete states remain usable", async ({
  page,
}, info: TestInfo) => {
  await openWorkspace(page, info);
  for (const kind of ["irregular", "many", "empty", "incomplete"] as const) {
    await openFile(page, fixture(kind));
    await capture(page, info, `fixture-${kind}`);
    await inspectLayout(page, info, kind);
    if (kind === "many") {
      await page.getByRole("button", { name: /^17\. 추가 웨이브 16/ }).click();
      await capture(page, info, "long-wave");
      await inspectAccessibility(page, info, "long-wave");
    }
    if (kind === "incomplete") {
      await page.getByRole("tab", { name: /진단/ }).click();
      await capture(page, info, "incomplete-diagnostics");
      await inspectAccessibility(page, info, "incomplete-diagnostics");
    }
  }
});

test("expanded panel, keyboard tabs, long title marker, pointer cancellation and reference deletion", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  const doc = fixture();
  doc.name = "매우 긴 한글 전장 이름 ".repeat(15);
  await openFile(page, doc);
  await expect(page.getByTestId("dirty")).toBeInViewport({ ratio: 1 });
  await page.getByRole("button", { name: "패널 확장", exact: true }).click();
  await page.getByRole("tab", { name: /웨이브/ }).focus();
  await page.keyboard.press("ArrowRight");
  await expect(page.getByRole("tab", { name: /진단/ })).toBeFocused();
  await page.keyboard.press("ArrowLeft");
  await expect(page.getByRole("tab", { name: /웨이브/ })).toBeFocused();
  await page
    .getByRole("button", { name: "1. 첫 번째 웨이브", exact: true })
    .click();
  await capture(page, info, "expanded-wave");
  await page.getByRole("button", { name: "패널 축소", exact: true }).click();
  await page.getByRole("button", { name: "장애물 O", exact: true }).click();
  const p = await position(page, "1,1");
  await page.mouse.move(p.x, p.y);
  await page.mouse.down();
  await page.getByRole("application").dispatchEvent("pointercancel");
  await page.mouse.up();
  expect((await exported(page)).terrain["1,1"]).toBeUndefined();
  await page.getByRole("button", { name: "주 이동 경로", exact: true }).click();
  await page
    .getByRole("button", { name: "선택 대상 삭제", exact: true })
    .click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "계속", exact: true })
    .click();
  let result = await exported(page);
  expect(result.routes).toHaveLength(0);
  expect(result.waves[0].routeId).toBe("route-main");
  await page.getByRole("tab", { name: /진단/ }).click();
  await expect(
    page.getByRole("button").filter({ hasText: "ROUTE_REFERENCE" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  result = await exported(page);
  expect(result.routes[0].id).toBe("route-main");
});
