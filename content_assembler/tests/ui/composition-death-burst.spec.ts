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

test("Composition: death burst edits Attack scaling and follows actual equipped kill delivery", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_DEATH_BURST_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_DEATH_BURST_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    percent: number;
    witness: {
      equipment_copy: string;
      attack: number;
      base_damage: number;
      pending: number;
      targets: number;
    };
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `장례-연쇄폭발-실제처치-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· funeral_echo$/ })
        .click();
    await select();
    const cap = page.getByRole("textbox", {
      name: "funeral_echo value.AttackProgram.death_burst.attack_percent",
      exact: true,
    });
    await cap.fill("150");
    await cap.press("Tab");
    const changed = original.replace(
      "attack_percent: 100",
      "attack_percent: 150",
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
    await capture(page, info, "composition-death-burst-properties");
    await inspectLayout(page, info, "composition-death-burst-properties", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-death-burst-properties",
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
    expect(run.latest_equipment_death_burst).toMatchObject(
      expected.find((e) => e.percent === 150)!.witness,
    );
    expect(run.latest_equipment_death_burst.origin).not.toBeNull();
    expect(run.latest_effect).toMatchObject({
      time_ms: 8500,
      hp_delta: -18,
      equipment: {
        persistent_copy_uuid: expected.find((e) => e.percent === 150)!.witness
          .equipment_copy,
      },
      field_path: "equipment_attacks[0].program.death_burst",
    });
    const timing = dialog.getByRole("group", {
      name: "처치 연쇄 폭발 관측",
    });
    await expect(timing).toContainText("폭발");
    await expect(timing).toContainText(
      `예약 공격력 ${run.latest_equipment_death_burst.attack}`,
    );
    await expect(timing).toContainText(
      `기초 피해 ${run.latest_equipment_death_burst.base_damage}`,
    );
    await expect(timing).toContainText(
      `이번 폭발 대상 ${run.latest_equipment_death_burst.targets}명`,
    );
    await expect(timing).toContainText(
      `남은 예약 ${run.latest_equipment_death_burst.pending}개`,
    );
    await expect(timing).toContainText(
      run.latest_equipment_death_burst.equipment_copy,
    );
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-death-burst-world");
    await inspectLayout(page, info, "composition-death-burst-world", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-death-burst-world");
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
    ).toContain("attack_percent: 150");
    await runSource.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-death-burst-source");
    await inspectLayout(page, info, "composition-death-burst-source", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-death-burst-source");
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
    const invalid = changed.replace("attack_percent: 150", "attack_percent: 0");
    await source.fill(invalid);
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(
      /death burst percent requires 1..10000/,
    );
    await diagnostic.click();
    await expect(source).toBeFocused();
    const invalidStart =
      invalid.indexOf("attack_percent: 0") + "attack_percent: ".length;
    expect(
      await source.evaluate((node: HTMLTextAreaElement) => ({
        start: node.selectionStart,
        end: node.selectionEnd,
        text: node.value.slice(node.selectionStart, node.selectionEnd),
      })),
    ).toEqual({ start: invalidStart, end: invalidStart + 1, text: "0" });
    await capture(page, info, "composition-death-burst-diagnostic");
    await inspectLayout(page, info, "composition-death-burst-diagnostic", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-death-burst-diagnostic",
    );
    await info.attach("death-burst-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
