import type { GraphDocument } from "../types";
import { p4ValidationContext } from "./p4-validation-context";
import { readTilePattern } from "./tile-pattern";

export function equipmentModifierDiagnostics(doc: GraphDocument) {
  const {
    out,
    source,
    sources,
    issue,
    requireKind,
    text,
    ofKind,
    plans,
    positive,
    positiveField,
  } = p4ValidationContext(doc);
  for (const n of doc.nodes) {
    if (n.kind === "skill-equipment-condition") {
      text(n, "definition");
      const visit = (id: string, path: Set<string>): boolean => {
        if (path.has(id)) return true;
        const node = doc.nodes.find((x) => x.id === id);
        if (!node) return false;
        return sources(node, "requires").some((x) =>
          visit(x.id, new Set([...path, id])),
        );
      };
      if (visit(n.id, new Set()))
        issue(
          n,
          "equipment-prerequisite-cycle",
          "선행 장착 조건을 순환 참조할 수 없습니다.",
          "requires",
        );
      for (const p of sources(n, "requires"))
        if (p.kind !== "skill-equipment-condition")
          issue(
            n,
            "equipment-condition-kind",
            "활성 장착 조건을 연결하세요.",
            "requires",
          );
    }
    if (!n.kind.startsWith("skill-modifier-")) continue;
    requireKind(n, "condition", ["equipment-condition"]);
    const target = source(n, "target");
    if (n.kind === "skill-modifier-piercing") {
      requireKind(n, "target", ["attack-profile"]);
      positiveField(n, "step");
      positiveField(n, "cap");
    }
    if (
      [
        "skill-modifier-projectile-status",
        "skill-modifier-projectile-push",
        "skill-modifier-projectile-explosion",
      ].includes(n.kind)
    ) {
      requireKind(n, "target", ["projectile-profile"]);
      if (
        n.kind !== "skill-modifier-projectile-status" &&
        target?.values.delivery !== "directional"
      )
        issue(
          n,
          "modifier-directional-required",
          "방향 관통 투사체에 연결하세요.",
          "target",
        );
    }
    if (n.kind === "skill-modifier-projectile-status")
      requireKind(n, "status", ["hit-status"]);
    if (n.kind === "skill-modifier-projectile-push") positiveField(n, "tiles");
    if (n.kind === "skill-modifier-projectile-wound") {
      positive(n, "duration");
      for (const p of sources(n, "profiles"))
        if (p.kind !== "skill-projectile-profile")
          issue(
            n,
            "modifier-projectile-required",
            "실제 투사체 정의만 연결하세요.",
            "profiles",
          );
    }
    if (n.kind === "skill-modifier-wound-grant")
      requireKind(n, "target", ["modifier-projectile-wound"]);
    if (n.kind === "skill-modifier-continuous-volley") {
      requireKind(n, "target", ["portal-program"]);
      positive(n, "interval");
      positive(n, "costInterval");
      positiveField(n, "percent");
    }
    if (n.kind === "skill-modifier-coincident-area-burst") {
      requireKind(n, "target", ["paired-areas"]);
      positive(n, "delay", true);
    }
    if (n.kind === "skill-modifier-single-support") {
      const effects = sources(n, "effects");
      if (n.values.mode === "attributes") {
        requireKind(n, "target", ["aura-buff-plan"]);
        if (
          !effects.length ||
          effects.some((e) => e.kind !== "skill-attribute-effect")
        )
          issue(
            n,
            "modifier-support-effects",
            "대체할 능력치 기여를 명시하세요.",
            "effects",
          );
        const attributes = effects.map((e) => e.values.attribute);
        if (new Set(attributes).size !== attributes.length)
          issue(
            n,
            "modifier-support-duplicate",
            "대체 능력치는 종류마다 하나만 연결하세요.",
            "effects",
          );
      } else {
        requireKind(n, "target", ["recipient-buff-plan"]);
        const state = target && source(target, "state");
        if (
          target?.values.recipient !== "allies" ||
          !state ||
          !sources(state, "effects").some(
            (e) => e.kind === "skill-personal-shield-effect",
          ) ||
          effects.length
        )
          issue(
            n,
            "modifier-support-shield",
            "범위 개인 보호막 행동만 연결하고 대체 능력치는 제거하세요.",
          );
      }
    }
    if (n.kind === "skill-modifier-duration")
      requireKind(n, "target", ["seconds", "count-duration"]);
    if (n.kind === "skill-modifier-capacity")
      requireKind(n, "target", ["stock"]);
    if (n.kind === "skill-modifier-population")
      requireKind(n, "target", ["population-power"]);
    if (n.kind === "skill-modifier-explosion-overlap")
      requireKind(n, "target", ["explosion-rule"]);
    if (n.kind === "skill-modifier-interruption") {
      const ports = ["program", "channel"].filter((p) => source(n, p));
      if (ports.length !== 1)
        issue(
          n,
          "modifier-target-choice",
          "행동 시작 설정 또는 집중 중 하나를 연결하세요.",
        );
      requireKind(n, "program", [
        "action-entry",
        "mode-program",
        "portal-program",
      ]);
      requireKind(n, "channel", ["channel-begin", "sequence-channel"]);
    }
    if (n.kind === "skill-modifier-mark-grant") {
      requireKind(n, "target", ["mark-weapon-hit"]);
      if (target && source(target, "marks")?.values.mode !== "stacks")
        issue(
          n,
          "modifier-stack-only",
          "정수 스택의 새 부여에만 연결하세요.",
          "target",
        );
    }
    if (n.kind === "skill-modifier-control-vulnerability") {
      const attached = ["hit", "ground", "reactive"].filter((p) =>
        source(n, p),
      );
      if (attached.length !== 1)
        issue(
          n,
          "modifier-control-choice",
          "기절·빙결 정의 하나를 연결하세요.",
        );
      for (const port of attached) {
        const effect = source(n, port);
        if (
          !effect ||
          !["stun", "freeze"].includes(String(effect.values.effect))
        )
          issue(
            n,
            "modifier-hard-control",
            "실제 기절·빙결만 이 조건을 만족합니다.",
            port,
          );
      }
    }
    if (n.kind === "skill-modifier-mobile-vulnerability")
      requireKind(n, "target", ["mobile-effect", "mobile-variant"]);
    if (n.kind === "skill-modifier-ramp")
      requireKind(n, "target", ["ground-ramping-slow"]);
    if (n.kind === "skill-modifier-grant")
      requireKind(n, "target", [
        "stock-periodic",
        "relation-periodic",
        "damage-recovery",
        "cohort-killed",
      ]);
    if (n.kind === "skill-modifier-relation-shield") {
      requireKind(n, "target", ["ally-relation"]);
      if (
        !ofKind("relation-emergency-shield").some(
          (p) => source(p, "relation")?.id === target?.id,
        )
      )
        issue(
          n,
          "modifier-shield-required",
          "이 관계에 선언된 긴급 보호막이 필요합니다.",
        );
    }
    if (
      n.kind === "skill-modifier-taunt-damage" &&
      (!target ||
        !sources(target, "effects").some(
          (p) => p.kind === "skill-target-taunt",
        ))
    )
      issue(
        n,
        "modifier-taunt-required",
        "실제 도발 효과를 포함한 상태에 연결하세요.",
      );
    if (
      n.kind === "skill-modifier-hit-count" &&
      target?.values.effect !== "vulnerability"
    )
      issue(
        n,
        "modifier-stacks-required",
        "스택형 약화의 새 부여에만 연결하세요.",
      );
    if (
      n.kind === "skill-modifier-primary-area" &&
      (target?.kind !== "skill-strike-plan" ||
        target.values.target !== "primary")
    )
      issue(
        n,
        "modifier-primary-required",
        "주 대상 직접 타격에만 연결하세요.",
      );
    if (n.kind === "skill-modifier-retarget") {
      requireKind(n, "target", ["action-entry"]);
      const plan = target && source(target, "plan");
      if (
        !plan ||
        !plans(plan).some(
          (p) =>
            p.kind === "skill-strike-plan" && p.values.target === "primary",
        )
      )
        issue(
          n,
          "modifier-primary-required",
          "주 대상 직접 타격을 포함한 행동이어야 합니다.",
        );
    }
    if (
      [
        "skill-modifier-control-vulnerability",
        "skill-modifier-mobile-vulnerability",
      ].includes(n.kind)
    )
      positive(n, "duration");
    if (n.kind === "skill-modifier-mobile-power")
      requireKind(n, "target", ["mobile-effect", "mobile-variant"]);
    if (n.kind === "skill-modifier-explosion-power")
      requireKind(n, "target", ["explosion-rule"]);
    if (n.kind === "skill-modifier-heal-buff") {
      const buff = requireKind(n, "state", ["target-state"]);
      if (
        buff &&
        (buff.values.sourceExit !== "independent" ||
          sources(buff, "effects").some(
            (p) => p.kind !== "skill-attribute-effect",
          ))
      )
        issue(
          n,
          "modifier-heal-buff-kind",
          "독립 수명의 능력치 버프 상태를 연결하세요.",
          "state",
        );
    }
    if (
      n.kind === "skill-modifier-taunt-damage" &&
      n.values.mode === "taken" &&
      typeof n.values.percent === "number" &&
      n.values.percent > 100
    )
      issue(
        n,
        "modifier-reduction-range",
        "피해 감소는 100%를 넘을 수 없습니다.",
      );
    if (n.kind === "skill-modifier-range") {
      if (n.values.mode === "replace" && !source(n, "replacement"))
        issue(
          n,
          "modifier-range-required",
          "명시한 확대 마스크를 연결하세요.",
          "replacement",
        );
      if (n.values.mode === "square-step" && source(n, "replacement"))
        issue(
          n,
          "modifier-range-unused",
          "정사각형 규칙에서는 교체 마스크를 사용하지 않습니다.",
          "replacement",
        );
      if (
        n.values.mode === "square-step" &&
        target?.kind === "skill-range" &&
        target.values.pattern !== ""
      ) {
        const pattern = readTilePattern(String(target.values.pattern));
        const cells = [...pattern.cells];
        if (target.values.includeAnchor) cells.push({ x: 0, y: 0 });
        const keys = new Set(cells.map((p) => `${p.x},${p.y}`));
        const single = keys.size === 1 && keys.has("0,0");
        const square =
          keys.size === 9 &&
          [-1, 0, 1].every((x) =>
            [-1, 0, 1].every((y) => keys.has(`${x},${y}`)),
          );
        if (!single && !square)
          issue(
            n,
            "modifier-range-shape",
            "이 형상은 명시한 확대 마스크를 사용해야 합니다.",
            "target",
          );
      }
    }
    if (
      n.kind === "skill-modifier-multihit" &&
      target &&
      !plans(target).some((p) => p.kind === "skill-strike-plan")
    )
      issue(
        n,
        "modifier-direct-hits",
        "직접 연타 본문을 연결하세요. 독립 효과의 반복은 직접 연타가 아닙니다.",
        "target",
      );
    const condition = source(n, "condition");
    if (
      target &&
      condition &&
      ofKind(n.kind.slice(6)).filter(
        (p) =>
          source(p, "target")?.id === target.id &&
          source(p, "condition")?.id === condition.id &&
          p.values.mode === n.values.mode,
      ).length > 1
    )
      issue(
        n,
        "modifier-duplicate",
        "같은 장착 조건의 동일 변경이 중복 연결되었습니다.",
        "target",
      );
  }
  return out;
}
