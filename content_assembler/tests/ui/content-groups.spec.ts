import { mkdtemp, readFile, writeFile, rm } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { contentOperation } from "../../server/content-files";
import {
  test,
  expect,
  openWorkspace,
  capture,
  inspectAccessibility,
  inspectLayout,
} from "./fixtures";
import { observeNodes } from "./node-observer";

test("groups create rename move cancel persist conflict and dissolve without changing graphs", async ({
  page,
}, info) => {
  // The UI boundary routes to the real filesystem service with a test-owned folder.
  // Each viewport has independent disk metadata, never a shared production folder.
  const root = await mkdtemp(join(tmpdir(), "assembler-ui-groups-"));
  const filename =
    "공명 소비 방식과 조건을 확인하는 아주 긴 한글 이름.node-graph.json";
  const second = "두 번째 예제.node-graph.json";
  const json = await readFile(
    join(process.cwd(), "content/hello-goodbye.node-graph.json"),
    "utf8",
  );
  let holdSave: Promise<void> | null = null;
  let releaseSave = () => {};
  try {
    await writeFile(join(root, filename), json);
    await writeFile(join(root, second), json);
    await page.route("**/__content/files", async (route) => {
      const request = route.request().postDataJSON();
      if (request.operation === "save-groups" && holdSave) await holdSave;
      try {
        await route.fulfill({ json: await contentOperation(root, request) });
      } catch (error) {
        await route.fulfill({ json: { error: String(error) } });
      }
    });
    const model = await observeNodes(page);
    await openWorkspace(page, info);
    await page
      .getByRole("navigation", { name: "작업 공간" })
      .getByRole("button", { name: "노드", exact: true })
      .click();
    await expect.poll(() => model.state()?.doc.nodes.length).toBe(0);
    const before = model.state().doc;
    const open = async () => {
      await page
        .getByRole("button", { name: "그래프 목록", exact: true })
        .click();
    };
    await open();
    const dialog = page.getByRole("dialog", {
      name: "그래프 목록",
      exact: true,
    });
    await dialog
      .getByRole("button", { name: "그룹 편집", exact: true })
      .click();
    await dialog
      .getByRole("button", { name: "그룹 추가", exact: true })
      .click();
    await dialog
      .getByLabel("그룹 이름 1", { exact: true })
      .fill("공명 소비 방식별 예제와 긴 한글 그룹 이름 확인");
    await dialog
      .getByLabel(`${filename} 소속 그룹`, { exact: true })
      .selectOption({
        label: "공명 소비 방식별 예제와 긴 한글 그룹 이름 확인",
      });
    await dialog
      .getByRole("button", { name: "그룹 추가", exact: true })
      .click();
    await dialog.getByLabel("그룹 이름 2", { exact: true }).fill("빈 그룹");
    await capture(page, info, "group-edit");
    await inspectAccessibility(page, info, "group-edit");
    await expect(dialog).toBeInViewport({ ratio: 1 });
    holdSave = new Promise((resolve) => {
      releaseSave = resolve;
    });
    await dialog
      .getByRole("button", { name: "그룹 적용", exact: true })
      .click();
    await expect(dialog.getByRole("status")).toContainText("그룹을 저장");
    await page.keyboard.press("Escape");
    await expect(dialog).toBeVisible();
    await capture(page, info, "group-saving");
    releaseSave();
    holdSave = null;
    await expect(
      dialog.getByRole("button", { name: "그룹 편집", exact: true }),
    ).toBeFocused();
    await expect(
      dialog.getByRole("button", { name: filename, exact: true }),
    ).not.toBeVisible();
    const summary = dialog
      .locator("summary")
      .filter({ hasText: "공명 소비 방식별" });
    await summary.focus();
    await page.keyboard.press("Enter");
    await expect(
      dialog.getByRole("button", { name: filename, exact: true }),
    ).toBeVisible();
    await dialog.locator("summary").filter({ hasText: "빈 그룹" }).click();
    await expect(dialog).toContainText("이 그룹에는 그래프가 없습니다");
    await capture(page, info, "group-expanded");
    await inspectAccessibility(page, info, "group-expanded");
    await expect(dialog).toBeInViewport({ ratio: 1 });
    const firstDisk = await readFile(join(root, ".graph-groups.json"), "utf8");
    await dialog
      .getByRole("button", { name: "그룹 편집", exact: true })
      .click();
    await dialog.getByLabel("그룹 이름 1", { exact: true }).fill("취소될 변경");
    await page.keyboard.press("Escape");
    await expect(
      dialog.getByRole("button", { name: "그룹 편집", exact: true }),
    ).toBeFocused();
    expect(await readFile(join(root, ".graph-groups.json"), "utf8")).toBe(
      firstDisk,
    );
    await page.keyboard.press("Escape");
    await expect(
      page.getByRole("button", { name: "그래프 목록", exact: true }),
    ).toBeFocused();
    await inspectLayout(page, info, "groups-closed");
    await open();
    await expect(dialog.locator("summary")).toHaveCount(2);
    await dialog
      .getByRole("button", { name: "그룹 편집", exact: true })
      .click();
    await dialog.getByLabel("그룹 이름 1", { exact: true }).fill("빈 그룹");
    await dialog
      .getByRole("button", { name: "그룹 적용", exact: true })
      .click();
    await expect(dialog.getByRole("alert")).toContainText("이름 중복");
    await dialog.getByLabel("그룹 이름 1", { exact: true }).fill("공명 예제");
    await dialog
      .getByLabel(`${filename} 소속 그룹`, { exact: true })
      .selectOption({ label: "빈 그룹" });
    await dialog
      .getByLabel(`${second} 소속 그룹`, { exact: true })
      .selectOption({ label: "공명 예제" });
    await dialog
      .getByRole("button", { name: "그룹 적용", exact: true })
      .click();
    await expect(
      dialog.locator("summary").filter({ hasText: "공명 예제" }),
    ).toBeVisible();
    const moved = JSON.parse(
      await readFile(join(root, ".graph-groups.json"), "utf8"),
    );
    expect(
      moved.groups.map((group: { files: string[] }) => group.files),
    ).toEqual([[second], [filename]]);
    await dialog
      .getByRole("button", { name: "그룹 편집", exact: true })
      .click();
    await writeFile(
      join(root, ".graph-groups.json"),
      JSON.stringify(moved) + "\n\n",
    );
    await dialog.getByLabel("그룹 이름 1", { exact: true }).fill("충돌한 변경");
    await dialog
      .getByRole("button", { name: "그룹 적용", exact: true })
      .click();
    await expect(dialog.getByRole("alert")).toContainText("외부에서 변경");
    await capture(page, info, "group-conflict");
    await inspectAccessibility(page, info, "group-conflict");
    await dialog
      .getByRole("button", { name: "편집 취소", exact: true })
      .click();
    await dialog.getByRole("button", { name: "새로고침", exact: true }).click();
    await dialog
      .getByRole("button", { name: "그룹 편집", exact: true })
      .click();
    await dialog
      .getByRole("button", { name: "빈 그룹 그룹 해제", exact: true })
      .click();
    await dialog
      .getByRole("button", { name: "그룹 적용", exact: true })
      .click();
    await expect(
      dialog.getByRole("button", { name: filename, exact: true }),
    ).toBeVisible();
    expect(model.state().doc).toEqual(before);
    expect(model.state().canUndo).toBe(false);
    expect(await readFile(join(root, filename), "utf8")).toBe(json);
    expect(await readFile(join(root, second), "utf8")).toBe(json);
    await dialog.getByRole("button", { name: filename, exact: true }).click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "교체", exact: true })
      .click();
    await expect.poll(() => model.state()?.doc.nodes.length).toBe(29);
  } finally {
    releaseSave();
    await rm(root, { recursive: true, force: true });
  }
});

test("empty folders accept groups and many long names remain editable within the dialog", async ({
  page,
}, info) => {
  const root = await mkdtemp(join(tmpdir(), "assembler-ui-many-groups-"));
  try {
    await page.route("**/__content/files", async (route) => {
      try {
        await route.fulfill({
          json: await contentOperation(root, route.request().postDataJSON()),
        });
      } catch (error) {
        await route.fulfill({ json: { error: String(error) } });
      }
    });
    await openWorkspace(page, info);
    await page
      .getByRole("navigation", { name: "작업 공간" })
      .getByRole("button", { name: "노드", exact: true })
      .click();
    await page
      .getByRole("button", { name: "그래프 목록", exact: true })
      .click();
    const dialog = page.getByRole("dialog", {
      name: "그래프 목록",
      exact: true,
    });
    await dialog
      .getByRole("button", { name: "그룹 편집", exact: true })
      .click();
    await expect(dialog).toContainText("빈 그룹을 미리 만들 수 있습니다");
    await dialog
      .getByRole("button", { name: "그룹 추가", exact: true })
      .click();
    await dialog
      .getByLabel("그룹 이름 1", { exact: true })
      .fill("빈 폴더에서 만든 그룹");
    await dialog
      .getByRole("button", { name: "그룹 적용", exact: true })
      .click();
    await expect(dialog.locator("summary")).toContainText(
      "빈 폴더에서 만든 그룹",
    );
    await capture(page, info, "group-empty-folder");
    const files = Array.from(
      { length: 60 },
      (_, index) =>
        `${String(index).padStart(2, "0")} · 여러 조건과 비용을 표현하는 긴 한글 예제.node-graph.json`,
    );
    for (const file of files)
      await writeFile(join(root, file), '{"format":"node-graph","version":1}');
    const groups = Array.from({ length: 12 }, (_, index) => ({
      id: `group-${index}`,
      name: `${index + 1} · 조건과 비용을 표현하는 긴 한글 그룹 이름`,
      files: files.slice(index * 5, index * 5 + 5),
    }));
    await writeFile(
      join(root, ".graph-groups.json"),
      JSON.stringify({ format: "graph-groups", version: 1, groups }),
    );
    await dialog.getByRole("button", { name: "새로고침", exact: true }).click();
    await expect(dialog.locator("summary")).toHaveCount(12);
    await capture(page, info, "group-many-list");
    await inspectAccessibility(page, info, "group-many-list");
    await dialog
      .getByRole("button", { name: "그룹 편집", exact: true })
      .click();
    await dialog
      .getByLabel("그룹 이름 12", { exact: true })
      .fill("마지막 그룹 이름 변경");
    await capture(page, info, "group-many-names");
    const last = dialog.getByLabel(`${files[59]} 소속 그룹`, { exact: true });
    await last.selectOption("");
    await last.focus();
    await expect(last).toBeInViewport({ ratio: 1 });
    await expect(dialog).toBeInViewport({ ratio: 1 });
    await capture(page, info, "group-many-assignments");
    await inspectAccessibility(page, info, "group-many-assignments");
    await dialog
      .getByRole("button", { name: "그룹 적용", exact: true })
      .click();
    await expect(dialog.locator("summary")).toHaveCount(12);
    const saved = JSON.parse(
      await readFile(join(root, ".graph-groups.json"), "utf8"),
    );
    expect(saved.groups[11].name).toBe("마지막 그룹 이름 변경");
    expect(saved.groups[11].files).toHaveLength(4);
    await dialog
      .getByRole("button", { name: files[59], exact: true })
      .scrollIntoViewIfNeeded();
    await expect(
      dialog.getByRole("button", { name: files[59], exact: true }),
    ).toBeVisible();
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
