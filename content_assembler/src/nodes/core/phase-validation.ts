import type { Diagnostic, GraphDocument, GraphNode } from "../types";
import { inputSources, inputSourceLists } from "./connections";
import { flowPaths } from "./flow-paths";
import { readTilePattern } from "./tile-pattern";

export function phaseDiagnostics(doc: GraphDocument): Diagnostic[] {
  const out: Diagnostic[] = [];
  const source = inputSources(doc),
    sources = inputSourceLists(doc);
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
      issue(
        n,
        "phase-source",
        "이 입력의 출처 종류가 올바르지 않습니다.",
        port,
      );
    return from?.kind === kind ? from : undefined;
  };
  const chain = (owner: GraphNode) => {
    const phases: GraphNode[] = [];
    let next = source(owner, "first");
    while (
      next &&
      next.kind === "skill-channel-phase" &&
      !phases.some((p) => p.id === next!.id)
    ) {
      phases.push(next);
      next = source(next, "next");
    }
    return { phases, invalid: Boolean(next) };
  };
  const effectArea = (effect: GraphNode) => {
    const query = source(effect, "member");
    return query && source(query, "area");
  };
  const effectOwner = (effect: GraphNode) => {
    const area = effectArea(effect);
    return area?.kind === "skill-phase-area"
      ? source(area, "channel")
      : undefined;
  };
  const countUse = (n: GraphNode) => {
    const count = source(n, "count");
    return count?.kind === "skill-counter-receipt"
      ? source(count, "receipt")
      : undefined;
  };
  for (const n of doc.nodes) {
    if (n.kind === "skill-channel-phase") {
      const duration = requireKind(n, "duration", "skill-seconds");
      if (duration?.values.value === 0)
        issue(
          n,
          "phase-duration",
          "단계 시간은 0초보다 커야 합니다.",
          "duration",
        );
      requireKind(n, "next", "skill-channel-phase");
      const placement = requireKind(n, "area", "skill-range-placement");
      if (
        placement &&
        placement.values.origin === "caster" &&
        (placement.values.positionTime !== "accepted" ||
          placement.values.facingTime !== "accepted")
      )
        issue(
          n,
          "phase-placement",
          "집중 단계의 위치와 방향은 접수 시 고정해야 합니다.",
          "area",
        );
    }
    if (n.kind === "skill-sequence-channel") {
      const use = requireKind(n, "use", "skill-resonance-admit");
      if (use && !mustPass(n, use, "out"))
        issue(
          n,
          "phase-use-scope",
          "같은 접수 성공 흐름에서 집중을 시작하세요.",
          "use",
        );
      requireKind(n, "first", "skill-channel-phase");
      const { phases, invalid } = chain(n);
      if (invalid)
        issue(
          n,
          "phase-sequence",
          "단계 순서는 순환 없는 유한한 단계 정의여야 합니다.",
          "first",
        );
      const permit = use && source(use, "permit"),
        request = permit && source(permit, "request");
      const selection = request && source(request, "selection");
      for (const phase of phases) {
        const placement = source(phase, "area"),
          range = placement && source(placement, "range");
        if (
          ["selected-tile", "selected-direction"].includes(
            String(placement?.values.origin),
          ) &&
          request?.kind !== "skill-counter-auto-request" &&
          !(
            selection?.kind === "skill-target-selection" &&
            selection.values.mode === "player-tile"
          )
        )
          issue(
            n,
            "phase-target-context",
            "지정 타일 단계에는 해당 타일을 접수하는 요청이 필요합니다.",
            "use",
          );
        if (range?.kind === "skill-count-range") {
          const count = source(range, "count");
          if (
            countUse(range)?.id !== use?.id ||
            !count ||
            !mustPass(n, count, "out")
          )
            issue(
              n,
              "phase-count-scope",
              "단계 범위에는 같은 사용에서 확정한 소비 개수만 사용할 수 있습니다.",
              "first",
            );
        }
      }
    }
    if (n.kind === "skill-phase-area") {
      const owner = requireKind(n, "channel", "skill-sequence-channel"),
        phase = requireKind(n, "phase", "skill-channel-phase");
      if (owner && phase && !chain(owner).phases.some((p) => p.id === phase.id))
        issue(
          n,
          "phase-owner",
          "이 집중 순서에 속한 단계만 참조할 수 있습니다.",
          "phase",
        );
    }
    if (n.kind === "skill-cast-outcomes") {
      const owner = requireKind(n, "channel", "skill-sequence-channel");
      for (const [port, kind] of [
        ["damage", "skill-ground-damage"],
        ["healing", "skill-ground-heal"],
      ]) {
        for (const effect of sources(n, port)) {
          if (
            effect.kind !== kind ||
            !owner ||
            effectOwner(effect)?.id !== owner.id
          )
            issue(
              n,
              "phase-result-owner",
              "같은 집중 단계에서 확정한 실제 결과만 집계할 수 있습니다.",
              port,
            );
        }
      }
    }
    if (n.kind === "skill-completion-award") {
      const summary = requireKind(n, "summary", "skill-cast-outcomes"),
        owner = summary && source(summary, "channel");
      if (owner && !mustPass(n, owner, "completed"))
        issue(
          n,
          "phase-natural-completion",
          "같은 집중의 자연 완료 경로에서만 보상하세요.",
          "in",
        );
      if (
        owner &&
        doc.nodes.some(
          (p) =>
            p.id !== n.id &&
            p.kind === n.kind &&
            source(p, "summary") &&
            source(source(p, "summary")!, "channel")?.id === owner.id,
        )
      )
        issue(
          n,
          "phase-award-duplicate",
          "시전별 완료 보상은 한 번만 계산해야 합니다.",
          "summary",
        );
    }
    if (
      [
        "skill-count-healing",
        "skill-count-damage",
        "skill-count-range",
      ].includes(n.kind)
    )
      requireKind(n, "count", "skill-counter-receipt");
    if (["skill-ground-heal", "skill-ground-damage"].includes(n.kind)) {
      const power = source(n, "power");
      if (
        power &&
        ["skill-count-healing", "skill-count-damage"].includes(power.kind)
      ) {
        const owner = effectOwner(n),
          count = source(power, "count");
        if (
          !owner ||
          !count ||
          countUse(power)?.id !== source(owner, "use")?.id ||
          !mustPass(owner, count, "out")
        )
          issue(
            n,
            "phase-count-scope",
            "같은 집중 시작 전에 확정한 소비 개수의 위력만 사용할 수 있습니다.",
            "power",
          );
      }
    }
    if (n.kind === "skill-count-range") {
      const tiers = sources(n, "tiers"),
        use = countUse(n),
        permit = use && source(use, "permit"),
        terms = permit && source(permit, "terms");
      const min = terms?.values.minimum,
        max = terms?.values.maximum;
      const counts = tiers.map((t) => t.values.count);
      if (
        tiers.some((t) => t.kind !== "skill-count-range-tier") ||
        new Set(counts).size !== counts.length ||
        (typeof min === "number" &&
          typeof max === "number" &&
          (tiers.length !== max - min + 1 ||
            counts.some((v) => typeof v !== "number" || v < min || v > max)))
      )
        issue(
          n,
          "phase-range-coverage",
          "접수 가능한 모든 소비 개수를 중복 없이 한 항목씩 연결하세요.",
          "tiers",
        );
      const ordered = [...tiers].sort(
        (a, b) => Number(a.values.count) - Number(b.values.count),
      );
      let previous: Set<string> | undefined;
      for (const tier of ordered) {
        const range = source(tier, "range");
        if (range?.kind !== "skill-range") {
          if (range)
            issue(
              n,
              "phase-range-pattern",
              "개수별 범위에는 명시적 타일 패턴을 연결하세요.",
              "tiers",
            );
          continue;
        }
        const pattern = readTilePattern(String(range.values.pattern));
        if (!pattern.assigned) {
          previous = undefined;
          continue;
        }
        const cells = new Set(pattern.cells.map((c) => `${c.x},${c.y}`));
        if (range.values.includeAnchor) cells.add("0,0");
        if (previous && [...previous].some((c) => !cells.has(c)))
          issue(
            n,
            "phase-range-shrinks",
            "높은 소비 개수의 범위가 낮은 개수의 범위를 포함해야 합니다.",
            "tiers",
          );
        previous = cells;
      }
    }
    if (n.kind === "skill-ground-pull") {
      const relation = source(n, "member"),
        filter = relation && source(relation, "filter");
      if (filter && filter.kind !== "skill-corrupted-filter")
        issue(
          n,
          "phase-pull-filter",
          "흡인은 침식체 적 조건의 대상 관계가 필요합니다.",
          "member",
        );
      if (n.values.tiles === 0)
        issue(n, "phase-pull-distance", "흡인 거리는 0보다 커야 합니다.");
    }
    if (
      ["skill-channel-cancel-request", "skill-channel-cancel"].includes(n.kind)
    ) {
      const owner = source(n, "channel"),
        use = owner && source(owner, "use"),
        permit = use && source(use, "permit"),
        request = permit && source(permit, "request");
      if (request?.kind === "skill-counter-auto-request")
        issue(
          n,
          "phase-auto-cancel",
          "상한 자동 시전에는 수동 취소를 연결할 수 없습니다.",
          "channel",
        );
    }
  }
  // A query's per-target branch is synchronous; its completion must not reuse a member.
  const queries = doc.nodes.filter((n) => n.kind === "skill-ground-targets");
  for (const n of doc.nodes) {
    if (
      [
        "delay",
        "skill-resonance-stage",
        "skill-resonance-end",
        "skill-sequence-channel",
        "skill-channel-begin",
        "skill-paired-areas",
      ].includes(n.kind) &&
      queries.some((q) => mayPass(n, q, "out"))
    )
      issue(
        n,
        "ground-query-sync",
        "대상별 동기 처리 안에서 대기·새 시전·사용 종료를 할 수 없습니다.",
        "in",
      );
    const relation = source(n, "member");
    if (
      relation &&
      queries.some((q) => q.id !== relation.id && mustPass(n, q, "out"))
    )
      issue(
        n,
        "ground-query-member",
        "다른 대상 조회의 개별 관계를 섞어 사용할 수 없습니다.",
        "member",
      );
  }
  return out;
}
