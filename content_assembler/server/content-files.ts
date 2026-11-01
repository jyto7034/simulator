import { createHash, randomUUID } from "node:crypto";
import { mkdir, lstat, open, readdir, rename, unlink } from "node:fs/promises";
import { join } from "node:path";
import type { Plugin } from "vite";
import type { IncomingMessage, ServerResponse } from "node:http";

import {
  contentName,
  emptyGroups,
  groupsFilename,
  parseGroups,
} from "../src/content/groups.ts";
export { contentName } from "../src/content/groups.ts";

const limit = 5_000_000;
const revision = (json: string) =>
  createHash("sha256").update(json).digest("hex");

async function ensureDirectory(directory: string) {
  await mkdir(directory, { recursive: true });
  const stat = await lstat(directory);
  if (!stat.isDirectory() || stat.isSymbolicLink())
    throw new Error("콘텐츠 폴더는 실제 디렉터리여야 합니다.");
}

async function read(directory: string, name: string) {
  const path = join(directory, name);
  const stat = await lstat(path);
  if (!stat.isFile() || stat.isSymbolicLink())
    throw new Error("일반 콘텐츠 파일만 열 수 있습니다.");
  if (stat.size > limit) throw new Error("파일 크기 제한은 5MB입니다.");
  const file = await open(path, "r");
  try {
    const buffer = Buffer.alloc(limit + 1);
    let size = 0;
    while (size < buffer.length) {
      const result = await file.read(buffer, size, buffer.length - size, null);
      if (!result.bytesRead) break;
      size += result.bytesRead;
    }
    if (size > limit) throw new Error("파일 크기 제한은 5MB입니다.");
    const json = new TextDecoder("utf-8", { fatal: true }).decode(
      buffer.subarray(0, size),
    );
    return { name, json, revision: revision(json) };
  } finally {
    await file.close();
  }
}

function missing(error: unknown) {
  return (
    !!error &&
    typeof error === "object" &&
    "code" in error &&
    error.code === "ENOENT"
  );
}

async function readGroups(directory: string) {
  try {
    const file = await read(directory, groupsFilename);
    return {
      document: parseGroups(JSON.parse(file.json)),
      revision: file.revision,
    };
  } catch (error) {
    if (!missing(error)) throw error;
    return { document: emptyGroups(), revision: null };
  }
}

// Both dev and preview operate on this project's content directory, never arbitrary paths.
export async function contentOperation(directory: string, input: unknown) {
  await ensureDirectory(directory);
  if (!input || typeof input !== "object" || !("operation" in input))
    throw new Error("잘못된 콘텐츠 요청입니다.");
  const kind = "kind" in input ? input.kind : "node-graph";
  if (kind !== "node-graph" && kind !== "composition")
    throw new Error("지원하지 않는 콘텐츠 종류입니다.");
  const composition = kind === "composition";
  const suffix = composition ? ".skill.yaml" : ".node-graph.json";
  if (input.operation === "list") {
    const entries = await readdir(directory, { withFileTypes: true });
    return {
      directory,
      organization: composition
        ? { document: emptyGroups(), revision: null }
        : await readGroups(directory),
      files: entries
        .filter((entry) => {
          if (!entry.isFile() || !entry.name.endsWith(suffix)) return false;
          try {
            contentName(entry.name, suffix);
            return true;
          } catch {
            return false;
          }
        })
        .map((entry) => entry.name)
        .sort(),
    };
  }
  const grouping = input.operation === "save-groups";
  if (grouping && composition)
    throw new Error("조합 원본은 그래프 그룹 파일을 수정할 수 없습니다.");
  const name = grouping
    ? groupsFilename
    : contentName("name" in input ? input.name : undefined, suffix);
  if (input.operation === "read") return read(directory, name);
  if (
    (input.operation !== "save" && !grouping) ||
    !("json" in input) ||
    typeof input.json !== "string" ||
    !("expected" in input) ||
    !(input.expected === null || typeof input.expected === "string")
  )
    throw new Error("잘못된 저장 요청입니다.");
  const json = input.json;
  if (Buffer.byteLength(json) > limit)
    throw new Error("파일 크기 제한은 5MB입니다.");
  if (composition && Buffer.byteLength(json) > 2_000_000)
    throw new Error("원본 크기 제한은 2MB입니다.");
  const document: unknown = composition ? null : JSON.parse(json);
  if (grouping) parseGroups(document);
  else if (
    !composition &&
    (!document ||
      typeof document !== "object" ||
      !("format" in document) ||
      document.format !== "node-graph" ||
      !("version" in document) ||
      document.version !== 1)
  )
    throw new Error("지원하는 노드 그래프 파일이 아닙니다.");
  const lockPath = join(directory, ".content-write.lock");
  let lock;
  try {
    lock = await open(lockPath, "wx");
  } catch (error) {
    if (
      !error ||
      typeof error !== "object" ||
      !("code" in error) ||
      error.code !== "EEXIST"
    )
      throw error;
    throw new Error(
      "콘텐츠 폴더에서 다른 저장이 진행 중입니다. 잠시 후 다시 저장하세요.",
      { cause: error },
    );
  }
  const temporary = join(directory, `.write-${randomUUID()}.tmp`);
  try {
    let previous: Awaited<ReturnType<typeof read>> | null = null;
    try {
      previous = await read(directory, name);
    } catch (error) {
      if (!missing(error)) throw error;
    }
    if ((previous?.revision ?? null) !== input.expected)
      throw new Error(
        grouping
          ? "그룹이 외부에서 변경되었습니다. 편집을 취소하고 새로고침한 뒤 다시 적용하세요."
          : "같은 이름의 파일이 있거나 파일이 외부에서 변경되었습니다. 다시 열거나 다른 이름으로 저장하세요.",
      );
    const file = await open(temporary, "wx");
    try {
      await file.writeFile(json, "utf8");
      await file.sync();
    } finally {
      await file.close();
    }
    await rename(temporary, join(directory, name));
    return grouping
      ? { document: parseGroups(document), revision: revision(json) }
      : { name, json, revision: revision(json) };
  } finally {
    try {
      await unlink(temporary).catch((error: unknown) => {
        if (!missing(error)) throw error;
      });
    } finally {
      await lock.close();
      await unlink(lockPath);
    }
  }
}

export function contentFilesPlugin(directory: string): Plugin {
  const middleware = (
    req: IncomingMessage,
    res: ServerResponse,
    next: () => void,
  ) => {
    if (req.url?.split("?")[0] !== "/__content/files") return next();
    const host = req.headers.host ?? "";
    // Custom header + same-origin + loopback host checks keep this local filesystem API off remote sites.
    if (
      req.method !== "POST" ||
      req.headers["x-content-assembler"] !== "local-files" ||
      !/^(localhost|127\.0\.0\.1|\[::1\])(:\d+)?$/.test(host) ||
      (req.headers.origin &&
        req.headers.origin !== `http://${host}` &&
        req.headers.origin !== `https://${host}`)
    ) {
      res.writeHead(403).end();
      return;
    }
    void (async () => {
      res.setHeader("Content-Type", "application/json; charset=utf-8");
      res.setHeader("Cache-Control", "no-store");
      try {
        let size = 0;
        const chunks: Buffer[] = [];
        for await (const chunk of req) {
          const buffer = Buffer.from(chunk);
          size += buffer.length;
          if (size > limit * 2 + 4096)
            throw new Error("저장 요청이 너무 큽니다.");
          chunks.push(buffer);
        }
        res.end(
          JSON.stringify(
            await contentOperation(
              directory,
              JSON.parse(Buffer.concat(chunks).toString("utf8")),
            ),
          ),
        );
      } catch (error) {
        res.end(
          JSON.stringify({
            error: error instanceof Error ? error.message : String(error),
          }),
        );
      }
    })();
  };
  return {
    name: "project-content-files",
    configureServer(server) {
      server.middlewares.use(middleware);
    },
    configurePreviewServer(server) {
      server.middlewares.use(middleware);
    },
  };
}
