import type { Diagnostic, GraphDocument, GraphNode } from "../types";
import { inputSources } from "./connections";
import { flowPaths } from "./flow-paths";

export function pairDiagnostics(doc: GraphDocument): Diagnostic[] {
  const out: Diagnostic[] = [];
  const source = inputSources(doc);
  const { mustPass, mayPass } = flowPaths(doc);
  const issue = (
    n: GraphNode,
    code: string,
    message: string,
    portId: string | null = null,
  ) =>
    out.push({
      code,
      nodeId: n.id,
      portId,
      message: `${n.title || n.kind} · ${message}`,
    });
  const requireKind = (n: GraphNode, port: string, kind: string) => {
    const from = source(n, port);
    if (from && from.kind !== kind)
      issue(n, "pair-source", "이 입력의 출처 종류가 올바르지 않습니다.", port);
    return from?.kind === kind ? from : undefined;
  };
  const pulses = doc.nodes.filter((n) => n.kind === "skill-pair-pulse");
  for (const n of doc.nodes) {
    if (n.kind === "skill-paired-areas") {
      const use = requireKind(n, "use", "skill-resonance-admit");
      const permit = use && source(use, "permit"),
        request = permit && source(permit, "request"),
        selection = request && source(request, "selection");
      if (
        use &&
        (!mustPass(n, use, "out") ||
          selection?.kind !== "skill-two-tile-selection")
      )
        issue(
          n,
          "pair-use-scope",
          "두 타일을 함께 접수한 같은 성공 흐름에서 생성하세요.",
          "use",
        );
      const duration = requireKind(n, "duration", "skill-seconds");
      if (duration?.values.value === 0)
        issue(
          n,
          "pair-duration",
          "두 영역의 수명은 0초보다 커야 합니다.",
          "duration",
        );
    }
    if (["skill-pair-pulse", "skill-pair-intersection"].includes(n.kind))
      requireKind(n, "pair", "skill-paired-areas");
    if (n.kind === "skill-pair-pulse") {
      const interval = requireKind(n, "interval", "skill-seconds");
      if (interval?.values.value === 0)
        issue(
          n,
          "pair-interval",
          "회복 주기는 0초보다 커야 합니다.",
          "interval",
        );
      if (
        pulses.some(
          (p) =>
            p.id !== n.id && source(p, "pair")?.id === source(n, "pair")?.id,
        )
      )
        issue(
          n,
          "pair-pulse-duplicate",
          "같은 두 영역은 하나의 동기 회복 주기를 공유해야 합니다.",
          "pair",
        );
    }
    if (["skill-pair-base-heal", "skill-pair-transfer"].includes(n.kind)) {
      const pulse = requireKind(n, "pulse", "skill-pair-pulse");
      if (pulse && !mustPass(n, pulse, "out"))
        issue(
          n,
          "pair-pulse-scope",
          "같은 회복 주기의 성공 흐름이 필요합니다.",
          "pulse",
        );
      if (
        pulse &&
        doc.nodes.some(
          (p) =>
            p.id !== n.id &&
            p.kind === n.kind &&
            source(p, "pulse")?.id === pulse.id,
        )
      )
        issue(
          n,
          "pair-settlement-duplicate",
          "기본 회복과 추가 회복은 각각 주기당 한 번만 확정하세요.",
          "pulse",
        );
      if (pulses.some((p) => p.id !== pulse?.id && mayPass(n, p, "out")))
        issue(
          n,
          "pair-pulse-scope",
          "서로 다른 회복 주기를 합칠 수 없습니다.",
          "in",
        );
      if (n.kind === "skill-pair-base-heal") {
        const power = source(n, "power");
        if (power?.kind === "skill-count-healing") {
          const count = source(power, "count"),
            pair = pulse && source(pulse, "pair");
          if (
            !count ||
            !pair ||
            source(count, "receipt")?.id !== source(pair, "use")?.id ||
            !mustPass(pair, count, "out")
          )
            issue(
              n,
              "pair-power-scope",
              "생성 전에 같은 사용에서 확정한 소비 개수만 회복량에 쓸 수 있습니다.",
              "power",
            );
        }
      }
      if (n.kind === "skill-pair-transfer") {
        const result = requireKind(n, "result", "skill-pair-base-heal");
        if (
          result &&
          (source(result, "pulse")?.id !== pulse?.id ||
            !mustPass(n, result, "out"))
        )
          issue(
            n,
            "pair-result-scope",
            "같은 주기의 양쪽 기본 회복이 모두 확정된 결과만 전달하세요.",
            "result",
          );
      }
    }
    if (n.kind === "skill-pair-intersection")
      requireKind(n, "buff", "skill-attack-speed-buff");
    if (n.kind === "skill-attack-speed-buff" && n.values.points === 0)
      issue(n, "pair-buff-positive", "공격 속도 증가량은 0보다 커야 합니다.");
    if (
      [
        "delay",
        "skill-resonance-stage",
        "skill-resonance-end",
        "skill-sequence-channel",
        "skill-channel-begin",
        "skill-paired-areas",
      ].includes(n.kind) &&
      pulses.some((p) => mayPass(n, p, "out"))
    )
      issue(
        n,
        "pair-settlement-sync",
        "두 영역의 기본/추가 회복 확정 흐름에는 대기·새 시전·사용 종료를 넣을 수 없습니다.",
        "in",
      );
  }
  return out;
}
