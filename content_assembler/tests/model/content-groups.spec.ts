import { test, expect } from "@playwright/test";
import { mkdtemp, readFile, writeFile, rm, symlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { contentOperation } from "../../server/content-files";
import {
  emptyGroups,
  parseGroups,
  moveToGroup,
} from "../../src/content/groups";

const file = "한글 그래프.node-graph.json";
const groups = () => ({
  ...emptyGroups(),
  groups: [{ id: "example", name: "예제", files: [file] }],
});

test("group transactions roundtrip, conflict, and preserve graph bytes", async () => {
  const root = await mkdtemp(join(tmpdir(), "assembler-groups-"));
  try {
    const graph = '{"format":"node-graph","version":1}';
    await writeFile(join(root, file), graph);
    const save = (document: unknown, expected: unknown = null) =>
      contentOperation(root, {
        operation: "save-groups",
        json: JSON.stringify(document),
        expected,
      });
    const first = await save(groups());
    if (!("revision" in first)) throw new Error("revision expected");
    const listing = await contentOperation(root, { operation: "list" });
    expect(listing).toMatchObject({ files: [file], organization: first });
    await expect(save(emptyGroups())).rejects.toThrow("외부에서 변경");
    expect(
      JSON.parse(await readFile(join(root, ".graph-groups.json"), "utf8")),
    ).toEqual(groups());
    await expect(
      save(
        { ...groups(), groups: [...groups().groups, ...groups().groups] },
        first.revision,
      ),
    ).rejects.toThrow("그룹 정보");
    const settled = await Promise.allSettled([
      save(emptyGroups(), first.revision),
      save(groups(), first.revision),
    ]);
    expect(
      settled.filter((value) => value.status === "fulfilled"),
    ).toHaveLength(1);
    const current = await contentOperation(root, { operation: "list" });
    if (!("organization" in current) || !current.organization)
      throw new Error("listing expected");
    await save(emptyGroups(), current.organization.revision);
    expect(await readFile(join(root, file), "utf8")).toBe(graph);
    await writeFile(join(root, ".graph-groups.json"), "broken");
    await expect(
      contentOperation(root, { operation: "list" }),
    ).rejects.toThrow();
    await expect(save(groups(), null)).rejects.toThrow("외부에서 변경");
    expect(await readFile(join(root, ".graph-groups.json"), "utf8")).toBe(
      "broken",
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("group membership is unique and moves do not mutate drafts", () => {
  const original = groups();
  original.groups.push({ id: "second", name: "빈 그룹", files: [] });
  const moved = moveToGroup(original, file, "second");
  expect(original.groups[0].files).toEqual([file]);
  expect(moved.groups.map((group) => group.files)).toEqual([[], [file]]);
  expect(
    moveToGroup(moved, file, "").groups.every((group) => !group.files.length),
  ).toBe(true);
  expect(() => moveToGroup(moved, file, "missing")).toThrow();
  for (const value of [
    { ...original, version: 2 },
    { ...original, groups: [{ id: "a", name: " ", files: [] }] },
    {
      ...original,
      groups: [
        { id: "a", name: "예제", files: ["../outside.node-graph.json"] },
      ],
    },
    { ...original, groups: [{ id: "a", name: "예제", files: [file, file] }] },
    {
      ...original,
      groups: [
        { id: "a", name: "예제", files: [] },
        { id: "b", name: "예제", files: [] },
      ],
    },
  ])
    expect(() => parseGroups(value)).toThrow();
  // External deletion does not destroy membership; a restored file returns to its group.
  expect(parseGroups(original)).toEqual(original);
});

test("group metadata symlinks cannot be read or overwritten", async () => {
  test.skip(
    process.platform === "win32",
    "Symlink creation requires Windows privileges",
  );
  const root = await mkdtemp(join(tmpdir(), "assembler-groups-link-"));
  const outside = root + "-outside";
  try {
    await writeFile(outside, JSON.stringify(groups()));
    await symlink(outside, join(root, ".graph-groups.json"));
    await expect(contentOperation(root, { operation: "list" })).rejects.toThrow(
      "일반 콘텐츠",
    );
    await expect(
      contentOperation(root, {
        operation: "save-groups",
        json: JSON.stringify(emptyGroups()),
        expected: null,
      }),
    ).rejects.toThrow("일반 콘텐츠");
    expect(JSON.parse(await readFile(outside, "utf8"))).toEqual(groups());
  } finally {
    await rm(root, { recursive: true, force: true });
    await rm(outside, { force: true });
  }
});
