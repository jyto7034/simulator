import { readFile, unlink } from "node:fs/promises";
import { resolve } from "node:path";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
} from "./fixtures";

test("Composition: source direct damage edits stacks and follows the real weapon status", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_SOURCE_DIRECT_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_SOURCE_DIRECT_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    percent: number;
    hp: number;
    latest: { time_ms: number; change: { expires_at_ms: number } };
    origin: { definition: string; calls: string[]; inputs: string[] };
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `간파-출처별중첩-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· insight$/ })
        .click();
    await select();
    const cap = page.getByRole("textbox", {
      name: "insight value.AttackProgram.on_hit_status.profile.kind.SourceDirectDamageTaken.per_stack_percent",
      exact: true,
    });
    await cap.fill("6");
    await cap.press("Tab");
    const changed = original.replace(
      "per_stack_percent: 3",
      "per_stack_percent: 6",
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
    await select();
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
    await cap.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-source-direct-properties");
    await inspectAccessibility(
      page,
      info,
      "composition-source-direct-properties",
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
    const wanted = expected.find((e) => e.percent === 6)!;
    expect(run.latest_sustained.time_ms).toBe(wanted.latest.time_ms);
    expect(run.latest_sustained.expires_at_ms).toBe(
      wanted.latest.change.expires_at_ms,
    );
    expect(run.latest_sustained.after.current_health).toBe(wanted.hp);
    expect(run.latest_sustained.origin).toEqual(wanted.origin);
    expect(run.latest_sustained.field_path).toBe(
      "equipment_attacks[0].program.on_hit_status",
    );
    expect(run.latest_sustained.source_file).toBe(run.identity.artifact_path);
    const timing = dialog.getByRole("group", { name: "지속 효과 관측" });
    await expect(timing).toContainText("equipment:standard_training_sword");
    await expect(timing).toContainText("만료 시각");
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-source-direct-world");
    await inspectAccessibility(page, info, "composition-source-direct-world");
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
    ).toContain("per_stack_percent: 6");
    await capture(page, info, "composition-source-direct-source");
    await inspectAccessibility(page, info, "composition-source-direct-source");
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
    await source.fill(
      changed.replace("per_stack_percent: 6", "per_stack_percent: 0"),
    );
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(
      /Source direct damage requires positive stacks within a cap/,
    );
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toContain("per_stack_percent: 0");
    await capture(page, info, "composition-source-direct-diagnostic");
    await inspectAccessibility(
      page,
      info,
      "composition-source-direct-diagnostic",
    );
    await info.attach("source-direct-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
