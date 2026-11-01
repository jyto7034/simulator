import type { GraphDocument } from "../types";
import { p4ValidationContext } from "./p4-validation-context";

export function portalDiagnostics(doc: GraphDocument) {
  const c = p4ValidationContext(doc);
  const {
    out,
    source,
    issue,
    requireKind,
    positive,
    ofKind,
    admittedProgram,
    requestFor,
    chargeForVolley,
  } = c;
  for (const n of doc.nodes) {
    if (n.kind === "skill-portal-policy") {
      for (const port of ["first", "last", "deadline", "decay"])
        positive(n, port);
      positive(n, "hold", true);
      const first = source(n, "first")?.values.value,
        last = source(n, "last")?.values.value;
      if (typeof first === "number" && typeof last === "number" && last > first)
        issue(
          n,
          "portal-acceleration",
          "후반 생성 간격이 첫 간격보다 길 수 없습니다.",
          "last",
        );
    }
    if (n.kind === "skill-portal-program") {
      requireKind(n, "bank", ["credit-ledger"]);
      requireKind(n, "policy", ["portal-policy"]);
    }
    if (n.kind === "skill-run-portals") {
      const { use } = admittedProgram(n, ["portal-program"]);
      const request = use && requestFor(use),
        selection = request && source(request, "selection");
      if (
        selection &&
        (selection.kind !== "skill-target-selection" ||
          selection.values.mode !== "self")
      )
        issue(
          n,
          "portal-request-selection",
          "전개 시작은 자기 자신 입력이며 매 발의 적 선택과 구분합니다.",
        );
      if (
        !ofKind("portal-release-request").some(
          (p) => source(p, "charge")?.id === n.id,
        )
      )
        issue(
          n,
          "portal-release-required",
          "전개의 사출 재입력 경로가 없습니다.",
        );
    }
    if (
      ["skill-portal-release-request", "skill-portal-stop-input"].includes(
        n.kind,
      )
    )
      requireKind(n, "charge", ["run-portals"]);
    if (n.kind === "skill-portal-release-request") {
      const consumers = ofKind("portal-freeze").filter(
        (p) => source(p, "right")?.id === n.id,
      );
      if (consumers.length !== 1)
        issue(
          n,
          "portal-right-consumer",
          "사출 권리를 수량 확정 하나에 연결하세요.",
        );
      const charge = source(n, "charge");
      if (
        charge &&
        ofKind("portal-release-request").filter(
          (p) => source(p, "charge")?.id === charge.id,
        ).length > 1
      )
        issue(
          n,
          "portal-release-duplicate",
          "같은 전개에 사출 재입력 구독이 중복됩니다.",
        );
    }
    if (n.kind === "skill-portal-freeze") {
      requireKind(n, "right", ["portal-release-request"], "out");
      const consumers = ofKind("serial-projectiles").filter(
        (p) => source(p, "volley")?.id === n.id,
      );
      if (consumers.length !== 1)
        issue(
          n,
          "portal-volley-consumer",
          "확정 사출 목록은 순차 사출 한 곳에서만 소비해야 합니다.",
        );
    }
    if (n.kind === "skill-serial-projectiles") {
      const volley = requireKind(n, "volley", ["portal-freeze"], "out");
      const profile = requireKind(n, "profile", ["projectile-profile"]),
        selector = requireKind(n, "selector", ["enemy-selector"]);
      if (profile && profile.values.delivery !== "impact")
        issue(
          n,
          "portal-projectile-kind",
          "순차 사출은 착탄 정산을 기다릴 표적 투사체를 사용해야 합니다.",
          "profile",
        );
      if (selector && profile && selector.values.air !== profile.values.air)
        issue(
          n,
          "portal-air-selection",
          "표적 선택과 착탄의 공중 자격이 다릅니다.",
        );
      if (selector?.values.order === "unmarked-route") {
        const charge = chargeForVolley(volley),
          program = charge && source(charge, "program"),
          bank = program && source(program, "bank");
        const mark = source(selector, "mark");
        const matches = ofKind("damage-recovery").filter(
          (p) =>
            source(p, "mark")?.id === mark?.id &&
            source(p, "bank")?.id === bank?.id,
        );
        if (bank && mark && matches.length !== 1)
          issue(
            n,
            "portal-recovery-owner",
            "같은 전개 원장에 지급하는 출처별 회수 상태로 표적을 선택해야 합니다.",
            "selector",
          );
      }
    }
  }
  return out;
}
