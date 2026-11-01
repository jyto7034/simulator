import { readFile, unlink } from "node:fs/promises";
import { resolve } from "node:path";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
} from "./fixtures";

test("Composition: cohort bounty edits thresholds and follows actual direct kill currency", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_COHORT_BOUNTY_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_COHORT_BOUNTY_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    threshold: number;
    bounty: {
      pending: number;
      confirmed: number;
      next_threshold: number;
      tier_attack: number;
      actual_attack: number;
      equipment_copy: string;
    };
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `수금-임계치-실제처치-${info.project.name}-${Date.now()}.skill.yaml`;
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
      name: "shared_hunt value.AttackProgram.cohort.bounty.tiers.0.threshold",
      exact: true,
    });
    await cap.fill("3");
    await cap.press("Tab");
    const changed = original.replace("threshold: 2", "threshold: 3");
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
    await capture(page, info, "composition-cohort-bounty-properties");
    await inspectAccessibility(
      page,
      info,
      "composition-cohort-bounty-properties",
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
    expect(run.latest_bounty).toMatchObject(
      expected.find((e) => e.threshold === 3)!.bounty,
    );
    expect(run.latest_bounty.origin).not.toBeNull();
    const timing = dialog.getByRole("group", {
      name: "수금과 공격력 성장 관측",
    });
    await expect(timing).toContainText("직접 처치 적립");
    await expect(timing).toContainText(
      `이번 전투 미확정 ${run.latest_bounty.pending}`,
    );
    await expect(timing).toContainText("원정 확정 0");
    await expect(timing).toContainText("다음 임계치 3");
    await expect(timing).toContainText(run.latest_bounty.equipment_copy);
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-cohort-bounty-world");
    await inspectAccessibility(page, info, "composition-cohort-bounty-world");
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
    ).toContain("threshold: 3");
    await runSource.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-cohort-bounty-source");
    await inspectAccessibility(page, info, "composition-cohort-bounty-source");
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
    await source.fill(changed.replace("threshold: 3", "threshold: 0"));
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(
      /bounty thresholds and total Attack must increase/,
    );
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toContain("threshold: 0");
    await capture(page, info, "composition-cohort-bounty-diagnostic");
    await inspectAccessibility(
      page,
      info,
      "composition-cohort-bounty-diagnostic",
    );
    await info.attach("cohort-bounty-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
