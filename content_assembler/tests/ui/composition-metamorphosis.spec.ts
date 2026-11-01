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

test("Composition: metamorphosis edits form and traces the permanent employee transformation", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_METAMORPHOSIS_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_METAMORPHOSIS_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: {
    form: string;
    witness: {
      time_ms: number;
      employee_uuid: string;
      actor: string;
      successor: string;
      equipment: { persistent_copy_uuid: string };
    };
    origin: { definition: string; calls: string[]; inputs: string[] };
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
  const name = `미미크리-변신-${info.project.name}-${Date.now()}.skill.yaml`;
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
        .getByRole("button", { name: /· mimicry-revival$/ })
        .click();
    await expect(
      page.getByRole("group", { name: "정의 선택" }).getByRole("button", {
        name: /장비 방어 · placeholder_nothing_there_accessory$/,
      }),
    ).toBeVisible();
    await select();
    const formInput = page.getByRole("textbox", {
      name: "mimicry-revival value.DefenseProgram.EmployeeRevival.transform_into",
      exact: true,
    });
    await formInput.fill("f-05-32_warm_hearted_woodsman");
    await formInput.press("Tab");
    const changed = original.replace(
      "transform_into: o-06-20_nothing_there",
      'transform_into: "f-05-32_warm_hearted_woodsman"',
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
    await formInput.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-metamorphosis-properties");
    await inspectLayout(page, info, "composition-metamorphosis-properties", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-metamorphosis-properties",
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
    const wanted = expected.find(
      (e) => e.form === "f-05-32_warm_hearted_woodsman",
    )!;
    const witness = run.latest_employee_transformation;
    expect(witness.time_ms).toBe(wanted.witness.time_ms);
    expect(witness.form).toBe(wanted.form);
    expect(witness.origin).toEqual(wanted.origin);
    expect(witness.actor).toBe(wanted.witness.actor);
    expect(witness.successor).toBe(wanted.witness.successor);
    expect(witness.actor).not.toBe(witness.successor);
    expect(witness.equipment).toEqual(wanted.witness.equipment);
    expect(witness.source_file).toBe(run.identity.artifact_path);
    const timing = dialog.getByRole("group", { name: "직원 변신 관측" });
    await expect(timing).toContainText("직원 영구 사망");
    await expect(timing).toContainText("승리 필수");
    await expect(timing).toContainText(wanted.witness.employee_uuid);
    await expect(timing).toContainText(
      wanted.witness.equipment.persistent_copy_uuid,
    );
    await timing.scrollIntoViewIfNeeded();
    await capture(page, info, "composition-metamorphosis-world");
    await inspectLayout(page, info, "composition-metamorphosis-world", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-metamorphosis-world");
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
    ).toContain('transform_into: "f-05-32_warm_hearted_woodsman"');
    await capture(page, info, "composition-metamorphosis-source");
    await inspectLayout(page, info, "composition-metamorphosis-source", {
      modal: true,
    });
    await inspectAccessibility(page, info, "composition-metamorphosis-source");
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
      changed.replace(
        'transform_into: "f-05-32_warm_hearted_woodsman"',
        "transform_into: ''",
      ),
    );
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const diagnostic = page
      .getByRole("group", { name: "조합 진단" })
      .getByRole("button")
      .first();
    await expect(diagnostic).toContainText(
      /transformation requires a definition/,
    );
    await diagnostic.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((node: HTMLTextAreaElement) =>
        node.value.slice(node.selectionStart, node.selectionEnd),
      ),
    ).toBe("''");
    await capture(page, info, "composition-metamorphosis-diagnostic");
    await inspectLayout(page, info, "composition-metamorphosis-diagnostic", {
      landmarks: false,
    });
    await inspectAccessibility(
      page,
      info,
      "composition-metamorphosis-diagnostic",
    );
    await info.attach("metamorphosis-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch(() => undefined);
  }
});
