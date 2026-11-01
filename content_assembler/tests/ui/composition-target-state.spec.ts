import { readFile, rm } from "node:fs/promises";
import { resolve } from "node:path";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
} from "./fixtures";

test("Composition: direct target state edits save reopen and run the real World", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_TARGET_STATE_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_TARGET_STATE_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const config = JSON.parse(
    await readFile(resolve(root, "trial.json"), "utf8"),
  );
  const name = `직접-강화-${info.project.name}-${Date.now()}.skill.yaml`;
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
      .getByRole("button", { name: /· resolve$/ })
      .click();
    const prefix =
      "resolve value.Effects.0.op.Sustained.definition.TargetState";
    const percent = page.getByRole("textbox", {
      name: `${prefix}.profile.attack_percent`,
      exact: true,
    });
    await percent.fill("50");
    await percent.press("Tab");
    const changed = original.replace(
      "attack_percent: 25",
      "attack_percent: 50",
    );
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
      .getByRole("button", { name: /· resolve$/ })
      .click();
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
    await percent.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-target-state-properties");
    await inspectAccessibility(
      page,
      info,
      "composition-target-state-properties",
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
    expect(run.identity.source_digest).toBe(run.source_digest);
    await expect(dialog.getByText(/시나리오 실행 완료/)).toBeVisible();
    expect(run.latest_sustained.ended).toBe("Expired");
    expect(run.latest_sustained.before.attack).toBe(18);
    expect(run.latest_sustained.after.attack).toBe(12);
    expect(run.latest_sustained.source_file).toBe(run.identity.artifact_path);
    const observed = dialog.getByRole("group", {
      name: "지속 효과 관측",
      exact: true,
    });
    await expect(observed.getByText(/공격력 18 → 12/)).toBeVisible();
    await expect(observed.getByText(/지속 시간 만료/)).toBeVisible();
    await capture(page, info, "composition-target-state-world-completed");
    await inspectAccessibility(
      page,
      info,
      "composition-target-state-world-completed",
    );
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
    const selectionText = () =>
      runSource.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      );
    expect(await selectionText()).toContain("Append:");
    await observed
      .getByRole("button", { name: "입력 1 보기", exact: true })
      .click();
    expect(await selectionText()).toContain("attack_percent: 50");
    await expect
      .poll(async () => {
        const area = await runSource.boundingBox();
        const modal = await dialog.boundingBox();
        return (
          !!area &&
          !!modal &&
          area.y >= modal.y &&
          area.y + area.height <= modal.y + modal.height
        );
      })
      .toBe(true);
    await capture(page, info, "composition-sustained-run-source");
    await inspectAccessibility(page, info, "composition-sustained-run-source");
    await observed
      .getByRole("button", { name: "호출 2 보기", exact: true })
      .click();
    expect(await selectionText()).toContain("definition: grant-state");
    for (let i = 0; i < 12; i++) {
      await page.keyboard.press("Tab");
      expect(
        await dialog.evaluate((node) => node.contains(document.activeElement)),
      ).toBe(true);
    }
    const repeatResponse = page.waitForResponse(
      (r) => r.url().endsWith("/__content/run-composition"),
      { timeout: 120_000 },
    );
    await dialog
      .getByRole("button", { name: "시나리오 실행", exact: true })
      .click();
    const repeated = await (await repeatResponse).json();
    expect(repeated.status, JSON.stringify(repeated.failure)).toBe("completed");
    expect(repeated.identity.run_id).not.toBe(run.identity.run_id);
    await expect(runSource).toHaveCount(0);
    await info.attach("sustained-repeated-world-result", {
      body: JSON.stringify(repeated, null, 2),
      contentType: "application/json",
    });
    await page.keyboard.press("Escape");
    await source.fill(
      changed.replace("attack_percent: 50", "attack_percent: 60"),
    );
    await page.getByRole("button", { name: "실행 시험", exact: true }).click();
    await observed
      .getByText("실행 원본의 정의·호출 위치", { exact: true })
      .click();
    await expect(observed.getByText(/현재 편집기와 다른 원본/)).toBeVisible();
    await observed
      .getByRole("button", { name: "정의 보기", exact: true })
      .click();
    expect(await selectionText()).toContain("Append:");
    await observed
      .getByRole("button", { name: "입력 1 보기", exact: true })
      .click();
    expect(await selectionText()).toContain("attack_percent: 50");
    await expect(
      dialog.getByRole("button", { name: "시나리오 실행", exact: true }),
    ).toBeDisabled();
    await capture(page, info, "composition-sustained-stale-source");
    await inspectAccessibility(
      page,
      info,
      "composition-sustained-stale-source",
    );
    await page.keyboard.press("Escape");
    await expect(source).toHaveValue(
      changed.replace("attack_percent: 50", "attack_percent: 60"),
    );
    await page.getByRole("button", { name: "실행 취소", exact: true }).click();
    await expect(source).toHaveValue(changed);
    await page.getByRole("button", { name: "실행 시험", exact: true }).click();
    await page.keyboard.press("Escape");
    await expect(
      page.getByRole("button", { name: "실행 시험", exact: true }),
    ).toBeFocused();
    const duration = page.getByRole("textbox", {
      name: `${prefix}.duration_ms`,
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
    await expect(diagnostic).toContainText(/positive duration/);
    await diagnostic.click();
    await expect(source).toBeFocused();
    const selected = await source.evaluate((node: HTMLTextAreaElement) =>
      node.value.slice(node.selectionStart, node.selectionEnd),
    );
    expect(selected).toContain("duration_ms: 0");
    await capture(page, info, "composition-target-state-duration-error");
    await inspectAccessibility(
      page,
      info,
      "composition-target-state-duration-error",
    );
    await page.getByRole("button", { name: "실행 취소", exact: true }).click();
    await expect(source).toHaveValue(changed);
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
    await info.attach("target-state-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await rm(path, { force: true });
  }
});
