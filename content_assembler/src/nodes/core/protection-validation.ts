import type { GraphDocument } from "../types";
import { p4ValidationContext } from "./p4-validation-context";

export function protectionDiagnostics(doc: GraphDocument) {
  const { out, source, sources, issue, required, requireKind, positive } =
    p4ValidationContext(doc);
  for (const n of doc.nodes) {
    if (n.kind === "skill-recipient-buff-plan") {
      requireKind(n, "state", ["target-state"]);
      if (n.values.recipient === "allies") required(n, "range");
      else if (source(n, "range"))
        issue(
          n,
          "protection-unused-range",
          "범위 입력은 범위 아군 부여에만 사용합니다.",
          "range",
        );
    }
    if (n.kind === "skill-next-basic-plan")
      requireKind(n, "reward", ["target-state"]);
    if (["skill-reactive-heal-plan", "skill-aura-buff-plan"].includes(n.kind))
      positive(n, "duration");
    if (n.kind === "skill-attribute-effect") {
      const expected =
        n.values.attribute === "basic-power"
          ? "percent"
          : ["attack-speed", "basic-speed"].includes(String(n.values.attribute))
            ? "points"
            : null;
      if (expected && n.values.operation !== expected)
        issue(
          n,
          "attribute-unit",
          "피해 증가는 백분율, 공격 속도는 포인트 기여로 작성하세요.",
          "operation",
        );
    }
    if (n.kind === "skill-aura-buff-plan") {
      const attributes = new Set<string>();
      for (const effect of sources(n, "effects")) {
        if (effect.kind !== "skill-attribute-effect")
          issue(
            n,
            "aura-effect-kind",
            "집중 범위에는 능력치 기여만 연결할 수 있습니다.",
            "effects",
          );
        const attribute = String(effect.values.attribute);
        if (attributes.has(attribute))
          issue(
            n,
            "aura-effect-duplicate",
            "같은 능력치 기여가 중복되었습니다.",
            "effects",
          );
        attributes.add(attribute);
      }
    }
    if (
      n.kind === "skill-target-state" &&
      sources(n, "effects").some(
        (e) => e.kind === "skill-personal-shield-effect",
      ) &&
      n.values.sourceExit !== "end"
    )
      issue(
        n,
        "shield-source-exit",
        "개인 보호막은 제공자 사망·철수에 함께 종료합니다.",
        "sourceExit",
      );
  }
  return out;
}
