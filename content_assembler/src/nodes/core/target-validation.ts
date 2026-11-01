import type { Diagnostic, GraphDocument, GraphNode } from "../types";
import { inputSources } from "./connections";
import { flowPaths } from "./flow-paths";

export function targetDiagnostics(doc: GraphDocument): Diagnostic[] {
  const out: Diagnostic[] = [];
  const source = inputSources(doc);
  const { mustPass, mayPass, ancestors } = flowPaths(doc);
  const nodes = new Map(doc.nodes.map((n) => [n.id, n]));
  const queries = doc.nodes.filter((n) => n.kind === "skill-range-targets");
  const issue = (n: GraphNode, code: string, message: string, portId: string) =>
    out.push({
      code,
      nodeId: n.id,
      portId,
      message: `${n.title || n.kind} · ${message}`,
    });
  const requireKind = (n: GraphNode, port: string, kind: string) => {
    const from = source(n, port);
    if (from && from.kind !== kind)
      issue(
        n,
        "target-source",
        "이 입력의 출처 종류가 올바르지 않습니다.",
        port,
      );
    return from?.kind === kind ? from : undefined;
  };
  const itemScopes = (n: GraphNode) =>
    queries.filter((q) => mustPass(n, q, "each"));
  const effects = (state: GraphNode) =>
    doc.edges
      .filter((e) => e.target.node === state.id && e.target.port === "effects")
      .map((e) => {
        const producer = nodes.get(e.source.node);
        return producer?.kind.startsWith("reroute-")
          ? source(producer, "in")
          : producer;
      })
      .filter((n): n is GraphNode => Boolean(n));
  const closedUse = (n: GraphNode, use: GraphNode) =>
    doc.nodes.some((p) => {
      if (p.kind === "skill-resonance-end")
        return source(p, "use")?.id === use.id && mayPass(n, p, "out");
      if (
        p.kind === "skill-resonance-pay" &&
        p.values.insufficient === "stop-use"
      ) {
        const step = source(p, "step");
        return (
          step &&
          source(step, "use")?.id === use.id &&
          mayPass(n, p, "rejected")
        );
      }
      return false;
    });
  for (const n of doc.nodes) {
    if (n.kind === "skill-range-targets") {
      if (queries.some((q) => mayPass(n, q, "each")))
        issue(
          n,
          "target-nested-query",
          "대상별 흐름 안에 다른 대상 조회를 중첩할 수 없습니다.",
          "in",
        );
      const use = requireKind(n, "use", "skill-resonance-admit");
      if (use && !mustPass(n, use, "out"))
        issue(
          n,
          "target-use-scope",
          "같은 사용의 접수 성공 흐름이 필요합니다.",
          "use",
        );
      if (use && closedUse(n, use))
        issue(
          n,
          "target-use-closed",
          "종료한 사용에서 대상을 조회할 수 없습니다.",
          "use",
        );
      const area = requireKind(n, "area", "skill-range-placement");
      requireKind(n, "filter", "skill-ground-filter");
      if (area?.values.origin === "selected-tile" && use) {
        const permit = source(use, "permit");
        const request = permit && source(permit, "request");
        const selection = request && source(request, "selection");
        if (selection?.values.mode !== "player-tile")
          issue(
            n,
            "target-selection",
            "이번 사용에 지정 타일 선택이 필요합니다.",
            "area",
          );
      }
    }
    if (n.kind === "skill-target-state") {
      const duration = requireKind(n, "duration", "skill-seconds");
      if (duration?.values.value === 0)
        issue(
          n,
          "target-positive-time",
          "공통 지속 시간은 0초보다 커야 합니다.",
          "duration",
        );
      const kinds = new Set<string>();
      for (const effect of effects(n)) {
        if (
          ![
            "skill-target-taunt",
            "skill-target-speed",
            "skill-target-attack-limit",
            "skill-attribute-effect",
            "skill-hp-floor-effect",
            "skill-personal-shield-effect",
          ].includes(effect.kind)
        )
          issue(
            n,
            "target-effect-source",
            "지원하는 대상 효과 설정을 연결하세요.",
            "effects",
          );
        const key =
          effect.kind === "skill-attribute-effect"
            ? `${effect.kind}:${effect.values.attribute}`
            : effect.kind;
        if (kinds.has(key))
          issue(
            n,
            "target-effect-duplicate",
            "같은 효과 종류를 한 상태에 중복 연결할 수 없습니다.",
            "effects",
          );
        kinds.add(key);
      }
      if (kinds.has("skill-target-taunt") && n.values.sourceExit !== "end")
        issue(
          n,
          "target-taunt-owner",
          "도발은 시전자 사망·철수에 함께 종료해야 합니다.",
          "effects",
        );
    }
    if (n.kind === "skill-target-speed" && n.values.percent === 0)
      issue(
        n,
        "target-positive-speed",
        "이동 속도 증가율은 0보다 커야 합니다.",
        "effect",
      );
    if (n.kind === "skill-target-attack-limit" && n.values.tiles === 0)
      issue(
        n,
        "target-positive-distance",
        "공격 거리 상한은 0보다 커야 합니다.",
        "effect",
      );
    if (n.kind === "skill-target-apply") {
      const state = requireKind(n, "state", "skill-target-state");
      const query = requireKind(n, "target", "skill-range-targets");
      if (
        state &&
        query &&
        effects(state).some((e) => e.kind === "skill-target-taunt") &&
        source(query, "filter")?.values.side !== "enemies"
      )
        issue(
          n,
          "target-taunt-enemies",
          "시전자 집결 도발은 적만 조회하는 조건이 필요합니다.",
          "target",
        );
      const scopes = itemScopes(n);
      if (query && (scopes.length !== 1 || scopes[0].id !== query.id))
        issue(
          n,
          "target-item-scope",
          "같은 조회의 대상별 흐름에서만 상태를 적용하세요.",
          "target",
        );
      const use = query && source(query, "use");
      if (use && closedUse(n, use))
        issue(
          n,
          "target-use-closed",
          "이미 종료한 사용의 대상에 새 상태를 부여할 수 없습니다.",
          "target",
        );
    }
    if (n.kind === "skill-target-clear" || n.kind === "skill-target-ended") {
      const apply = requireKind(n, "instance", "skill-target-apply");
      if (apply && n.kind === "skill-target-clear") {
        const ended = doc.nodes.some(
          (p) =>
            p.kind === "skill-target-ended" &&
            source(p, "instance")?.id === apply.id &&
            mustPass(n, p, "out"),
        );
        const query = source(apply, "target");
        const scopes = itemScopes(n);
        if (
          !ended &&
          (!mustPass(n, apply, "applied") ||
            scopes.length !== 1 ||
            scopes[0].id !== query?.id)
        )
          issue(
            n,
            "target-instance-scope",
            "같은 적용 성공 또는 해당 활성 종료 문맥이 필요합니다.",
            "instance",
          );
      }
    }
    const scopes = queries.filter((q) => mayPass(n, q, "each"));
    if (
      scopes.length &&
      (n.kind === "delay" || n.kind === "skill-resonance-step")
    )
      issue(
        n,
        "target-synchronous",
        "대상별 조회 흐름에는 시간 대기를 둘 수 없습니다.",
        "in",
      );
    if (
      n.kind === "skill-resonance-end" &&
      scopes.some((q) => source(q, "use")?.id === source(n, "use")?.id)
    )
      issue(
        n,
        "target-end-per-item",
        "사용 완료는 대상별 흐름 대신 전체 처리 완료에 연결하세요.",
        "in",
      );
    if (
      n.kind === "skill-target-apply" &&
      doc.nodes.some(
        (p) => p.kind === "skill-target-ended" && ancestors(n).has(p.id),
      )
    )
      issue(
        n,
        "target-ended-reapply",
        "종료 사건으로 이전 조회 대상에 상태를 재적용할 수 없습니다.",
        "in",
      );
  }
  return out;
}
