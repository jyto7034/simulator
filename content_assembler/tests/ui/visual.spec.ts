import { test, expect, openWorkspace } from "./fixtures";
import { fixture } from "../model/fixtures";

// Opt-in comparison, after baseline images have been visually reviewed.
test("workspace and menu match reviewed design @visual", async ({
  page,
}, testInfo) => {
  await openWorkspace(page, testInfo);
  // Fixed document IDs make the optional visual comparison reproducible.
  await page.getByLabel("전투 맵 JSON 열기").setInputFiles({
    name: "fixture.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(fixture())),
  });
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "계속", exact: true })
    .click();
  await expect(page.getByRole("status", { name: "문서 상태" })).toContainText(
    "파일을 열었습니다",
  );
  await expect(page).toHaveScreenshot("workspace.png");
  await page.getByRole("button", { name: "화면 테마 변경" }).click();
  await expect(page.getByRole("menu")).toBeVisible();
  await expect(page).toHaveScreenshot("theme-menu.png");
});
