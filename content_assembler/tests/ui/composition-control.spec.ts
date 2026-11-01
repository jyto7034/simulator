import { readFile, rm } from "node:fs/promises";
import { resolve } from "node:path";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
} from "./fixtures";

test("Composition: finite control edits and real World source navigation", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_CONTROL_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_CONTROL_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const config = JSON.parse(
    await readFile(resolve(root, "trial.json"), "utf8"),
  );
  const name = `제어-변경-${info.project.name}-${Date.now()}.skill.yaml`;
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
      .getByRole("button", { name: /· approach-control$/ })
      .click();
    const prefix = "approach-control value";
    const duration = page.getByRole("textbox", {
      name: `${prefix}.Effects.0.op.Sustained.definition.Control.duration_ms`,
      exact: true,
    });
    await duration.fill("2001");
    await duration.press("Tab");
    const changed = original.replace("duration_ms: 1501", "duration_ms: 2001");
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
      .getByRole("button", { name: /· approach-control$/ })
      .click();
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
    await duration.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-control-properties");
    await inspectAccessibility(page, info, "composition-control-properties");

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
    expect(run.latest_sustained.time_ms).toBe(25_503);
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
    expect(await selected()).toContain("duration_ms: 2001");
    expect(await selected()).toContain("ApproachTaunt:");
    expect(run.latest_sustained.before.move_speed_units_per_ms).toBe(
      run.latest_sustained.after.move_speed_units_per_ms * 2,
    );
    await expect(observed.getByText(/이동 속도.*타일\/초/)).toBeVisible();
    await observed
      .getByRole("button", { name: "호출 2 보기", exact: true })
      .click();
    expect(await selected()).toContain("definition: apply-control");
    await capture(page, info, "composition-control-world-source");
    await inspectAccessibility(page, info, "composition-control-world-source");
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

    const controlDuration = page.getByRole("textbox", {
      name: `${prefix}.Effects.0.op.Sustained.definition.Control.duration_ms`,
      exact: true,
    });
    await controlDuration.fill("0");
    await controlDuration.press("Tab");
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
    await capture(page, info, "composition-control-duration-error");
    await inspectAccessibility(
      page,
      info,
      "composition-control-duration-error",
    );
    await page.getByRole("button", { name: "실행 취소", exact: true }).click();
    await expect(source).toHaveValue(changed);
    await info.attach("control-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await rm(path, { force: true });
  }
});
