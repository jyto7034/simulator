import { readFile, unlink } from "node:fs/promises";
import { resolve } from "node:path";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
} from "./fixtures";

test("Composition: shared cohort edits hostile HP and follows an undeployed accessory copy", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_SHARED_COHORT_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_SHARED_COHORT_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    hp: number;
    cohort: {
      initial_hp: number;
      provider_count: number;
      fragment_recipient: string | null;
    };
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `공유사냥감-실제출현-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· shared_hunt$/ })
        .click();
    await select();
    const cap = page.getByRole("textbox", {
      name: "shared_hunt value.AttackProgram.cohort.spawn.hp",
      exact: true,
    });
    await cap.fill("450");
    await cap.press("Tab");
    const changed = original.replace("hp: 300", "hp: 450");
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
    await cap.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-shared-cohort-properties");
    await inspectAccessibility(
      page,
      info,
      "composition-shared-cohort-properties",
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
    expect(run.latest_shared_cohort).toMatchObject(
      expected.find((e) => e.hp === 450)!.cohort,
    );
    expect(run.latest_shared_cohort.origin).not.toBeNull();
    const timing = dialog.getByRole("group", {
      name: "공유 사냥감 관측",
    });
    await expect(timing).toContainText("출현 시 HP 450");
    await expect(timing).toContainText("공유 주기 1개");
    await expect(timing).toContainText("파편 보상 수신 사본: 없음");
    await expect(timing).toContainText(run.latest_shared_cohort.equipment_copy);
    await expect(timing).toContainText(run.latest_shared_cohort.unit);
    await capture(page, info, "composition-shared-cohort-world");
    await inspectAccessibility(page, info, "composition-shared-cohort-world");
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
    ).toContain("hp: 450");
    await capture(page, info, "composition-shared-cohort-source");
    await inspectAccessibility(page, info, "composition-shared-cohort-source");
    for (let i = 0; i < 8; i++) {
      await page.keyboard.press("Tab");
      expect(
        await dialog.evaluate((node) => node.contains(document.activeElement)),
      ).toBe(true);
    }
    await page.keyboard.press("Escape");
    await expect(
      page.getByRole("button", { name: "실행 시험", exact: true }),
    ).toBeFocused();
    await source.fill(changed.replace("interval_ms: 5000", "interval_ms: 0"));
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(/hostile value must be positive/);
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toBe("0");
    await capture(page, info, "composition-shared-cohort-diagnostic");
    await inspectAccessibility(
      page,
      info,
      "composition-shared-cohort-diagnostic",
    );
    await info.attach("shared-cohort-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
