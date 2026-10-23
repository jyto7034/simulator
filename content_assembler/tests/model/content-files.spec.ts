import { test, expect } from "@playwright/test";
import {
  mkdtemp,
  readFile,
  writeFile,
  rm,
  symlink,
  readdir,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { contentOperation } from "../../server/content-files";
import { helloGoodbye } from "../../src/nodes/core/hello-goodbye";

test("project folder roundtrip and stale saves preserve the disk file", async () => {
  const root = await mkdtemp(join(tmpdir(), "assembler-content-"));
  const name = "한글 콘텐츠.node-graph.json";
  const json = JSON.stringify(helloGoodbye());
  try {
    const result = await contentOperation(root, {
      operation: "save",
      name,
      json,
      expected: null,
    });
    if (!("revision" in result)) throw new Error("file expected");
    expect(await readFile(join(root, name), "utf8")).toBe(json);
    expect(await contentOperation(root, { operation: "read", name })).toEqual(
      result,
    );
    expect(await contentOperation(root, { operation: "list" })).toEqual({
      directory: root,
      files: [name],
      organization: {
        document: { format: "graph-groups", version: 1, groups: [] },
        revision: null,
      },
    });
    await expect(
      contentOperation(root, { operation: "save", name, json, expected: null }),
    ).rejects.toThrow("같은 이름");
    await writeFile(join(root, name), json + "\n");
    await expect(
      contentOperation(root, {
        operation: "save",
        name,
        json,
        expected: result.revision,
      }),
    ).rejects.toThrow("외부에서 변경");
    expect(await readFile(join(root, name), "utf8")).toBe(json + "\n");
    const current = await contentOperation(root, { operation: "read", name });
    if (!("revision" in current)) throw new Error("file expected");
    await contentOperation(root, {
      operation: "save",
      name,
      json,
      expected: current.revision,
    });
    expect(await readdir(root)).toEqual([name]);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("invalid paths, invalid documents and oversized input never create content files", async () => {
  const root = await mkdtemp(join(tmpdir(), "assembler-content-"));
  try {
    const json = JSON.stringify(helloGoodbye());
    for (const name of [
      "../outside.node-graph.json",
      "..\\outside.node-graph.json",
      "C:escape.node-graph.json",
      "CON.node-graph.json",
      "plain.json",
    ]) {
      await expect(
        contentOperation(root, {
          operation: "save",
          name,
          json,
          expected: null,
        }),
      ).rejects.toThrow("파일 이름");
    }
    for (const json of ["not json", "{}", " ".repeat(5_000_001)]) {
      await expect(
        contentOperation(root, {
          operation: "save",
          name: "invalid.node-graph.json",
          json,
          expected: null,
        }),
      ).rejects.toThrow();
    }
    expect(await readdir(root)).toEqual([]);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("symlinks cannot escape the content folder and competing writes cannot both succeed", async () => {
  test.skip(
    process.platform === "win32",
    "Symlink creation requires Windows privileges",
  );
  const root = await mkdtemp(join(tmpdir(), "assembler-content-"));
  const outside = root + "-outside";
  try {
    const json = JSON.stringify(helloGoodbye());
    await writeFile(outside, "preserve");
    await symlink(outside, join(root, "link.node-graph.json"));
    await expect(
      contentOperation(root, {
        operation: "read",
        name: "link.node-graph.json",
      }),
    ).rejects.toThrow("일반 콘텐츠");
    await expect(
      contentOperation(root, {
        operation: "save",
        name: "link.node-graph.json",
        json,
        expected: null,
      }),
    ).rejects.toThrow("일반 콘텐츠");
    expect(await readFile(outside, "utf8")).toBe("preserve");
    const saves = await Promise.allSettled(
      [1, 2].map(() =>
        contentOperation(root, {
          operation: "save",
          name: "race.node-graph.json",
          json,
          expected: null,
        }),
      ),
    );
    expect(
      saves.filter((result) => result.status === "fulfilled"),
    ).toHaveLength(1);
    expect(await readFile(join(root, "race.node-graph.json"), "utf8")).toBe(
      json,
    );
  } finally {
    await rm(root, { recursive: true, force: true });
    await rm(outside, { force: true });
  }
});
