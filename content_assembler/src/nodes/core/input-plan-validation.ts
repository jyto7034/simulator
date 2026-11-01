import type { GraphDocument, GraphNode } from "../types";
import { p4ValidationContext } from "./p4-validation-context";

export function inputPlanDiagnostics(doc: GraphDocument) {
  const c = p4ValidationContext(doc);
  const {
    out,
    source,
    sources,
    issue,
    requireKind,
    ofKind,
    mustPass,
    preparation,
  } = c;
  const aimKinds = ["plan-precise-aim", "plan-focused-aim"];
  const sameLayout = (a: GraphNode, b: GraphNode) =>
    source(a, "layout")?.id === source(b, "layout")?.id;
  for (const n of doc.nodes) {
    if (n.kind === "skill-layout-memory") {
      const normal = requireKind(n, "normal", ["stock"]),
        enhanced = requireKind(n, "enhanced", ["stock"]);
      if (normal && enhanced && normal.id === enhanced.id)
        issue(
          n,
          "plan-stock-kinds",
          "일반 창과 강화 창은 서로 다른 재고를 참조해야 합니다.",
        );
      for (const bank of [normal, enhanced])
        if (bank && bank.values.lifetime !== "battle")
          issue(
            n,
            "plan-stock-lifetime",
            "이전 배치 기억과 두 창 재고는 전투 수명을 사용해야 합니다.",
          );
    }
    if (n.kind === "skill-plan-prepare") {
      requireKind(n, "memory", ["layout-memory"]);
      requireKind(n, "settings", ["resonance-settings"]);
      requireKind(n, "requirement", ["resonance-requirement"]);
      if (
        !ofKind("plan-commit").some(
          (p) => preparation(source(p, "plan"))?.id === n.id,
        )
      )
        issue(
          n,
          "plan-commit-required",
          "입력 준비의 최종 계획을 일괄 확정에 연결하세요. 일반 접수로 먼저 소비하지 않습니다.",
        );
    }
    if (n.kind === "skill-plan-layout")
      requireKind(n, "preparation", ["plan-prepare"], "out");
    if (n.kind === "skill-plan-aim-mode")
      requireKind(n, "layout", ["plan-layout"], "out");
    if (aimKinds.some((k) => n.kind === `skill-${k}`)) {
      requireKind(n, "layout", ["plan-layout"], "out");
      const branch =
        n.kind === "skill-plan-precise-aim" ? "precise" : "focused";
      if (
        !ofKind("plan-aim-mode").some(
          (mode) => sameLayout(n, mode) && mustPass(n, mode, branch),
        )
      )
        issue(
          n,
          "plan-aim-branch",
          "같은 배치의 해당 사격 방식 분기를 통과해야 합니다.",
          "in",
        );
    }
    if (n.kind === "skill-plan-merge") {
      const precise = requireKind(n, "precise", ["plan-precise-aim"]),
        focused = requireKind(n, "focused", ["plan-focused-aim"]);
      if (precise && focused && !sameLayout(precise, focused))
        issue(
          n,
          "plan-merge-owner",
          "다른 준비 또는 배치의 사격 계획을 합류할 수 없습니다.",
        );
      const flow = sources(n, "in");
      if (
        precise &&
        focused &&
        (flow.length !== 2 ||
          !flow.some((p) => p.id === precise.id) ||
          !flow.some((p) => p.id === focused.id))
      )
        issue(
          n,
          "plan-merge-flow",
          "각 사격 계획 생산자의 완료 흐름을 한 번씩 연결하세요.",
          "in",
        );
    }
    if (n.kind === "skill-resonance-qualify") {
      const request = source(n, "request"),
        plan = source(n, "plan");
      if (request?.kind === "skill-plan-prepare") {
        if (!plan)
          issue(
            n,
            "plan-final-input",
            "정지 준비의 공명 허가는 최종 사격 계획을 함께 검사해야 합니다.",
            "plan",
          );
        if (
          plan &&
          (preparation(plan)?.id !== request.id || !mustPass(n, plan, "out"))
        )
          issue(
            n,
            "plan-final-scope",
            "최종 계획과 요청의 준비 회차·완료 흐름이 다릅니다.",
            "plan",
          );
        const read = source(n, "snapshot");
        if (
          read &&
          (source(read, "settings")?.id !== source(request, "settings")?.id ||
            source(n, "requirement")?.id !== source(request, "requirement")?.id)
        )
          issue(
            n,
            "plan-resonance-scope",
            "준비 사전 조회와 최종 허가는 같은 공명 설정·요구 조건을 사용해야 합니다.",
          );
      } else if (plan)
        issue(
          n,
          "plan-request-kind",
          "사격 계획은 그 입력 준비가 만든 최종 요청과 함께 사용해야 합니다.",
          "request",
        );
    }
    if (n.kind === "skill-resonance-admit") {
      const permit = source(n, "permit"),
        request = permit && source(permit, "request");
      if (request?.kind === "skill-plan-prepare")
        issue(
          n,
          "plan-early-admission",
          "창/공명을 먼저 지불하지 마세요. 전체 계획과 일괄 방출을 같은 확정 경계에 연결해야 합니다.",
        );
    }
    if (n.kind === "skill-plan-commit") {
      const permit = requireKind(n, "permit", ["resonance-qualify"], "out");
      const plan = requireKind(n, "plan", ["plan-merge", ...aimKinds]);
      const prep = preparation(plan);
      if (
        permit &&
        plan &&
        (source(permit, "plan")?.id !== plan.id ||
          source(permit, "request")?.id !== prep?.id)
      )
        issue(
          n,
          "plan-commit-scope",
          "최종 허가와 확정할 사격 계획의 준비 회차가 다릅니다.",
        );
      for (const port of ["normal", "enhanced"]) {
        const profile = requireKind(n, port, ["projectile-profile"]);
        if (profile && profile.values.delivery !== "directional")
          issue(
            n,
            "plan-projectile-kind",
            "창 배치 계획은 원점·방향을 갖는 관통 탄환을 방출해야 합니다.",
            port,
          );
      }
      if (
        prep &&
        ofKind("plan-commit").filter(
          (p) => preparation(source(p, "plan"))?.id === prep.id,
        ).length > 1
      )
        issue(
          n,
          "plan-commit-duplicate",
          "같은 준비의 소비·방출 확정이 여러 개입니다.",
        );
    }
  }
  return out;
}
