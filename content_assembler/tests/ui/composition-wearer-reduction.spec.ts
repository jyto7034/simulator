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

test("Composition: wearer reduction edits the same source and traces current HP and actual armor", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_WEARER_REDUCTION_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_WEARER_REDUCTION_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    percent: number;
    witness: {
      time_ms: number;
      target: string;
      percent: number;
      current_hp: number;
      max_hp: number;
      strike: number;
      equipment: { persistent_copy_uuid: string };
      source_file: string;
      field_path: string;
    };
    origin: { definition: string; calls: string[]; inputs: string[] };
    last_hp_loss: number;
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `철상자-HP감면-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· current-hp-reduction$/ })
        .click();
    await expect(
      page.getByRole("group", { name: "정의 선택" }).getByRole("button", {
        name: /장비 방어 · placeholder_schadenfreude_armor$/,
      }),
    ).toBeVisible();
    await select();
    const reward = page.getByRole("textbox", {
      name: "current-hp-reduction value.DefenseProgram.WearerReduction.percent.root.Clamp.max.Const.value",
      exact: true,
    });
    await reward.fill("35");
    await reward.press("Tab");
    const changed = original.replace("value: 50,", "value: 35,");
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
    await capture(page, info, "composition-wearer-reduction-properties");
    await inspectLayout(page, info, "composition-wearer-reduction-properties", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-wearer-reduction-properties",
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
    const wanted = expected.find((e) => e.percent === 35)!;
    const observed = run.latest_wearer_reduction;
    for (const key of [
      "time_ms",
      "target",
      "percent",
      "current_hp",
      "max_hp",
      "strike",
      "equipment",
    ] as const)
      expect(observed[key]).toEqual(wanted.witness[key]);
    expect(observed.origin).toEqual(wanted.origin);
    expect(observed.source_file).toBe(run.identity.artifact_path);
    const timing = dialog.getByRole("group", {
      name: "현재 HP 기반 감면 관측",
    });
    await expect(timing).toContainText("피해 감소 35%");
    await expect(timing).toContainText(
      `HP ${wanted.witness.current_hp} / ${wanted.witness.max_hp}`,
    );
    await expect(timing).toContainText(
      wanted.witness.equipment.persistent_copy_uuid,
    );
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-wearer-reduction-world");
    await inspectLayout(page, info, "composition-wearer-reduction-world", {
      modal: true,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-wearer-reduction-world",
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
    ).toContain("value: 35,");
    await capture(page, info, "composition-wearer-reduction-source");
    await inspectLayout(page, info, "composition-wearer-reduction-source", {
      modal: true,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-wearer-reduction-source",
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
    await source.fill(changed.replace("moment: Live", "moment: Snapshot"));
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(/wearer amount requires Live/);
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toContain("moment: Snapshot");
    await capture(page, info, "composition-wearer-reduction-diagnostic");
    await inspectLayout(page, info, "composition-wearer-reduction-diagnostic", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-wearer-reduction-diagnostic",
    );
    await info.attach("wearer-reduction-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
