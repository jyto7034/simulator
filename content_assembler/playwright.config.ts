import { defineConfig } from "@playwright/test";

const viewports = [
  { name: "minimum", width: 960, height: 640 },
  { name: "default", width: 1280, height: 800 },
  { name: "large", width: 1920, height: 1080 },
];

export default defineConfig({
  testDir: "./tests/ui",
  outputDir: "./artifacts/ui/results",
  timeout: 45_000,
  expect: {
    timeout: 7_000,
    toHaveScreenshot: { animations: "disabled", caret: "hide", scale: "css" },
  },
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: 0,
  workers: 2,
  // Missing baselines must fail comparison, never silently approve a new design.
  updateSnapshots: "none",
  snapshotPathTemplate:
    "{testDir}/baselines/{platform}/{projectName}/{testFilePath}/{arg}{ext}",
  reporter: [
    ["list"],
    ["html", { outputFolder: "artifacts/ui/report", open: "never" }],
    ["json", { outputFile: "artifacts/ui/report.json" }],
  ],
  use: {
    browserName: "chromium",
    baseURL: "http://127.0.0.1:1422",
    headless: true,
    locale: "ko-KR",
    timezoneId: "Asia/Seoul",
    deviceScaleFactor: 1,
    reducedMotion: "reduce",
    serviceWorkers: "block",
    actionTimeout: 7_000,
    navigationTimeout: 15_000,
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
  },
  projects: viewports.flatMap(({ name, width, height }) =>
    (["light", "dark"] as const).map((colorScheme) => ({
      name: `chromium-${name}-${colorScheme}`,
      use: { viewport: { width, height }, colorScheme },
    })),
  ),
  webServer:
    process.env.CONTENT_ASSEMBLER_UI_TRANSPORT === "pipe"
      ? undefined
      : {
          command: "npm run dev -- --host 127.0.0.1 --port 1422 --strictPort",
          // Vite's strictPort fails if another process owns the port. Wait for this
          // process's ready output: unavailable loopback HTTP probes can stall in WSL.
          wait: { stdout: /Local:.*http:\/\/127\.0\.0\.1:1422\// },
          env: { TAURI_DEV_HOST: "" },
          // Never inspect a stale server or an unrelated app already using this port.
          reuseExistingServer: false,
          timeout: 30_000,
        },
});
