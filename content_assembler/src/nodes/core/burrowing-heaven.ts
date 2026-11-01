import { skillExampleBuilder } from "./skill-example-builder";
import { exampleModifiers } from "./example-modifiers";
import type { GraphNode } from "../types";
import { makeNode } from "./catalog";
import { id } from "./model";
import { resonanceExample } from "./resonance-examples";

// The base fragment's authored structure. Unagreed balance values stay empty.
export function burrowingHeaven() {
  const doc = resonanceExample("all");
  doc.name = "파고드는 천국 · 집중 장판";
  doc.frames[0].title = "공명 만충·전량 소비 · 최대치는 미정";
  const find = (kind: string) =>
    doc.nodes.find((n) => n.kind === `skill-${kind}`)!;
  const end = find("resonance-end");
  doc.nodes = doc.nodes.filter((n) => n.id !== end.id);
  doc.edges = doc.edges.filter(
    (e) => e.source.node !== end.id && e.target.node !== end.id,
  );
  find("resonance-amount").values.value = "";
  find("resonance-amount").title = "최대 공명";
  find("request").title = "파편 사용 요청";
  const frame = {
    id: id(),
    title: "집중 장판 · 시전 위치 고정·기본 공격 중지",
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
  const selection = add("target-selection", "자기 위치에서 시전", 40, 1000, {
    mode: "self",
  });
  link(selection, "selection", find("request"), "selection");
  const duration = add("seconds", "최대 집중 시간", 360, 1000);
  const begin = add("channel-begin", "집중 시작", 680, 1000, {
    basicAttack: "pause",
    interruption: "standard",
  });
  link(duration, "value", begin, "duration");
  link(find("resonance-admit"), "use", begin, "use");
  link(find("resonance-result"), "out", begin, "in");
  const pattern = add("range", "장판 효과 범위", 40, 1500);
  const placement = add("range-placement", "시전 위치 고정", 360, 1500, {
    origin: "caster",
    positionTime: "accepted",
    facingTime: "accepted",
  });
  const area = add("channel-area", "집중에 종속된 장판", 1000, 1000);
  link(pattern, "range", placement, "range");
  link(placement, "area", area, "area");
  link(begin, "out", area, "in");
  link(begin, "channel", area, "channel");
  const cancelInput = add("channel-cancel-request", "직접 취소 입력", 40, 2000);
  const cancel = add("channel-cancel", "플레이어 직접 취소", 360, 2000);
  const ended = add("channel-ended", "집중 종료와 정리", 680, 2000);
  for (const n of [cancelInput, cancel, ended])
    link(begin, "channel", n, "channel");
  link(cancelInput, "out", cancel, "in");
  const startCue = add("cue", "빨간 눈 열기", 1320, 1000, {
    description:
      "시전자 위에 빨간 눈을 표시합니다. 공격·피격·소환 유닛이 아닌 집중 연출입니다.",
  });
  const endCue = add("cue", "빨간 눈 닫기", 1000, 2000, {
    description: "집중의 모든 종료 원인에서 눈을 닫고 제거합니다.",
  });
  link(begin, "cue", startCue, "event");
  link(ended, "cue", endCue, "event");
  const filter = add("ground-filter", "장판 안 적군", 40, 2500, {
    side: "enemies",
    caster: "exclude",
  });
  const enter = add("ground-enter", "진입 즉시 약화", 360, 2500);
  const ramp = add("ground-ramping-slow", "체류에 따른 점진 둔화", 360, 3000);
  const threshold = add("seconds", "기절까지 체류 시간", 40, 3000);
  link(threshold, "value", ramp, "duration");
  for (const n of [enter, ramp]) {
    link(area, "area", n, "area");
    link(filter, "filter", n, "filter");
  }
  const amplify = add("ground-status", "받는 피해 20% 증가", 680, 2500, {
    effect: "damage-taken",
    percent: 20,
    overlap: "strongest",
    lifetime: "membership",
  });
  const stun = add("ground-status", "장판 종료까지 기절", 680, 3000, {
    effect: "stun",
    lifetime: "area",
  });
  for (const [event, status, title, y] of [
    [enter, amplify, "진입 피해 증폭 적용", 2500],
    [ramp, stun, "체류 도달 기절 적용", 3000],
  ] as const) {
    const apply = add("ground-apply-status", title, 1000, y);
    link(event, "out", apply, "in");
    link(event, "member", apply, "member");
    link(status, "status", apply, "status");
  }
  const b = skillExampleBuilder(doc.name, doc);
  b.frame("장착 유물 → 집중·체류·성공한 기절의 독립 변경");
  const m = exampleModifiers(b);
  m.range(pattern);
  m.duration(duration);
  m.vulnerability(stun, "ground");
  m.bind(
    "unbroken-trance",
    "무아지경 · 진행 중 집중 보호",
    "interruption",
    begin,
    "channel",
    {},
    "channel",
  );
  m.bind(
    "tightening-shackles",
    "조여오는 족쇄 · 대상별 체류 진행",
    "ramp",
    ramp,
    "member",
  );
  return b.finish();
}
