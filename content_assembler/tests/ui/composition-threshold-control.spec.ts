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

test("Composition: threshold counters edit the same source and trace actual weapon consumption", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_THRESHOLD_CONTROL_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_THRESHOLD_CONTROL_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    threshold: number;
    time_ms: number;
    target_unit: string;
    outcome: Record<string, unknown>;
    equipment: unknown;
    origin: unknown;
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `서리마검-적중누적-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· frostbite$/ })
        .click();
    await expect(
      page.getByRole("group", { name: "정의 선택" }).getByRole("button", {
        name: /장비 공격 · frost_splinter$/,
      }),
    ).toBeVisible();
    await select();
    const reward = page.getByRole("textbox", {
      name: "frostbite value.AttackProgram.on_hit_status.profile.kind.ThresholdControl.threshold",
      exact: true,
    });
    await reward.fill("3");
    await reward.press("Tab");
    const changed = original.replace("threshold: 5", "threshold: 3");
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
    await capture(page, info, "composition-threshold-control-properties");
    await inspectLayout(
      page,
      info,
      "composition-threshold-control-properties",
      {
        landmarks: false,
      },
    );
    await inspectAccessibility(
      page,
      info,
      "composition-threshold-control-properties",
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
    const wanted = expected.find((e) => e.threshold === 3)!;
    const observed = run.latest_hit_counter;
    expect(observed.time_ms).toBe(wanted.time_ms);
    expect(observed.target_unit).toBe(wanted.target_unit);
    expect(observed.origin).toEqual(wanted.origin);
    expect(observed.equipment).toEqual(wanted.equipment);
    for (const [key, value] of Object.entries(wanted.outcome))
      expect(observed[key]).toEqual(value);
    expect(observed.source_file).toBe(run.identity.artifact_path);
    const timing = dialog.getByRole("group", { name: "적중 누적 관측" });
    await expect(timing).toContainText("임계치 3");
    await expect(timing).toContainText("2 + 1 → 0");
    await expect(timing).toContainText("임계치 소모 · 기절 적용");
    await expect(timing).toContainText(observed.equipment.persistent_copy_uuid);
    await expect(dialog).not.toContainText(
      "최근 관측에 이 조합의 HP 변화가 없습니다.",
    );
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-threshold-control-world");
    await inspectLayout(page, info, "composition-threshold-control-world", {
      modal: true,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-threshold-control-world",
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
    ).toContain("threshold: 3");
    await capture(page, info, "composition-threshold-control-source");
    await inspectLayout(page, info, "composition-threshold-control-source", {
      modal: true,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-threshold-control-source",
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
    await source.fill(changed.replace("threshold: 3", "threshold: 1001"));
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(/Threshold control requires/);
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toContain("1001");
    await capture(page, info, "composition-threshold-control-diagnostic");
    await inspectLayout(
      page,
      info,
      "composition-threshold-control-diagnostic",
      {
        landmarks: false,
      },
    );
    await inspectAccessibility(
      page,
      info,
      "composition-threshold-control-diagnostic",
    );
    await info.attach("threshold-control-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
