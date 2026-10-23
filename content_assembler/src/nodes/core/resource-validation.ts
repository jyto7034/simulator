import type { Diagnostic, GraphDocument, GraphNode } from "../types";
import { inputSources } from "./connections";
import { flowPaths } from "./flow-paths";

export function resourceDiagnostics(doc: GraphDocument): Diagnostic[] {
  const out: Diagnostic[] = [];
  const source = inputSources(doc);
  const { mustPass } = flowPaths(doc);
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
  const termsFor = (use: GraphNode | undefined) => {
    const permit = use && source(use, "permit");
    return permit && source(permit, "terms");
  };
  const requestFor = (use: GraphNode | undefined) => {
    const permit = use && source(use, "permit");
    return permit && source(permit, "request");
  };
  const requireKind = (n: GraphNode, port: string, kind: string) => {
    const from = source(n, port);
    if (from && from.kind !== kind)
      issue(
        n,
        "counter-source",
        "이 입력의 출처 종류가 올바르지 않습니다.",
        port,
      );
    return from?.kind === kind ? from : undefined;
  };
  const autos = doc.nodes.filter(
    (n) => n.kind === "skill-counter-auto-request",
  );
  for (const n of doc.nodes) {
    if (
      [
        "skill-counter-terms",
        "skill-counter-periodic",
        "skill-counter-gain",
        "skill-aim-memory",
        "skill-counter-auto-request",
      ].includes(n.kind)
    )
      requireKind(n, "counter", "skill-battle-counter");
    if (n.kind === "skill-counter-terms") {
      const counter = source(n, "counter");
      const min = n.values.minimum,
        max = n.values.maximum,
        cap = counter?.values.capacity;
      if (
        typeof min === "number" &&
        typeof max === "number" &&
        (min > max || (typeof cap === "number" && max > cap))
      )
        issue(n, "counter-range", "최소 ≤ 최대 ≤ 자원 상한이어야 합니다.");
    }
    if (n.kind === "skill-counter-periodic") {
      const interval = requireKind(n, "interval", "skill-seconds");
      if (interval?.values.value === 0)
        issue(
          n,
          "counter-period",
          "획득 주기는 0초보다 커야 합니다.",
          "interval",
        );
      if (
        doc.nodes.some(
          (p) =>
            p.id !== n.id &&
            p.kind === n.kind &&
            source(p, "counter")?.id === source(n, "counter")?.id,
        )
      )
        issue(
          n,
          "counter-period-owner",
          "같은 자원의 기본 획득 주기는 하나여야 합니다.",
          "counter",
        );
    }
    if (n.kind === "skill-menu-request") {
      const entry =
        typeof n.values.entry === "string" ? n.values.entry.trim() : "";
      if (
        !entry ||
        doc.nodes.some(
          (p) =>
            p.id !== n.id &&
            p.kind === n.kind &&
            p.values.entry === n.values.entry,
        )
      )
        issue(
          n,
          "counter-menu-entry",
          "메뉴 항목 ID는 비어 있지 않고 문서 안에서 고유해야 합니다.",
        );
    }
    if (n.kind === "skill-counter-auto-request") {
      const counter = source(n, "counter"),
        memory = requireKind(n, "memory", "skill-aim-memory");
      if (counter && memory && source(memory, "counter")?.id !== counter.id)
        issue(
          n,
          "counter-owner",
          "자원과 조준 기억은 같은 사본 선언을 참조해야 합니다.",
          "memory",
        );
      if (
        counter &&
        autos.some(
          (p) => p.id !== n.id && source(p, "counter")?.id === counter.id,
        )
      )
        issue(
          n,
          "counter-auto-duplicate",
          "같은 자원 상한의 자동 요청은 하나여야 합니다.",
          "counter",
        );
      const admits = doc.nodes.filter(
        (p) => p.kind === "skill-resonance-admit" && requestFor(p)?.id === n.id,
      );
      if (admits.length !== 1)
        issue(
          n,
          "counter-auto-unhandled",
          "자동 요청을 하나의 접수와 집중 수명에 연결하세요.",
          "request",
        );
      for (const admit of admits) {
        const owners = doc.nodes.filter(
          (p) =>
            p.kind === "skill-sequence-channel" &&
            source(p, "use")?.id === admit.id,
        );
        if (
          owners.length !== 1 ||
          owners[0]?.values.interruption !== "standard"
        )
          issue(
            n,
            "counter-auto-lifetime",
            "자동 요청은 표준 외부 중단을 따르는 하나의 단계 집중이 소유해야 합니다.",
          );
      }
    }
    if (n.kind === "skill-resonance-qualify") {
      const terms = requireKind(n, "terms", "skill-counter-terms"),
        request = source(n, "request");
      const counter = terms && source(terms, "counter");
      if (request?.kind === "skill-counter-auto-request") {
        const requirement = source(n, "requirement"),
          cost = source(n, "cost");
        if (
          !counter ||
          source(request, "counter")?.id !== counter.id ||
          terms?.values.minimum !== counter.values.capacity ||
          terms?.values.maximum !== counter.values.capacity ||
          terms?.values.spend !== "end-all" ||
          requirement?.values.mode !== "none" ||
          cost?.values.mode !== "zero"
        )
          issue(
            n,
            "counter-auto-terms",
            "상한 자동 요청은 같은 자원의 상한 전용·종료 소진 조건과 공명 조건 없음·비용 0을 사용해야 합니다.",
            "terms",
          );
      } else if (terms) {
        if (terms.values.spend === "end-all")
          issue(
            n,
            "counter-end-spend",
            "종료 시 소진은 상한 자동 요청 전용입니다.",
            "terms",
          );
        if (
          counter &&
          autos.some((p) => source(p, "counter")?.id === counter.id) &&
          typeof terms.values.maximum === "number" &&
          typeof counter.values.capacity === "number" &&
          terms.values.maximum >= counter.values.capacity
        )
          issue(
            n,
            "counter-auto-priority",
            "자동 요청을 예약하는 상한에서는 새 수동 입력을 허용할 수 없습니다.",
            "terms",
          );
      }
    }
    if (n.kind === "skill-counter-receipt") {
      const use = requireKind(n, "receipt", "skill-resonance-admit");
      if (
        use &&
        (!mustPass(n, use, "out") ||
          termsFor(use)?.values.spend !== "admission-all")
      )
        issue(
          n,
          "counter-receipt-scope",
          "전량 소비를 확정한 같은 접수 성공 흐름에서 읽으세요.",
          "receipt",
        );
    }
    if (n.kind === "skill-remember-aim") {
      const use = requireKind(n, "use", "skill-resonance-admit"),
        memory = requireKind(n, "memory", "skill-aim-memory");
      const request = requestFor(use),
        selection = request && source(request, "selection"),
        terms = termsFor(use);
      if (
        use &&
        (!mustPass(n, use, "out") ||
          selection?.kind !== "skill-target-selection" ||
          selection.values.mode !== "player-tile" ||
          !terms ||
          source(terms, "counter")?.id !==
            (memory && source(memory, "counter")?.id))
      )
        issue(
          n,
          "counter-aim-scope",
          "같은 자원 사본의 지정 타일 접수 성공 흐름에서만 기록하세요.",
          "use",
        );
    }
    if (n.kind === "skill-counter-gain") {
      const counter = source(n, "counter"),
        amount = source(n, "amount");
      const producers = doc.nodes.filter(
        (p) =>
          ["skill-counter-periodic", "skill-completion-award"].includes(
            p.kind,
          ) && mustPass(n, p, "out"),
      );
      const producer = producers[0];
      let owner: GraphNode | undefined;
      if (producer?.kind === "skill-counter-periodic")
        owner = source(producer, "counter");
      else if (producer) {
        const summary = source(producer, "summary"),
          channel = summary && source(summary, "channel");
        const terms = termsFor(channel && source(channel, "use"));
        owner = terms && source(terms, "counter");
      }
      if (
        producers.length !== 1 ||
        !owner ||
        owner.id !== counter?.id ||
        (producer?.kind === "skill-completion-award" &&
          amount?.id !== producer.id)
      )
        issue(
          n,
          "counter-gain-scope",
          "같은 자원의 주기 또는 같은 시전의 전체 완료 보상에서 한 번 획득하세요.",
          "in",
        );
      if (
        producer?.kind === "skill-counter-periodic" &&
        amount &&
        (amount.kind !== "number" ||
          typeof amount.values.value !== "number" ||
          !Number.isSafeInteger(amount.values.value) ||
          amount.values.value <= 0)
      )
        issue(
          n,
          "counter-gain-amount",
          "주기 획득에는 양수 정수 개수를 연결하세요.",
          "amount",
        );
      if (
        producer &&
        doc.nodes.some(
          (p) =>
            p.id !== n.id && p.kind === n.kind && mustPass(p, producer, "out"),
        )
      )
        issue(
          n,
          "counter-gain-duplicate",
          "같은 사건의 자원 획득을 중복 연결할 수 없습니다.",
          "in",
        );
    }
  }
  return out;
}
