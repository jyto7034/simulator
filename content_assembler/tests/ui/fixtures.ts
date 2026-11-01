import AxeBuilder from "@axe-core/playwright";
import { installSocketFreeTransport } from "./socket-free-transport";
import {
  test as base,
  expect,
  type Page,
  type TestInfo,
} from "@playwright/test";

type BrowserIssue = {
  kind: "console" | "exception" | "request" | "http";
  message: string;
};

// Attach listeners before navigation; diagnostics are kept even when assertions fail.
export const test = base.extend<{ browserDiagnostics: void }>({
  browserDiagnostics: [
    async ({ page }, runTest, testInfo) => {
      await installSocketFreeTransport(page);
      const errors: BrowserIssue[] = [];
      const warnings: string[] = [];
      page.on("console", (message) => {
        if (message.type() === "error") {
          errors.push({ kind: "console", message: message.text() });
        } else if (message.type() === "warning") {
          warnings.push(message.text());
        }
      });
      page.on("pageerror", (error) => {
        errors.push({
          kind: "exception",
          message: error.stack ?? error.message,
        });
      });
      page.on("requestfailed", (request) => {
        errors.push({
          kind: "request",
          message: `${request.method()} ${request.url()}: ${request.failure()?.errorText}`,
        });
      });
      page.on("response", (response) => {
        if (response.status() >= 400) {
          errors.push({
            kind: "http",
            message: `${response.status()} ${response.url()}`,
          });
        }
      });

      try {
        await runTest();
      } finally {
        await testInfo.attach("browser-diagnostics", {
          body: JSON.stringify({ errors, warnings }, null, 2),
          contentType: "application/json",
        });
        expect(
          errors,
          "Browser errors; see browser-diagnostics attachment",
        ).toEqual([]);
      }
    },
    { auto: true },
  ],
});

export { expect };

export async function openWorkspace(page: Page, testInfo: TestInfo) {
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "전투 맵 조립기", exact: true }),
  ).toBeVisible();
  await page.evaluate(() => document.fonts.ready.then(() => undefined));

  // Each test has fresh storage; the initial theme follows the emulated OS.
  await expect(page.locator("html")).toHaveCSS(
    "color-scheme",
    testInfo.project.use.colorScheme === "dark" ? "dark" : "light",
  );
}

export async function capture(page: Page, testInfo: TestInfo, name: string) {
  const path = testInfo.outputPath(`${name}.png`);
  await page.screenshot({
    path,
    fullPage: false,
    animations: "disabled",
    caret: "hide",
    scale: "css",
  });
  await testInfo.attach(name, { path, contentType: "image/png" });
}

export async function inspectLayout(
  page: Page,
  testInfo: TestInfo,
  name: string,
  options: { modal?: boolean; landmarks?: boolean } = {},
) {
  const layout = await page.evaluate(() => {
    const root = document.documentElement;
    const body = document.body;
    const width = root.clientWidth;
    const height = root.clientHeight;
    const scrollWidth = Math.max(root.scrollWidth, body.scrollWidth);
    const scrollHeight = Math.max(root.scrollHeight, body.scrollHeight);
    return {
      viewport: { width, height },
      document: { width: scrollWidth, height: scrollHeight },
      // This desktop shell scrolls inside panels, not at the document level.
      overflowX: scrollWidth > width + 1,
      overflowY: scrollHeight > height + 1,
    };
  });
  await testInfo.attach(`${name}-layout`, {
    body: JSON.stringify(layout, null, 2),
    contentType: "application/json",
  });
  expect
    .soft(layout.overflowX, "Unexpected horizontal document overflow")
    .toBe(false);
  expect
    .soft(layout.overflowY, "Unexpected vertical document overflow")
    .toBe(false);
  if (options.modal) {
    await expect.soft(page.getByRole("dialog")).toBeInViewport({ ratio: 1 });
    return;
  }
  // Composition uses panel groups instead of the map shell landmarks.
  if (options.landmarks === false) return;
  for (const role of ["banner", "main", "contentinfo"] as const) {
    await expect.soft(page.getByRole(role)).toBeInViewport({ ratio: 1 });
  }
}

export async function inspectAccessibility(
  page: Page,
  testInfo: TestInfo,
  name: string,
) {
  const { violations, incomplete } = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
    .analyze();
  await testInfo.attach(`${name}-accessibility`, {
    body: JSON.stringify({ violations, incomplete }, null, 2),
    contentType: "application/json",
  });
  expect
    .soft(violations, "Accessibility violations; see attachment")
    .toEqual([]);
}
