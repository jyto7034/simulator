import { readFile } from "node:fs/promises";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectLayout,
  inspectAccessibility,
} from "./fixtures";
import { diagnoseSkill, parseSkill } from "../../src/skill/validation";

test("hood review preserves both aiming paths, projectile effects and undecided values through file roundtrip", async ({
  page,
}, info) => {
  const source = await readFile(
    new URL("../../docs/examples/hood-review.skill.json", import.meta.url),
    "utf8",
  );
  const review = parseSkill(source);
  await openWorkspace(page, info);
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "스킬", exact: true })
    .click();
  await page.getByLabel("스킬 JSON 열기").setInputFiles({
    name: "hood-review.skill.json",
    mimeType: "application/json",
    buffer: Buffer.from(source),
  });
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "계속", exact: true })
    .click();
  const ready = () =>
    expect(page.getByTestId("skill-graph")).toHaveAttribute(
      "data-layout",
      "ready",
    );
  const node = (id: string) =>
    page.locator(`.skill-graph-node[data-item-id="hood.review.${id}"]`);
  const select = async (id: string) => {
    await node(id).locator(".skill-node-content").click();
  };
  await ready();
  await expect(page.locator(".skill-graph-node.event")).toHaveCount(4);
  await select("mode");
  await expect(page.getByLabel("선택할 내용", { exact: true })).toHaveValue(
    /정밀.*집중.*재사용/,
  );
  await capture(page, info, "hood-review-overview");
  await inspectLayout(page, info, "hood-review-overview");
  await inspectAccessibility(page, info, "hood-review-overview");

  await node("aim")
    .getByRole("button", { name: "3. 조건 분기 펼치기", exact: true })
    .click();
  await ready();
  await select("precise");
  await expect(page.getByLabel("선택할 내용", { exact: true })).toHaveValue(
    /클릭은 궤적 기록만/,
  );
  await select("target");
  await expect(page.getByLabel("결과 이름", { exact: true })).toHaveValue(
    "집중 목표 타일",
  );
  await select("directions");
  await expect(page.getByLabel("계산식", { exact: true })).toHaveValue(
    /각 원점에서 집중 목표 타일 중심/,
  );
  await page
    .getByRole("button", { name: "선택 동작 보기", exact: true })
    .click();
  await capture(page, info, "hood-review-preparation");

  await node("hit-kind")
    .getByRole("button", { name: "1. 조건 분기 펼치기", exact: true })
    .click();
  await ready();
  await select("bind");
  await expect(
    page.getByLabel("지속 시간 (초 또는 식)", { exact: true }),
  ).toHaveValue("2");
  await select("normal-damage");
  await expect(page.getByLabel("피해량 또는 식", { exact: true })).toHaveValue(
    "",
  );
  await page
    .getByRole("button", { name: "선택 동작 보기", exact: true })
    .click();
  await capture(page, info, "hood-review-impact");
  await inspectLayout(page, info, "hood-review-impact");

  await expect(page.getByTestId("skill-dirty")).toContainText("파일 확인됨");
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "저장", exact: true }).click();
  const result = await download;
  expect(parseSkill(await readFile((await result.path())!, "utf8"))).toEqual(
    review,
  );
  await info.attach("review-document-diagnostics", {
    body: JSON.stringify(diagnoseSkill(review), null, 2),
    contentType: "application/json",
  });
});
