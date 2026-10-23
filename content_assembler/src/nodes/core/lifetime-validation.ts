import type { Diagnostic, GraphDocument, GraphNode } from "../types";
import { inputSources } from "./connections";
import { flowPaths } from "./flow-paths";
import { planStructure } from "./plan-structure";

// Lifetimes created by P3 also constrain existing consumers of an admitted use.
export function lifetimeDiagnostics(doc: GraphDocument): Diagnostic[] {
  const source = inputSources(doc);
  const { mayPass } = flowPaths(doc);
  const { chargeForVolley } = planStructure(doc);
  const owners = doc.nodes.filter((n) =>
    [
      "skill-sequence-channel",
      "skill-channel-begin",
      "skill-paired-areas",
      "skill-run-action",
      "skill-run-portals",
    ].includes(n.kind),
  );
  const out: Diagnostic[] = [];
  const issue = (n: GraphNode, code: string, message: string) =>
    out.push({
      code,
      nodeId: n.id,
      portId: "use",
      message: `${n.title || n.kind} · ${message}`,
    });
  for (const n of doc.nodes) {
    const use = source(n, "use");
    if (!use || use.kind !== "skill-resonance-admit") continue;
    const bound = owners.filter((o) => source(o, "use")?.id === use.id);
    if (owners.includes(n) && bound.length > 1)
      issue(
        n,
        "use-lifetime-owner",
        "한 사용에 둘 이상의 집중/영역/행동 실행 수명을 연결할 수 없습니다.",
      );
    if (n.kind === "skill-resonance-end" && bound.length)
      issue(
        n,
        "use-lifetime-end",
        "집중/영역/행동 실행이 사용을 종료하므로 별도 사용 종료를 연결할 수 없습니다.",
      );
    if (
      bound.some((o) =>
        o.kind === "skill-paired-areas"
          ? mayPass(n, o, "out")
          : o.kind === "skill-run-portals"
            ? mayPass(n, o, "expired") ||
              mayPass(n, o, "cancelled") ||
              doc.nodes.some(
                (p) =>
                  p.kind === "skill-serial-projectiles" &&
                  chargeForVolley(source(p, "volley"))?.id === o.id &&
                  (mayPass(n, p, "completed") || mayPass(n, p, "cancelled")),
              )
            : mayPass(n, o, "completed") || mayPass(n, o, "cancelled"),
      )
    )
      issue(
        n,
        "use-lifetime-closed",
        "이미 완료/중단된 사용을 다시 실행할 수 없습니다.",
      );
    if (
      owners.includes(n) &&
      doc.nodes.some((p) => {
        if (p.kind === "skill-resonance-end")
          return source(p, "use")?.id === use.id && mayPass(n, p, "out");
        if (
          p.kind !== "skill-resonance-pay" ||
          p.values.insufficient !== "stop-use"
        )
          return false;
        const step = source(p, "step");
        return (
          step &&
          source(step, "use")?.id === use.id &&
          mayPass(n, p, "rejected")
        );
      })
    )
      issue(
        n,
        "use-lifetime-closed",
        "이미 종료된 접수의 사용 수명을 새로 시작할 수 없습니다.",
      );
  }
  return out;
}
