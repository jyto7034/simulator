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

test("Composition: channel opening edits armor protection and traces actual hostile damage", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_CHANNEL_OPENING_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_CHANNEL_OPENING_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    percent: number;
    witness: {
      equipment: { persistent_copy_uuid: string };
      time_ms: number;
      started_at_ms: number;
      window_end_ms: number;
      reduction_percent: number;
    };
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `집중초반보호-실제방어구-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· opening-protection$/ })
        .click();
    await select();
    const cap = page.getByRole("textbox", {
      name: "opening-protection value.DefenseProgram.ChannelOpeningGuard.reduction_percent",
      exact: true,
    });
    await cap.fill("75");
    await cap.press("Tab");
    const changed = original.replace(
      "reduction_percent: 50",
      "reduction_percent: 75",
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
    await capture(page, info, "composition-channel-opening-properties");
    await inspectLayout(page, info, "composition-channel-opening-properties", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-channel-opening-properties",
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
    expect(run.latest_channel_opening).toMatchObject(
      expected.find((e) => e.percent === 75)!.witness,
    );
    expect(run.latest_channel_opening.origin).not.toBeNull();
    const wanted = expected.find((e) => e.percent === 75)!.witness;
    const timing = dialog.getByRole("group", { name: "집중 초반 보호 관측" });
    await expect(timing).toContainText(
      `피해 감소 ${wanted.reduction_percent}%`,
    );
    await expect(timing).toContainText(`집중 시작 ${wanted.started_at_ms}ms`);
    await expect(timing).toContainText(`보호 한계 ${wanted.window_end_ms}ms`);
    await expect(timing).toContainText(wanted.equipment.persistent_copy_uuid);
    await expect(timing).toContainText("집중이 끝나면 보호도 끝납니다.");
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-channel-opening-world");
    await inspectLayout(page, info, "composition-channel-opening-world", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-channel-opening-world");
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
    ).toContain("reduction_percent: 75");
    await runSource.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-channel-opening-source");
    await inspectLayout(page, info, "composition-channel-opening-source", {
      modal: true,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-channel-opening-source",
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
    const invalid = changed.replace(
      "reduction_percent: 75",
      "reduction_percent: 0",
    );
    await source.fill(invalid);
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(
      /opening reduction must be from 1 to 100/,
    );
    await diagnostic.click();
    await expect(source).toBeFocused();
    const invalidStart =
      invalid.indexOf("reduction_percent: 0") + "reduction_percent: ".length;
    expect(
      await source.evaluate((node: HTMLTextAreaElement) => ({
        start: node.selectionStart,
        end: node.selectionEnd,
        text: node.value.slice(node.selectionStart, node.selectionEnd),
      })),
    ).toEqual({ start: invalidStart, end: invalidStart + 1, text: "0" });
    await capture(page, info, "composition-channel-opening-diagnostic");
    await inspectLayout(page, info, "composition-channel-opening-diagnostic", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-channel-opening-diagnostic",
    );
    await info.attach("channel-opening-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
