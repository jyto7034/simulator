import type { Page } from "@playwright/test";
import { readFile } from "node:fs/promises";
import { resolve, relative, extname } from "node:path";
import {
  runCompiler,
  compositionCompilerLimits,
  compositionRunLimits,
} from "../../server/compiler";
import { contentOperation } from "../../server/content-files";

/** Explicit test-only transport. Real built UI, compiler process and disk store;
 * no localhost HTTP or Tauri IPC claims are made by this fallback. */
export async function installSocketFreeTransport(page: Page) {
  if (process.env.CONTENT_ASSEMBLER_UI_TRANSPORT !== "pipe") return;
  const root = resolve("dist");
  await page.route("http://127.0.0.1:1422/**", async (route) => {
    const request = route.request();
    const pathname = new URL(request.url()).pathname;
    try {
      if (pathname.startsWith("/__content/")) {
        const input = request.postDataJSON();
        let result: unknown;
        if (pathname === "/__content/files")
          result = await contentOperation(resolve("content"), input);
        else {
          const executable =
            pathname === "/__content/compile-composition"
              ? process.env.CONTENT_ASSEMBLER_COMPOSITION_COMPILER
              : pathname === "/__content/run-composition"
                ? process.env.CONTENT_ASSEMBLER_COMPOSITION_RUNNER
                : pathname === "/__content/compile"
                  ? process.env.CONTENT_ASSEMBLER_COMPILER
                  : undefined;
          if (!executable)
            throw new Error(
              "Core compiler is not configured for this test transport",
            );
          result = await runCompiler(
            executable,
            JSON.stringify(input),
            pathname === "/__content/compile-composition"
              ? compositionCompilerLimits
              : pathname === "/__content/run-composition"
                ? compositionRunLimits
                : undefined,
          );
        }
        await route.fulfill({
          contentType: "application/json",
          body: JSON.stringify(result),
        });
        return;
      }
      const path = resolve(
        root,
        pathname === "/" ? "index.html" : `.${decodeURIComponent(pathname)}`,
      );
      if (relative(root, path).startsWith(".."))
        throw new Error("Asset path escapes build directory");
      const mime: Record<string, string> = {
        ".html": "text/html",
        ".js": "application/javascript",
        ".css": "text/css",
        ".svg": "image/svg+xml",
        ".png": "image/png",
        ".txt": "text/plain",
        ".woff2": "font/woff2",
      };
      await route.fulfill({
        contentType: mime[extname(path)] ?? "application/octet-stream",
        body: await readFile(path),
      });
    } catch (error) {
      if (!pathname.startsWith("/__content/")) throw error;
      await route.fulfill({
        contentType: "application/json",
        body: JSON.stringify({
          error: error instanceof Error ? error.message : String(error),
        }),
      });
    }
  });
}
