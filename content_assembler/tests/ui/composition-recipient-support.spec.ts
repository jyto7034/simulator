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

test("Composition: recipient support edits the same source and traces actual block healing and control duration", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_RECIPIENT_SUPPORT_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_RECIPIENT_SUPPORT_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: Array<{
    rate: number;
    healing: Record<string, unknown>;
    control: Record<string, unknown>;
    heal_origin: unknown;
    control_origin: unknown;
  }> = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `추억의브로치-회복과제어-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· recipient-support$/ })
        .click();
    await expect(
      page.getByRole("group", { name: "정의 선택" }).getByRole("button", {
        name: /장비 지원 · black_swan_memory_brooch$/,
      }),
    ).toBeVisible();
    await select();
    const reward = page.getByRole("textbox", {
      name: "recipient-support value.SupportProgram.received_heal_percent.root.Clamp.value.Mul.scalar.Const.value",
      exact: true,
    });
    await reward.fill("35");
    await reward.press("Tab");
    const duration = page.getByRole("textbox", {
      name: "recipient-support value.SupportProgram.control_duration.reduction_percent",
      exact: true,
    });
    await duration.fill("25");
    await duration.press("Tab");
    const changed = original
      .replace("value: 20,", "value: 35,")
      .replace("reduction_percent: 50", "reduction_percent: 25");
    await expect(source).toHaveValue(changed);
    await page.getByRole("button", { name: "실행 취소", exact: true }).click();
    await page.getByRole("button", { name: "실행 취소", exact: true }).click();
    await expect(source).toHaveValue(original);
    await page.getByRole("button", { name: "다시 실행", exact: true }).click();
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
    await capture(page, info, "composition-recipient-support-properties");
    await inspectLayout(
      page,
      info,
      "composition-recipient-support-properties",
      {
        landmarks: false,
      },
    );
    await inspectAccessibility(
      page,
      info,
      "composition-recipient-support-properties",
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
    const wanted = expected.find((e) => e.rate === 35)!;
    for (const [key, witness, origin] of [
      ["latest_received_heal", wanted.healing, wanted.heal_origin],
      ["latest_control_duration", wanted.control, wanted.control_origin],
    ] as const) {
      for (const [field, value] of Object.entries(witness))
        if (field !== "source_file" && field !== "incoming")
          expect(run[key][field]).toEqual(value);
      expect(run[key].origin).toEqual(origin);
      expect(run[key].source_file).toBe(run.identity.artifact_path);
    }
    const timing = dialog.getByRole("group", { name: "받는 회복 관측" });
    await expect(timing).toContainText(
      `저지 ${wanted.healing.blocked_hostiles}명`,
    );
    await expect(timing).toContainText("증가 35%");
    await expect(
      dialog.getByRole("group", { name: "제어 시간 감소 관측" }),
    ).toContainText("2000ms → 1500ms");
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-recipient-support-world");
    await inspectLayout(page, info, "composition-recipient-support-world", {
      modal: true,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-recipient-support-world",
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
    await capture(page, info, "composition-recipient-support-source");
    await inspectLayout(page, info, "composition-recipient-support-source", {
      modal: true,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-recipient-support-source",
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
      changed.replace("value: 1, unit: Count", "value: 0, unit: Count"),
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
    ).toContain("value: 0, unit: Count");
    await capture(page, info, "composition-recipient-support-diagnostic");
    await inspectLayout(
      page,
      info,
      "composition-recipient-support-diagnostic",
      {
        landmarks: false,
      },
    );
    await inspectAccessibility(
      page,
      info,
      "composition-recipient-support-diagnostic",
    );
    await info.attach("recipient-support-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
