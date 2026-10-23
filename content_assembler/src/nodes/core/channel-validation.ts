import type { Diagnostic, GraphDocument, GraphNode } from "../types";
import { inputSources } from "./connections";
import { flowPaths } from "./flow-paths";

export function channelDiagnostics(doc: GraphDocument): Diagnostic[] {
  const out: Diagnostic[] = [];
  const source = inputSources(doc);
  const { mustPass, mayPass, ancestors } = flowPaths(doc);
  const channels = doc.nodes.filter((n) => n.kind === "skill-channel-begin");
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
  for (const n of doc.nodes) {
    if (n.kind === "skill-channel-begin") {
      const use = source(n, "use");
      if (
        use &&
        (use.kind !== "skill-resonance-admit" || !mustPass(n, use, "out"))
      )
        issue(
          n,
          "channel-use-scope",
          "같은 사용의 접수 성공 흐름에서 집중을 시작하세요.",
          "use",
        );
      const path = ancestors(n);
      if (
        use &&
        doc.nodes.some((p) => {
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
          "channel-use-closed",
          "후속 지불 실패로 종료한 사용에서 집중을 시작할 수 없습니다.",
          "use",
        );
      if (
        use &&
        doc.nodes.some(
          (p) =>
            path.has(p.id) &&
            p.kind === "skill-resonance-end" &&
            source(p, "use")?.id === use.id,
        )
      )
        issue(
          n,
          "channel-use-closed",
          "이미 종료한 사용에서 집중을 시작할 수 없습니다.",
          "use",
        );
      if (
        use &&
        channels.some((p) => p.id !== n.id && source(p, "use")?.id === use.id)
      )
        issue(
          n,
          "channel-use-owner",
          "한 사용의 집중 수명 소유자는 하나여야 합니다.",
          "use",
        );
      const duration = source(n, "duration");
      if (duration && duration.kind !== "skill-seconds")
        issue(
          n,
          "channel-duration-source",
          "최대 집중 시간에는 고정 시간 값을 연결하세요.",
          "duration",
        );
      if (duration?.values.value === 0)
        issue(
          n,
          "channel-positive-time",
          "최대 집중 시간은 0초보다 커야 합니다.",
          "duration",
        );
    }
    if (
      [
        "skill-channel-area",
        "skill-channel-cancel",
        "skill-channel-cancel-request",
        "skill-channel-ended",
      ].includes(n.kind)
    ) {
      const owner = source(n, "channel");
      if (
        owner &&
        !["skill-channel-begin", "skill-sequence-channel"].includes(owner.kind)
      )
        issue(
          n,
          "channel-source",
          "집중 시작이 만든 이번 집중을 연결하세요.",
          "channel",
        );
      if (
        owner &&
        ["skill-channel-area", "skill-channel-cancel"].includes(n.kind)
      ) {
        const livePath =
          mustPass(n, owner, "out") ||
          (n.kind === "skill-channel-cancel" &&
            doc.nodes.some(
              (p) =>
                p.kind === "skill-channel-cancel-request" &&
                source(p, "channel")?.id === owner.id &&
                mustPass(n, p, "out"),
            ));
        if (!livePath)
          issue(
            n,
            "channel-flow-scope",
            "같은 살아 있는 집중의 흐름에서만 실행할 수 있습니다.",
            "in",
          );
      }
      if (n.kind === "skill-channel-area") {
        const placement = source(n, "area");
        if (
          placement &&
          (placement.kind !== "skill-range-placement" ||
            placement.values.origin !== "caster" ||
            placement.values.positionTime !== "accepted" ||
            placement.values.facingTime !== "accepted")
        )
          issue(
            n,
            "channel-area-origin",
            "집중 장판은 시전자 위치·방향을 입력 접수 시 고정한 배치가 필요합니다.",
            "area",
          );
      }
      if (
        n.kind === "skill-channel-cancel-request" &&
        !doc.nodes.some(
          (p) =>
            p.kind === "skill-channel-cancel" &&
            source(p, "channel")?.id === owner?.id &&
            mustPass(p, n, "out"),
        )
      )
        issue(
          n,
          "channel-cancel-unhandled",
          "직접 취소 입력을 같은 집중의 취소 동작에 연결하세요.",
          "out",
        );
    }
    if (n.kind === "skill-resonance-end") {
      const use = source(n, "use");
      if (use && channels.some((p) => source(p, "use")?.id === use.id))
        issue(
          n,
          "channel-use-end",
          "집중이 소유한 사용은 집중의 공통 종료 절차가 끝냅니다. 별도 사용 완료를 제거하세요.",
          "use",
        );
    }
  }
  return out;
}
