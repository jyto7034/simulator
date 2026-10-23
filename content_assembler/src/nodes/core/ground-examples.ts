import type { GraphNode } from "../types";
import { makeNode } from "./catalog";
import { canonicalize, id } from "./model";
import { spatialExample } from "./spatial-examples";

// Authoring examples, not the balance or targeting policy of a shipped skill.
export function groundExample(mode: "damage" | "heal" | "status") {
  const doc = spatialExample("selected-ground");
  doc.name = {
    damage: "장판 구성 · 주기 피해",
    heal: "장판 구성 · 주기 회복",
    status: "장판 구성 · 진입·체류 상태",
  }[mode];
  const create = doc.nodes.find((n) => n.kind === "skill-ground-area")!;
  create.note =
    "장판 사건·효과·수명 연결 예제입니다. 실제 스킬의 수치와 미정 정책을 대신 확정하지 않습니다.";
  const frame = {
    id: id(),
    title: "장판 사건과 효과 · 수치와 미정 정책은 직접 지정",
  };
  doc.frames.push(frame);
  const add = (
    kind: string,
    title: string,
    x: number,
    y: number,
    values: GraphNode["values"] = {},
  ) => {
    const n = makeNode(`skill-${kind}`, { x, y });
    n.title = title;
    n.values = { ...n.values, ...values };
    n.frame = frame.id;
    doc.nodes.push(n);
    return n;
  };
  const link = (a: GraphNode, ap: string, b: GraphNode, bp: string) =>
    doc.edges.push({
      id: id(),
      source: { node: a.id, port: ap },
      target: { node: b.id, port: bp },
    });
  const filter = add("ground-filter", "장판 대상 조건", 40, 3500, {
    side: mode === "heal" ? "allies" : "enemies",
    caster: mode === "heal" ? "include" : "exclude",
  });
  const event = (kind: string, title: string, x: number, y: number) => {
    const n = add(kind, title, x, y);
    link(create, "area", n, "area");
    return n;
  };
  const ended = event("ground-ended", "이 장판 종료", 40, 4700);
  const endCue = add("cue", "장판 종료 표시", 380, 4700, {
    description:
      "이 장판의 관계·주기·상태 기여분 정리 뒤 장판 외형을 제거합니다.",
  });
  link(ended, "cue", endCue, "event");
  if (mode === "status") {
    const enter = event("ground-enter", "장판 진입", 380, 3500);
    const dwell = event("ground-dwell", "체류 조건 도달", 380, 3900);
    const exit = event("ground-exit", "장판 이탈", 380, 4300);
    for (const n of [enter, dwell, exit]) link(filter, "filter", n, "filter");
    const delay = add("seconds", "필요 체류 시간", 40, 3900);
    link(delay, "value", dwell, "duration");
    const slow = add("ground-status", "진입 둔화 정의", 720, 3500, {
      effect: "slow",
    });
    const stun = add("ground-status", "체류 기절 정의", 720, 3900, {
      effect: "stun",
    });
    for (const [event, status, title, y] of [
      [enter, slow, "진입 대상 둔화", 3500],
      [dwell, stun, "체류 대상 기절", 3900],
    ] as const) {
      const apply = add("ground-apply-status", title, 1060, y);
      link(event, "out", apply, "in");
      link(event, "member", apply, "member");
      link(status, "status", apply, "status");
    }
    const exitCue = add("cue", "장판 이탈 표시", 720, 4300, {
      description:
        "이탈한 대상의 표시를 정리합니다. 상태 해제 여부는 각 상태 정의의 수명을 따릅니다.",
    });
    link(exit, "cue", exitCue, "event");
  } else {
    const periodic = event("ground-periodic", "장판 주기", 380, 3500);
    const interval = add("seconds", "장판 발동 주기", 40, 3900);
    const each = event("ground-targets", "현재 장판 대상", 720, 3500);
    link(interval, "value", periodic, "interval");
    link(periodic, "out", each, "in");
    link(filter, "filter", each, "filter");
    const power = add(
      mode === "damage" ? "damage-amount" : "healing-amount",
      mode === "damage" ? "장판 피해 위력" : "장판 회복 요청량",
      720,
      3900,
    );
    const effect = add(
      mode === "damage" ? "ground-damage" : "ground-heal",
      mode === "damage" ? "대상별 피해" : "대상별 회복",
      1060,
      3500,
    );
    link(each, "out", effect, "in");
    link(each, "member", effect, "member");
    link(power, "power", effect, "power");
  }
  canonicalize(doc);
  return doc;
}
