import type { Diagnostic, GraphDocument, GraphNode } from "../types";
import { inputEndpoints, inputSources } from "./connections";
import { requestKinds } from "./resource-catalog";
import { flowPaths } from "./flow-paths";
import {
  resonanceCatalog,
  resonanceInputActive,
  resonanceRequiredPorts,
} from "./resonance-catalog";

const definitions = new Map(resonanceCatalog.map((d) => [d.kind, d]));
const kind = (name: string) => `skill-resonance-${name}`;

// Static authoring checks only. Runtime amounts, owner IDs and occurrence IDs
// remain the responsibility of the explicit admission/payment contracts.
export function resonanceDiagnostics(doc: GraphDocument): Diagnostic[] {
  const out: Diagnostic[] = [];
  const source = inputSources(doc),
    endpoint = inputEndpoints(doc);
  const { mustPass, ancestors } = flowPaths(doc);
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
      message: `${n.title || definitions.get(n.kind)?.label || n.kind} · ${message}`,
    });
  const requireSource = (
    n: GraphNode,
    port: string,
    kinds: string[],
    flowPort?: string,
  ) => {
    const p = source(n, port);
    if (!p) return;
    if (!kinds.includes(p.kind) || (flowPort && !mustPass(n, p, flowPort))) {
      issue(
        n,
        "skill-path-scope",
        "이 입력값을 만든 분기를 모든 실행 경로가 통과해야 합니다.",
        port,
      );
      return;
    }
    return p;
  };
  const qualified = (admit: GraphNode) => source(admit, "permit");
  const lookup = (admit: GraphNode) => {
    const q = qualified(admit);
    return q && source(q, "vacancy");
  };
  const requestFor = (admit: GraphNode) => {
    const q = qualified(admit);
    return q && source(q, "request");
  };
  const readFor = (cost: GraphNode) => source(cost, "snapshot");
  const admittedUseFor = (n: GraphNode) => {
    const use = source(n, "use");
    return use?.kind === kind("admit") ? use : undefined;
  };
  const inUse = (n: GraphNode, use: GraphNode) =>
    mustPass(n, use, "out") ||
    doc.nodes.some(
      (event) =>
        event.kind === kind("periodic") &&
        admittedUseFor(event)?.id === use.id &&
        (mustPass(n, event, "out") || mustPass(n, event, "completed")),
    );
  const closedBefore = (n: GraphNode, use: GraphNode) => {
    const upstream = ancestors(n);
    return doc.nodes.some((p) => {
      if (!upstream.has(p.id)) return false;
      if (p.kind === kind("end")) return admittedUseFor(p)?.id === use.id;
      if (p.kind !== kind("pay") || p.values.insufficient !== "stop-use")
        return false;
      const step = source(p, "step");
      // A branch after stop-use may not resume the ended use. A merge containing
      // that branch is also invalid, even if a success branch reaches it.
      return (
        step &&
        admittedUseFor(step)?.id === use.id &&
        reachableFrom(p.id, "rejected", n.id)
      );
    });
  };
  const reachableFrom = (id: string, port: string, target: string) => {
    const pending = doc.edges
      .filter(
        (e) =>
          e.source.node === id &&
          e.source.port === port &&
          e.target.port === "in",
      )
      .map((e) => e.target.node);
    const seen = new Set<string>();
    while (pending.length) {
      const next = pending.pop()!;
      if (next === target) return true;
      if (seen.has(next)) continue;
      seen.add(next);
      pending.push(
        ...doc.edges
          .filter((e) => e.source.node === next && e.target.port === "in")
          .map((e) => e.target.node),
      );
    }
    return false;
  };
  // Follow data producers to detect cross-request values and success-scoped
  // receipts even when a cost/requirement or reroute sits between them.
  const dependencies = (n: GraphNode) => {
    const found = new Map<string, GraphNode>();
    const visit = (current: GraphNode) => {
      for (const edge of doc.edges.filter(
        (e) =>
          e.target.node === current.id &&
          e.target.port !== "in" &&
          resonanceInputActive(current, e.target.port),
      )) {
        const p = source(current, edge.target.port);
        if (!p || found.has(p.id)) continue;
        found.set(p.id, p);
        if (![kind("result"), kind("read"), kind("input")].includes(p.kind))
          visit(p);
      }
    };
    visit(n);
    return [...found.values()];
  };
  const checkValues = (
    n: GraphNode,
    expectedRequest: GraphNode | undefined,
    use?: GraphNode,
    execution = true,
  ) => {
    for (const p of dependencies(n)) {
      if ([kind("read"), kind("input")].includes(p.kind)) {
        const request = source(p, "request");
        if (expectedRequest && request && request.id !== expectedRequest.id)
          issue(
            n,
            "resonance-owner-scope",
            "공명 값과 실행의 요청·소유자·출처가 다릅니다.",
          );
      }
      if (execution && p.kind === kind("result")) {
        const payment = source(p, "receipt");
        const step =
          payment?.kind === kind("pay") ? source(payment, "step") : undefined;
        const paidUse =
          payment?.kind === kind("admit")
            ? payment
            : step && admittedUseFor(step);
        if (!mustPass(n, p, "out") || (use && paidUse?.id !== use.id))
          issue(
            n,
            "resonance-result-scope",
            "같은 사용의 지불 성공 경로에서 확정한 결과만 사용할 수 있습니다.",
          );
      }
    }
  };
  const literal = (n: GraphNode, port: string): number | undefined => {
    const p = source(n, port);
    if (
      p?.kind === kind("amount") ||
      p?.kind === "number" ||
      p?.kind === "skill-seconds"
    )
      return typeof p.values.value === "number" ? p.values.value : undefined;
  };

  for (const n of doc.nodes) {
    if (!definitions.has(n.kind)) continue;
    for (const port of resonanceRequiredPorts(n))
      if (!source(n, port))
        issue(
          n,
          "resonance-input-required",
          "선택한 방식에 필요한 입력이 연결되지 않았습니다.",
          port,
        );

    if (n.kind === kind("read")) {
      requireSource(n, "request", requestKinds);
      requireSource(n, "settings", [kind("settings")]);
    }
    if (n.kind === kind("input")) requireSource(n, "request", requestKinds);
    if (n.kind === kind("cost")) {
      const read = requireSource(n, "snapshot", [kind("read")]);
      if (n.values.mode === "selected") {
        const selected = source(n, "selected");
        if (
          selected &&
          (selected.kind !== kind("input") ||
            endpoint(n, "selected")?.port !== "amount")
        )
          issue(
            n,
            "resonance-selection-source",
            "선택량은 같은 요청의 투입량 출력을 연결해야 합니다.",
            "selected",
          );
        const min = literal(n, "minimum"),
          max = literal(n, "maximum");
        if (min !== undefined && max !== undefined && min > max)
          issue(
            n,
            "resonance-selection-range",
            "선택 최소량이 최대량보다 큽니다.",
            "maximum",
          );
      }
      if (n.values.mode === "count") {
        const count = literal(n, "count");
        if (count !== undefined && (!Number.isSafeInteger(count) || count < 0))
          issue(
            n,
            "resonance-count",
            "소비 횟수는 0 이상의 안전한 정수여야 합니다.",
            "count",
          );
      }
      if (read) checkValues(n, source(read, "request"), undefined, false);
    }
    if (n.kind === kind("qualify")) {
      const l = requireSource(n, "vacancy", ["skill-lookup"], "absent");
      const read = requireSource(n, "snapshot", [kind("read")]);
      const cost = requireSource(n, "cost", [kind("cost")]);
      requireSource(n, "requirement", [kind("requirement")]);
      const request = requireSource(n, "request", requestKinds, "out");
      if (request && l && source(l, "request")?.id !== request.id)
        issue(
          n,
          "resonance-owner-scope",
          "비활성 조회와 사용 요청이 다릅니다.",
          "vacancy",
        );
      if (request && read && source(read, "request")?.id !== request.id)
        issue(
          n,
          "resonance-owner-scope",
          "공명 조회와 비활성 조회의 요청·소유자·출처가 다릅니다.",
          "snapshot",
        );
      if (read && cost && readFor(cost)?.id !== read.id)
        issue(
          n,
          "resonance-cost-scope",
          "조건 검사와 비용 계산은 같은 공명 조회를 참조해야 합니다.",
          "cost",
        );
      checkValues(n, request);
    }
    if (n.kind === kind("admit")) {
      const permit = requireSource(n, "permit", [kind("qualify")], "out");
      if (permit) {
        const request = source(permit, "request");
        const peers = doc.nodes.filter(
          (p) => p.kind === kind("admit") && requestFor(p)?.id === request?.id,
        );
        if (request && peers.length > 1)
          issue(
            n,
            "resonance-duplicate-admission",
            "같은 요청에 접수 지불이 여러 개 있습니다. 접수 경계를 하나로 합치세요.",
            "permit",
          );
        checkValues(n, request);
      }
    }
    if (
      [
        kind("install"),
        kind("stage"),
        kind("end"),
        "skill-ground-area",
      ].includes(n.kind)
    ) {
      const use = requireSource(n, "use", [kind("admit")]);
      if (use && (!inUse(n, use) || closedBefore(n, use)))
        issue(
          n,
          "resonance-use-scope",
          "종료되지 않은 같은 사용의 성공 흐름에서만 실행할 수 있습니다.",
          "use",
        );
      if (use && n.kind === kind("install")) {
        const l = lookup(use);
        const expected = l && source(l, "state"),
          actual = source(n, "state");
        if (actual && (!l || (expected && expected.id !== actual.id)))
          issue(
            n,
            "skill-start-state",
            "첫 사용 허가와 시작할 상태 선언이 다릅니다.",
            "state",
          );
      }
    }
    if (n.kind === kind("periodic")) {
      requireSource(n, "use", [kind("admit")]);
      const interval = literal(n, "interval");
      if (interval !== undefined && interval <= 0)
        issue(
          n,
          "resonance-period",
          "지불 주기는 0초보다 커야 합니다.",
          "interval",
        );
      if (
        n.values.limit === "count" &&
        typeof n.values.count === "number" &&
        !Number.isSafeInteger(n.values.count)
      )
        issue(
          n,
          "resonance-count",
          "최대 지불 회수는 1 이상의 안전한 정수여야 합니다.",
        );
      if (
        n.values.limit !== "count" &&
        doc.edges.some(
          (e) => e.source.node === n.id && e.source.port === "completed",
        )
      )
        issue(
          n,
          "resonance-period-completion",
          "지정 회수 방식에서만 주기 완료 흐름을 사용할 수 있습니다.",
          "completed",
        );
      const consumers = doc.nodes.filter(
        (p) => p.kind === kind("pay") && source(p, "step")?.id === n.id,
      );
      if (!consumers.length)
        issue(
          n,
          "resonance-period-payment",
          "주기 사건의 회차 권리를 후속 지불에 연결하세요.",
          "step",
        );
    }
    if (n.kind === kind("pay")) {
      const step = requireSource(
        n,
        "step",
        [kind("stage"), kind("periodic")],
        "out",
      );
      const cost = requireSource(n, "cost", [kind("cost")]);
      const use = step && admittedUseFor(step);
      if (use) {
        if (closedBefore(n, use))
          issue(
            n,
            "resonance-use-scope",
            "종료된 사용에는 후속 비용을 지불할 수 없습니다.",
            "step",
          );
        const q = qualified(use),
          initialRead = q && source(q, "snapshot");
        const read = cost && readFor(cost);
        if (
          initialRead &&
          read &&
          source(initialRead, "settings")?.id !== source(read, "settings")?.id
        )
          issue(
            n,
            "resonance-cost-scope",
            "후속 비용은 이번 사용과 같은 공명 저장 설정을 참조해야 합니다.",
            "cost",
          );
        checkValues(n, requestFor(use), use);
      }
      if (
        step &&
        doc.nodes.filter(
          (p) => p.kind === kind("pay") && source(p, "step")?.id === step.id,
        ).length > 1
      )
        issue(
          n,
          "resonance-duplicate-payment",
          "같은 단계·회차 권리에 지불 노드가 여러 개 있습니다.",
          "step",
        );
    }
    if (n.kind === kind("result")) {
      requireSource(n, "receipt", [kind("admit"), kind("pay")], "out");
    }
  }
  return out;
}
