import { readFile } from "node:fs/promises";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectLayout,
  inspectAccessibility,
} from "./fixtures";
import { observeNodes } from "./node-observer";
import type { GraphDocument } from "../../src/nodes/types";

for (const file of [
  "melting-love",
  "funeral-of-dying-butterflies",
  "funeral-of-dying-butterflies-bloom",
  "funeral-of-dying-butterflies-orb",
]) {
  test(`P5 ${file}: find effect, change collision policy, undo and save/reopen`, async ({
    page,
  }, info) => {
    const model = await observeNodes(page);
    await openWorkspace(page, info);
    await page
      .getByRole("navigation", { name: "작업 공간" })
      .getByRole("button", { name: "노드", exact: true })
      .click();
    const doc: GraphDocument = JSON.parse(
      await readFile(`content/${file}.node-graph.json`, "utf8"),
    );
    await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
      name: `${file}.json`,
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify(doc)),
    });
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "교체", exact: true })
      .click();
    await expect.poll(() => model.state()?.doc).toEqual(doc);
    const rule = doc.nodes.find((n) => n.kind === "skill-explosion-rule")!;
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(rule.id);
    await page
      .getByTestId(`rf__node-${rule.id}`)
      .getByRole("button", {
        name: `${rule.title} 동일 대상 피해 수정`,
        exact: true,
      })
      .click();
    const dialog = page.getByRole("dialog");
    await dialog
      .getByLabel("동일 대상 피해", { exact: true })
      .selectOption("blast");
    await dialog.getByRole("button", { name: "값 적용", exact: true }).click();
    await expect
      .poll(
        () =>
          model.state().doc.nodes.find((n) => n.id === rule.id)?.values.dedup,
      )
      .toBe("blast");
    await page
      .getByRole("button", { name: "노드 실행 취소", exact: true })
      .click();
    await expect.poll(() => model.state().doc).toEqual(doc);
    const effect = doc.nodes.find((n) =>
      [
        "skill-mark-burst-plan",
        "skill-path-burst-plan",
        "skill-stack-burst-plan",
      ].includes(n.kind),
    )!;
    await page.getByRole("button", { name: "실행 중심", exact: true }).click();
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(effect.id);
    await capture(page, info, "effect-and-authored-inputs");
    await inspectLayout(page, info, "effect-and-authored-inputs");
    await inspectAccessibility(page, info, "effect-and-authored-inputs");
    const pending = page.waitForEvent("download");
    await page
      .getByRole("button", { name: "JSON 내보내기", exact: true })
      .click();
    const download = await pending;
    const saved = JSON.parse(await readFile((await download.path())!, "utf8"));
    expect(saved).toEqual(doc);
    await page
      .getByRole("button", { name: "다운로드 확인", exact: true })
      .click();
    await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
      name: "reopened.json",
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify(saved)),
    });
    await expect.poll(() => model.state().doc).toEqual(saved);
    expect(model.state().dirty).toBe(false);
  });
}
