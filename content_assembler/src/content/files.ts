import { invoke, isTauri } from "@tauri-apps/api/core";
import { parseDocument } from "../nodes/core/validation";

import { parseGroups, type GroupSnapshot, type GraphGroups } from "./groups";

export type ContentFile = { name: string; json: string; revision: string };
export type ContentListing = {
  directory: string;
  files: string[];
  organization: GroupSnapshot;
};
export type ContentRequest =
  | { operation: "save-groups"; json: string; expected: string | null }
  | { operation: "list" }
  | { operation: "read"; name: string }
  | { operation: "save"; name: string; json: string; expected: string | null };

async function request(input: ContentRequest): Promise<unknown> {
  if (isTauri()) return invoke("content_files", { input });
  const response = await fetch("/__content/files", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Content-Assembler": "local-files",
    },
    body: JSON.stringify(input),
  });
  if (
    !response.ok ||
    !response.headers.get("content-type")?.includes("application/json")
  )
    throw new Error(
      "콘텐츠 폴더에 연결하지 못했습니다. 프로젝트 개발 서버 또는 데스크톱 앱에서 열어 주세요.",
    );
  const result: unknown = await response.json();
  if (result && typeof result === "object" && "error" in result)
    throw new Error(String(result.error));
  return result;
}

export async function listContent(): Promise<ContentListing> {
  const result = await request({ operation: "list" });
  if (
    !result ||
    typeof result !== "object" ||
    !("directory" in result) ||
    typeof result.directory !== "string" ||
    !("files" in result) ||
    !Array.isArray(result.files) ||
    !result.files.every((name: unknown) => typeof name === "string")
  )
    throw new Error("콘텐츠 목록 응답이 올바르지 않습니다.");
  return {
    directory: result.directory,
    files: result.files,
    organization: parseSnapshot(
      "organization" in result ? result.organization : undefined,
    ),
  };
}

export async function contentFile(
  input: Exclude<ContentRequest, { operation: "list" | "save-groups" }>,
): Promise<ContentFile> {
  if (input.operation === "save") parseDocument(input.json);
  const result = await request(input);
  if (
    !result ||
    typeof result !== "object" ||
    !("name" in result) ||
    typeof result.name !== "string" ||
    !("json" in result) ||
    typeof result.json !== "string" ||
    !("revision" in result) ||
    typeof result.revision !== "string"
  )
    throw new Error("콘텐츠 파일 응답이 올바르지 않습니다.");
  return { name: result.name, json: result.json, revision: result.revision };
}

function parseSnapshot(value: unknown): GroupSnapshot {
  if (
    !value ||
    typeof value !== "object" ||
    !("document" in value) ||
    !("revision" in value) ||
    !(value.revision === null || typeof value.revision === "string")
  )
    throw new Error("그룹 응답이 올바르지 않습니다.");
  return { document: parseGroups(value.document), revision: value.revision };
}
export async function saveGroups(
  document: GraphGroups,
  expected: string | null,
): Promise<GroupSnapshot> {
  return parseSnapshot(
    await request({
      operation: "save-groups",
      json: JSON.stringify(parseGroups(document), null, 2) + "\n",
      expected,
    }),
  );
}
