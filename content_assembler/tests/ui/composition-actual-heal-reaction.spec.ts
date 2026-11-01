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

test("Composition: actual heal reactions edit ratio and trace actual armor and accessory", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_ACTUAL_HEAL_REACTION_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_ACTUAL_HEAL_REACTION_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    ratio: number;
    reaction: NonNullable<
      import("../../src/composition/execution").RunResponse["latest_actual_heal_reaction"]
    >;
    origins: unknown[];
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `여왕벌-치유반응-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· heal-shield$/ })
        .click();
    await expect(
      page.getByRole("group", { name: "정의 선택" }).getByRole("button", {
        name: /장비 지원 · placeholder_queen_bee_armor$/,
      }),
    ).toBeVisible();
    await select();
    const reward = page.getByRole("textbox", {
      name: "heal-shield value.SupportProgram.actual_heal_reaction.self_shield.actual_heal_percent",
      exact: true,
    });
    await reward.fill("50");
    await reward.press("Tab");
    const changed = original.replace(
      "actual_heal_percent: 25",
      "actual_heal_percent: 50",
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
    await reward.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-actual-heal-reaction-properties");
    await inspectLayout(
      page,
      info,
      "composition-actual-heal-reaction-properties",
      {
        landmarks: false,
      },
    );
    await inspectAccessibility(
      page,
      info,
      "composition-actual-heal-reaction-properties",
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
    const wanted = expected.find((e) => e.ratio === 50)!;
    const observed = run.latest_actual_heal_reaction;
    for (const key of ["time_ms", "actor", "target", "actual_healed"] as const)
      expect(observed[key]).toBe(wanted.reaction[key]);
    expect(observed.origins).toEqual(wanted.origins);
    expect(observed.contributors).toHaveLength(2);
    for (const [index, item] of wanted.reaction.contributors.entries()) {
      expect(observed.contributors[index].equipment).toEqual(item.equipment);
      expect(observed.contributors[index].self_shield).toEqual(
        item.self_shield,
      );
      expect(observed.contributors[index].target_haste).toEqual(
        item.target_haste,
      );
      expect(observed.contributors[index].source_file).toBe(
        run.identity.artifact_path,
      );
    }
    const timing = dialog.getByRole("group", { name: "실제 치유 반응 관측" });
    await expect(timing).toContainText(
      `이 대상의 실제 회복 ${wanted.reaction.actual_healed} HP`,
    );
    await expect(timing).toContainText("실제 회복의 50%");
    await expect(timing).toContainText("공격 속도 +30");
    await expect(dialog).not.toContainText(
      "최근 관측에 이 조합의 HP 변화가 없습니다.",
    );
    for (const item of wanted.reaction.contributors)
      await expect(timing).toContainText(item.equipment.persistent_copy_uuid!);
    const shieldIndex = wanted.reaction.contributors.findIndex(
      (c) => c.self_shield !== null,
    );
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-actual-heal-reaction-world");
    await inspectLayout(page, info, "composition-actual-heal-reaction-world", {
      modal: true,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-actual-heal-reaction-world",
    );
    await timing
      .getByText("실행 원본의 정의·호출 위치", { exact: true })
      .nth(shieldIndex)
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
    ).toContain("actual_heal_percent: 50");
    await capture(page, info, "composition-actual-heal-reaction-source");
    await inspectLayout(page, info, "composition-actual-heal-reaction-source", {
      modal: true,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-actual-heal-reaction-source",
    );
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
    await source.fill(
      changed.replace("actual_heal_percent: 50", "actual_heal_percent: 10001"),
    );
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(
      /heal shield ratio must be from 1 to 10000 percent/,
    );
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toContain("10001");
    await capture(page, info, "composition-actual-heal-reaction-diagnostic");
    await inspectLayout(
      page,
      info,
      "composition-actual-heal-reaction-diagnostic",
      {
        landmarks: false,
      },
    );
    await inspectAccessibility(
      page,
      info,
      "composition-actual-heal-reaction-diagnostic",
    );
    await info.attach("actual-heal-reaction-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
