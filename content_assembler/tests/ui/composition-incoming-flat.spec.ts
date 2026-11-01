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

test("Composition: incoming flat edits the same source and traces actual armor and fixed physical reduction", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_INCOMING_FLAT_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_INCOMING_FLAT_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    percent: number;
    witness: {
      time_ms: number;
      target: string;
      defense: number;
      allowance: number;
      used: number;
      remaining: number;
      equipment: { persistent_copy_uuid: string };
      definition_id: string;
      field_path: string;
    };
    origin: { definition: string; calls: string[]; inputs: string[] };
    last_hp_loss: number;
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `쐐기풀외투-고정차감-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· incoming-flat$/ })
        .click();
    await expect(
      page.getByRole("group", { name: "정의 선택" }).getByRole("button", {
        name: /장비 방어 · black_swan_nettle_coat$/,
      }),
    ).toBeVisible();
    await select();
    const reward = page.getByRole("textbox", {
      name: "incoming-flat value.DefenseProgram.PhysicalBasicFlatReduction.amount.root.Clamp.value.Ratio.numerator.Mul.scalar.Const.value",
      exact: true,
    });
    await reward.fill("75");
    await reward.press("Tab");
    const changed = original.replace("value: 50,", "value: 75,");
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
    await capture(page, info, "composition-incoming-flat-properties");
    await inspectLayout(page, info, "composition-incoming-flat-properties", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-incoming-flat-properties",
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
    const wanted = expected.find((e) => e.percent === 75)!;
    const observed = run.latest_incoming_flat;
    for (const key of [
      "time_ms",
      "target",
      "defense",
      "allowance",
      "used",
      "remaining",
      "equipment",
      "definition_id",
      "field_path",
    ] as const)
      expect(observed[key]).toEqual(wanted.witness[key]);
    expect(observed.origin).toEqual(wanted.origin);
    expect(observed.source_file).toBe(run.identity.artifact_path);
    const timing = dialog.getByRole("group", {
      name: "물리 기본 타격 차감 관측",
    });
    await expect(timing).toContainText(`차감 ${wanted.witness.used}`);
    await expect(timing).toContainText(`방어력 ${wanted.witness.defense}`);
    await expect(timing).toContainText(`타격 한도 ${wanted.witness.allowance}`);
    await expect(timing).toContainText(
      wanted.witness.equipment.persistent_copy_uuid,
    );
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-incoming-flat-world");
    await inspectLayout(page, info, "composition-incoming-flat-world", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-incoming-flat-world");
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
    ).toContain("value: 75,");
    await capture(page, info, "composition-incoming-flat-source");
    await inspectLayout(page, info, "composition-incoming-flat-source", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-incoming-flat-source");
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
      changed.replace("value: 100, unit: Scalar", "value: 0, unit: Scalar"),
    );
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(/Ratio denominator is zero/);
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toContain("value: 0, unit: Scalar");
    await capture(page, info, "composition-incoming-flat-diagnostic");
    await inspectLayout(page, info, "composition-incoming-flat-diagnostic", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-incoming-flat-diagnostic",
    );
    await info.attach("incoming-flat-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
