import { readFile, writeFile, unlink } from "node:fs/promises";
import { join } from "node:path";
import { randomUUID } from "node:crypto";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
  inspectLayout,
} from "./fixtures";
import { observeNodes, readNodeState } from "./node-observer";

test("open starts in the project content folder and disk save roundtrips without download confirmation", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  await openWorkspace(page, info);
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "노드", exact: true })
    .click();
  await expect(page.getByTestId("node-canvas")).toBeVisible();
  await expect.poll(() => model.state()?.doc.nodes.length).toBe(0);
  await expect(
    page.getByText("첫 노드를 추가하세요", { exact: true }),
  ).toBeVisible();
  const name = `화면 검사-${randomUUID()}.node-graph.json`;
  const path = join(process.cwd(), "content", name);
  try {
    await page
      .getByRole("button", { name: "그래프 목록", exact: true })
      .click();
    let dialog = page.getByRole("dialog", { name: "그래프 목록" });
    await expect(dialog.getByLabel("콘텐츠 폴더 경로")).toContainText(
      "content",
    );
    await expect(
      dialog.getByRole("button", {
        name: "hello-goodbye.node-graph.json",
        exact: true,
      }),
    ).toBeVisible();
    await capture(page, info, "content-open");
    await inspectAccessibility(page, info, "content-open");
    await expect(dialog).toBeInViewport({ ratio: 1 });
    await page.keyboard.press("Escape");
    await expect(
      page.getByRole("button", { name: "그래프 목록", exact: true }),
    ).toBeFocused();
    await inspectLayout(page, info, "content-workspace");
    await page.keyboard.press("Control+o");
    await dialog
      .getByRole("button", {
        name: "hello-goodbye.node-graph.json",
        exact: true,
      })
      .click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "교체", exact: true })
      .click();
    await expect.poll(() => model.state()?.doc.nodes.length).toBe(29);
    await expect.poll(() => model.state().dirty).toBe(false);
    await page
      .getByLabel("그래프 이름", { exact: true })
      .fill("콘텐츠 폴더 · 한글 저장과 다시 열기");
    await page.getByRole("button", { name: "변경 적용", exact: true }).click();
    const saved = model.state().doc;
    await page.getByRole("button", { name: "저장", exact: true }).click();
    dialog = page.getByRole("dialog", { name: "콘텐츠 폴더에 저장" });
    await dialog.getByLabel("파일 이름", { exact: true }).fill(name);
    await capture(page, info, "content-save");
    await inspectAccessibility(page, info, "content-save");
    await dialog
      .getByRole("button", { name: "파일 저장", exact: true })
      .click();
    await expect(dialog).toHaveCount(0);
    await expect.poll(() => model.state().dirty).toBe(false);
    expect(JSON.parse(await readFile(path, "utf8"))).toEqual(saved);
    await expect(
      page.getByRole("button", { name: "다운로드 확인" }),
    ).toHaveCount(0);
    await page.getByRole("button", { name: "새 그래프", exact: true }).click();
    await expect.poll(() => model.state().doc.nodes.length).toBe(0);
    expect(model.state().doc.id).not.toBe(saved.id);
    expect(model.state().canUndo).toBe(false);
    await expect(page.getByRole("menu")).toHaveCount(0);
    await capture(page, info, "graph-new-empty");
    await inspectLayout(page, info, "graph-new-empty");
    await inspectAccessibility(page, info, "graph-new-empty");
    await page
      .getByRole("button", { name: "그래프 목록", exact: true })
      .click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name, exact: true })
      .click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "교체", exact: true })
      .click();
    await expect.poll(() => model.state().canUndo).toBe(false);
    expect(model.state().doc).toEqual(saved);
    // A write by another editor must not be overwritten using an old disk revision.
    await writeFile(path, JSON.stringify(saved) + "\n");
    await page
      .getByLabel("그래프 이름", { exact: true })
      .fill("외부 변경 뒤의 미저장 편집");
    await page.getByRole("button", { name: "변경 적용", exact: true }).click();
    await page.getByRole("button", { name: "저장", exact: true }).click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "파일 저장", exact: true })
      .click();
    await expect(page.getByRole("dialog").getByRole("alert")).toContainText(
      "외부에서 변경",
    );
    expect(model.state().dirty).toBe(true);
    expect(JSON.parse(await readFile(path, "utf8"))).toEqual(saved);
    await capture(page, info, "content-save-conflict");
    await page.keyboard.press("Escape");
  } finally {
    await unlink(path).catch((error: NodeJS.ErrnoException) => {
      if (error.code !== "ENOENT") throw error;
    });
  }
});

test("content folder loading, failure, retry, empty list and invalid file keep the current graph", async ({
  page,
}, info) => {
  const model = await observeNodes(page);
  let mode: "loading" | "error" | "empty" | "many" | "invalid" = "loading";
  let release = () => {};
  const loading = new Promise<void>((resolve) => {
    release = resolve;
  });
  await page.route("**/__content/files", async (route) => {
    if (mode === "loading") await loading;
    const request = route.request().postDataJSON();
    const payload =
      mode === "error"
        ? { error: "콘텐츠 폴더를 읽을 수 없습니다." }
        : request.operation === "read"
          ? { name: "invalid.node-graph.json", json: "{}", revision: "invalid" }
          : {
              directory: "content/",
              organization: {
                document: { format: "graph-groups", version: 1, groups: [] },
                revision: null,
              },
              files:
                mode === "invalid"
                  ? ["invalid.node-graph.json"]
                  : mode === "many"
                    ? Array.from(
                        { length: 60 },
                        (_, i) =>
                          `${i} · 아주 긴 한글 콘텐츠 이름과 폴더 목록 줄바꿈 확인.node-graph.json`,
                      )
                    : [],
            };
    await route.fulfill({ json: payload });
  });
  await openWorkspace(page, info);
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "노드", exact: true })
    .click();
  await expect(page.getByTestId("node-canvas")).toBeVisible();
  const before = (await readNodeState(page)).doc;
  await page.getByRole("button", { name: "새 그래프", exact: true }).click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "취소", exact: true })
    .click();
  expect((await readNodeState(page)).doc).toEqual(before);
  await expect(
    page.getByRole("button", { name: "새 그래프", exact: true }),
  ).toBeFocused();
  await page.getByRole("button", { name: "그래프 목록", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "그래프 목록" });
  await expect(dialog.getByRole("status")).toContainText("목록을 읽고");
  await capture(page, info, "content-loading");
  mode = "error";
  release();
  await expect(dialog.getByRole("alert")).toContainText("읽을 수 없습니다");
  await capture(page, info, "content-error");
  mode = "empty";
  await dialog.getByRole("button", { name: "새로고침" }).click();
  await expect(dialog).toContainText("저장된 노드 그래프가 없습니다");
  await capture(page, info, "content-empty");
  mode = "many";
  await dialog.getByRole("button", { name: "새로고침" }).click();
  await expect(
    dialog.getByLabel("저장된 콘텐츠").getByRole("button"),
  ).toHaveCount(60);
  await expect(dialog).toBeInViewport({ ratio: 1 });
  await capture(page, info, "content-many");
  mode = "invalid";
  await dialog.getByRole("button", { name: "새로고침" }).click();
  await dialog
    .getByRole("button", { name: "invalid.node-graph.json", exact: true })
    .click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "취소", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "그래프 목록", exact: true }),
  ).toBeFocused();
  expect((await readNodeState(page)).doc).toEqual(before);
  await page.getByRole("button", { name: "그래프 목록", exact: true }).click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "invalid.node-graph.json", exact: true })
    .click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "교체", exact: true })
    .click();
  await expect(
    page.getByTestId("node-editor").getByRole("alert"),
  ).toBeVisible();
  expect(model.state().doc).toEqual(before);
});
