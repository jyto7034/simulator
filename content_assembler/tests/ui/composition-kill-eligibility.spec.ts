import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
} from "./fixtures";

test("Composition: kill eligibility edits authority and rank and runs the World source", async ({
  page,
}, info) => {
  test.setTimeout(180_000);
  const root = process.env.CONTENT_ASSEMBLER_KILL_ELIGIBILITY_TRIAL_FIXTURE;
  if (!root)
    throw new Error(
      "CONTENT_ASSEMBLER_KILL_ELIGIBILITY_TRIAL_FIXTURE must identify the exported World fixture",
    );
  const original = await readFile(resolve(root, "source.skill.yaml"), "utf8");
  const config = JSON.parse(
    await readFile(resolve(root, "trial.json"), "utf8"),
  );
  const expected: {
    grant: number;
    balance: number;
    threat: string;
    rewards: { heals: number[] }[];
  }[] = JSON.parse(await readFile(resolve(root, "expected.json"), "utf8"));
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
    .getByRole("button", { name: /· transformation$/ })
    .click();
  const prefix =
    "transformation value.Effects.0.op.Sustained.definition.State.program.handlers.0";
  const eligibility = page.getByRole("textbox", {
    name: `${prefix}.event.BasicAttackKill.eligibility`,
    exact: true,
  });
  await eligibility.fill("AnyHostile");
  await eligibility.press("Tab");
  await expect(source).toHaveValue(
    original.replace(
      "eligibility: EncounterCombatant",
      'eligibility: "AnyHostile"',
    ),
  );
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  await expect(source).toHaveValue(original);
  const threat = page.getByRole("textbox", {
    name: `${prefix}.event.BasicAttackKill.threat`,
    exact: true,
  });
  await threat.fill("elite");
  await threat.press("Tab");
  await expect(source).toHaveValue(
    original.replace("threat: normal", 'threat: "elite"'),
  );
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  await expect(source).toHaveValue(original);
  const grant = page.getByRole("textbox", {
    name: `${prefix}.body.1.Resources.operations.0.operation.Grant`,
    exact: true,
  });
  await grant.fill("8");
  await grant.press("Tab");
  const granted = original.replace("Grant: 2", "Grant: 8");
  await expect(source).toHaveValue(granted);
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  await expect(source).toHaveValue(original);
  await page.getByRole("button", { name: "다시 실행", exact: true }).click();
  await expect(source).toHaveValue(granted);
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  await expect(page.getByText(/현재 원본 컴파일 완료/)).toBeVisible();
  await threat.scrollIntoViewIfNeeded();
  await capture(page, info, "composition-kill-eligibility-operation");
  await inspectAccessibility(
    page,
    info,
    "composition-kill-eligibility-operation",
  );
  const amount = page.getByRole("textbox", {
    name: `${prefix}.body.2.HealOwner.amount.root.Const.value`,
    exact: true,
  });
  await amount.fill("5");
  await amount.press("Tab");
  const changed = granted.replace(
    "value: 7, unit: Magnitude",
    "value: 5, unit: Magnitude",
  );
  await expect(source).toHaveValue(changed);
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  await expect(source).toHaveValue(granted);
  await page.getByRole("button", { name: "다시 실행", exact: true }).click();
  await expect(source).toHaveValue(changed);
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
  expect(run.accepted_steps).toBe(
    config.scenario.preparation_steps.length + config.scenario.steps.length,
  );
  expect(run.latest_effect.hp_delta).toBe(
    expected.find((e) => e.grant === 8)!.rewards[0].heals[0],
  );
  expect(run.latest_effect.origin).not.toBeNull();
  await expect(dialog.getByText(/시나리오 실행 완료/)).toBeVisible();
  await capture(page, info, "composition-kill-eligibility-world-completed");
  await inspectAccessibility(
    page,
    info,
    "composition-kill-eligibility-world-completed",
  );
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
  await source.fill(changed.replace("threat: normal", "threat: PendingRank"));
  await page.getByRole("button", { name: "Core 컴파일", exact: true }).click();
  const diagnostic = page
    .getByRole("group", { name: "조합 진단" })
    .getByRole("button")
    .first();
  await expect(diagnostic).toContainText(/PendingRank/);
  await diagnostic.click();
  await expect(source).toBeFocused();
  expect(
    await source.evaluate((node: HTMLTextAreaElement) =>
      node.value.slice(node.selectionStart, node.selectionEnd),
    ),
  ).toBe("PendingRank");
  await capture(page, info, "composition-kill-eligibility-event-error");
  await inspectAccessibility(
    page,
    info,
    "composition-kill-eligibility-event-error",
  );
  await info.attach("kill-eligibility-world-result", {
    body: JSON.stringify(run, null, 2),
    contentType: "application/json",
  });
});
