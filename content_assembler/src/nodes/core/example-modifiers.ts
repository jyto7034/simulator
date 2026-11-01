import type { GraphNode } from "../types";
import type { skillExampleBuilder } from "./skill-example-builder";
type Builder = ReturnType<typeof skillExampleBuilder>;

// Authoring helpers only. Every target and condition remains an ordinary typed edge.
export function exampleModifiers(b: Builder) {
  const conditions = new Map<string, GraphNode>();
  const condition = (definition: string, title: string) => {
    let node = conditions.get(definition);
    if (!node) {
      node = b.add("equipment-condition", title, {
        definition,
        slot: "relic",
        copies: "once",
      });
      conditions.set(definition, node);
    }
    return node;
  };
  const bind = (
    definition: string,
    title: string,
    kind: string,
    target: GraphNode,
    output: string,
    values: GraphNode["values"] = {},
    port = "target",
  ) => {
    const relicName = title.split(" · ")[0];
    const node = b.add(
      `modifier-${kind}`,
      `${relicName} → ${target.title}`,
      values,
    );
    b.link(
      condition(definition, `${relicName} · 장착 여부`),
      "condition",
      node,
      "condition",
    );
    b.link(target, output, node, port);
    return node;
  };
  const duration = (target: GraphNode) =>
    bind(
      "long-breath",
      "긴 호흡 · 지정한 유지 시간만 +25%",
      "duration",
      target,
      "value",
      { percent: 25 },
    );
  const range = (target: GraphNode, square = false) => {
    const node = bind(
      "widened-horizon",
      "넓어진 지평 · 효과 타일만 변경",
      "range",
      target,
      "range",
      { mode: square ? "square-step" : "replace" },
    );
    if (!square)
      b.link(
        b.add("range", "이 형상의 확대 타일 · 미정"),
        "range",
        node,
        "replacement",
      );
    return node;
  };
  const vulnerability = (
    target: GraphNode,
    port: "hit" | "ground" | "reactive",
  ) => {
    const node = bind(
      "broken-balance",
      "깨진 균형 · 실제 기절/빙결 뒤 5초",
      "control-vulnerability",
      target,
      "status",
      {},
      port,
    );
    b.link(b.seconds("약화의 독립 수명", 5), "value", node, "duration");
    return node;
  };
  const summonVulnerability = (target: GraphNode) => {
    const node = bind(
      "summon-vulnerability",
      "사냥의 낙인 · 소환 피해 뒤 5초",
      "mobile-vulnerability",
      target,
      "profile",
    );
    b.link(
      b.seconds("소환 피해 약화의 독립 수명", 5),
      "value",
      node,
      "duration",
    );
    return node;
  };
  let healedBuff: GraphNode | undefined;
  const healingBuff = (target: GraphNode) => {
    if (!healedBuff) {
      healedBuff = b.add(
        "target-state",
        "남은 온기 · 버프 정의와 독립 수명 미정",
        { sourceExit: "independent" },
      );
      b.link(
        b.seconds("남은 온기 · 버프 유지 시간 미정"),
        "value",
        healedBuff,
        "duration",
      );
      b.link(
        b.add("attribute-effect", "남은 온기 · 버프 종류/방식/수치 미정"),
        "effect",
        healedBuff,
        "effects",
      );
    }
    const node = bind(
      "lingering-warmth",
      "남은 온기 · 실제 회복 대상 버프",
      "heal-buff",
      target,
      "power",
    );
    b.link(healedBuff, "state", node, "state");
    return node;
  };
  return {
    condition,
    bind,
    duration,
    range,
    vulnerability,
    summonVulnerability,
    healingBuff,
  };
}
