import { invoke, isTauri } from "@tauri-apps/api/core";
import type { CompileResult, CompileTicket, CoreDiagnostic } from "./session";

const record = (value: unknown): value is Record<string, unknown> =>
  !!value && typeof value === "object" && !Array.isArray(value);
const strings = (value: unknown): value is string[] =>
  Array.isArray(value) && value.every((item) => typeof item === "string");
function diagnostic(value: unknown): value is CoreDiagnostic {
  return (
    record(value) &&
    ["code", "document", "source_digest", "reason"].every(
      (key) => typeof value[key] === "string",
    ) &&
    record(value.source) &&
    typeof value.source.definition === "string" &&
    strings(value.source.calls) &&
    strings(value.source.inputs)
  );
}
export async function digest(source: string) {
  return Array.from(
    new Uint8Array(
      await crypto.subtle.digest("SHA-256", new TextEncoder().encode(source)),
    ),
  )
    .map((value) => value.toString(16).padStart(2, "0"))
    .join("");
}
export async function compileComposition(
  ticket: CompileTicket,
): Promise<CompileResult> {
  const sourceDigest = await digest(ticket.source);
  let response: unknown;
  if (isTauri())
    response = await invoke("compile_composition", { input: ticket });
  else {
    const result = await fetch("/__content/compile-composition", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Content-Assembler": "compile",
      },
      body: JSON.stringify(ticket),
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
    response.epoch !== ticket.epoch ||
    response.revision !== ticket.revision ||
    response.request_id !== ticket.request_id ||
    response.source_digest !== sourceDigest ||
    response.target !== "composition-yaml-v1"
  )
    throw new Error("컴파일 결과가 요청한 원본 리비전과 일치하지 않습니다.");
  if (
    response.status === "compiled" &&
    typeof response.artifact === "string" &&
    response.artifact.length > 0 &&
    response.artifact_digest === (await digest(response.artifact)) &&
    !("diagnostic" in response)
  )
    return response as CompileResult;
  if (
    response.status === "rejected" &&
    diagnostic(response.diagnostic) &&
    response.diagnostic.source_digest === sourceDigest &&
    !("artifact" in response)
  )
    return response as CompileResult;
  throw new Error("컴파일 응답의 형식 또는 내용 식별자가 올바르지 않습니다.");
}
