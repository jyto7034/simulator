import { equipmentModifierDiagnostics } from "./equipment-modifier-validation";
import { mobileEffectDiagnostics } from "./mobile-effect-validation";
import { offensiveDiagnostics } from "./offensive-validation";
import { protectionDiagnostics } from "./protection-validation";
import { markDiagnostics } from "./mark-validation";
import { portalDiagnostics } from "./portal-validation";
import { inputPlanDiagnostics } from "./input-plan-validation";
import { actionPlanDiagnostics } from "./action-plan-validation";
import { inventoryDiagnostics } from "./inventory-validation";
import { p4FieldActive } from "./p4-fields";
import type { Diagnostic, GraphDocument, GraphNode } from "../types";
import { skillCatalog } from "./skill-catalog";
import { isLegacyArea, readTilePattern } from "./tile-pattern";
import { inputSources } from "./connections";
import { flowPaths } from "./flow-paths";
import { resonanceFieldActive } from "./resonance-catalog";
import { resonanceDiagnostics } from "./resonance-validation";
import { spatialFieldActive } from "./spatial-catalog";
import { spatialDiagnostics } from "./spatial-validation";
import { groundFieldActive } from "./ground-catalog";
import { resourceDiagnostics } from "./resource-validation";
import { phaseDiagnostics } from "./phase-validation";
import { lifetimeDiagnostics } from "./lifetime-validation";
import { pairDiagnostics } from "./pair-validation";
import { groundDiagnostics } from "./ground-validation";
import { channelDiagnostics } from "./channel-validation";
import { targetDiagnostics } from "./target-validation";

const definitions = new Map(skillCatalog.map((d) => [d.kind, d]));
const activationEvents = new Set([
  "skill-reinput",
  "skill-expiry",
  "skill-owner-exit",
  "skill-claim",
]);

// Narrow authoring checks. This is not an executor or a proof of arbitrary flow semantics.
export function skillDiagnostics(doc: GraphDocument): Diagnostic[] {
  const out: Diagnostic[] = [];
  const nodes = new Map(doc.nodes.map((n) => [n.id, n]));
  const source = inputSources(doc);
  const { mustPass, ancestors } = flowPaths(doc);
  const issue = (
    node: GraphNode,
    code: string,
    message: string,
    portId: string | null = null,
  ) => {
    out.push({
      code,
      nodeId: node.id,
      portId,
      message: `${node.title || definitions.get(node.kind)?.label || node.kind} · ${message}`,
    });
  };
  const installers = new Map<string, GraphNode[]>();
  for (const n of doc.nodes) {
    if (
      !["skill-overlay", "skill-begin", "skill-resonance-install"].includes(
        n.kind,
      )
    )
      continue;
    const state = source(n, "state");
    if (!state) continue;
    const previous = installers.get(state.id) ?? [];
    previous.push(n);
    installers.set(state.id, previous);
  }
  for (const n of doc.nodes) {
    const def = definitions.get(n.kind);
    if (!def || n.kind.startsWith("reroute-")) continue;
    if (isLegacyArea(n)) {
      issue(
        n,
        "skill-area-legacy",
        "이전 도형 설정입니다. 효과 타일 범위를 다시 지정하세요. 도형 값은 자동 변환하지 않습니다.",
      );
      continue;
    }
    if (["skill-area", "skill-range"].includes(n.kind)) {
      const pattern = readTilePattern(String(n.values.pattern));
      if (
        !pattern.assigned ||
        (!pattern.cells.length && !n.values.includeAnchor)
      )
        issue(n, "skill-value-unset", "효과 타일을 하나 이상 지정하세요.");
    }
    for (const field of def.fields) {
      if (
        !resonanceFieldActive(n, field) ||
        !spatialFieldActive(n, field) ||
        !groundFieldActive(n, field) ||
        !p4FieldActive(n, field)
      )
        continue;
      if (
        field.options &&
        (n.values[field.id] === "unresolved" ||
          (n.kind === "skill-area-damage" &&
            field.id === "damageType" &&
            n.values[field.id] === undefined))
      )
        issue(n, "skill-policy-unset", `${field.label}이 미지정입니다.`);
      let required = field.valueType === "optional-number";
      if (n.kind === "skill-time-power" && field.id === "cap")
        required = n.values.curve === "capped-linear";
      if (required && n.values[field.id] === "")
        issue(
          n,
          "skill-value-unset",
          `${field.label}을 아직 정하지 않았습니다.`,
        );
    }
    if (
      n.kind === "skill-time-power" &&
      n.values.curve === "capped-linear" &&
      typeof n.values.cap === "number" &&
      typeof n.values.base === "number" &&
      n.values.cap < n.values.base
    )
      issue(n, "skill-power-cap", "위력 상한이 기본 위력보다 작습니다.");
    if (n.kind === "skill-cue" && !String(n.values.description).trim())
      issue(n, "skill-cue-unset", "표시할 연출 내용을 입력하세요.");
    if (n.kind === "skill-state" && (installers.get(n.id)?.length ?? 0) > 1)
      issue(
        n,
        "skill-state-installers",
        "같은 상태 선언에 여러 공격 교체가 연결되어 있습니다. 활성 소유 범위를 구분하세요.",
      );
    if (
      ["skill-basic-event", "skill-lookup", ...activationEvents].includes(
        n.kind,
      )
    ) {
      const state = source(n, "state");
      if (state && !installers.has(state.id))
        issue(
          n,
          "skill-state-uninstalled",
          "참조한 상태를 적용하는 기본 공격 교체가 없습니다.",
          "state",
        );
    }
    const requireSource = (port: string, kind: string, flowPort: string) => {
      const producer = source(n, port);
      if (!producer) return;
      if (producer.kind !== kind || !mustPass(n, producer, flowPort)) {
        issue(
          n,
          "skill-path-scope",
          "이 입력값을 만든 분기를 모든 실행 경로가 통과해야 합니다.",
          port,
        );
        return;
      }
      return producer;
    };
    if (n.kind === "skill-lookup")
      requireSource("request", "skill-request", "out");
    if (n.kind === "skill-qualify")
      requireSource("vacancy", "skill-lookup", "absent");
    if (n.kind === "skill-begin") {
      const qualification = requireSource("permit", "skill-qualify", "out");
      const lookup = qualification && source(qualification, "vacancy");
      const expected = lookup && source(lookup, "state");
      const actual = source(n, "state");
      if (expected && actual && expected.id !== actual.id)
        issue(
          n,
          "skill-start-state",
          "첫 사용 허가와 시작할 상태 선언이 다릅니다.",
          "state",
        );
    }
    if (n.kind === "skill-permission") {
      const lookup = requireSource("activation", "skill-lookup", "present");
      requireSource("request", "skill-request", "out");
      const request = source(n, "request");
      if (lookup && request && source(lookup, "request")?.id !== request.id)
        issue(
          n,
          "skill-request-scope",
          "활성을 조회한 요청과 재입력 요청이 다릅니다.",
          "request",
        );
    }
    if (n.kind === "skill-claim")
      requireSource("permit", "skill-permission", "out");
    if (n.kind === "skill-capture-time" || n.kind === "skill-finish") {
      const event = source(n, "activation");
      if (!event) continue; // Missing ports are reported by the graph validator.
      const upstream = ancestors(n);
      const roots = [...upstream].filter((id) =>
        activationEvents.has(nodes.get(id)?.kind ?? ""),
      );
      if (
        roots.length !== 1 ||
        roots[0] !== event.id ||
        !mustPass(n, event, "out")
      )
        issue(
          n,
          "skill-activation-scope",
          "실행 경로와 이번 활성의 출처가 다르거나 모호합니다.",
          "activation",
        );
      if (
        n.kind === "skill-capture-time" &&
        !["skill-reinput", "skill-claim"].includes(event.kind)
      )
        issue(
          n,
          "skill-capture-source",
          "잔여 시간은 유효한 재입력의 활성에서 고정해야 합니다.",
          "activation",
        );
      if (n.kind === "skill-finish") {
        const expected = {
          "skill-reinput": "manual",
          "skill-claim": "manual",
          "skill-expiry": "expiry",
          "skill-owner-exit": "owner-exit",
        }[event.kind];
        if (expected && expected !== n.values.reason)
          issue(
            n,
            "skill-end-reason",
            "활성을 보낸 사건과 종료 원인이 다릅니다.",
          );
      }
    }
    if (n.kind === "skill-area-damage") {
      const power = source(n, "power");
      const capture =
        power?.kind === "skill-time-power"
          ? source(power, "seconds")
          : undefined;
      if (
        capture?.kind === "skill-capture-time" &&
        !mustPass(n, capture, "out")
      )
        issue(
          n,
          "skill-result-scope",
          "이 실행 경로에서 고정되지 않은 잔여 시간 결과를 사용합니다.",
          "power",
        );
      if (capture?.kind === "skill-capture-time") {
        const activation = source(capture, "activation");
        const ended =
          activation &&
          doc.nodes.some(
            (finish) =>
              finish.kind === "skill-finish" &&
              finish.values.reason === "manual" &&
              source(finish, "activation")?.id === activation.id &&
              mustPass(n, finish, "out"),
          );
        if (!ended)
          issue(
            n,
            "skill-finish-required",
            "잔여 시간을 사용하는 일격은 같은 활성의 수동 종료를 거쳐야 합니다.",
            "in",
          );
      }
    }
  }
  return [
    ...out,
    ...resonanceDiagnostics(doc),
    ...spatialDiagnostics(doc),
    ...groundDiagnostics(doc),
    ...channelDiagnostics(doc),
    ...targetDiagnostics(doc),
    ...pairDiagnostics(doc),
    ...lifetimeDiagnostics(doc),
    ...phaseDiagnostics(doc),
    ...resourceDiagnostics(doc),
    ...portalDiagnostics(doc),
    ...inputPlanDiagnostics(doc),
    ...protectionDiagnostics(doc),
    ...offensiveDiagnostics(doc),
    ...mobileEffectDiagnostics(doc),
    ...equipmentModifierDiagnostics(doc),
    ...markDiagnostics(doc),
    ...actionPlanDiagnostics(doc),
    ...inventoryDiagnostics(doc),
  ];
}
