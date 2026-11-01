import { readFile, unlink } from "node:fs/promises";
import { resolve } from "node:path";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
} from "./fixtures";

test("Composition: manual state edits child power and runs the same input in World", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_MANUAL_STATE_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_MANUAL_STATE_TRIAL_FIXTURE must identify the exported manual state World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected = JSON.parse(
    await readFile(resolve(root, "expected.json"), "utf8"),
  ) as { cap: number; latest: { change: { before: number; after: number } } }[];
  const edited = expected.find((entry) => entry.cap === 35)!;
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
  const definitions = page.getByRole("group", { name: "정의 선택" });
  await definitions.getByRole("button", { name: /· finish-power$/ }).click();
  const cap = page.getByRole("textbox", {
    name: "finish-power value.Amount.root.Clamp.max.Const.value",
    exact: true,
  });
  await cap.fill("35");
  await cap.press("Tab");
  await expect(source).toHaveValue(
    original.replace(
      "value: 45, unit: Magnitude",
      "value: 35, unit: Magnitude",
    ),
  );
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
  await capture(page, info, "composition-manual-state-power");
  await inspectAccessibility(page, info, "composition-manual-state-power");
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  await expect(source).toHaveValue(original);
  await page.getByRole("button", { name: "다시 실행", exact: true }).click();
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
  await page.getByRole("button", { name: "실행 시험", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "조합 실행 시험" });
  await dialog
    .getByLabel("실행 설정 파일", { exact: true })
    .setInputFiles(resolve(root, "trial.json"));
  await expect(dialog.getByText("trial.json", { exact: true })).toBeVisible();
  const response = page.waitForResponse(
    (r) => r.url().endsWith("/__content/run-composition"),
    { timeout: 120_000 },
  );
  await dialog
    .getByRole("button", { name: "시나리오 실행", exact: true })
    .click();
  const run = await (await response).json();
  expect(run.status, JSON.stringify(run.failure)).toBe("completed");
  expect(run.accepted_steps).toBe(29);
  expect(run.latest_effect.skill_id).toBe("authored-finish");
  expect(run.latest_effect.hp_delta).toBe(
    edited.latest.change.after - edited.latest.change.before,
  );
  expect(run.latest_effect.origin).not.toBeNull();
  await expect(dialog.getByText(/작성 효과 관측/)).toContainText(
    "authored-finish",
  );
  await capture(page, info, "composition-manual-state-world-completed");
  await inspectAccessibility(
    page,
    info,
    "composition-manual-state-world-completed",
  );
  for (let index = 0; index < 8; index++) {
    await page.keyboard.press("Tab");
    expect(
      await dialog.evaluate((node) => node.contains(document.activeElement)),
    ).toBe(true);
  }
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("button", { name: "실행 시험", exact: true }),
  ).toBeFocused();
  await definitions.getByRole("button", { name: /· transformation$/ }).click();
  const child = page.getByRole("textbox", {
    name: "transformation value.Effects.0.op.Sustained.definition.State.program.handlers.0.body.0.EndAndInvoke.skill",
    exact: true,
  });
  await child.fill("missing-child");
  await child.press("Tab");
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  const diagnostic = page.getByRole("button", { name: /^Native ·/ });
  await expect(diagnostic).toContainText("EndAndInvoke child 'missing-child'");
  await diagnostic.click();
  await expect(source).toBeFocused();
  expect(
    await source.evaluate((element: HTMLTextAreaElement) =>
      element.value.slice(element.selectionStart, element.selectionEnd),
    ),
  ).toBe('"missing-child"');
  await capture(page, info, "composition-manual-state-child-error");
  await inspectAccessibility(
    page,
    info,
    "composition-manual-state-child-error",
  );
  await info.attach("manual-state-world-result", {
    body: JSON.stringify(run, null, 2),
    contentType: "application/json",
  });
});

test("Composition: authored state property runs in World and runtime failure preserves prior result", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_STATE_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_STATE_TRIAL_FIXTURE must identify the exported state World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected = JSON.parse(
    await readFile(resolve(root, "expected.json"), "utf8"),
  ) as { heals: number[] };
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
  await initial.fill("1");
  await initial.press("Tab");
  await expect(source).toHaveValue(
    original.replace("initial: 2", "initial: 1"),
  );
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
  await page.getByRole("button", { name: "실행 시험", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "조합 실행 시험" });
  await dialog
    .getByLabel("실행 설정 파일", { exact: true })
    .setInputFiles(resolve(root, "trial.json"));
  await expect(dialog.getByText("trial.json", { exact: true })).toBeVisible();
  const runResponse = page.waitForResponse(
    (r) => r.url().endsWith("/__content/run-composition"),
    { timeout: 120_000 },
  );
  await dialog
    .getByRole("button", { name: "시나리오 실행", exact: true })
    .click();
  const run = await (await runResponse).json();
  expect(run.status, JSON.stringify(run.failure)).toBe("completed");
  // One charge must produce the baseline fixture's first observed employee heal.
  expect(run.latest_effect.hp_delta).toBe(expected.heals[0]);
  expect(run.latest_effect.field_path).toContain(
    ".op.Sustained.definition.State.program.",
  );
  expect(run.latest_effect.origin).not.toBeNull();
  await expect(dialog.getByText(/작성 효과 관측/)).toContainText(
    `HP 변화 ${expected.heals[0]}`,
  );
  await capture(page, info, "composition-state-world-completed");
  await inspectAccessibility(page, info, "composition-state-world-completed");
  for (let index = 0; index < 8; index++) {
    await page.keyboard.press("Tab");
    expect(
      await dialog.evaluate((node) => node.contains(document.activeElement)),
    ).toBe(true);
  }
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("button", { name: "실행 시험", exact: true }),
  ).toBeFocused();
  await source.fill(original.replace("value: -1", "value: 1"));
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
  await page.getByRole("button", { name: "실행 시험", exact: true }).click();
  const faultResponse = page.waitForResponse(
    (r) => r.url().endsWith("/__content/run-composition"),
    { timeout: 120_000 },
  );
  await dialog
    .getByRole("button", { name: "시나리오 실행", exact: true })
    .click();
  const fault = await (await faultResponse).json();
  expect(fault.status).toBe("failed");
  expect(fault.failure.stage).toBe("execution");
  expect(JSON.stringify(fault.failure)).toContain("value_expression_fault");
  await expect(dialog.getByRole("alert")).toContainText("실행 실패");
  await expect(dialog.getByRole("alert")).toContainText(
    "field remaining outside declared bounds",
  );
  await expect(
    dialog.getByText(`실행 ${run.identity.run_id}`, { exact: true }),
  ).toBeVisible();
  await expect(dialog.getByText(/이전 완료 실행/)).toBeVisible();
  await capture(page, info, "composition-state-world-failed");
  await inspectAccessibility(page, info, "composition-state-world-failed");
  for (let index = 0; index < 8; index++) {
    await page.keyboard.press("Tab");
    expect(
      await dialog.evaluate((node) => node.contains(document.activeElement)),
    ).toBe(true);
  }
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("button", { name: "실행 시험", exact: true }),
  ).toBeFocused();
  await info.attach("authored-state-world-results", {
    body: JSON.stringify({ run, fault }, null, 2),
    contentType: "application/json",
  });
});

test("Composition: UI source runs through real World with pinned versions and preserves results after rejection", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_TRIAL_FIXTURE must identify the exported real World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const changed = await readFile(resolve(root, "changed.skill.yaml"), "utf8");
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
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
  await page.getByRole("button", { name: "실행 시험", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "조합 실행 시험" });
  await dialog
    .getByLabel("실행 설정 파일", { exact: true })
    .setInputFiles(resolve(root, "trial.json"));
  await expect(dialog.getByText("trial.json", { exact: true })).toBeVisible();
  const oldResponse = page.waitForResponse(
    (r) => r.url().endsWith("/__content/run-composition"),
    { timeout: 120_000 },
  );
  await dialog
    .getByRole("button", { name: "시나리오 실행", exact: true })
    .click();
  await expect(dialog.getByText(/실행 중…/)).toBeVisible();
  await capture(page, info, "composition-trial-running");
  await inspectAccessibility(page, info, "composition-trial-running");
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("button", { name: "실행 시험", exact: true }),
  ).toBeFocused();
  await source.fill(changed);
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
  const old = await (await oldResponse).json();
  expect(old.status, JSON.stringify(old.failure)).toBe("completed");
  expect(old.latest_effect.origin).not.toBeNull();
  expect(old.latest_effect.hp_delta).toBeLessThan(0);
  await page.getByRole("button", { name: "실행 시험", exact: true }).click();
  await expect(
    dialog.getByRole("status").filter({ hasText: "시나리오 실행 완료" }),
  ).toContainText("원본 0:1");
  await expect(
    dialog.getByText("실행할 원본 0:2", { exact: true }),
  ).toBeVisible();
  await capture(page, info, "composition-trial-old-version");
  const newResponse = page.waitForResponse(
    (r) => r.url().endsWith("/__content/run-composition"),
    { timeout: 120_000 },
  );
  await dialog
    .getByRole("button", { name: "시나리오 실행", exact: true })
    .click();
  const next = await (await newResponse).json();
  expect(next.status, JSON.stringify(next.failure)).toBe("completed");
  expect(next.identity.run_id).not.toBe(old.identity.run_id);
  expect(next.source_digest).not.toBe(old.source_digest);
  expect(next.latest_effect.hp_delta).toBeLessThan(old.latest_effect.hp_delta);
  await expect(
    dialog.getByRole("status").filter({ hasText: "시나리오 실행 완료" }),
  ).toContainText("원본 0:2");
  await capture(page, info, "composition-trial-completed");
  await inspectAccessibility(page, info, "composition-trial-completed");
  const folder = dialog.getByRole("textbox", {
    name: "실행 콘텐츠 폴더",
    exact: true,
  });
  await folder.fill(`${await folder.inputValue()}/absent`);
  await dialog
    .getByRole("button", { name: "시나리오 실행", exact: true })
    .click();
  await expect(dialog.getByRole("alert")).toContainText("실행 준비 거절");
  await expect(
    dialog.getByText(`실행 ${next.identity.run_id}`, { exact: true }),
  ).toBeVisible();
  await capture(page, info, "composition-trial-rejected");
  await info.attach("real-world-version-results", {
    body: JSON.stringify({ old, next }, null, 2),
    contentType: "application/json",
  });
});

test("Composition: personal shield shared lifetime edits and runs in World", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_SHIELD_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_SHIELD_TRIAL_FIXTURE must identify the exported shield World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
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
    .getByRole("button", { name: /· protection-time$/ })
    .click();
  const lifetime = page.getByRole("textbox", {
    name: "protection-time value.Milliseconds",
    exact: true,
  });
  await lifetime.fill("5000");
  await lifetime.press("Tab");
  await expect(source).toHaveValue(
    original.replace("Milliseconds: 6000", "Milliseconds: 5000"),
  );
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
  await capture(page, info, "composition-shield-lifetime");
  await inspectAccessibility(page, info, "composition-shield-lifetime");
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  await expect(source).toHaveValue(original);
  await page.getByRole("button", { name: "다시 실행", exact: true }).click();
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
  await page.getByRole("button", { name: "실행 시험", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "조합 실행 시험" });
  await dialog
    .getByLabel("실행 설정 파일", { exact: true })
    .setInputFiles(resolve(root, "trial.json"));
  await expect(dialog.getByText("trial.json", { exact: true })).toBeVisible();
  const response = page.waitForResponse(
    (r) => r.url().endsWith("/__content/run-composition"),
    { timeout: 120_000 },
  );
  await dialog
    .getByRole("button", { name: "시나리오 실행", exact: true })
    .click();
  const run = await (await response).json();
  expect(run.status, JSON.stringify(run.failure)).toBe("completed");
  expect(run.accepted_steps).toBe(47);
  // Shield consumption is asserted by the exported World test. The existing
  // result UI reports HP observations and does not label them as shield proof.
  await expect(dialog.getByText(/시나리오 실행 완료/)).toBeVisible();
  await capture(page, info, "composition-shield-world-completed");
  await inspectAccessibility(page, info, "composition-shield-world-completed");
  for (let index = 0; index < 8; index++) {
    await page.keyboard.press("Tab");
    expect(
      await dialog.evaluate((node) => node.contains(document.activeElement)),
    ).toBe(true);
  }
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("button", { name: "실행 시험", exact: true }),
  ).toBeFocused();
  await lifetime.fill("0");
  await lifetime.press("Tab");
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  const invalidDuration = page.getByRole("button", { name: /^Native ·/ });
  await expect(invalidDuration).toContainText("positive duration/reach");
  await invalidDuration.click();
  await expect(source).toBeFocused();
  expect(
    await source.evaluate((element: HTMLTextAreaElement) =>
      element.value.slice(element.selectionStart, element.selectionEnd),
    ),
  ).toContain("PersonalShieldGrant");
  await capture(page, info, "composition-shield-duration-error");
  await inspectAccessibility(page, info, "composition-shield-duration-error");
  await info.attach("personal-shield-world-result", {
    body: JSON.stringify(run, null, 2),
    contentType: "application/json",
  });
});

test("Composition: resource declaration edits drive real World healing", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_RESOURCE_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_RESOURCE_TRIAL_FIXTURE must identify the exported resource World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const expected: { cap: number; heals: number[] }[] = JSON.parse(
    await readFile(resolve(root, "expected.json"), "utf8"),
  );
  const name = `자원-검증-${info.project.name}-${Date.now()}.skill.yaml`;
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
    await page
      .getByRole("group", { name: "정의 선택" })
      .getByRole("button", { name: "자원 · fragment_meter", exact: true })
      .click();
    const cap = page.getByRole("textbox", {
      name: "fragment_meter cap",
      exact: true,
    });
    await cap.fill("6");
    await cap.press("Tab");
    const changed = original.replace(
      "initial: 0, cap: 15",
      "initial: 0, cap: 6",
    );
    await expect(source).toHaveValue(changed);
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
    await expect(page.locator(".react-flow__edge")).toHaveCount(2);
    await expect(
      page.locator(
        '.react-flow__edge[aria-label="attempt-charge → fragment_meter: Resource 3곳"]',
      ),
    ).toHaveCount(1);
    const controls = await page.getByTestId("rf__controls").boundingBox();
    expect(controls).not.toBeNull();
    for (const node of await page.locator(".react-flow__node").all()) {
      const bounds = await node.boundingBox();
      expect(bounds).not.toBeNull();
      expect(
        bounds!.x + bounds!.width <= controls!.x ||
          controls!.x + controls!.width <= bounds!.x ||
          bounds!.y + bounds!.height <= controls!.y ||
          controls!.y + controls!.height <= bounds!.y,
        "Graph controls must not cover a definition node after fit",
      ).toBe(true);
    }
    await capture(page, info, "composition-resource-cap");
    await inspectAccessibility(page, info, "composition-resource-cap");
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
    await page
      .getByRole("group", { name: "정의 선택" })
      .getByRole("button", { name: "자원 · fragment_meter", exact: true })
      .click();
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
    await page.getByRole("button", { name: "실행 시험", exact: true }).click();
    const dialog = page.getByRole("dialog", { name: "조합 실행 시험" });
    await expect(dialog.getByText(/작성한 스킬과 자원 정의/)).toBeVisible();
    await dialog
      .getByLabel("실행 설정 파일", { exact: true })
      .setInputFiles(resolve(root, "trial.json"));
    await expect(dialog.getByText("trial.json", { exact: true })).toBeVisible();
    const response = page.waitForResponse(
      (r) => r.url().endsWith("/__content/run-composition"),
      { timeout: 120_000 },
    );
    await dialog
      .getByRole("button", { name: "시나리오 실행", exact: true })
      .click();
    const run = await (await response).json();
    expect(run.status, JSON.stringify(run.failure)).toBe("completed");
    expect(run.accepted_steps).toBe(28);
    expect(run.latest_effect.hp_delta).toBe(
      expected.find((e) => e.cap === 6)!.heals[0],
    );
    expect(run.latest_effect.origin).not.toBeNull();
    await expect(dialog.getByText(/시나리오 실행 완료/)).toBeVisible();
    await capture(page, info, "composition-resource-world-completed");
    await inspectAccessibility(
      page,
      info,
      "composition-resource-world-completed",
    );
    for (let index = 0; index < 8; index++) {
      await page.keyboard.press("Tab");
      expect(
        await dialog.evaluate((node) => node.contains(document.activeElement)),
      ).toBe(true);
    }
    await page.keyboard.press("Escape");
    await expect(
      page.getByRole("button", { name: "실행 시험", exact: true }),
    ).toBeFocused();
    const initial = page.getByRole("textbox", {
      name: "fragment_meter initial",
      exact: true,
    });
    await initial.fill("7");
    await initial.press("Tab");
    await page
      .getByRole("button", { name: "Core 컴파일", exact: true })
      .click();
    const invalid = page.getByRole("button", { name: /^Native ·/ });
    await expect(invalid).toContainText("initial exceeds cap");
    await invalid.click();
    await expect(source).toBeFocused();
    expect(
      await source.evaluate((element: HTMLTextAreaElement) =>
        element.value.slice(element.selectionStart, element.selectionEnd),
      ),
    ).toBe("7");
    if (info.project.use.viewport?.width === 960) {
      expect(
        await source.evaluate((element) => element.scrollLeft),
      ).toBeGreaterThan(0);
    }
    await capture(page, info, "composition-resource-initial-error");
    await inspectAccessibility(
      page,
      info,
      "composition-resource-initial-error",
    );
    await info.attach("resource-world-result", {
      body: JSON.stringify(run, null, 2),
      contentType: "application/json",
    });
  } finally {
    await unlink(path).catch((error) => {
      if (error.code !== "ENOENT") throw error;
    });
  }
});
