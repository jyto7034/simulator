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

test("Composition: named predicate edits its shared condition and changes the real equipped outcome", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_PREDICATE_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_PREDICATE_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    minimum: number;
    actual_time_ms: number;
    latest: { time_ms: number; change: { expires_at_ms: number } };
    origin: { definition: string; calls: string[]; inputs: string[] };
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `공통조건-활성시간-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /공통 조건 · active-window$/ })
        .click();
    await expect(
      page
        .getByRole("group", { name: "정의 선택" })
        .getByRole("button", { name: /장비 상태 · state_fixture_teardrop$/ }),
    ).toBeVisible();
    await select();
    const threshold = page.getByRole("textbox", {
      name: "active-window body.Logic.All.1.Compare.right.Literal.Amount.root.Const.value",
      exact: true,
    });
    await threshold.fill("1");
    await threshold.press("Tab");
    const changed = original.replace("value: 0", "value: 1");
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
    await threshold.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-predicate-properties");
    await inspectLayout(page, info, "composition-predicate-properties", {
      landmarks: false,
    });
    await inspectAccessibility(page, info, "composition-predicate-properties");
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
    const wanted = expected.find((e) => e.minimum === 1)!;
    expect(run.latest_sustained.time_ms).toBe(wanted.latest.time_ms);
    expect(run.latest_sustained.expires_at_ms).toBe(
      wanted.latest.change.expires_at_ms,
    );
    expect(run.latest_sustained.time_ms).toBe(wanted.actual_time_ms);
    expect(run.latest_sustained.ended).toBe("Terminal");
    expect(run.latest_sustained.origin).toEqual(wanted.origin);
    expect(run.latest_sustained.field_path).toBe(
      "equipment_states[0].program.handlers[0].body[3].then[0].duration",
    );
    expect(run.latest_sustained.source_file).toBe(run.identity.artifact_path);
    const timing = dialog.getByRole("group", { name: "지속 효과 관측" });
    await expect(timing).toContainText("state_fixture_teardrop");
    await expect(timing).toContainText("만료 시각");
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-predicate-world");
    await inspectLayout(page, info, "composition-predicate-world", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-predicate-world");
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
    ).toContain("name: needs-reset");
    await capture(page, info, "composition-predicate-source");
    await inspectLayout(page, info, "composition-predicate-source", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-predicate-source");
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
    const invalid = changed.lastIndexOf("Parameter: stacks");
    expect(invalid).toBeGreaterThan(0);
    await source.fill(
      changed.slice(0, invalid) +
        changed
          .slice(invalid)
          .replace("Parameter: stacks", "Parameter: missing"),
    );
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(/unknown predicate parameter/);
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toContain("Parameter: missing");
    await capture(page, info, "composition-predicate-diagnostic");
    await inspectLayout(page, info, "composition-predicate-diagnostic", {
      landmarks: false,
    });
    await inspectAccessibility(page, info, "composition-predicate-diagnostic");
    await info.attach("predicate-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
