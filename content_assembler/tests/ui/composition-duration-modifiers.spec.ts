import { readFile, rm } from "node:fs/promises";
import { resolve } from "node:path";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
} from "./fixtures";

test("Composition: duration modifiers edits and real World source navigation", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_DURATION_MODIFIERS_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_DURATION_MODIFIERS_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const config = JSON.parse(
    await readFile(resolve(root, "trial.json"), "utf8"),
  );
  const name = `지속-변경-${info.project.name}-${Date.now()}.skill.yaml`;
  const path = resolve("content", name);
  try {
    await openWorkspace(page, info);
    await page
      .getByRole("navigation", { name: "작업 공간" })
      .getByRole("button", { name: "조합", exact: true })
      .click();
    const source = page.getByRole("textbox", {
      name: "스킬 조합 원본",
      exact: true,
    });
    await source.fill(original);
    await page
      .getByRole("group", { name: "정의 선택" })
      .getByRole("button", { name: /· duration-increase$/ })
      .click();
    const prefix = "duration-increase value";
    const percent = page.getByRole("textbox", {
      name: `${prefix}.Count`,
      exact: true,
    });
    await percent.fill("50");
    await percent.press("Tab");
    const changed = original.replace("Count: 25", "Count: 50");
    await expect(source).toHaveValue(changed);
    await page.getByRole("button", { name: "실행 취소", exact: true }).click();
    await expect(source).toHaveValue(original);
    await page.getByRole("button", { name: "다시 실행", exact: true }).click();
    await expect(source).toHaveValue(changed);
    await page
      .getByRole("textbox", { name: "조합 파일 이름", exact: true })
      .fill(name);
    await page.getByRole("button", { name: /^저장/ }).click();
    await expect(
      page.getByRole("status").filter({ hasText: "저장했습니다." }),
    ).toBeVisible();
    expect(await readFile(path, "utf8")).toBe(changed);
    await page.getByRole("button", { name: "열기", exact: true }).click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name, exact: true })
      .click();
    await expect(source).toHaveValue(changed);
    await page
      .getByRole("group", { name: "정의 선택" })
      .getByRole("button", { name: /· duration-increase$/ })
      .click();
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
    await percent.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-duration-modifiers-properties");
    await inspectAccessibility(
      page,
      info,
      "composition-duration-modifiers-properties",
    );

    await page.getByRole("button", { name: "실행 시험", exact: true }).click();
    const dialog = page.getByRole("dialog", { name: "조합 실행 시험" });
    await dialog
      .getByLabel("실행 설정 파일", { exact: true })
      .setInputFiles(resolve(root, "trial.json"));
    const response = page.waitForResponse(
      (r) => r.url().endsWith("/__content/run-composition"),
      { timeout: 120_000 },
    );
    await dialog
      .getByRole("button", { name: "시나리오 실행", exact: true })
      .click();
    const run = await (await response).json();
    expect(run.status, JSON.stringify(run.failure)).toBe("completed");
    expect(run.accepted_steps).toBe(
      config.scenario.preparation_steps.length + config.scenario.steps.length,
    );
    expect(run.latest_sustained.ended).toBe("Expired");
    expect(run.latest_sustained.time_ms).toBe(25_253);
    expect(run.latest_sustained.before.attack).toBe(
      run.latest_sustained.after.attack,
    );
    expect(run.latest_sustained.source_file).toBe(run.identity.artifact_path);
    const observed = dialog.getByRole("group", {
      name: "지속 효과 관측",
      exact: true,
    });
    await expect(observed.getByText(/지속 시간 만료/)).toBeVisible();
    await observed
      .getByText("실행 원본의 정의·호출 위치", { exact: true })
      .click();
    await observed
      .getByRole("button", { name: "정의 보기", exact: true })
      .click();
    const runSource = observed.getByRole("textbox", {
      name: "실행 당시 스킬 조합 원본",
      exact: true,
    });
    await expect(runSource).toBeFocused();
    await expect(runSource).not.toBeEditable();
    await expect(runSource).toHaveValue(changed);
    const selected = () =>
      runSource.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      );
    expect(await selected()).toContain("Append:");
    await observed
      .getByRole("button", { name: "입력 1 보기", exact: true })
      .click();
    expect(await selected()).toContain("duration_ms: 1501");
    const inputs = observed.getByRole("button", { name: /^입력 \d+ 보기$/ });
    let foundDuration = false;
    for (let i = 0; i < (await inputs.count()); i++) {
      await inputs.nth(i).click();
      if ((await selected()).includes("Count: 50")) {
        foundDuration = true;
        break;
      }
    }
    expect(foundDuration).toBe(true);
    await observed
      .getByRole("button", { name: "호출 2 보기", exact: true })
      .click();
    expect(await selected()).toContain("definition: maintain-support");
    await capture(page, info, "composition-duration-modifiers-world-source");
    await inspectAccessibility(
      page,
      info,
      "composition-duration-modifiers-world-source",
    );
    for (let i = 0; i < 8; i++) {
      await page.keyboard.press("Tab");
      expect(
        await dialog.evaluate((node) => node.contains(document.activeElement)),
      ).toBe(true);
    }
    await page.keyboard.press("Escape");
    await expect(dialog).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: "실행 시험", exact: true }),
    ).toBeFocused();

    const duration = page.getByRole("textbox", {
      name: `${prefix}.Count`,
      exact: true,
    });
    await duration.fill("0");
    await duration.press("Tab");
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(/positive/);
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toBe("0");
    await capture(page, info, "composition-duration-modifiers-duration-error");
    await inspectAccessibility(
      page,
      info,
      "composition-duration-modifiers-duration-error",
    );
    await page.getByRole("button", { name: "실행 취소", exact: true }).click();
    await expect(source).toHaveValue(changed);
    await info.attach("duration-modifiers-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await rm(path, { force: true });
  }
});
