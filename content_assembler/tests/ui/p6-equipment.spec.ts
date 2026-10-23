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

for (const [file, kind, field, label] of [
  ["snow-queen", "skill-modifier-duration", "percent", "시간 변화율 (%)"],
  ["knight-of-despair", "skill-modifier-multihit", "step", "적중당 증가 (%p)"],
  ["queen-of-hatred", "skill-modifier-healing", "percent", "증가율 (%)"],
] as const) {
  test(`P6 relic ${file}: conditional edit, variant, undo and save/reopen`, async ({
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
    const rule = doc.nodes.find((n) => n.kind === kind)!;
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(rule.id);
    await page
      .getByTestId(`rf__node-${rule.id}`)
      .getByRole("button", {
        name: `${rule.title} ${label} 수정`,
        exact: true,
      })
      .click();
    const dialog = page.getByRole("dialog");
    await dialog.getByLabel(label, { exact: true }).fill("12");
    await dialog.getByRole("button", { name: "값 적용", exact: true }).click();
    await expect
      .poll(
        () =>
          model.state().doc.nodes.find((n) => n.id === rule.id)?.values[field],
      )
      .toBe(12);
    await page
      .getByRole("button", { name: "노드 실행 취소", exact: true })
      .click();
    await expect.poll(() => model.state().doc).toEqual(doc);
    const edge = doc.edges.find(
      (e) => e.target.node === rule.id && e.target.port === "condition",
    )!;
    const condition = doc.nodes.find((n) => n.id === edge.source.node)!;
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(condition.id);
    await page
      .getByTestId(`rf__node-${condition.id}`)
      .getByRole("button", {
        name: `${condition.title} 장착 변형 수정`,
        exact: true,
      })
      .click();
    await page
      .getByRole("dialog")
      .getByLabel("장착 변형", { exact: true })
      .selectOption("corrupt");
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "값 적용", exact: true })
      .click();
    await expect
      .poll(
        () =>
          model.state().doc.nodes.find((n) => n.id === condition.id)?.values
            .variant,
      )
      .toBe("corrupt");
    await page
      .getByRole("button", { name: "노드 실행 취소", exact: true })
      .click();
    await expect.poll(() => model.state().doc).toEqual(doc);
    await page.getByRole("button", { name: "실행 중심", exact: true }).click();
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(rule.id);
    await capture(page, info, "condition-and-modifier");
    await inspectLayout(page, info, "condition-and-modifier");
    await inspectAccessibility(page, info, "condition-and-modifier");
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
