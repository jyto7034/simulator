import type { Diagnostic, GraphDocument, GraphNode } from "../types";
import { inputSources, inputSourceLists, inputEndpoints } from "./connections";
import { flowPaths } from "./flow-paths";
import { planStructure } from "./plan-structure";

// Shared primitives for the P4 validators, with no runtime interpretation.
export function p4ValidationContext(doc: GraphDocument) {
  const out: Diagnostic[] = [];
  const source = inputSources(doc),
    sources = inputSourceLists(doc),
    endpoint = inputEndpoints(doc);
  const paths = flowPaths(doc),
    structure = planStructure(doc);
  const issue = (
    n: GraphNode,
    code: string,
    message: string,
    portId: string | null = null,
  ) => {
    out.push({
      code,
      nodeId: n.id,
      portId,
      message: `${n.title || n.kind} · ${message}`,
    });
  };
  const required = (n: GraphNode, port: string) => {
    const p = source(n, port);
    if (!p)
      issue(
        n,
        "p4-input-required",
        "선택한 방식에 필요한 입력을 연결하세요.",
        port,
      );
    return p;
  };
  const requireKind = (
    n: GraphNode,
    port: string,
    kinds: string[],
    flow?: string,
  ) => {
    const p = source(n, port);
    if (!p) return;
    if (
      !kinds.map((k) => `skill-${k}`).includes(p.kind) ||
      (flow && !paths.mustPass(n, p, flow))
    ) {
      issue(
        n,
        "p4-source-scope",
        "같은 회차의 올바른 생산자와 성공 흐름을 연결하세요.",
        port,
      );
      return;
    }
    return p;
  };
  const positive = (n: GraphNode, port: string, zero = false) => {
    const p = source(n, port);
    if (
      p &&
      p.kind !== "skill-seconds" &&
      !(
        n.kind === "skill-timed-plan" &&
        port === "duration" &&
        p.kind === "skill-count-duration"
      )
    )
      issue(
        n,
        "p4-time-scope",
        "이 시간은 고정 시간 값 또는 해당 유지 행동의 개수별 시간이어야 합니다.",
        port,
      );
    if (
      p?.kind === "skill-seconds" &&
      typeof p.values.value === "number" &&
      (zero ? p.values.value < 0 : p.values.value <= 0)
    )
      issue(
        n,
        "p4-positive-time",
        zero ? "시간은 0 이상이어야 합니다." : "시간은 0초보다 커야 합니다.",
        port,
      );
  };
  const positiveField = (n: GraphNode, field: string) => {
    const value = n.values[field];
    if (typeof value === "number" && value <= 0)
      issue(n, "p4-positive-value", "이 값은 0보다 커야 합니다.", field);
  };
  const text = (n: GraphNode, field: string) => {
    if (!String(n.values[field] ?? "").trim())
      issue(
        n,
        "p4-identifier-unset",
        "식별자를 입력하세요. 표시 이름으로 대신하지 않습니다.",
        field,
      );
  };
  const ofKind = (name: string) =>
    doc.nodes.filter((n) => n.kind === `skill-${name}`);
  const requestFor = (use: GraphNode) => {
    const permit = source(use, "permit");
    return permit && source(permit, "request");
  };
  const admittedProgram = (n: GraphNode, expected: string[]) => {
    const use = requireKind(n, "use", ["resonance-admit"], "out");
    const program = requireKind(n, "program", expected);
    const permit = use && source(use, "permit");
    if (program && permit && source(permit, "program")?.id !== program.id)
      issue(
        n,
        "p4-program-scope",
        "사용 자격을 검사한 행동 설정과 실제 실행 설정이 다릅니다.",
        "program",
      );
    return { use, program };
  };
  return {
    out,
    source,
    sources,
    endpoint,
    issue,
    required,
    requireKind,
    positive,
    positiveField,
    text,
    ofKind,
    requestFor,
    admittedProgram,
    ...paths,
    ...structure,
  };
}
