import { readFile } from "node:fs/promises";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectLayout,
  inspectAccessibility,
} from "./fixtures";
import { createSkill, makeBlock } from "../../src/skill/model";
import { exampleSkill } from "../../src/skill/examples";
import { parseSkill } from "../../src/skill/validation";
import type { Page } from "@playwright/test";
async function skill(page: Page) {
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "스킬", exact: true })
    .click();
  await expect(
    page.getByRole("heading", { name: "스킬 조립기", exact: true }),
  ).toBeVisible();
}
async function ready(page: Page) {
  await expect(page.getByTestId("skill-graph")).toHaveAttribute(
    "data-layout",
    "ready",
  );
}
async function structure(page: Page) {
  await page.getByRole("button", { name: "스킬 구성", exact: true }).click();
}
async function accept(page: Page) {
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "계속", exact: true })
    .click();
}
async function fill(page: Page, label: string, value: string) {
  const input = page.getByLabel(label, { exact: true });
  await input.fill(value);
  await input.press("Enter");
}

test("skill workspace examples, edits, nesting and independent map history", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await page.getByRole("button", { name: "맵 정보", exact: true }).click();
  await fill(page, "맵 이름", "보존할 전투 맵");
  await skill(page);
  await ready(page);
  await expect(page.locator(".skill-graph-node.event")).toHaveCount(3);
  await expect(
    page.getByRole("button", { name: "2. 유한 반복 펼치기", exact: true }),
  ).toBeVisible();
  const node = page.locator(".skill-graph-node.event").first();
  const beforeZoom = await node.evaluate(
    (el) => el.getBoundingClientRect().width,
  );
  await page.getByRole("button", { name: "그래프 확대", exact: true }).click();
  await expect
    .poll(() => node.evaluate((el) => el.getBoundingClientRect().width))
    .toBeGreaterThan(beforeZoom);
  await page.getByRole("button", { name: "전체 보기", exact: true }).click();
  await page.getByRole("button", { name: "미니맵", exact: true }).click();
  await expect(page.locator(".react-flow__minimap")).toBeVisible();
  await page.getByRole("button", { name: "미니맵", exact: true }).click();
  await expect(page.locator(".react-flow__minimap")).toHaveCount(0);
  await capture(page, info, "skill-butterfly");
  await inspectLayout(page, info, "skill-butterfly");
  await inspectAccessibility(page, info, "skill-butterfly");
  await page
    .getByRole("button", { name: "1. 대상 조회 선택", exact: true })
    .click();
  await fill(page, "선정 조건", "스택이 1 이상인 타일");
  await page
    .getByRole("button", { name: "2. 유한 반복 선택", exact: true })
    .click();
  await page
    .getByRole("button", { name: "안쪽 절차에 추가", exact: true })
    .click();
  await capture(page, info, "skill-block-picker");
  await page
    .getByRole("button", { name: "기다리기 블록 추가", exact: true })
    .click();
  await fill(page, "대기 시간 (초 또는 식)", "0.5");
  await expect(
    page.getByRole("button", { name: "4. 기다리기 선택", exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "블록 위로 이동" }).click();
  await expect(
    page.getByRole("button", { name: "3. 기다리기 선택", exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "블록 복제" }).click();
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  // Undo removes the selected clone; reopen the now-unselected group to inspect its contents.
  await page
    .getByRole("button", { name: "2. 유한 반복 펼치기", exact: true })
    .click();
  await expect(
    page.locator(".skill-node-content").filter({ hasText: "기다리기" }),
  ).toHaveCount(1);
  await page
    .getByRole("button", { name: "3. 기다리기 선택", exact: true })
    .click();
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "맵", exact: true })
    .click();
  await expect(page.getByLabel("맵 이름", { exact: true })).toHaveValue(
    "보존할 전투 맵",
  );
  await page.getByRole("button", { name: "실행 취소", exact: true }).click();
  await expect(page.getByLabel("맵 이름", { exact: true })).toHaveValue(
    "새 전투 맵",
  );
  await skill(page);
  await expect(
    page.getByLabel("대기 시간 (초 또는 식)", { exact: true }),
  ).toHaveValue("0.5");
  await page.getByRole("button", { name: "예제", exact: true }).click();
  await page.getByRole("menuitem", { name: "두건 예제" }).click();
  await accept(page);
  await ready(page);
  await expect(page.locator(".skill-graph-node.event")).toHaveCount(4);
  await capture(page, info, "skill-hood");
  await inspectAccessibility(page, info, "skill-hood");
  await page
    .getByRole("button", { name: "2. 조건 분기 펼치기", exact: true })
    .click();
  await ready(page);
  await expect(
    page.locator(".react-flow__edge-text").filter({ hasText: "아니면" }),
  ).toHaveCount(1);
  await capture(page, info, "skill-hood-expanded");
  await page
    .getByRole("button", { name: "2. 조건 분기 선택", exact: true })
    .click();
  await page
    .getByRole("button", { name: "아니면 절차에 추가", exact: true })
    .click();
  await page.getByLabel("블록 검색").fill("기다리기");
  await page
    .getByRole("button", { name: "기다리기 블록 추가", exact: true })
    .click();
  await expect(
    page.getByLabel("대기 시간 (초 또는 식)", { exact: true }),
  ).toHaveValue("");
});

test("skill save reopen, diagnostics and malformed file preserve the current design", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await skill(page);
  await structure(page);
  await page.getByRole("button", { name: "기본 설정", exact: true }).click();
  await fill(
    page,
    "스킬 이름",
    "아주 긴 한글 스킬 이름 · 여러 타일의 나비를 모아 폭발시키는 개화형 설계 문서",
  );
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "저장", exact: true }).click();
  const result = await download;
  const json = await readFile((await result.path())!, "utf8");
  const doc = parseSkill(json);
  expect(doc.states).toHaveLength(2);
  expect(doc.routines[2].blocks[1].children).toHaveLength(3);
  await page
    .getByRole("button", { name: "다운로드 확인", exact: true })
    .click();
  await expect(page.getByTestId("skill-dirty")).toContainText("파일 확인됨");
  await ready(page);
  await page
    .getByRole("button", { name: "2. 유한 반복 펼치기", exact: true })
    .click();
  await ready(page);
  await page
    .getByRole("button", { name: "2. 유한 반복 접기", exact: true })
    .click();
  await expect(page.getByTestId("skill-dirty")).toContainText("파일 확인됨");
  await structure(page);
  await page.getByRole("button", { name: "기본 설정", exact: true }).click();
  await page.getByLabel("스킬 JSON 열기").setInputFiles({
    name: "bad.json",
    mimeType: "application/json",
    buffer: Buffer.from('{"format":"other"}'),
  });
  await expect(
    page.getByRole("status", { name: "스킬 문서 상태" }),
  ).toContainText("열기 실패");
  await expect(page.getByLabel("스킬 이름", { exact: true })).toHaveValue(
    doc.name,
  );
  await page.getByRole("button", { name: "새로 만들기", exact: true }).click();
  await expect(page.getByLabel("스킬 이름", { exact: true })).toHaveValue(
    "새 스킬",
  );
  await page.getByLabel("스킬 JSON 열기").setInputFiles({
    name: "saved.json",
    mimeType: "application/json",
    buffer: Buffer.from(json),
  });
  await accept(page);
  await expect(page.getByLabel("스킬 이름", { exact: true })).toHaveValue(
    doc.name,
  );
  await page.getByRole("button", { name: /^작성 진단/ }).click();
  await page
    .getByRole("button", {
      name: "피해 적용: 피해량 또는 식 미지정",
      exact: true,
    })
    .click();
  await expect(
    page.getByLabel("피해량 또는 식", { exact: true }),
  ).toBeFocused();
  await expect
    .poll(() =>
      page
        .locator('.skill-graph-node[data-selected="true"]')
        .evaluate((node) => {
          const n = node.getBoundingClientRect();
          const c = node
            .closest(".skill-graph-canvas")!
            .getBoundingClientRect();
          // Compare rendered pixel bounds: IntersectionObserver can report 0.99999994 at fractional zoom.
          return (
            Math.round(n.left) >= Math.round(c.left) &&
            Math.round(n.right) <= Math.round(c.right) &&
            Math.round(n.top) >= Math.round(c.top) &&
            Math.round(n.bottom) <= Math.round(c.bottom)
          );
        }),
    )
    .toBe(true);
  await capture(page, info, "skill-long-name-diagnostic");
  await inspectLayout(page, info, "skill-long-name");
});

test("skill empty and large designs remain editable with keyboard dialog cancellation", async ({
  page,
}, info) => {
  await openWorkspace(page, info);
  await skill(page);
  const doc = createSkill();
  doc.routines = [];
  await page.getByLabel("스킬 JSON 열기").setInputFiles({
    name: "empty.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(doc)),
  });
  await accept(page);
  await ready(page);
  await capture(page, info, "skill-empty");
  await page.getByRole("button", { name: "동작 추가", exact: true }).click();
  await page
    .getByRole("button", { name: "선택 항목 삭제", exact: true })
    .click();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toBeHidden();
  await expect(
    page.getByRole("button", { name: "선택 항목 삭제", exact: true }),
  ).toBeFocused();
  const large = exampleSkill("hood");
  large.routines[2].blocks = Array.from({ length: 60 }, () =>
    makeBlock("damage", {
      target: "현재 타일의 적",
      amount: "공격력 × 계수",
      damageType: "미정",
    }),
  );
  await page.getByLabel("스킬 JSON 열기").setInputFiles({
    name: "large.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(large)),
  });
  await accept(page);
  await ready(page);
  await page.getByRole("button", { name: "노드 찾기", exact: true }).click();
  await page.getByLabel("노드 검색").fill("60. 피해 적용");
  await page.locator(".skill-node-results").getByRole("button").click();
  await expect(
    page.getByRole("button", { name: "60. 피해 적용 선택", exact: true }),
  ).toBeInViewport({ ratio: 1 });
  await capture(page, info, "skill-many-blocks");
  await inspectLayout(page, info, "skill-many-blocks");
  await inspectAccessibility(page, info, "skill-many-blocks");
});

test("skill layout failure can retry without changing the design", async ({
  page,
}, info) => {
  // Explicit browser test boundary: fail ELK workers while preserving the real graph/model code.
  await page.addInitScript(() => {
    const OriginalWorker = window.Worker;
    Object.assign(window, { failSkillLayout: true });
    window.Worker = class extends OriginalWorker {
      constructor(url: string | URL, options?: WorkerOptions) {
        super(url, options);
        if (
          String(url).includes("elk-worker") &&
          Reflect.get(window, "failSkillLayout")
        )
          queueMicrotask(() =>
            this.dispatchEvent(
              new ErrorEvent("error", { message: "검사용 배치 실패" }),
            ),
          );
      }
    };
  });
  await openWorkspace(page, info);
  await skill(page);
  await expect(page.getByTestId("skill-graph")).toHaveAttribute(
    "data-layout",
    "error",
  );
  await expect(
    page.getByRole("status").filter({ hasText: "자동 배치 실패" }),
  ).toContainText("검사용 배치 실패");
  await capture(page, info, "skill-layout-error");
  await page.evaluate(() => Reflect.set(window, "failSkillLayout", false));
  await page.getByRole("button", { name: "다시 배치", exact: true }).click();
  await ready(page);
  await expect(page.locator(".skill-graph-node.event")).toHaveCount(3);
  await expect(
    page.getByRole("button", { name: "실행 취소", exact: true }),
  ).toBeDisabled();
});
