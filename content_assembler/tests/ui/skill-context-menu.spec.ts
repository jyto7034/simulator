import { readFile } from "node:fs/promises";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectLayout,
  inspectAccessibility,
} from "./fixtures";
import { parseSkill } from "../../src/skill/validation";
import type { Page } from "@playwright/test";
async function openSkill(page: Page) {
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "스킬", exact: true })
    .click();
  await ready(page);
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
  const doc = parseSkill(await readFile((await result.path())!, "utf8"));
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  return doc;
}

test("node context menu targets the clicked block, returns focus, and shares reversible edits", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await openSkill(page);
  const original = await save(page);
  const repeatId = original.routines[2].blocks[1].id;
  const repeat = page.locator(`.skill-graph-node[data-item-id="${repeatId}"]`);
  const button = repeat.locator(".skill-node-content");
  await button.click({ button: "right" });
  const menu = page.getByRole("menu");
  await expect(menu).toBeVisible();
  await expect(repeat).toHaveAttribute("data-selected", "true");
  await expect(
    menu.getByRole("menuitem", { name: "블록 아래로 이동", exact: true }),
  ).toHaveAttribute("aria-disabled", "true");
  await expect(menu).toBeInViewport({ ratio: 1 });
  await capture(page, info, "node-context-menu");
  await inspectAccessibility(page, info, "node-context-menu");
  await page.keyboard.press("Escape");
  await expect(menu).toHaveCount(0);
  await expect(button).toBeFocused();
  await expect(
    page.getByRole("button", { name: "실행 취소", exact: true }),
  ).toBeDisabled();
  await expect(page.getByTestId("skill-dirty")).toContainText("파일 확인됨");

  await button.press("Shift+F10");
  await expect(menu).toBeVisible();
  await menu
    .getByRole("menuitem", { name: "안쪽 절차에 추가", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await expect(page.getByLabel("블록 검색")).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(button).toBeFocused();

  await button.click({ button: "right" });
  await menu.getByRole("menuitem", { name: "블록 복제", exact: true }).click();
  await ready(page);
  const duplicated = await save(page);
  const clone = duplicated.routines[2].blocks[2];
  expect(clone.id).not.toBe(repeatId);
  expect(clone.children).toHaveLength(3);
  expect(clone.children.map((b) => b.id)).not.toEqual(
    original.routines[2].blocks[1].children.map((b) => b.id),
  );
  await expect(
    page.locator(`.skill-graph-node[data-item-id="${clone.id}"]`),
  ).toHaveAttribute("data-selected", "true");
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  await ready(page);

  await button.click({ button: "right" });
  await menu
    .getByRole("menuitem", { name: "블록 위로 이동", exact: true })
    .click();
  await expect(button).toHaveAccessibleName("1. 유한 반복 선택");
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  await expect(button).toHaveAccessibleName("2. 유한 반복 선택");

  await button.click({ button: "right" });
  await menu.getByRole("menuitem", { name: "블록 삭제", exact: true }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(button).toBeFocused();
  await expect(button).toBeVisible();
  await button.click({ button: "right" });
  await menu.getByRole("menuitem", { name: "블록 삭제", exact: true }).click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "계속", exact: true })
    .click();
  await expect(repeat).toHaveCount(0);
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  await ready(page);
  expect(await save(page)).toEqual(original);
  await inspectLayout(page, info, "context-menu-restored");
});

test("event, branch and synthetic join menus expose appropriate actions", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await openSkill(page);
  await page.getByRole("button", { name: "예제", exact: true }).click();
  await page.getByRole("menuitem", { name: "두건 예제", exact: true }).click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "계속", exact: true })
    .click();
  await ready(page);
  const menu = page.getByRole("menu");
  const event = page
    .locator(".skill-graph-node.event")
    .last()
    .locator(".skill-node-content");
  await event.click({ button: "right" });
  await expect(
    menu.getByRole("menuitem", { name: "동작 삭제", exact: true }),
  ).toBeVisible();
  await expect(
    menu.getByRole("menuitem", { name: "블록 복제", exact: true }),
  ).toHaveCount(0);
  await menu
    .getByRole("menuitem", { name: "마지막에 블록 추가", exact: true })
    .click();
  await page.keyboard.press("Escape");
  await expect(event).toBeFocused();

  const branch = page.getByRole("button", {
    name: "2. 조건 분기 선택",
    exact: true,
  });
  await branch.click({ button: "right" });
  await expect(
    menu.getByRole("menuitem", { name: "아니면 절차에 추가", exact: true }),
  ).toBeVisible();
  await expect(menu).toBeInViewport({ ratio: 1 });
  await capture(page, info, "branch-context-menu");
  await menu
    .getByRole("menuitem", { name: "내부 절차 펼치기", exact: true })
    .click();
  await ready(page);
  const join = page.locator(".skill-graph-node.join").first();
  await join.click({ button: "right" });
  await expect(menu.getByRole("menuitem")).toHaveCount(1);
  await menu
    .getByRole("menuitem", { name: "원본 블록 보기", exact: true })
    .click();
  await expect(branch).toHaveAttribute("aria-pressed", "true");
  await branch.click({ button: "right" });
  await menu
    .getByRole("menuitem", { name: "아니면 절차에 추가", exact: true })
    .click();
  await page.getByLabel("블록 검색").fill("기다리기");
  await page
    .getByRole("button", { name: "기다리기 블록 추가", exact: true })
    .click();
  await expect(
    page.getByLabel("대기 시간 (초 또는 식)", { exact: true }),
  ).toHaveValue("");
  const saved = await save(page);
  expect(saved.routines[2].blocks[1].otherwise.map((b) => b.kind)).toEqual([
    "input",
    "wait",
  ]);
});
