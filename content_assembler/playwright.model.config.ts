import { defineConfig } from "@playwright/test";
// Pure model tests use no browser and start no development server.
export default defineConfig({
  testDir: "./tests/model",
  outputDir: "./artifacts/model",
  reporter: "list",
  workers: 1,
});
