import type { GraphDocument } from "../types";
import { p4ValidationContext } from "./p4-validation-context";

export function mobileEffectDiagnostics(doc: GraphDocument) {
  const {
    out,
    source,
    sources,
    issue,
    requireKind,
    required,
    positive,
    text,
    ofKind,
  } = p4ValidationContext(doc);
  const profiles = ["mobile-effect", "mobile-variant"];
  for (const n of doc.nodes) {
    if (n.kind === "skill-mobile-effect") {
      positive(n, "delay");
      if (n.values.motion === "wait-leap") {
        required(n, "lifetime");
        positive(n, "lifetime");
      } else if (source(n, "lifetime"))
        issue(
          n,
          "mobile-unused-lifetime",
          "추격 폭발은 폭발 지연을 사용하며 별도 대기 수명을 받지 않습니다.",
          "lifetime",
        );
    }
    if (n.kind === "skill-mobile-variant") {
      let base = requireKind(n, "base", profiles);
      const seen = new Set([n.id]);
      while (base?.kind === "skill-mobile-variant") {
        if (seen.has(base.id)) {
          issue(
            n,
            "mobile-variant-cycle",
            "이동 효과 변형이 서로를 순환 참조합니다.",
            "base",
          );
          break;
        }
        seen.add(base.id);
        base = source(base, "base");
      }
    }
    if (n.kind === "skill-fusion-family") {
      const tiers = sources(n, "tiers"),
        levels = tiers.map((t) => t.values.minimum);
      if (new Set(levels).size !== levels.length)
        issue(
          n,
          "fusion-tier-duplicate",
          "합체 최소 수가 중복되었습니다.",
          "tiers",
        );
      if (!levels.includes(1))
        issue(
          n,
          "fusion-tier-base",
          "1단위부터 사용할 기본 항목이 필요합니다.",
          "tiers",
        );
      for (const tier of tiers)
        if (tier.kind !== "skill-fusion-tier")
          issue(n, "fusion-tier-kind", "합체 항목을 연결하세요.", "tiers");
    }
    if (n.kind === "skill-fusion-tier") requireKind(n, "profile", profiles);
    if (["skill-place-fusion-plan", "skill-fusion-death-seed"].includes(n.kind))
      requireKind(n, "family", ["fusion-family"]);
    if (n.kind === "skill-fusion-death-seed") {
      text(n, "relic");
      positive(n, "duration");
    }
    if (
      ["skill-spawn-mobile-plan", "skill-status-death-mobile"].includes(n.kind)
    ) {
      requireKind(n, "profile", profiles);
      requireKind(n, "limit", ["count-positive"]);
      const same = doc.nodes.filter(
        (p) =>
          ["skill-spawn-mobile-plan", "skill-status-death-mobile"].includes(
            p.kind,
          ) && source(p, "profile")?.id === source(n, "profile")?.id,
      );
      if (same.some((p) => source(p, "limit")?.id !== source(n, "limit")?.id))
        issue(
          n,
          "mobile-shared-limit",
          "같은 이동 효과의 기본·사망 생성은 같은 동시 상한을 참조해야 합니다.",
          "limit",
        );
    }
    if (
      n.kind === "skill-spawn-mobile-plan" &&
      !ofKind("on-hit-plan").some((p) => source(p, "after")?.id === n.id)
    )
      issue(
        n,
        "mobile-hit-context",
        "기본 생성은 유효 적중 후속 문맥에 연결하세요.",
        "profile",
      );
    if (n.kind === "skill-status-death-mobile") {
      text(n, "relic");
      for (const status of sources(n, "states"))
        if (status.kind !== "skill-reactive-status")
          issue(
            n,
            "mobile-death-status-kind",
            "사건 부여 상태 정의를 연결하세요.",
            "states",
          );
    }
    if (n.kind === "skill-reactive-status") {
      positive(n, "duration");
      requireKind(n, "blockedBy", ["reactive-status"]);
      if (source(n, "blockedBy")?.id === n.id)
        issue(
          n,
          "status-self-block",
          "자기 자신을 신규 부여 차단 상태로 연결할 수 없습니다.",
          "blockedBy",
        );
    }
    if (n.kind === "skill-owned-damage-status") {
      const status = requireKind(n, "status", ["reactive-status"]);
      if (
        status &&
        ofKind("owned-damage-status").filter(
          (p) => source(p, "status")?.id === status.id,
        ).length > 1
      )
        issue(
          n,
          "status-subscription-duplicate",
          "같은 상태의 피해 구독이 중복되었습니다.",
          "status",
        );
    }
    if (n.kind === "skill-convert-status-plan") {
      requireKind(n, "body", ["strike-plan"]);
      requireKind(n, "from", ["reactive-status"]);
      requireKind(n, "to", ["reactive-status"]);
      if (source(n, "from") && source(n, "from")?.id === source(n, "to")?.id)
        issue(
          n,
          "status-conversion-identical",
          "서로 다른 원본·변환 상태를 연결하세요.",
          "to",
        );
    }
  }
  return out;
}
