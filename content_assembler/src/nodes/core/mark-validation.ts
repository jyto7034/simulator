import type { GraphDocument, GraphNode } from "../types";
import { p4ValidationContext } from "./p4-validation-context";

export function markDiagnostics(doc: GraphDocument) {
  const {
    out,
    source,
    issue,
    requireKind,
    positive,
    positiveField,
    text,
    ofKind,
    plans,
  } = p4ValidationContext(doc);
  const sameMarks = (n: GraphNode, port: string) => {
    const other = source(n, port),
      own = source(n, "marks");
    if (other && own && source(other, "marks")?.id !== own.id)
      issue(
        n,
        "mark-source-owner",
        "연결한 기능과 표식 저장소의 출처가 다릅니다.",
        port,
      );
  };
  const mode = (n: GraphNode, value: string) => {
    const store = requireKind(n, "marks", ["target-marks"]);
    if (store && store.values.mode !== value)
      issue(
        n,
        "mark-storage-mode",
        value === "presence"
          ? "존재형 숙주 저장소가 필요합니다."
          : "정수 스택 저장소가 필요합니다.",
        "marks",
      );
    return store;
  };
  const unique = (n: GraphNode, port = "marks") => {
    const owner = source(n, port);
    if (
      owner &&
      doc.nodes.some(
        (p) =>
          p.id !== n.id &&
          p.kind === n.kind &&
          source(p, port)?.id === owner.id &&
          !(
            n.kind === "skill-mark-weapon-hit" &&
            p.values.weapon === n.values.weapon &&
            source(p, "evolution")?.id === source(n, "evolution")?.id &&
            source(n, "evolution") &&
            [p.values.binding, n.values.binding].sort().join() ===
              "weapon-base,weapon-upgrade"
          ),
      )
    )
      issue(
        n,
        "mark-subscription-duplicate",
        "같은 저장소의 같은 기능을 중복 구독할 수 없습니다.",
        port,
      );
  };
  for (const n of doc.nodes) {
    if (n.kind === "skill-weapon-evolution") {
      for (const field of ["fragment", "form", "weapon", "base", "upgraded"])
        text(n, field);
      if (n.values.base === n.values.upgraded)
        issue(
          n,
          "weapon-evolution-identity",
          "승급 전후 단계는 서로 달라야 합니다.",
          "upgraded",
        );
    }
    if (n.kind === "skill-weapon-kill-growth") {
      text(n, "weapon");
      text(n, "key");
      positiveField(n, "damage");
      for (const port of ["normal", "elite", "abnormality"]) {
        const value = source(n, port);
        if (
          value &&
          (!["number", "skill-integer-amount"].includes(value.kind) ||
            (typeof value.values.value === "number" &&
              (!Number.isSafeInteger(value.values.value) ||
                value.values.value < 0)))
        )
          issue(
            n,
            "weapon-growth-amount",
            "신규 획득량은 0 이상의 안전한 정수 고정값이어야 합니다.",
            port,
          );
      }
      if (
        ofKind("weapon-kill-growth").some(
          (p) =>
            p.id !== n.id &&
            p.values.weapon === n.values.weapon &&
            p.values.key === n.values.key,
        )
      )
        issue(
          n,
          "weapon-growth-duplicate",
          "같은 무기 상태를 중복 성장시킬 수 없습니다.",
          "key",
        );
    }
    if (n.kind === "skill-weapon-mark-owner") {
      text(n, "key");
      const store = requireKind(n, "marks", ["target-marks"]);
      requireKind(n, "evolution", ["weapon-evolution"]);
      unique(n);
      if (
        store &&
        (store.values.mode !== "stacks" || store.values.lifetime !== "battle")
      )
        issue(
          n,
          "mark-weapon-storage",
          "무기 표식은 전투 수명의 정수 스택 저장소여야 합니다.",
        );
    }
    if (
      n.kind === "skill-target-marks" &&
      n.values.mode === "presence" &&
      n.values.death === "tile"
    )
      issue(
        n,
        "mark-presence-tile",
        "숙주 존재형은 사망 타일의 정수 스택으로 이전할 수 없습니다.",
        "death",
      );
    if (n.kind === "skill-mark-weapon-hit") {
      const store = requireKind(n, "marks", ["target-marks"]);
      const owner =
        store &&
        ofKind("weapon-mark-owner").find(
          (p) => source(p, "marks")?.id === store.id,
        );
      requireKind(n, "filter", ["mark-filter"]);
      if (
        ["weapon-upgrade", "weapon-base"].includes(String(n.values.binding))
      ) {
        text(n, "weapon");
        const evolution = requireKind(n, "evolution", ["weapon-evolution"]);
        if (!evolution)
          issue(
            n,
            "weapon-evolution-missing",
            "무기의 전후 단계를 선언한 영구 승급을 연결하세요.",
            "evolution",
          );
        if (evolution && evolution.values.weapon !== n.values.weapon)
          issue(
            n,
            "weapon-evolution-owner",
            "적중 구독과 승급 대상 무기가 다릅니다.",
            "evolution",
          );
        if (
          store &&
          (!owner || source(owner, "evolution")?.id !== evolution?.id)
        )
          issue(
            n,
            "mark-weapon-owner",
            "무기 적중은 같은 무기 사본의 저장소를 사용해야 합니다.",
            "marks",
          );
      } else if (owner)
        issue(
          n,
          "mark-weapon-owner",
          "파편 장착 구독에 무기 사본 저장소를 사용할 수 없습니다.",
          "marks",
        );
      unique(n);
    }
    if (n.kind === "skill-mark-spread") {
      mode(n, "presence");
      requireKind(n, "source", ["mark-weapon-hit"]);
      requireKind(n, "filter", ["mark-filter"]);
      sameMarks(n, "source");
      unique(n);
      const hit = source(n, "source");
      if (hit && source(n, "filter")?.id !== source(hit, "filter")?.id)
        issue(
          n,
          "mark-filter-mismatch",
          "최초 감염과 전파에는 같은 대상 자격을 연결하세요.",
          "filter",
        );
    }
    if (n.kind === "skill-population-power") {
      const store = mode(n, "presence");
      positive(n, "grace", true);
      positiveField(n, "cap");
      unique(n);
      if (store && store.values.lifetime !== "deployment")
        issue(
          n,
          "mark-power-lifetime",
          "연속 위력과 숙주는 배치 이탈 때 함께 폐기해야 합니다.",
          "marks",
        );
    }
    if (
      n.kind === "skill-mark-periodic-damage" ||
      n.kind === "skill-mark-burst-plan"
    ) {
      mode(n, "presence");
      requireKind(n, "power", ["population-power"]);
      sameMarks(n, "power");
      positiveField(n, "coefficient");
      if (n.kind === "skill-mark-periodic-damage") {
        positive(n, "interval");
        unique(n);
      }
    }
    if (
      n.kind === "skill-path-burst-plan" ||
      n.kind === "skill-stack-burst-plan"
    )
      positive(n, "delay", true);
    if (n.kind === "skill-stack-burst-plan") {
      const store = mode(n, "stacks");
      if (store && store.values.death !== "tile")
        issue(
          n,
          "mark-stack-transfer",
          "타일 합산 폭발은 사망 시 타일로 이전하는 저장소가 필요합니다.",
          "marks",
        );
      positive(n, "interval");
      requireKind(n, "grade", ["stack-grade"]);
      requireKind(n, "conversion", ["burst-conversion"]);
    }
    if (n.kind === "skill-burst-conversion") positive(n, "presentation");
    if (
      [
        "skill-mark-burst-plan",
        "skill-path-burst-plan",
        "skill-stack-burst-plan",
      ].includes(n.kind)
    )
      requireKind(n, "explosion", ["explosion-rule"]);
    if (n.kind === "skill-action-entry") {
      const body = plans(source(n, "plan"));
      const bursts = body.filter((p) =>
        [
          "skill-mark-burst-plan",
          "skill-path-burst-plan",
          "skill-stack-burst-plan",
        ].includes(p.kind),
      );
      if (
        bursts.length &&
        (body.length !== 1 ||
          n.values.start !== "immediate" ||
          source(n, "selector"))
      )
        issue(
          n,
          "burst-admission-boundary",
          "독립 폭발은 즉시 시작하는 단일 계획으로 연결하세요. 표식 소비와 필수 대상을 비용 확정 전에 검사해야 합니다.",
          "plan",
        );
    }
  }
  // A declared population is source-owned; multiple periodic clocks or growth
  // declarations cannot silently multiply its effects.
  for (const store of ofKind("target-marks")) {
    if (
      !ofKind("mark-weapon-hit").some(
        (n) => source(n, "marks")?.id === store.id,
      )
    )
      issue(
        store,
        "mark-source-missing",
        "표식을 부여할 무기 적중 출처를 연결하세요.",
      );
  }
  return out;
}
