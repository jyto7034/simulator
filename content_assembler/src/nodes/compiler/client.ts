import { invoke, isTauri } from "@tauri-apps/api/core";
import type { GraphDocument, Version } from "../types";

export type CoreLocation = {
  node: string | null;
  port: string | null;
  edge: string | null;
  field: string | null;
};
export type CoreDiagnostic = {
  code: string;
  phase: string;
  reason: string;
  primary: CoreLocation;
  related: CoreLocation[];
};
export type CompileResult = Version & {
  request_id: number;
  source_digest: string;
  skill_id: string;
  target: string;
} & (
    | { status: "compiled"; ron: string }
    | { status: "rejected"; diagnostics: CoreDiagnostic[] }
  );
const target = "native-manual-employee-v1";
function record(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}
function location(value: unknown): value is CoreLocation {
  return (
    record(value) &&
    ["node", "port", "edge", "field"].every(
      (k) => value[k] === null || typeof value[k] === "string",
    )
  );
}
function diagnostic(value: unknown): value is CoreDiagnostic {
  return (
    record(value) &&
    ["code", "phase", "reason"].every((k) => typeof value[k] === "string") &&
    location(value.primary) &&
    Array.isArray(value.related) &&
    value.related.every(location)
  );
}
export async function compileGraph(
  doc: GraphDocument,
  version: Version,
  requestId: number,
): Promise<CompileResult> {
  const source = JSON.stringify(doc);
  const digest = Array.from(
    new Uint8Array(
      await crypto.subtle.digest("SHA-256", new TextEncoder().encode(source)),
    ),
  )
    .map((n) => n.toString(16).padStart(2, "0"))
    .join("");
  const input = {
    source,
    skill_id: doc.id,
    epoch: version.epoch,
    revision: version.revision,
    request_id: requestId,
  };
  let response: unknown;
  if (isTauri()) response = await invoke("compile_graph", { input });
  else {
    const result = await fetch("/__content/compile", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Content-Assembler": "compile",
      },
      body: JSON.stringify(input),
    });
    if (
      !result.ok ||
      !result.headers.get("content-type")?.includes("application/json")
    )
      throw new Error("Core 컴파일러에 연결하지 못했습니다.");
    response = await result.json();
  }
  if (record(response) && typeof response.error === "string")
    throw new Error(response.error);
  if (
    !record(response) ||
    response.epoch !== version.epoch ||
    response.revision !== version.revision ||
    response.request_id !== requestId ||
    response.skill_id !== doc.id ||
    response.source_digest !== digest ||
    response.target !== target
  )
    throw new Error(
      "컴파일 응답이 요청한 문서 또는 실행 대상과 일치하지 않습니다.",
    );
  if (
    response.status === "compiled" &&
    typeof response.ron === "string" &&
    response.ron.length > 0 &&
    !("diagnostics" in response)
  )
    return response as CompileResult;
  if (
    response.status === "rejected" &&
    Array.isArray(response.diagnostics) &&
    response.diagnostics.length > 0 &&
    response.diagnostics.every(diagnostic) &&
    !("ron" in response)
  )
    return response as CompileResult;
  throw new Error("Core 컴파일 결과 형식이 올바르지 않습니다.");
}
export function sameVersion(a: Version, b: Version) {
  return a.epoch === b.epoch && a.revision === b.revision;
}
export function diagnosticTitle(code: string) {
  const titles: Record<string, string> = {
    "graph.target_unsupported": "실행 연결이 아직 지원되지 않는 동작",
    "graph.value_unresolved": "값을 입력해야 합니다",
    "graph.policy_unresolved": "정책을 선택해야 합니다",
    "graph.input_required": "필수 입력이 연결되지 않았습니다",
    "graph.entry_unbound": "실행 진입점에 연결되지 않은 노드",
    "graph.entry_ambiguous": "실행 진입점을 확인해야 합니다",
    "graph.lowering_contract": "실행 경로의 연결을 확인해야 합니다",
    "graph.numeric_target_range": "Core에서 표현할 수 없는 수치",
    "graph.result_scope": "결과를 읽을 수 없는 실행 경로",
    "graph.owner_scope": "사용 소유자가 일치하지 않습니다",
    "graph.use_after_end": "종료된 사용을 참조하고 있습니다",
    "graph.duplicate_admission": "같은 요청을 중복 접수하고 있습니다",
  };
  return titles[code] ?? "Core 검증에서 확인할 문제가 있습니다";
}
