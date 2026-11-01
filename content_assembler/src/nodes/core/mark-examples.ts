import { exampleModifiers } from "./example-modifiers";
import type { GraphNode } from "../types";
import { skillExampleBuilder } from "./skill-example-builder";

type Builder = ReturnType<typeof skillExampleBuilder>;
function active(b: Builder, plan: GraphNode) {
  b.frame("시작 → 자격·비용 → 폭발 → 사용 종료");
  const program = b.add("action-entry", "폭발 즉시 시작 · 비용 전 최종 검사", {
    start: "immediate",
    interruption: "standard",
  });
  b.link(plan, "plan", program, "plan");
  const selection = b.add("target-selection", "자기 자신에게 시전", {
    mode: "self",
  });
  const request = b.add("request", "일제 폭발 시전 입력");
  b.link(selection, "selection", request, "selection");
  const use = b.admission(request, b.settings(), false, undefined, program);
  const run = b.add("run-action", "표식 고정·소비와 폭발 계획 실행");
  b.link(use, "out", run, "in");
  b.link(use, "use", run, "use");
  b.link(program, "program", run, "program");
}
function explosion(b: Builder, plan: GraphNode) {
  const rule = b.add(
    "explosion-rule",
    "차례당 대상 1회 · 폭발 피해 증가 별도",
    { dedup: "wave", damageType: "magic", bonus: 0 },
  );
  const range = b.add("range", "각 폭발 중심 1칸", {
    pattern: "@",
    includeAnchor: true,
  });
  b.link(rule, "explosion", plan, "explosion");
  b.link(range, "range", plan, "range");
  return { rule, range };
}
function store(b: Builder, stacks: boolean) {
  const marks = b.add(
    "target-marks",
    stacks ? "무기 사본의 유닛→타일 나비 스택" : "파편 사본의 포자 감염 숙주",
    {
      mode: stacks ? "stacks" : "presence",
      lifetime: stacks ? "battle" : "deployment",
      death: stacks ? "tile" : "remove",
    },
  );
  const filter = b.add("mark-filter", "지상 적 · 보스 포함 · 소환·임시 제외");
  const hit = b.add(
    "mark-weapon-hit",
    stacks
      ? "승급 미니건의 직접 적중당 1스택"
      : "자기 기본 공격의 대상별 최초 감염",
    {
      binding: stacks ? "weapon-upgrade" : "fragment",
      weapon: stacks ? "funeral-minigun" : "",
      chance: stacks ? 100 : "",
      amount: 1,
      grantMultiplier: 1,
    },
  );
  b.link(marks, "marks", hit, "marks");
  b.link(filter, "filter", hit, "filter");
  const hits = [hit];
  if (stacks) {
    const evolution = b.add(
      "weapon-evolution",
      "개화 거래 → 선택한 전용 무기 한 사본 영구 승급",
      {
        fragment: "funeral-of-dying-butterflies",
        form: "bloom",
        weapon: "funeral-minigun",
        base: "paired-pistols",
        upgraded: "minigun",
      },
    );
    const owner = b.add(
      "weapon-mark-owner",
      "형태·파편 교체에도 같은 무기 사본의 나비",
      { key: "butterflies" },
    );
    b.link(marks, "marks", owner, "marks");
    b.link(evolution, "evolution", owner, "evolution");
    const base = b.add(
      "mark-weapon-hit",
      "쌍권총의 직접 적중 · 기본 부여량 미정",
      {
        binding: "weapon-base",
        weapon: "funeral-minigun",
        chance: 100,
        amount: "",
        grantMultiplier: 1,
      },
    );
    b.link(marks, "marks", base, "marks");
    b.link(filter, "filter", base, "filter");
    for (const binding of [hit, base])
      b.link(evolution, "evolution", binding, "evolution");
    hits.push(base);
  }
  return { marks, filter, hit, hits };
}
export function meltingLove() {
  const b = skillExampleBuilder("녹아내리는 사랑 · 공격형 포자 감염");
  b.frame("장착 intrinsic · 최초 감염과 비치명 피격/사망 전파");
  const { marks, filter, hit } = store(b, false);
  const spread = b.add("mark-spread", "사망 우선 · 다음 세대는 별도 사건");
  b.link(marks, "marks", spread, "marks");
  b.link(filter, "filter", spread, "filter");
  b.link(hit, "source", spread, "source");
  b.frame("감염체 수×시간의 성장 · 독립 피해 주기");
  const power = b.add(
    "population-power",
    "출처별 연속 위력 · 0숙주 유예 후 감소",
  );
  b.link(marks, "marks", power, "marks");
  b.link(b.seconds("0숙주 위력 보존 유예"), "value", power, "grace");
  const periodic = b.add(
    "mark-periodic-damage",
    "현재 공유 위력으로 각 숙주 주기 피해",
    { damageType: "magic", coefficient: 1 },
  );
  b.link(marks, "marks", periodic, "marks");
  b.link(power, "power", periodic, "power");
  b.link(b.seconds("감염 피해 주기"), "value", periodic, "interval");
  b.frame("숙주 존재 검사 · 위치·위력 고정 · 전량 소비");
  const burst = b.add(
    "mark-burst-plan",
    "일제 기폭 · 자기 숙주 3배 · 전파 억제",
    { coefficient: 1, hostMultiplier: 3 },
  );
  b.link(marks, "marks", burst, "marks");
  b.link(power, "power", burst, "power");
  const { rule, range } = explosion(b, burst);
  active(b, burst);
  b.frame("장착 유물 → 지정 효과의 변경 · 원본 값 보존");
  const m = exampleModifiers(b);
  const interval = b.doc.nodes.find((n) => n.title === "감염 피해 주기")!;
  m.bind(
    "hastened-pulse",
    "재촉하는 맥박 · 피해 주기만 단축",
    "duration",
    interval,
    "value",
  );
  m.bind(
    "overlapping-explosions",
    "중첩된폭발 · 별개 폭발의 중복",
    "explosion-overlap",
    rule,
    "explosion",
  );
  m.range(range, true);
  m.bind(
    "explosion-vulnerability",
    "압축 화약 · 폭발 피해만 증가",
    "explosion-power",
    rule,
    "explosion",
  );
  m.bind(
    "growth-medium",
    "증식의 배양액 · 새 성장량만 증가",
    "population",
    power,
    "power",
    { mode: "growth" },
  );
  m.bind(
    "sealed-culture",
    "밀봉된 배양병 · 0숙주 자연 감소 방지",
    "population",
    power,
    "power",
    { mode: "preserve" },
  );
  return b.finish();
}
export function funeralButterflies(bloom = false, orb = false) {
  const b = skillExampleBuilder(
    `죽을 나비의 장례 · ${bloom ? "개화형" : "기본형"}${orb ? " · 검은 고치" : ""}`,
  );
  b.frame(
    bloom
      ? "승급 무기 적중 → 유닛 스택 → 사망 중심 타일 이전"
      : "방출 때 모든 경로 타일 고정 → 지연 → 현재 적 판정",
  );
  const power = b.add("damage-amount", "폭발 기준 위력 · 미정");
  const plan = b.add(
    bloom ? "stack-burst-plan" : "path-burst-plan",
    bloom
      ? "타일·생존 스택 합산 후 차례별 폭발"
      : "모든 walkable path의 지연 폭발",
  );
  b.link(b.seconds("첫 폭발까지 지연"), "value", plan, "delay");
  const { marks, hits } = store(b, true);
  if (bloom) {
    b.link(marks, "marks", plan, "marks");
    const grade = b.add("stack-grade", "q=0이면 S회 · q≥1이면 q등급 q회");
    b.link(power, "power", grade, "power");
    b.link(grade, "grade", plan, "grade");
    b.link(b.seconds("연속 폭발 차례 간격"), "value", plan, "interval");
    if (orb) {
      const conversion = b.add(
        "burst-conversion",
        "검은 고치 · 총 스택 임계치의 단일 대체 폭발",
      );
      b.link(
        exampleModifiers(b).condition(
          "funeral-butterfly-orb",
          "검은 고치 · 장착 여부",
        ),
        "condition",
        conversion,
        "condition",
      );
      b.link(power, "power", conversion, "power");
      b.link(
        b.add("range", "구체 피해 범위 · 반드시 별도 지정"),
        "range",
        conversion,
        "range",
      );
      b.link(
        b.seconds("구체 집결의 전투 정지 연출 시간"),
        "value",
        conversion,
        "presentation",
      );
      b.link(conversion, "conversion", plan, "conversion");
    }
  } else b.link(power, "power", plan, "power");
  const { rule, range } = explosion(b, plan);
  active(b, plan);
  b.frame("장착 유물 → 새 부여·폭발 변경 · 이전 스택 보존");
  const m = exampleModifiers(b);
  m.bind(
    "overlapping-explosions",
    "중첩된폭발 · 별개 폭발의 중복",
    "explosion-overlap",
    rule,
    "explosion",
  );
  m.range(range, true);
  m.bind(
    "explosion-vulnerability",
    "압축 화약 · 일반/등급/구체 폭발",
    "explosion-power",
    rule,
    "explosion",
  );
  for (const hit of hits)
    m.bind(
      "deep-imprint",
      "깊은 각인 · 무기 신규 부여만 2배",
      "mark-grant",
      hit,
      "source",
      { factor: 2 },
    );
  return b.finish();
}
