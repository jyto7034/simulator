import { exampleModifiers } from "./example-modifiers";
import type { GraphNode } from "../types";
import { skillExampleBuilder } from "./skill-example-builder";
type Builder = ReturnType<typeof skillExampleBuilder>;
function sequence(
  b: Builder,
  first: GraphNode,
  next: GraphNode,
  title = "앞 행동 완료 뒤 다음 행동",
) {
  const result = b.add("sequence-plan", title);
  b.link(first, "plan", result, "first");
  b.link(next, "plan", result, "next");
  return result;
}
function wait(b: Builder, title: string) {
  const node = b.add("wait-plan", title);
  b.link(b.seconds(`${title} 시간 · 미정`), "value", node, "duration");
  return node;
}
function selector(b: Builder, title: string) {
  const node = b.add("enemy-selector", title, {
    order: "route",
    air: "include",
  });
  b.link(b.add("range", `${title} 범위 · 미정`), "range", node, "range");
  return node;
}
function impact(b: Builder, time: "accepted" | "start") {
  const plan = b.add(
    "delayed-area-plan",
    "목표 타일 고정 → 지연 → 현재 범위 적 타격",
    { damageType: "physical", selectionTime: time },
  );
  const select = selector(b, "루트 진행도 우선 · 타일 고정");
  b.link(select, "selector", plan, "selector");
  b.link(b.seconds("방출 후 도착 지연 · 미정"), "value", plan, "delay");
  b.link(b.add("range", "착탄 피해 범위 · 미정"), "range", plan, "range");
  b.link(
    b.add("weapon-power", "무기 Attack 비례 · 계수 미정"),
    "power",
    plan,
    "power",
  );
  return { plan, select };
}
export function warmHeartedWoodsman() {
  const b = skillExampleBuilder("따뜻한 마음의 나무꾼 · 범위 3연타");
  b.frame("1타 끌어당김 → 2타 기절 → 3타 대상별 잃은 HP 증가");
  const strikes = [1, 2, 3].map((i) => {
    const strike = b.add("strike-plan", `${i}타 · 현재 전방 범위 재조회`, {
      target: "forward-area",
      damageType: "physical",
    });
    const range = b.add(
      "range",
      i === 1
        ? "최대 2칸 대상 범위 · 실제 타일 미정"
        : `${i}타 효과 타일 · 미정`,
    );
    if (i === 1) {
      const limit = b.add("limited-range", "1타 기본 대상 범위 최대 2칸", {
        tiles: 2,
      });
      b.link(range, "range", limit, "base");
      b.link(limit, "range", strike, "range");
    } else b.link(range, "range", strike, "range");
    const base = b.add("weapon-power", `${i}타 무기 Attack 계수 · 미정`);
    if (i === 3) {
      const hp = b.add("missing-hp-power", "3타 피해 직전 대상별 잃은 HP 비율");
      b.link(base, "power", hp, "base");
      b.link(hp, "power", strike, "power");
    } else b.link(base, "power", strike, "power");
    if (i === 1)
      b.link(
        b.add("hit-pull", "1타 피해 뒤 중심으로 당기기 · 거리 미정"),
        "status",
        strike,
        "after",
      );
    if (i === 2) {
      const stun = b.add("hit-status", "2타 뒤 기절", { effect: "stun" });
      b.link(b.seconds("기절 지속 시간 · 미정"), "value", stun, "duration");
      b.link(stun, "status", strike, "after");
    }
    return strike;
  });
  const first = sequence(b, strikes[0], wait(b, "1타 뒤 간격"));
  const second = sequence(b, strikes[1], wait(b, "2타 뒤 간격"));
  const complete = sequence(b, first, sequence(b, second, strikes[2]));
  const { entry } = b.action(complete);
  b.link(
    selector(b, "첫 유효 적 · 진행도 우선"),
    "selector",
    entry,
    "selector",
  );
  b.frame("장착 유물 → 직접 3연타와 기절만 변경");
  const m = exampleModifiers(b);
  m.bind(
    "deep-wedge",
    "깊은 쐐기 · 시전/대상별 직접 적중",
    "multihit",
    complete,
    "plan",
    { step: 2, cap: 20 },
  );
  m.bind(
    "unbroken-trance",
    "무아지경 · 진행 중 3연타 보호",
    "interruption",
    entry,
    "program",
    {},
    "program",
  );
  for (const strike of strikes) {
    const edge = b.doc.edges.find(
      (e) => e.target.node === strike.id && e.target.port === "range",
    )!;
    m.range(b.doc.nodes.find((n) => n.id === edge.source.node)!);
  }
  const stun = b.doc.nodes.find((n) => n.kind === "skill-hit-status")!;
  m.vulnerability(stun, "hit");
  m.duration(b.doc.nodes.find((n) => n.title === "기절 지속 시간 · 미정")!);
  return b.finish();
}
export function cloudedMonk() {
  const b = skillExampleBuilder("풍운승려 · 아귀의 탐식");
  b.frame("접수 위치 고정 · 준비 → 독립 전달 → 유효 적중당 시전 1회 회복");
  const { plan, select } = impact(b, "accepted");
  const onHit = b.add("on-hit-plan", "적 하나 이상 유효 적중 뒤 자기 회복 1회");
  b.link(plan, "plan", onHit, "body");
  b.link(
    b.add("self-heal-plan", "자기 최대 HP 비율 회복 · 미정"),
    "plan",
    onHit,
    "after",
  );
  const { entry } = b.action(sequence(b, wait(b, "방출 전 준비"), onHit));
  b.link(select, "selector", entry, "selector");
  return b.finish();
}
export function rudolta() {
  const b = skillExampleBuilder("썰매의 루돌타 · 악의의 선물");
  b.frame("시전 중심 고정 · 유한 기회 · 빈 기회 소비 · 낙하 타일 고정");
  const { plan } = impact(b, "start");
  const schedule = b.add("scheduled-plan", "기본 공격 병행 · 독립 투하 시계", {
    first: "immediate",
  });
  b.link(plan, "plan", schedule, "body");
  b.link(
    b.add("count-positive", "총 투하 기회 · 미정"),
    "count",
    schedule,
    "count",
  );
  b.link(b.seconds("투하 기회 간격 · 미정"), "value", schedule, "interval");
  b.action(schedule);
  return b.finish();
}
export function blackSwan() {
  const b = skillExampleBuilder("흑조의 꿈 · 검은 우산");
  b.frame("장착·배치 중 상시 감면과 적 직접 공격 확률 반사");
  const guard = b.add("passive-guard", "이 파편 출처의 상시 피해 감소");
  const reflection = b.add(
    "direct-reflection",
    "반사 연쇄 제외 · 패링 첫 타격 중복 제외",
    { damageType: "original" },
  );
  b.link(guard, "guard", reflection, "guard");
  b.frame("패링 성공 0.5초 면역·받아치기 / 실패 시 자기 감면만 공백");
  const parry = b.add("parry-plan", "첫 타격 선점 → 성공 또는 실패", {
    damageType: "magic",
  });
  b.link(guard, "guard", parry, "guard");
  b.link(b.seconds("받아내기 판정 창"), "value", parry, "window");
  b.link(
    b.seconds("성공 뒤 피해·행동제어 면역", 0.5),
    "value",
    parry,
    "immunity",
  );
  b.link(b.seconds("실패 뒤 감면 공백"), "value", parry, "failure");
  b.link(
    b.add("weapon-power", "받아치기 무기 Attack 계수"),
    "power",
    parry,
    "power",
  );
  b.action(parry);
  b.frame("유물 · 자기 감면 축적 → 성공 때 전량 방출");
  const charge = b.add("guard-charge", "깨어난 흑조 · 자기 감면만 축적");
  b.link(guard, "guard", charge, "guard");
  b.link(parry, "plan", charge, "parry");
  b.link(
    exampleModifiers(b).condition(
      "awakened-black-swan",
      "깨어난 흑조 · 실제 장착 여부",
    ),
    "condition",
    charge,
    "condition",
  );
  b.link(
    b.add("range", "깨어난 흑조 · 주변 파급 타일"),
    "range",
    charge,
    "range",
  );
  return b.finish();
}
