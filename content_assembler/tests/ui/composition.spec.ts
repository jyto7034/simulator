import { readFile, unlink } from "node:fs/promises";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
} from "./fixtures";

test("Composition: same-source properties, undo, real compile, diagnostics and incomplete file roundtrip", async ({
  page,
}, info) => {
  const name = `조합-검증-${info.project.name}-${Date.now()}.skill.yaml`;
  const path = `content/${name}`;
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
    await expect(source).toBeVisible();
    const original = await source.inputValue();
    const prepare = page.getByRole("textbox", {
      name: "풍운승려 검증 사본 prepare_ms",
      exact: true,
    });
    await prepare.fill("150");
    await prepare.press("Tab");
    await expect(source).toHaveValue(
      original.replace("prepare_ms: 100", "prepare_ms: 150"),
    );
    await page.getByRole("button", { name: "실행 취소", exact: true }).click();
    await expect(source).toHaveValue(original);
    await page.getByRole("button", { name: "다시 실행", exact: true }).click();
    const response = page.waitForResponse((r) =>
      r.url().endsWith("/__content/compile-composition"),
    );
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const compiled = await (await response).json();
    expect(compiled.status).toBe("compiled");
    expect(compiled.artifact).toContain("skill-composition-native");
    await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
    await capture(page, info, "composition-compiled");
    await inspectAccessibility(page, info, "composition-compiled");
    expect(
      await page.evaluate(
        () =>
          document.documentElement.scrollWidth <= innerWidth + 1 &&
          document.documentElement.scrollHeight <= innerHeight + 1,
      ),
    ).toBe(true);
    await source.fill(
      original.replace("Shared: burst-power", "Shared: missing-power"),
    );
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page.getByRole("button", { name: /^Reference ·/ });
    await expect(diagnostic).toBeVisible();
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((element: HTMLTextAreaElement) =>
        element.value.slice(element.selectionStart, element.selectionEnd),
      ),
    ).toContain("missing-power");
    await capture(page, info, "composition-reference-error");
    await inspectAccessibility(page, info, "composition-reference-error");
    const incomplete = "# 미완성 원문 그대로 보존\nversion: 1\nskills: [";
    await source.fill(incomplete);
    await page
      .getByRole("textbox", { name: "조합 파일 이름", exact: true })
      .fill(name);
    await page.getByRole("button", { name: /^저장/ }).click();
    await expect(
      page.getByRole("status").filter({ hasText: "저장했습니다." }),
    ).toBeVisible();
    expect(await readFile(path, "utf8")).toBe(incomplete);
    await page.getByRole("button", { name: "열기", exact: true }).click();
    const dialog = page.getByRole("dialog");
    await expect(
      dialog.getByRole("button", { name, exact: true }),
    ).toBeVisible();
    await capture(page, info, "composition-open");
    await inspectAccessibility(page, info, "composition-open");
    for (let index = 0; index < 8; index++) {
      await page.keyboard.press("Tab");
      expect(
        await dialog.evaluate((node) => node.contains(document.activeElement)),
      ).toBe(true);
    }
    await page.keyboard.press("Escape");
    await expect(dialog).not.toBeVisible();
    await expect(
      page.getByRole("button", { name: "열기", exact: true }),
    ).toBeFocused();
    await page.getByRole("button", { name: "열기", exact: true }).click();
    await dialog.getByRole("button", { name, exact: true }).click();
    await expect(dialog).not.toBeVisible();
    await expect(source).toHaveValue(incomplete);
    await capture(page, info, "composition-incomplete");
    await inspectAccessibility(page, info, "composition-incomplete");
  } finally {
    await unlink(path).catch((error) => {
      if (error.code !== "ENOENT") throw error;
    });
  }
});

test("Composition: long Korean names and empty definitions remain usable", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "조합", exact: true })
    .click();
  const source = page.getByRole("textbox", {
    name: "스킬 조합 원본",
    exact: true,
  });
  const original = await source.inputValue();
  await source.fill(
    original.replace(
      "풍운승려 검증 사본",
      "긴 한글 이름의 공유 정의와 사용처를 함께 확인하는 풍운승려 검증 사본",
    ),
  );
  await capture(page, info, "composition-long-name");
  await inspectAccessibility(page, info, "composition-long-name");
  await source.fill(
    "version: 1\nid: empty\nvalues: []\ndefinitions: []\nskills: []\n",
  );
  await capture(page, info, "composition-empty");
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <= innerWidth + 1 &&
        document.documentElement.scrollHeight <= innerHeight + 1,
    ),
  ).toBe(true);
});

test("Composition: authored state fields preserve source edits and Core rejects invalid bounds", async ({
  page,
}, info) => {
  const original = await readFile(
    new URL(
      "../../../core_enhanced/tests/fixtures/skill_composition/state_events.skill.yaml",
      import.meta.url,
    ),
    "utf8",
  );
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
  await page
    .getByRole("group", { name: "정의 선택" })
    .getByRole("button", { name: /· aid$/ })
    .click();
  const initial = page.getByRole("textbox", {
    name: "aid value.Effects.0.op.Sustained.definition.State.program.fields.0.value.Integer.initial",
    exact: true,
  });
  await expect(initial).toHaveValue("2");
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
  await initial.scrollIntoViewIfNeeded();
  await initial.focus();
  await capture(page, info, "composition-state-fields");
  await inspectAccessibility(page, info, "composition-state-fields");
  await initial.fill("3");
  await initial.press("Tab");
  await expect(source).toHaveValue(
    original.replace("initial: 2", "initial: 3"),
  );
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  await expect(page.getByRole("group", { name: "조합 진단" })).toContainText(
    "invalid type or field bounds",
  );
  await capture(page, info, "composition-state-bounds-error");
  await inspectAccessibility(page, info, "composition-state-bounds-error");
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  await expect(source).toHaveValue(original);
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
});
