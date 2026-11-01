import type { GraphNode } from "../types";
import { skillExampleBuilder } from "./skill-example-builder";
import { exampleModifiers } from "./example-modifiers";
type Builder = ReturnType<typeof skillExampleBuilder>;

function state(
  b: Builder,
  effects: GraphNode[],
  sourceExit: "end" | "independent",
) {
  const result = b.add("target-state", "같은 수명의 효과 묶음", { sourceExit });
  b.link(b.seconds("상태 지속 시간 · 미정"), "value", result, "duration");
  for (const effect of effects) b.link(effect, "effect", result, "effects");
  return result;
}
function attribute(
  b: Builder,
  name: string,
  key: string,
  amount: number | "" = "",
  operation = "percent",
) {
  return b.add("attribute-effect", name, { attribute: key, amount, operation });
}
function recipient(
  b: Builder,
  profile: GraphNode,
  recipient: "self" | "allies",
) {
  const plan = b.add(
    "recipient-buff-plan",
    "방출 시 대상 확정 · 독립 상태 부여",
    { recipient },
  );
  b.link(profile, "state", plan, "state");
  if (recipient === "allies")
    b.link(b.add("range", "아군 효과 범위 · 미정"), "range", plan, "range");
  b.action(plan);
  return plan;
}
export function porccubus() {
  const b = skillExampleBuilder("포큐버스 · 황홀의 포자막");
  b.frame("대상마다 보호량·잔량·시간 분리");
  const shield = b.add(
    "personal-shield-effect",
    "시전자 최대 HP 기반 · 개인 보호량 미정",
  );
  const plan = recipient(b, state(b, [shield], "end"), "allies");
  const single = exampleModifiers(b).bind(
    "single-target-conversion",
    "오롯한 축복 · 개인 보호량 집중",
    "single-support",
    plan,
    "plan",
    { mode: "shield" },
  );
  b.link(
    b.add("range", "오롯한 축복 · 아군 선택 범위 미정"),
    "range",
    single,
    "range",
  );
  return b.finish();
}
export function schadenfreude() {
  const b = skillExampleBuilder("샤덴프로이데 · 멈추지 않는 분쇄");
  b.frame("자기 강화 · 피해의 HP 1 하한 · 종료 시 기여분만 제거");
  const floor = b.add("hp-floor-effect", "전투 피해의 HP 1 하한", {
    minimum: 1,
  });
  recipient(
    b,
    state(
      b,
      [
        floor,
        attribute(b, "공격력 증가", "attack"),
        attribute(b, "공격 속도 포인트 증가", "attack-speed", "", "points"),
        attribute(b, "방어력 증가", "defense"),
      ],
      "end",
    ),
    "self",
  );
  return b.finish();
}
export function scarecrow() {
  const b = skillExampleBuilder("지혜를 구하는 허수아비 · 지혜 수확");
  b.frame("다음 정상 공격 준비 → release 소비 → 직접 처치 1회 보상");
  const reward = state(
    b,
    [attribute(b, "처치 후 공격력 증가 · 20~30% 미정", "attack")],
    "end",
  );
  const plan = b.add(
    "next-basic-plan",
    "무기 기본 공격 1회 강화 · 방출 뒤 환불 없음",
  );
  b.link(reward, "state", plan, "reward");
  b.action(plan);
  return b.finish();
}
export function queenBee() {
  const b = skillExampleBuilder("여왕벌 · 일벌의 가호");
  b.frame("선택한 아군 · 공격당 반응 1회 · 횟수 소비 후 회복");
  const plan = b.add(
    "reactive-heal-plan",
    "왕실의 벌집 · 횟수·10~20% 회복 미정",
  );
  b.link(b.seconds("벌집 지속 시간 · 미정"), "value", plan, "duration");
  b.action(plan, "player-ally");
  return b.finish();
}
export function namelessFetus() {
  const b = skillExampleBuilder("이름이 없는 태아 · 멈추지 않는 울음");
  b.frame("집중 중 아군 범위 기여 · 이탈·중단 시 즉시 해제");
  const plan = b.add(
    "aura-buff-plan",
    "자기 제외 아군 직원 · 기본 공격만 강화",
  );
  const duration = b.seconds("비침식 집중 시간");
  const range = b.add("range", "시전자 중심 원형의 실제 타일");
  b.link(duration, "value", plan, "duration");
  b.link(range, "range", plan, "range");
  for (const effect of [
    attribute(b, "기본 공격 피해 증가", "basic-power", 20),
    attribute(b, "기본 공격 속도 포인트 증가", "basic-speed", 15, "points"),
  ])
    b.link(effect, "effect", plan, "effects");
  const { run } = b.action(plan);
  b.link(
    run,
    "run",
    b.add("action-stop-input", "직접 시전 중지 · 공명 환불 없음"),
    "run",
  );
  const modifiers = exampleModifiers(b);
  modifiers.duration(duration);
  const expanded = modifiers.bind(
    "resonator",
    "울림통 · 기본 원형 반경 +25%의 타일",
    "range",
    range,
    "range",
    { mode: "replace" },
  );
  b.link(
    b.add("range", "울림통 · 반경 +25%를 반영한 타일"),
    "range",
    expanded,
    "replacement",
  );
  const single = modifiers.bind(
    "single-target-conversion",
    "오롯한 축복 · 한 명 강화",
    "single-support",
    plan,
    "plan",
    { mode: "attributes" },
  );
  b.link(
    b.add("range", "오롯한 축복 · 자기 제외 아군 선택 범위"),
    "range",
    single,
    "range",
  );
  for (const effect of [
    attribute(b, "단일 기본 공격 피해 증가", "basic-power", 40),
    attribute(
      b,
      "단일 기본 공격 속도 포인트 증가",
      "basic-speed",
      20,
      "points",
    ),
  ])
    b.link(effect, "effect", single, "effects");
  return b.finish();
}
