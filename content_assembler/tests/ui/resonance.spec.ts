import { readFile } from "node:fs/promises";
import type { Page, TestInfo } from "@playwright/test";
import type { GraphDocument } from "../../src/nodes/types";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectLayout,
  inspectAccessibility,
} from "./fixtures";
import { observeNodes } from "./node-observer";

async function openExample(page: Page, info: TestInfo, name: string) {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "노드", exact: true })
    .click();
  await page.getByRole("button", { name: "그래프 목록", exact: true }).click();
  await page
    .getByRole("dialog")
    .locator("summary")
    .filter({ hasText: "공명 소비 예제" })
    .click();
  await page
    .getByRole("dialog", { name: "그래프 목록", exact: true })
    .getByRole("button", {
      name: `resonance-${name}.node-graph.json`,
      exact: true,
    })
    .click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "교체", exact: true })
    .click();
  await expect
    .poll(() => model.state()?.doc.name)
    .toContain("검증 예제 · 공명");
  await expect.poll(() => model.state()?.diagnostics).toEqual([]);
  return model;
}
async function focus(page: Page, doc: GraphDocument, title: string) {
  const n = doc.nodes.find((n) => n.title === title)!;
  await page.getByLabel("노드 또는 구간으로 이동").selectOption(n.id);
  await expect(page.getByTestId(`rf__node-${n.id}`)).toBeInViewport();
  return n;
}
async function inspect(page: Page, info: TestInfo, name: string) {
  await capture(page, info, name);
  await inspectLayout(page, info, name);
  await inspectAccessibility(page, info, name);
}
async function roundtrip(page: Page) {
  const download = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "JSON 내보내기", exact: true })
    .click();
  const file = await download;
  const text = await readFile((await file.path())!, "utf8");
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "resonance-roundtrip.node-graph.json",
    mimeType: "application/json",
    buffer: Buffer.from(text),
  });
  return JSON.parse(text) as GraphDocument;
}

test("resonance modes expose relevant fields, retain unset values and undo through a real file roundtrip", async ({
  page,
}, info) => {
  const model = await openExample(page, info, "all");
  const cost = await focus(page, model.state().doc, "첫 지불 비용");
  const mode = page.getByLabel("소비 방식", { exact: true });
  const percent = page.getByLabel("소비 비율 (%)", { exact: true });
  await expect(mode).toHaveValue("all");
  await expect(percent).toHaveCount(0);
  await mode.selectOption("current-percent");
  await expect(percent).toHaveValue("");
  await percent.fill("25");
  await expect(page.getByLabel("비율 반올림", { exact: true })).toHaveCount(0);
  await expect(page.getByLabel("노드 속성")).toContainText("항상 올림");
  await page.getByLabel("비율 최소 비용", { exact: true }).fill("0");
  const before = model.state().doc;
  await percent.fill("25.5");
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await expect(
    page.getByText("소비 비율 (%): 소수 없이 안전한 정수를 입력하세요.", {
      exact: true,
    }),
  ).toBeVisible();
  expect(model.state().doc).toEqual(before);
  await percent.fill("25");
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await expect.poll(() => model.state().diagnostics).toEqual([]);
  await focus(page, model.state().doc, "첫 지불 비용");
  await inspect(page, info, "resonance-percent");
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect(mode).toHaveValue("all");
  await expect(percent).toHaveCount(0);
  await page
    .getByRole("button", { name: "노드 다시 실행", exact: true })
    .click();
  await expect(percent).toHaveValue("25");
  await percent.fill("");
  // Export also commits the pending field, preserving missing instead of zero.
  const saved = await roundtrip(page);
  expect(saved.nodes.find((n) => n.id === cost.id)?.values.percent).toBe("");
  await expect.poll(() => model.state().doc).toEqual(saved);
  await focus(page, saved, "첫 지불 비용");
  await expect(percent).toHaveValue("");
  await expect
    .poll(() =>
      model.state().diagnostics.some((d) => d.code === "skill-value-unset"),
    )
    .toBe(true);
  await mode.selectOption("fixed");
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await expect(percent).toHaveCount(0);
  await expect
    .poll(() => model.state().diagnostics.map((d) => d.code))
    .toEqual(["resonance-input-required"]);
  await page.getByRole("button", { name: /^진단 \d+/ }).click();
  await page
    .getByRole("region", { name: "노드 작성 진단" })
    .getByRole("button", {
      name: "첫 지불 비용 · 선택한 방식에 필요한 입력이 연결되지 않았습니다.",
      exact: true,
    })
    .click();
  await inspect(page, info, "resonance-missing-fixed-input");
});

test("periodic payment exposes lifetime, insufficient-cost policy and validated time with undo", async ({
  page,
}, info) => {
  const model = await openExample(page, info, "periodic");
  await focus(page, model.state().doc, "유지 비용 주기");
  await expect(page.getByLabel("최대 지불 회수", { exact: true })).toHaveValue(
    "3",
  );
  await expect(page.getByLabel("노드 속성")).toContainText(
    "사용 종료·취소 시 구독을 정리",
  );
  await inspect(page, info, "resonance-periodic");
  const interval = await focus(page, model.state().doc, "유지 비용 간격");
  const seconds = page.getByLabel("시간 (초)", { exact: true });
  const before = model.state().doc;
  await seconds.fill("0.0005");
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await expect(
    page.getByText(
      "시간 (초): 0 이상의 시간을 0.001초 단위로 입력하세요. 밀리초 값은 안전한 정수 범위여야 합니다.",
      { exact: true },
    ),
  ).toBeVisible();
  expect(model.state().doc).toEqual(before);
  await seconds.fill("1.001");
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await expect
    .poll(
      () =>
        model.state().doc.nodes.find((n) => n.id === interval.id)?.values.value,
    )
    .toBe(1.001);
  await inspect(page, info, "resonance-millisecond-time");
  await page.getByLabel("시간 (초)", { exact: true }).fill("0");
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await expect
    .poll(() => model.state().diagnostics.map((d) => d.code))
    .toEqual(["resonance-period"]);
  await page
    .getByRole("button", { name: "노드 실행 취소", exact: true })
    .click();
  await expect.poll(() => model.state().diagnostics).toEqual([]);
  const pay = await focus(page, model.state().doc, "단계의 공명 지불");
  const policy = page.getByLabel("비용 부족 처리", { exact: true });
  await expect(policy).toHaveValue("stop-use");
  await policy.selectOption("skip-step");
  await page.getByRole("button", { name: "변경 적용", exact: true }).click();
  await focus(page, model.state().doc, "단계의 공명 지불");
  await inspect(page, info, "resonance-payment-policy");
  const saved = await roundtrip(page);
  expect(saved.nodes.find((n) => n.id === interval.id)?.values.value).toBe(
    1.001,
  );
  expect(saved.nodes.find((n) => n.id === pay.id)?.values.insufficient).toBe(
    "skip-step",
  );
  await expect.poll(() => model.state().doc).toEqual(saved);
  await expect.poll(() => model.state().diagnostics).toEqual([]);
  await focus(page, saved, "후속 지불 결과");
  await expect(page.getByLabel("노드 속성")).toContainText(
    "실제 소비량과 지불 직후 잔량",
  );
  await inspect(page, info, "resonance-receipt");
});
