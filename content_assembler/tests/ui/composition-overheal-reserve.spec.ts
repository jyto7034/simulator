import { readFile, unlink } from "node:fs/promises";
import { resolve } from "node:path";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
} from "./fixtures";

test("Composition: overheal reserve edits the cap and follows an equipped armor copy", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_OVERHEAL_RESERVE_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_OVERHEAL_RESERVE_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    ratio: number;
    reserve: {
      stored_before: number;
      stored_after: number;
      actual_healed: number;
    };
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `태극-초과회복-저장방출-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· overheal-defense$/ })
        .click();
    await select();
    const cap = page.getByRole("textbox", {
      name: "overheal-defense value.DefenseProgram.OverhealReserve.max_hp_cap_percent",
      exact: true,
    });
    await cap.fill("75");
    await cap.press("Tab");
    const changed = original.replace(
      "max_hp_cap_percent: 50",
      "max_hp_cap_percent: 75",
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
    await capture(page, info, "composition-overheal-reserve-properties");
    await inspectAccessibility(
      page,
      info,
      "composition-overheal-reserve-properties",
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
    expect(run.latest_overheal_reserve).toMatchObject(
      expected.find((e) => e.ratio === 75)!.reserve,
    );
    expect(run.latest_overheal_reserve.origin).not.toBeNull();
    const timing = dialog.getByRole("group", {
      name: "초과 회복 저장 관측",
    });
    await expect(timing).toContainText(
      `저장량 ${run.latest_overheal_reserve.stored_before} → ${run.latest_overheal_reserve.stored_after}`,
    );
    await expect(timing).toContainText(
      `실제 방출 회복 ${run.latest_overheal_reserve.actual_healed}`,
    );
    await expect(timing).toContainText("overheal_reserve_fixture_armor");
    await capture(page, info, "composition-overheal-reserve-world");
    await inspectAccessibility(
      page,
      info,
      "composition-overheal-reserve-world",
    );
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
    ).toContain("max_hp_cap_percent: 75");
    await capture(page, info, "composition-overheal-reserve-source");
    await inspectAccessibility(
      page,
      info,
      "composition-overheal-reserve-source",
    );
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
      changed.replace("max_hp_cap_percent: 75", "max_hp_cap_percent: 101"),
    );
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(/reserve cap must be from 1 to 100/);
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toBe("101");
    await capture(page, info, "composition-overheal-reserve-diagnostic");
    await inspectAccessibility(
      page,
      info,
      "composition-overheal-reserve-diagnostic",
    );
    await info.attach("overheal-reserve-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
