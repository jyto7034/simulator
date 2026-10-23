import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectLayout,
  inspectAccessibility,
} from "./fixtures";

test("workspace and open theme menu", async ({ page }, testInfo) => {
  await openWorkspace(page, testInfo);
  const centerOffset = () =>
    page.locator(".map-canvas").evaluate((canvas) => {
      const viewport = canvas.getBoundingClientRect();
      const cells = Array.from(canvas.querySelectorAll("[data-cell]"), (cell) =>
        cell.getBoundingClientRect(),
      );
      const left = Math.min(...cells.map((cell) => cell.left));
      const right = Math.max(...cells.map((cell) => cell.right));
      const top = Math.min(...cells.map((cell) => cell.top));
      const bottom = Math.max(...cells.map((cell) => cell.bottom));
      return Math.max(
        Math.abs((left + right - viewport.left - viewport.right) / 2),
        Math.abs((top + bottom - viewport.top - viewport.bottom) / 2),
      );
    });
  await expect.poll(centerOffset).toBeLessThan(1);
  const canvas = (await page.locator(".map-canvas").boundingBox())!;
  await page.mouse.move(
    canvas.x + canvas.width / 2,
    canvas.y + canvas.height / 2,
  );
  await page.mouse.down({ button: "middle" });
  await page.mouse.move(
    canvas.x + canvas.width / 2 + 40,
    canvas.y + canvas.height / 2 + 30,
  );
  await page.mouse.up({ button: "middle" });
  await expect.poll(centerOffset).toBeGreaterThan(20);
  await page.getByRole("button", { name: "화면 맞춤", exact: true }).click();
  await expect.poll(centerOffset).toBeLessThan(1);
  await capture(page, testInfo, "workspace");
  await inspectLayout(page, testInfo, "workspace");
  await inspectAccessibility(page, testInfo, "workspace");

  await page.getByRole("button", { name: "화면 테마 변경" }).click();
  const menu = page.getByRole("menu");
  await expect(menu).toBeVisible();
  await expect(menu).toHaveAttribute(
    "id",
    (await page
      .getByRole("button", { name: "화면 테마 변경" })
      .getAttribute("aria-controls"))!,
  );
  await capture(page, testInfo, "theme-menu");
  await expect.soft(menu).toBeInViewport({ ratio: 1 });
  await inspectLayout(page, testInfo, "theme-menu");
  await inspectAccessibility(page, testInfo, "theme-menu");
});

test("keyboard menu, focus return and persisted theme", async ({
  page,
}, testInfo) => {
  await openWorkspace(page, testInfo);
  const trigger = page.getByRole("button", { name: "화면 테마 변경" });
  // Traverse the document toolbar with the keyboard.
  for (let i = 0; i < 20; i++) {
    await page.keyboard.press("Tab");
    if (await trigger.evaluate((el) => el === document.activeElement)) break;
  }
  await expect(trigger).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("menu")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("menu")).toBeHidden();
  await expect(trigger).toBeFocused();

  const nextTheme =
    testInfo.project.use.colorScheme === "dark" ? "light" : "dark";
  await page.keyboard.press("Enter");
  await page.keyboard.press("Home");
  if (nextTheme === "dark") await page.keyboard.press("ArrowDown");
  await expect(
    page.getByRole("menuitemradio", {
      name: nextTheme === "dark" ? "어둡게" : "밝게",
      exact: true,
    }),
  ).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("menu")).toBeHidden();
  await expect(trigger).toBeFocused();
  await expect(page.locator("html")).toHaveCSS("color-scheme", nextTheme);

  await page.reload();
  await expect(page.locator("html")).toHaveCSS("color-scheme", nextTheme);
  await trigger.click();
  await expect(
    page.getByRole("menuitemradio", {
      name: nextTheme === "dark" ? "어둡게" : "밝게",
      exact: true,
    }),
  ).toHaveAttribute("aria-checked", "true");
});
