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

test("Composition: last will edits armor blast and follows lethal protection through incapacitation", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_LAST_WILL_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_LAST_WILL_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    percent: number;
    witness: {
      time_ms: number;
      equipment: { persistent_copy_uuid: string };
      attack: number;
      base_damage: number;
      hp_before: number;
      hp_protected: number;
      hp_after: number;
      incapacitated: boolean;
      targets: number;
    };
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `마지막유언-실제폭발-퇴장-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· farewell$/ })
        .click();
    await select();
    const cap = page.getByRole("textbox", {
      name: "farewell value.DefenseProgram.LastWill.attack_percent",
      exact: true,
    });
    await cap.fill("450");
    await cap.press("Tab");
    const changed = original.replace(
      "attack_percent: 300",
      "attack_percent: 450",
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
    await capture(page, info, "composition-last-will-properties");
    await inspectLayout(page, info, "composition-last-will-properties", {
      landmarks: false,
    });
    await inspectAccessibility(page, info, "composition-last-will-properties");
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
    expect(run.latest_last_will).toMatchObject(
      expected.find((e) => e.percent === 450)!.witness,
    );
    expect(run.latest_last_will.origin).not.toBeNull();
    const wanted = expected.find((e) => e.percent === 450)!.witness;
    expect(run.latest_effect).toMatchObject({
      time_ms: wanted.time_ms,
      hp_delta: -wanted.base_damage,
      equipment: {
        persistent_copy_uuid: wanted.equipment.persistent_copy_uuid,
      },
      field_path: "equipment_defenses[0].program.LastWill",
    });
    const timing = dialog.getByRole("group", { name: "마지막 유언 관측" });
    await expect(timing).toContainText("전투불능");
    await expect(timing).toContainText(
      `HP ${wanted.hp_before} → 1 보호 → 0 퇴장`,
    );
    await expect(timing).toContainText(`공격력 ${wanted.attack}`);
    await expect(timing).toContainText(`기초 피해 ${wanted.base_damage}`);
    await expect(timing).toContainText(`폭발 대상 ${wanted.targets}명`);
    await expect(timing).toContainText(wanted.equipment.persistent_copy_uuid);
    await expect(timing).toContainText("이번 전투 사용 완료");
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-last-will-world");
    await inspectLayout(page, info, "composition-last-will-world", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-last-will-world");
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
    ).toContain("attack_percent: 450");
    await runSource.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-last-will-source");
    await inspectLayout(page, info, "composition-last-will-source", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-last-will-source");
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
    const invalid = changed.replace("attack_percent: 450", "attack_percent: 0");
    await source.fill(invalid);
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(
      /last will percent requires 1..10000/,
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
    await capture(page, info, "composition-last-will-diagnostic");
    await inspectLayout(page, info, "composition-last-will-diagnostic", {
      landmarks: false,
    });
    await inspectAccessibility(page, info, "composition-last-will-diagnostic");
    await info.attach("last-will-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
