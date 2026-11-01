import { readFile, unlink } from "node:fs/promises";
import { resolve } from "node:path";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
  inspectLayout,
} from "./fixtures";

test("Composition: lifesteal edits percent and traces the actual weapon heal", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_LIFESTEAL_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_LIFESTEAL_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    percent: number;
    heals: {
      time_ms: number;
      delta: number;
      equipment: { persistent_copy_uuid: string };
      origin: { definition: string; calls: string[]; inputs: string[] };
    }[];
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `미미크리-흡혈-${info.project.name}-${Date.now()}.skill.yaml`;
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
    const select = () =>
      page
        .getByRole("group", { name: "정의 선택" })
        .getByRole("button", { name: /· mimicry-lifesteal$/ })
        .click();
    await expect(
      page.getByRole("group", { name: "정의 선택" }).getByRole("button", {
        name: /장비 공격 · placeholder_nothing_there_weapon$/,
      }),
    ).toBeVisible();
    await select();
    const reward = page.getByRole("textbox", {
      name: "mimicry-lifesteal value.AttackProgram.basic_lifesteal.percent",
      exact: true,
    });
    await reward.fill("50");
    await reward.press("Tab");
    const changed = original.replace("percent: 25", "percent: 50");
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
    await select();
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
    await reward.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-lifesteal-properties");
    await inspectLayout(page, info, "composition-lifesteal-properties", {
      landmarks: false,
    });
    await inspectAccessibility(page, info, "composition-lifesteal-properties");
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
    const wanted = expected.find((e) => e.percent === 50)!.heals.at(-1)!;
    expect(run.latest_effect.time_ms).toBe(wanted.time_ms);
    expect(run.latest_effect.hp_delta).toBe(wanted.delta);
    expect(run.latest_effect.origin).toEqual(wanted.origin);
    expect(run.latest_effect.skill_id).toBeNull();
    expect(run.latest_effect.equipment).toEqual(wanted.equipment);
    expect(run.latest_effect.source_file).toBe(run.identity.artifact_path);
    const timing = dialog.getByRole("group", { name: "HP 효과 관측" });
    await expect(timing).toContainText("placeholder_nothing_there_weapon");
    await expect(timing).toContainText(`HP 변화 ${wanted.delta}`);
    await expect(timing).toContainText(wanted.equipment.persistent_copy_uuid);
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-lifesteal-world");
    await inspectLayout(page, info, "composition-lifesteal-world", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-lifesteal-world");
    await timing
      .getByText("실행 원본의 정의·호출 위치", { exact: true })
      .click();
    await timing
      .getByRole("button", { name: "정의 보기", exact: true })
      .click();
    const runSource = timing.getByRole("textbox", {
      name: "실행 당시 스킬 조합 원본",
    });
    await expect(runSource).toHaveValue(changed);
    await expect(runSource).toHaveAttribute("readonly", "");
    expect(
      await runSource.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toContain("percent: 50");
    await capture(page, info, "composition-lifesteal-source");
    await inspectLayout(page, info, "composition-lifesteal-source", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-lifesteal-source");
    for (let i = 0; i < 8; i++) {
      await page.keyboard.press("Tab");
      expect(
        await dialog.evaluate((node) => node.contains(document.activeElement)),
      ).toBe(true);
    }
    for (let i = 0; i < 8; i++) {
      await page.keyboard.press("Shift+Tab");
      expect(
        await dialog.evaluate((node) => node.contains(document.activeElement)),
      ).toBe(true);
    }
    await page.keyboard.press("Escape");
    await expect(
      page.getByRole("button", { name: "실행 시험", exact: true }),
    ).toBeFocused();
    await source.fill(changed.replace("percent: 50", "percent: 101"));
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(/lifesteal percent requires 1..100/);
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toBe("101");
    await capture(page, info, "composition-lifesteal-diagnostic");
    await inspectLayout(page, info, "composition-lifesteal-diagnostic", {
      landmarks: false,
    });
    await inspectAccessibility(page, info, "composition-lifesteal-diagnostic");
    await info.attach("lifesteal-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
