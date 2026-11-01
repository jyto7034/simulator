import { readFile, rm } from "node:fs/promises";
import { resolve } from "node:path";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
} from "./fixtures";

test("Composition: area effect source edits and real World execution", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_AREA_EFFECT_SOURCE_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_AREA_EFFECT_SOURCE_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const config = JSON.parse(
    await readFile(resolve(root, "trial.json"), "utf8"),
  );
  const name = `장판생성약화-조합-${info.project.name}-${Date.now()}.skill.yaml`;
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
      .getByRole("button", { name: /· vulnerability-percent$/ })
      .click();
    const prefix = "vulnerability-percent value";
    const percent = page.getByRole("textbox", {
      name: `${prefix}.Count`,
      exact: true,
    });
    await percent.fill("30");
    await percent.press("Tab");
    const changed = original.replace("Count: 15", "Count: 30");
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
      .getByRole("button", { name: /· vulnerability-percent$/ })
      .click();
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
    await percent.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-area-effect-source-properties");
    await inspectAccessibility(
      page,
      info,
      "composition-area-effect-source-properties",
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
    expect(run.latest_area.change).toEqual({
      kind: "ended",
      reason: "Expired",
    });
    expect(run.latest_area.started_at_ms).toBe(23001);
    expect(run.latest_area.expires_at_ms).toBe(25002);
    expect(run.latest_area.time_ms).toBe(25002);
    expect(run.latest_area.before.contributions).toBeGreaterThanOrEqual(3);
    expect(run.latest_area.after).toBeNull();
    expect(run.latest_area.source_file).toBe(run.identity.artifact_path);
    expect(run.latest_area.field_path).toBe("skills[0].steps[0].delivery");
    expect(run.latest_area.origin.inputs).toContain("values[0].value");
    expect(run.latest_area.origin.inputs).toContain("values[5].value");
    expect(run.latest_area.origin.inputs).toContain("values[6].value");
    expect(run.latest_area.origin.inputs).toContain("skills[0].modifiers[0]");
    expect(run.latest_sustained.ended).toBe("Expired");
    expect(run.latest_sustained.time_ms).toBe(29001);
    expect(run.latest_sustained.expires_at_ms).toBe(29001);
    expect(run.latest_sustained.area_source.area_id).toBe(
      run.latest_area.area_id,
    );
    expect(run.latest_sustained.area_source.status_index).toBe(1);
    expect(run.latest_sustained.area_source.modifier_source.definition).toBe(
      "broken-balance",
    );
    expect(run.latest_sustained.source_file).toBe(run.identity.artifact_path);
    expect(run.latest_sustained.field_path).toBe("skills[0].steps[0].delivery");
    expect(run.latest_sustained.origin.inputs).toContain("values[5].value");
    expect(run.latest_sustained.origin.inputs).toContain("values[6].value");
    const area = dialog.getByRole("group", {
      name: "지속 효과 관측",
      exact: true,
    });
    await expect(area).toContainText("지속 효과 종료 · 29001ms");
    await expect(area).toContainText("만료 시각 · 29001ms");
    await expect(area).toContainText("생성한 장판");
    await expect(area).toContainText("유물 · broken-balance");
    await expect(area).toContainText("종료 사유 · 지속 시간 만료");
    await area.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-area-effect-source-world");
    await inspectAccessibility(
      page,
      info,
      "composition-area-effect-source-world",
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

    const percentInput = page.getByRole("textbox", {
      name: `${prefix}.Count`,
      exact: true,
    });
    await percentInput.fill("45");
    await percentInput.press("Tab");
    await page.getByRole("button", { name: "실행 시험", exact: true }).click();
    await area.getByText("실행 원본의 정의·호출 위치", { exact: true }).click();
    await expect(area).toContainText("현재 편집기와 다른 원본입니다.");
    await area.getByRole("button", { name: "정의 보기", exact: true }).click();
    const frozen = area.getByRole("textbox", {
      name: "실행 당시 스킬 조합 원본",
      exact: true,
    });
    await expect(frozen).toHaveValue(changed);
    expect(
      await frozen.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toContain("Parameter");
    await area
      .getByRole("button", { name: "호출 1 보기", exact: true })
      .click();
    expect(
      await frozen.evaluate(
        (node: HTMLTextAreaElement) => node.selectionEnd > node.selectionStart,
      ),
    ).toBe(true);
    await area
      .getByRole("button", {
        name: `입력 ${run.latest_area.origin.inputs.indexOf("values[5].value") + 1} 보기`,
        exact: true,
      })
      .click();
    expect(
      await frozen.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toContain("Count: 30");
    await area
      .getByRole("button", {
        name: `입력 ${run.latest_area.origin.inputs.indexOf("values[6].value") + 1} 보기`,
        exact: true,
      })
      .click();
    expect(
      await frozen.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toContain("Milliseconds: 5000");
    await capture(page, info, "composition-area-effect-source-frozen-source");
    await inspectAccessibility(
      page,
      info,
      "composition-area-effect-source-frozen-source",
    );
    await page.keyboard.press("Escape");
    await expect(dialog).toHaveCount(0);
    await page.getByRole("button", { name: "실행 취소", exact: true }).click();
    await expect(source).toHaveValue(changed);
    await percentInput.fill("0");
    await percentInput.press("Tab");
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
    await capture(page, info, "composition-area-effect-source-value-error");
    await inspectAccessibility(
      page,
      info,
      "composition-area-effect-source-value-error",
    );
    await page.getByRole("button", { name: "실행 취소", exact: true }).click();
    await expect(source).toHaveValue(changed);
    await info.attach("control-vulnerability-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await rm(path, { force: true });
  }
});
