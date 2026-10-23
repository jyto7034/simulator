import type { GraphDocument } from "../types";
import { p4ValidationContext } from "./p4-validation-context";

const grantKinds = [
  "stock-periodic",
  "relation-periodic",
  "cohort-killed",
  "damage-recovery",
];
export function inventoryDiagnostics(doc: GraphDocument) {
  const c = p4ValidationContext(doc);
  const {
    out,
    source,
    issue,
    requireKind,
    positive,
    positiveField,
    text,
    ofKind,
    mustPass,
    counts,
  } = c;
  for (const n of doc.nodes) {
    if (n.kind === "skill-stock") {
      if (
        n.values.limit === "bounded" &&
        typeof n.values.initial === "number" &&
        typeof n.values.capacity === "number" &&
        n.values.initial > n.values.capacity
      )
        issue(
          n,
          "stock-initial-cap",
          "초기 재고가 지정한 상한보다 큽니다.",
          "capacity",
        );
    }
    if (n.kind === "skill-count-stock") requireKind(n, "bank", ["stock"]);
    if (n.kind === "skill-stock-periodic") {
      const bank = requireKind(n, "bank", ["stock"]);
      positive(n, "interval");
      if (
        bank?.values.lifetime === "deployment" &&
        n.values.scope === "equipped"
      )
        issue(
          n,
          "stock-clock-lifetime",
          "배치 수명 저장소의 주기는 배치 중에만 진행할 수 있습니다.",
          "scope",
        );
    }
    if (n.kind === "skill-resource-credit") {
      const grant = requireKind(n, "grant", grantKinds, "out");
      const bank = requireKind(n, "bank", ["stock", "credit-ledger"]);
      if (grant && bank && source(grant, "bank")?.id !== bank.id)
        issue(
          n,
          "resource-grant-owner",
          "획득 권리에 결속한 저장소와 지급할 저장소가 다릅니다.",
          "bank",
        );
    }
    if (grantKinds.some((k) => n.kind === `skill-${k}`)) {
      const consumers = ofKind("resource-credit").filter(
        (p) => source(p, "grant")?.id === n.id,
      );
      if (consumers.length !== 1)
        issue(
          n,
          "resource-grant-consumer",
          "획득 권리마다 지급 동작을 정확히 하나 연결하세요.",
          "grant",
        );
      const identity =
        n.kind === "skill-relation-periodic"
          ? "relation"
          : n.kind === "skill-cohort-killed"
            ? "cohort"
            : n.kind === "skill-damage-recovery"
              ? "mark"
              : "bank";
      if (
        source(n, identity) &&
        doc.nodes.some(
          (p) =>
            p.id !== n.id &&
            p.kind === n.kind &&
            source(p, identity)?.id === source(n, identity)?.id &&
            source(p, "bank")?.id === source(n, "bank")?.id,
        )
      )
        issue(
          n,
          "resource-event-duplicate",
          "같은 출처 사건과 저장소의 획득 구독이 중복됩니다.",
        );
    }
    if (n.kind === "skill-ally-relation") {
      text(n, "entry");
      requireKind(n, "blocked", ["battle-mode"]);
      if (
        String(n.values.entry).trim() &&
        ofKind("ally-relation").some(
          (p) => p.id !== n.id && p.values.entry === n.values.entry,
        )
      )
        issue(
          n,
          "relation-menu-duplicate",
          "관계 메뉴 항목 ID가 중복됩니다.",
          "entry",
        );
    }
    if (
      [
        "skill-relation-reduction",
        "skill-relation-emergency-shield",
        "skill-relation-periodic",
        "skill-relation-hp-zero",
        "skill-relation-clear",
      ].includes(n.kind)
    )
      requireKind(n, "relation", ["ally-relation"]);
    if (n.kind === "skill-relation-reduction") positiveField(n, "percent");
    if (n.kind === "skill-relation-emergency-shield") {
      positive(n, "duration");
      positiveField(n, "threshold");
      positiveField(n, "shield");
      const r = source(n, "relation");
      if (
        r &&
        ofKind("relation-emergency-shield").filter(
          (p) => source(p, "relation")?.id === r.id,
        ).length > 1
      )
        issue(
          n,
          "relation-shield-record",
          "같은 관계에 전투당 한 번 기록을 중복 선언할 수 없습니다.",
        );
    }
    if (n.kind === "skill-relation-periodic") {
      requireKind(n, "bank", ["stock"]);
      positive(n, "interval");
    }
    if (n.kind === "skill-defer-mode") {
      const event = requireKind(n, "event", ["relation-hp-zero"], "out"),
        mode = requireKind(n, "mode", ["battle-mode"]);
      const relation = event && source(event, "relation");
      if (relation && mode && source(relation, "blocked")?.id !== mode.id)
        issue(
          n,
          "relation-mode-owner",
          "관계의 새 지정을 막는 상태와 전환할 상태를 일치시키세요.",
          "mode",
        );
      if (
        event &&
        ofKind("defer-mode").filter((p) => source(p, "event")?.id === event.id)
          .length > 1
      )
        issue(
          n,
          "relation-transition-duplicate",
          "한 관계 사망 사건의 전환 예약이 중복됩니다.",
        );
    }
    if (n.kind === "skill-mode-entered")
      requireKind(n, "mode", ["battle-mode"]);
    if (n.kind === "skill-relation-clear") {
      const relation = source(n, "relation"),
        mode = relation && source(relation, "blocked");
      if (
        mode &&
        !ofKind("mode-entered").some(
          (e) => source(e, "mode")?.id === mode.id && mustPass(n, e, "out"),
        )
      )
        issue(
          n,
          "relation-clear-scope",
          "해당 관계를 차단하는 상태의 진입 흐름에서 해제하세요.",
          "in",
        );
    }
    if (n.kind === "skill-self-stock-damage") {
      positiveField(n, "percent");
      const count = source(n, "count");
      if (
        count &&
        (count.kind !== "skill-count-stock" ||
          counts(count).some((p) => p.kind === "skill-count-paid"))
      )
        issue(
          n,
          "self-stock-count",
          "상태 진입 자기 피해는 현재 재고 고정을 참조해야 합니다.",
          "count",
        );
      if (!ofKind("mode-entered").some((e) => mustPass(n, e, "out")))
        issue(
          n,
          "self-stock-event",
          "자기 피해는 상태 진입 성공 흐름에서 한 번 처리해야 합니다.",
          "in",
        );
    }
    if (n.kind === "skill-hostile-profile") {
      positive(n, "interval");
      positiveField(n, "speed");
    }
    if (n.kind === "skill-hostile-family-weakness") {
      requireKind(n, "base", ["hostile-profile"]);
      text(n, "family");
      positiveField(n, "percent");
    }
    if (n.kind === "skill-hostile-spawner") {
      requireKind(n, "profile", ["hostile-profile", "hostile-family-weakness"]);
      positive(n, "interval");
    }
    if (n.kind === "skill-cohort-killed") {
      requireKind(n, "cohort", ["hostile-spawner"]);
      const bank = requireKind(n, "bank", ["stock"]);
      if (bank && bank.values.lifetime !== "battle")
        issue(
          n,
          "cohort-reward-lifetime",
          "이미 생성한 적의 처치 보상은 전투 수명 재고에 결속해야 합니다.",
          "bank",
        );
    }
    if (n.kind === "skill-damage-mark") positive(n, "duration");
    if (n.kind === "skill-damage-recovery") {
      requireKind(n, "mark", ["damage-mark"]);
      requireKind(n, "bank", ["credit-ledger", "stock"]);
      const bank = source(n, "bank");
      if (bank?.kind === "skill-stock" && bank.values.lifetime !== "battle")
        issue(
          n,
          "recovery-bank-lifetime",
          "출처별 사망 회수는 전투 수명 저장소를 사용해야 합니다.",
          "bank",
        );
    }
  }
  return out;
}
