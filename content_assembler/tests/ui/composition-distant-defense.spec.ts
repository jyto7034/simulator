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

test("Composition: distant defense edits the same source and traces actual armor and current hostile distance", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_DISTANT_DEFENSE_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_DISTANT_DEFENSE_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    percent: number;
    witness: {
      time_ms: number;
      actor: string;
      wearer: string;
      target: string;
      percent: number;
      distance_squared: number;
      equipment: { persistent_copy_uuid: string };
      contributors: { source_file: string; field_path: string }[];
    };
    origin: { definition: string; calls: string[]; inputs: string[] };
    last_hp_loss: number;
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `황홀의외피-거리감면-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· distant-defense$/ })
        .click();
    await expect(
      page.getByRole("group", { name: "정의 선택" }).getByRole("button", {
        name: /장비 방어 · porccubus_ecstasy_coat$/,
      }),
    ).toBeVisible();
    await select();
    const reward = page.getByRole("textbox", {
      name: "distant-defense value.DefenseProgram.ConditionalReduction.percent.root.Const.value",
      exact: true,
    });
    await reward.fill("35");
    await reward.press("Tab");
    const changed = original.replace("value: 20,", "value: 35,");
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
    await capture(page, info, "composition-distant-defense-properties");
    await inspectLayout(page, info, "composition-distant-defense-properties", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-distant-defense-properties",
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
    const observed = run.latest_conditional_reduction;
    for (const key of [
      "time_ms",
      "actor",
      "wearer",
      "target",
      "percent",
      "distance_squared",
      "equipment",
    ] as const)
      expect(observed[key]).toEqual(wanted.witness[key]);
    expect(observed.origins[0]).toEqual(wanted.origin);
    expect(observed.contributors[0].source_file).toBe(
      run.identity.artifact_path,
    );
    const timing = dialog.getByRole("group", { name: "조건부 감면 관측" });
    await expect(timing).toContainText("받는 피해 −35%");
    await expect(timing).toContainText(
      `판정 거리 · ${Math.sqrt(wanted.witness.distance_squared).toFixed(3)}타일`,
    );
    await expect(timing).toContainText(
      wanted.witness.equipment.persistent_copy_uuid,
    );
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-distant-defense-world");
    await inspectLayout(page, info, "composition-distant-defense-world", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-distant-defense-world");
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
    await capture(page, info, "composition-distant-defense-source");
    await inspectLayout(page, info, "composition-distant-defense-source", {
      modal: true,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-distant-defense-source",
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
    await source.fill(changed.replace("unit: Scalar", "unit: Magnitude"));
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(
      /conditional damage percentage requires Scalar/,
    );
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toContain("unit: Magnitude");
    await capture(page, info, "composition-distant-defense-diagnostic");
    await inspectLayout(page, info, "composition-distant-defense-diagnostic", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-distant-defense-diagnostic",
    );
    await info.attach("distant-defense-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
