import { skillExampleBuilder } from "./skill-example-builder";
import { exampleModifiers } from "./example-modifiers";
import type { GraphNode } from "../types";
import { makeNode } from "./catalog";
import { id } from "./model";
import { resonanceExample } from "./resonance-examples";

export function mountainOfSmilingBodies() {
  const doc = resonanceExample("all");
  doc.name = "웃는 시체의 산 · 시체들의 합창";
  doc.frames[0].title = "공명 만충·전량 소비 · 최대치는 미정";
  const find = (kind: string) =>
    doc.nodes.find((n) => n.kind === `skill-${kind}`)!;
  const end = find("resonance-end");
  doc.edges = doc.edges.filter((e) => e.target.node !== end.id);
  find("resonance-amount").values.value = "";
  find("resonance-amount").title = "최대 공명";
  find("request").title = "파편 사용 요청";
  const frame = { id: id(), title: "일회 범위 조회 · 대상별 공통 수명" };
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
  const selection = add("target-selection", "자기 중심 시전", 40, 1000, {
    mode: "self",
  });
  link(selection, "selection", find("request"), "selection");
  const pattern = add("range", "광역 도발 범위", 40, 1500);
  const placement = add("range-placement", "방출 시 시전자 중심", 360, 1500, {
    origin: "caster",
    positionTime: "release",
    facingTime: "release",
  });
  link(pattern, "range", placement, "range");
  const filter = add("ground-filter", "범위 안 살아 있는 적", 40, 2000, {
    side: "enemies",
    caster: "exclude",
  });
  const query = add("range-targets", "도발 대상 한 번 조회", 680, 1000);
  link(find("resonance-result"), "out", query, "in");
  link(find("resonance-admit"), "use", query, "use");
  link(placement, "area", query, "area");
  link(filter, "filter", query, "filter");
  link(query, "out", end, "in");
  link(find("resonance-admit"), "use", end, "use");
  end.position = { x: 1000, y: 600 };
  end.title = "부여 완료 · 공명 충전 재개";
  const duration = add("seconds", "도발 공통 지속 시간", 360, 2000);
  const state = add("target-state", "도발·접근 제약의 공통 수명", 680, 1600, {
    sourceExit: "end",
  });
  link(duration, "value", state, "duration");
  const taunt = add("target-taunt", "시전자만 저지 가능한 도발", 40, 2500);
  const speed = add("target-speed", "접근 이동 속도 증가", 360, 2500);
  const limit = add("target-attack-limit", "1타일 이내 공격 제한", 680, 2500, {
    tiles: 1,
  });
  for (const effect of [taunt, speed, limit])
    link(effect, "effect", state, "effects");
  const apply = add("target-apply", "대상별 도발 묶음 적용", 1000, 1000);
  link(query, "each", apply, "in");
  link(query, "target", apply, "target");
  link(state, "state", apply, "state");
  const ended = add("target-ended", "이번 대상의 세 효과 정리", 1320, 1600);
  link(apply, "instance", ended, "instance");
  for (const [event, title, y, description] of [
    [
      apply,
      "대상 도발 표시",
      1000,
      "상태 적용에 성공한 적에게 시체들의 합창 표식을 표시합니다. 직접 피해나 게임 유닛을 만들지 않습니다.",
    ],
    [
      ended,
      "이번 도발 표식 종료",
      1600,
      "종료한 대상 활성의 표식만 제거합니다. 대체된 새 도발이나 다른 대상의 표식을 지우지 않습니다.",
    ],
  ] as const) {
    const cue = add("cue", title, 1640, y, { description });
    link(event, "cue", cue, "event");
  }
  const b = skillExampleBuilder(doc.name, doc);
  b.frame("장착 유물 → 도발 범위·수명·실제 관계의 피해");
  const m = exampleModifiers(b);
  m.range(pattern);
  m.duration(duration);
  m.bind(
    "taunted-enemy-damage-reduction",
    "도전자의 갑주 · 자기 도발 적에게 받는 피해",
    "taunt-damage",
    state,
    "state",
    { mode: "taken" },
  );
  m.bind(
    "taunted-enemy-basic-attack-boost",
    "응징의 표식 · 자기 도발 적에게 기본 공격",
    "taunt-damage",
    state,
    "state",
    { mode: "dealt-basic" },
  );
  b.frame("장착 무기 사본의 주 대상 막타 성장 · 파편 도발은 획득 없음");
  const growth = b.add(
    "weapon-kill-growth",
    "웃음 · 주 대상 막타만 시체 질량 획득",
    { weapon: "laughter", key: "corpse-mass" },
  );
  for (const [port, label] of [
    ["normal", "일반"],
    ["elite", "엘리트"],
    ["abnormality", "환상체"],
  ]) {
    const amount = b.add("integer-amount", `${label} 막타의 새 질량 획득량`);
    b.link(amount, "value", growth, port);
    m.bind(
      "double-harvest",
      `두 겹의 수확 · ${label} 신규 질량만 2배`,
      "number",
      amount,
      "value",
      { factor: 2 },
    );
  }
  return b.finish();
}
