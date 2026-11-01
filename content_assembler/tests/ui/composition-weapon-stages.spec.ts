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

test("Composition: weapon stages edit upgraded push and follow actual bloom copy through projectile impact", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_WEAPON_STAGES_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_WEAPON_STAGES_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    distance: number;
    shot: {
      stage: string;
      time_ms: number;
      interval_ms: number;
      windup_ms: number;
      actual_hp_loss: number;
      push_distance_units: number;
      equipment: { persistent_copy_uuid: string };
      from_data_units: [number, number];
      to_data_units: [number, number];
    };
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `나비무기-승급-실제관통-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· funeral_shots$/ })
        .click();
    await select();
    const cap = page.getByRole("textbox", {
      name: "funeral_shots value.AttackProgram.weapon_stages.upgraded.push_distance_units",
      exact: true,
    });
    await cap.fill("200000");
    await cap.press("Tab");
    const changed = original.replace(
      "push_distance_units: 100000",
      "push_distance_units: 200000",
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
    await capture(page, info, "composition-weapon-stages-properties");
    await inspectLayout(page, info, "composition-weapon-stages-properties", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-weapon-stages-properties",
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
    const wanted = expected.find((e) => e.distance === 200000)!.shot;
    expect(run.latest_weapon_stage).toMatchObject(wanted);
    expect(run.latest_weapon_stage.origin).not.toBeNull();
    const configuration = JSON.parse(
      await readFile(resolve(root, "trial.json"), "utf8"),
    );
    expect(run.accepted_steps).toBe(
      configuration.scenario.preparation_steps.length +
        configuration.scenario.steps.length,
    );
    const timing = dialog.getByRole("group", { name: "무기 단계 관측" });
    await expect(timing).toContainText("승급형");
    await expect(timing).toContainText("설정 사격 간격 250ms · 준비 50ms");
    await expect(dialog).not.toContainText(
      "최근 관측에 이 조합의 HP 변화가 없습니다.",
    );
    await expect(timing).toContainText("실제 HP 피해 12 · 밀치기 최대 0.2타일");
    await expect(timing).toContainText(wanted.equipment.persistent_copy_uuid);
    for (const coordinates of [wanted.from_data_units, wanted.to_data_units]) {
      await expect(timing).toContainText(
        coordinates.map((n) => n / 1000000).join(", "),
      );
    }
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-weapon-stages-world");
    await inspectLayout(page, info, "composition-weapon-stages-world", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-weapon-stages-world");
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
    ).toContain("push_distance_units: 200000");
    await runSource.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-weapon-stages-source");
    await inspectLayout(page, info, "composition-weapon-stages-source", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-weapon-stages-source");
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
    const invalid = changed.replace("interval_ms: 250", "interval_ms: 0");
    await source.fill(invalid);
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(
      /weapon stage requires positive attack interval/,
    );
    await diagnostic.click();
    await expect(source).toBeFocused();
    const invalidStart =
      invalid.indexOf("interval_ms: 0") + "interval_ms: ".length;
    expect(
      await source.evaluate((node: HTMLTextAreaElement) => ({
        start: node.selectionStart,
        end: node.selectionEnd,
        text: node.value.slice(node.selectionStart, node.selectionEnd),
      })),
    ).toEqual({ start: invalidStart, end: invalidStart + 1, text: "0" });
    await capture(page, info, "composition-weapon-stages-diagnostic");
    await inspectLayout(page, info, "composition-weapon-stages-diagnostic", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-weapon-stages-diagnostic",
    );
    await info.attach("weapon-stages-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
