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

test("graph readability: follow admission, edit local and connected values, undo and reload", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "노드", exact: true })
    .click();
  const doc: GraphDocument = JSON.parse(
    await readFile("content/knight-of-despair.node-graph.json", "utf8"),
  );
  doc.nodes[0].position.x += 40;
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "knight.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(doc)),
  });
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "교체", exact: true })
    .click();
  await expect.poll(() => model.state()?.doc).toEqual(doc);
  const focus = async (id: string) => {
    await page.getByLabel("노드 또는 구간으로 이동").selectOption(id);
    await expect(page.getByTestId(`rf__node-${id}`)).toBeInViewport();
  };
  const shot = async (name: string) => {
    await capture(page, info, name);
    await inspectLayout(page, info, name);
    await inspectAccessibility(page, info, name);
  };
  const qualify = doc.nodes.find((n) => n.kind === "skill-resonance-qualify")!;
  await focus(qualify.id);
  await shot("admission-before-layout");
  await page.getByRole("button", { name: "흐름 정렬", exact: true }).click();
  await expect.poll(() => model.state().undoCount).toBe(1);
  expect(model.state().doc.edges).toEqual(doc.edges);
  await page.getByRole("button", { name: "실행 중심", exact: true }).click();
  await focus(qualify.id);
  await shot("admission-after-layout");
  const qualificationCard = page.getByTestId(`rf__node-${qualify.id}`);
  const collapsed = await qualificationCard.boundingBox();
  const expansion = qualificationCard.getByRole("button", {
    name: /추가 입력 \d+개$/,
  });
  await expect(expansion).toHaveAttribute("aria-expanded", "false");
  await expansion.focus();
  await page.keyboard.press("Enter");
  await expect(expansion).toHaveAttribute("aria-expanded", "true");
  expect((await qualificationCard.boundingBox())!.height).toBeGreaterThan(
    collapsed!.height,
  );
  expect(model.state().undoCount).toBe(1);
  await page.keyboard.press("Enter");
  await expect(expansion).toHaveAttribute("aria-expanded", "false");
  const canvasBox = await page.locator(".react-flow").boundingBox();
  expect(collapsed!.y + collapsed!.height).toBeLessThanOrEqual(
    canvasBox!.y + canvasBox!.height,
  );

  const count = doc.nodes.find((n) => n.kind === "skill-count-quotient")!;
  await focus(count.id);
  const editCount = page
    .getByTestId(`rf__node-${count.id}`)
    .getByRole("button", { name: `${count.title} 나눌 수 수정`, exact: true });
  await editCount.focus();
  await page.keyboard.press("Enter");
  const dialog = page.getByRole("dialog");
  await dialog.getByLabel("나눌 수", { exact: true }).fill("99");
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
  await expect(editCount).toBeFocused();
  expect(
    model.state().doc.nodes.find((n) => n.id === count.id)?.values.divisor,
  ).toBe(4);
  await page.keyboard.press("Enter");
  await dialog.getByLabel("나눌 수", { exact: true }).fill("5");
  await dialog.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(dialog).toHaveCount(0);
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === count.id)?.values.divisor,
    )
    .toBe(5);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === count.id)?.values.divisor,
    )
    .toBe(4);
  const shield = doc.nodes.find(
    (n) => n.kind === "skill-relation-emergency-shield",
  )!;
  const durationEdge = doc.edges.find(
    (e) => e.target.node === shield.id && e.target.port === "duration",
  )!;
  await focus(shield.id);
  const durationButton = page
    .getByTestId(`rf__node-${shield.id}`)
    .getByRole("button", { name: /연결값 수정$/ });
  await durationButton.click();
  const beforeInvalid = model.state();
  await dialog.getByRole("textbox").fill("12.0001");
  await dialog.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(dialog.getByRole("alert")).toBeVisible();
  expect(model.state()).toEqual(beforeInvalid);
  await dialog.getByRole("textbox").fill("12");
  await dialog.getByRole("button", { name: "값 적용", exact: true }).click();
  await expect(dialog).toHaveCount(0);
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === durationEdge.source.node)
          ?.values.value,
    )
    .toBe(12);
  expect(model.state().doc.edges).toEqual(doc.edges);
  await shot("connected-duration-edited");
  const pending = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const download = await pending;
  const saved = JSON.parse(await readFile((await download.path())!, "utf8"));
  expect(saved).toEqual(model.state().doc);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "reloaded.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(saved)),
  });
  await expect.poll(() => model.state().doc).toEqual(saved);
  await focus(shield.id);
  await expect(
    page
      .getByTestId(`rf__node-${shield.id}`)
      .getByRole("button", { name: /연결값 수정$/ }),
  ).toContainText("12초");
});
