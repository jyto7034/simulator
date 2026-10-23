import { exampleModifiers } from "./example-modifiers";
import type { GraphNode } from "../types";
import { skillExampleBuilder } from "./skill-example-builder";

export function knightOfDespair(bloom = false) {
  const b = skillExampleBuilder(
    bloom ? "절망의 기사 · 개화형" : "절망의 기사 · 기본형",
  );
  const { add, link, seconds, frame } = b;
  const count = (title: string, value: number | "" = "") =>
    add("count-constant", title, { value });
  const power = (title: string) => add("damage-amount", title);
  const sequence = (title: string, first: GraphNode, next: GraphNode) => {
    const n = add("sequence-plan", title);
    link(first, "plan", n, "first");
    link(next, "plan", n, "next");
    return n;
  };
  const status = (
    title: string,
    effect: string,
    duration: number,
    stacks?: GraphNode,
  ) => {
    const n = add("hit-status", title, {
      effect,
      ...(effect === "vulnerability" ? { percent: 5 } : {}),
    });
    link(seconds(`${title} · 지속 시간`, duration), "value", n, "duration");
    if (stacks) link(stacks, "count", n, "count");
    return n;
  };
  const strike = (
    title: string,
    damage: GraphNode,
    target: string,
    after?: GraphNode,
    range?: GraphNode,
  ) => {
    const n = add("strike-plan", title, { target, damageType: "physical" });
    link(damage, "power", n, "power");
    if (after) link(after, "status", n, "after");
    if (range) link(range, "range", n, "range");
    return n;
  };
  const entry = (
    title: string,
    plan: GraphNode,
    start: string,
    interruption = "standard",
  ) => {
    const n = add("action-entry", title, { start, interruption });
    link(plan, "plan", n, "plan");
    return n;
  };
  const wait = (title: string) => {
    const n = add("wait-plan", title);
    link(seconds(`${title} · 시간 미정`), "value", n, "duration");
    return n;
  };
  frame("전투 사본 · 가호와 검");
  const swords = add("stock", "검 재고 · 기본 상한 4", {
    initial: 0,
    lifetime: "battle",
    limit: "bounded",
    capacity: 4,
  });
  const mode = add("battle-mode", "절망 · 전투 중 유지");
  const relation = add("ally-relation", "가호 대상 지정·해제·교체", {
    entry: "blessing",
  });
  link(mode, "mode", relation, "blocked");
  const reduction = add("relation-reduction", "가호 피해 감소 · 20~30% 조정");
  link(relation, "relation", reduction, "relation");
  const shield = add("relation-emergency-shield", "긴급 보호막 · 전투당 1회", {
    threshold: 10,
    shield: 50,
  });
  link(relation, "relation", shield, "relation");
  link(seconds("보호막 15초", 15), "value", shield, "duration");
  const timer = add("relation-periodic", "가호 부착부터 검 생성 주기");
  link(relation, "relation", timer, "relation");
  link(swords, "bank", timer, "bank");
  link(seconds("검 생성 주기 · 미정"), "value", timer, "interval");
  const gain = add("resource-credit", "주기 완료 검 1개", { amount: 1 });
  link(timer, "out", gain, "in");
  link(timer, "grant", gain, "grant");
  link(swords, "bank", gain, "bank");
  frame("행동 사이의 절망 전환");
  const death = add("relation-hp-zero", "가호 대상의 HP 0 퇴각만");
  link(relation, "relation", death, "relation");
  const defer = add("defer-mode", "현재 행동 전체 종료 뒤 전환 예약");
  link(death, "out", defer, "in");
  link(death, "event", defer, "event");
  link(mode, "mode", defer, "mode");
  const entered = add("mode-entered", "다음 행동 직전 절망 진입");
  link(mode, "mode", entered, "mode");
  const clear = add("relation-clear", "절망 진입 · 가호 관계 정리");
  link(entered, "out", clear, "in");
  link(relation, "relation", clear, "relation");
  const stockCount = add("count-stock", "행동 시작의 검 수 S");
  link(swords, "bank", stockCount, "bank");
  const self = add("self-stock-damage", "검당 최대 HP 5% · 보호막 무시", {
    percent: 5,
    damage: "fixed-bypass",
  });
  link(clear, "out", self, "in");
  link(stockCount, "count", self, "count");
  let normal: GraphNode, despair: GraphNode;
  if (!bloom) {
    frame("일반 · 시작 검 수로 고정한 단계 소비");
    const perStep = add("count-quotient", "단계당 검 비용 · 시작 재고의 몫", {
      divisor: 4,
      minimum: 1,
    });
    link(stockCount, "count", perStep, "count");
    const paid = add("count-paid", "단계 실제 소비 검 수");
    const wound = status(
      "단계 직후 상처 · 소비 검당 1",
      "vulnerability",
      5,
      paid,
    );
    const stun = status("찌르기 기절 1초", "stun", 1);
    const first = strike(
      "검 없이도 첫 찌르기",
      power("첫 찌르기 위력 · 미정"),
      "primary",
      stun,
    );
    const attackLock = status("아탕호 · 기본 공격 불가 3초", "basic-lock", 3);
    const a = sequence(
      "아탕호 준비 → 타격",
      wait("아탕호 준비"),
      strike("아탕호 타격", power("아탕호 위력 · 미정"), "primary", attackLock),
    );
    const e = sequence(
      "에스토카다 준비 → 타격",
      wait("에스토카다 준비"),
      strike("에스토카다 타격", power("에스토카다 위력 · 미정"), "primary"),
    );
    const stance = add("stance-plan", "데스비오 · 방어 자세", { percent: 20 });
    link(seconds("방어 자세 1초", 1), "value", stance, "duration");
    const front = add("range", "데스비오 · 전방 2칸", {
      pattern: "X\nX\n@",
      includeAnchor: false,
    });
    const d = sequence(
      "데스비오 자세 → 전방 타격",
      stance,
      strike(
        "데스비오 전방 타격",
        power("데스비오 위력 · 미정"),
        "forward-area",
        stun,
        front,
      ),
    );
    const n = add("count-positive", "엑시큐션 기본 타수 n · 미정"),
      duration = seconds("엑시큐션 공통 총시간 · 미정"),
      executionPower = power("엑시큐션 타격당 위력 · 미정");
    const normalRepeat = add("repeat-plan", "일반 엑시큐션 · n회", {
      timing: "fixed-total",
    });
    link(
      strike("일반 엑시큐션 개별 타격", executionPower, "primary"),
      "plan",
      normalRepeat,
      "body",
    );
    link(n, "count", normalRepeat, "count");
    link(duration, "value", normalRepeat, "time");
    const x = sequence(
      "엑시큐션 준비 → 연타",
      wait("엑시큐션 준비"),
      normalRepeat,
    );
    const stages = [a, e, d, x].map((body, i) => {
      const step = add(
        "spend-plan",
        `${["아탕호", "에스토카다", "데스비오", "엑시큐션"][i]} · c개 소비 → 공격 → 상처`,
        { spend: "amount" },
      );
      link(swords, "bank", step, "bank");
      link(perStep, "count", step, "count");
      link(body, "plan", step, "body");
      link(wound, "status", step, "after");
      return step;
    });
    let tail = stages[3];
    for (let i = 2; i >= 0; i--)
      tail = sequence(`일반 연속기 · ${i + 1}단계 이후`, stages[i], tail);
    normal = entry(
      "일반 · 다음 기본 공격 한 번 교체",
      sequence("첫 찌르기 → 검 소비 연속기", first, tail),
      "next-basic",
    );
    frame("절망 · 검 보존·같은 총시간·대상별 후속 증가");
    const ramp = add("hit-ramp", "이번 엑시큐션 후속 타격만 +10%p", {
      percent: 10,
    });
    const hit = strike("절망 엑시큐션 개별 타격", executionPower, "primary");
    link(ramp, "ramp", hit, "ramp");
    const total = add("count-sum", "절망 타수 n + S");
    link(n, "count", total, "first");
    link(stockCount, "count", total, "second");
    const repeat = add("repeat-plan", "절망 엑시큐션 · 같은 총시간", {
      timing: "fixed-total",
    });
    link(hit, "plan", repeat, "body");
    link(total, "count", repeat, "count");
    link(duration, "value", repeat, "time");
    despair = entry("절망 · 별도 시전으로 즉시 엑시큐션", repeat, "immediate");
    const selector = add("enemy-selector", "절망 주 대상 · 저지 상대 우선", {
      order: "block-route",
      air: "exclude",
    });
    link(
      add("range", "절망 주 대상 선택 범위 · 미정"),
      "range",
      selector,
      "range",
    );
    link(selector, "selector", despair, "selector");
  } else {
    frame("개화 일반 · 큰 검 방출 때 전량 소비");
    const footprint = add("range", "큰 검·작은 검 공통 전체 범위 · 미정");
    const stun = status("큰 검만 기절 2초", "stun", 2);
    const big = strike(
      "큰 검 · 현재 범위 전체",
      power("큰 검 위력 · 미정"),
      "selected-area",
      stun,
      footprint,
    );
    const small = strike(
      "작은 검 하나 · 같은 전체 범위",
      power("작은 검 위력 · 미정"),
      "selected-area",
      undefined,
      footprint,
    );
    const paid = add("count-paid", "큰 검 방출 시 실제 소비 수");
    const repeat = add("repeat-plan", "작은 검 · 실제 소비 수만큼 순차", {
      timing: "interval",
      first: "after-interval",
    });
    link(small, "plan", repeat, "body");
    link(paid, "count", repeat, "count");
    link(seconds("작은 검 낙하 간격 · 미정"), "value", repeat, "time");
    const spend = add("spend-plan", "큰 검 방출 · 현재 검 전량 소비", {
      spend: "all",
    });
    link(swords, "bank", spend, "bank");
    link(
      sequence("큰 검 → 작은 검 순차 낙하", big, repeat),
      "plan",
      spend,
      "body",
    );
    normal = entry(
      "개화 일반 · 지정 타일 순차 낙검",
      sequence("큰 검 준비 → 방출 시 소비", wait("큰 검 방출 전 준비"), spend),
      "immediate",
    );
    frame("개화 절망 · 검 보존·유한 유지·두 독립 주기");
    const wound = status(
      "주기 타격 뒤 상처 1스택",
      "vulnerability",
      5,
      count("주기 상처 1개", 1),
    );
    const area = add("range", "개화 절망 고정 타일 범위 · 미정");
    const hit = strike(
      "0.5초마다 현재 적 피해 → 상처",
      power("개화 절망 주기 위력 · 미정"),
      "selected-area",
      wound,
      area,
    );
    const duration = add("count-duration", "검 수에 따른 유한 최대 유지 시간");
    link(stockCount, "count", duration, "count");
    link(seconds("개화 절망 기본 시간 · 미정"), "value", duration, "base");
    link(seconds("검 1개당 추가 시간 · 미정"), "value", duration, "perCount");
    const timed = add("timed-plan", "개화 절망 · 최대 시간까지 유지", {
      first: "after-interval",
    });
    link(hit, "plan", timed, "body");
    link(duration, "value", timed, "duration");
    link(seconds("범위 타격 주기 0.5초", 0.5), "value", timed, "interval");
    despair = entry(
      "개화 절망 · 침묵에도 즉시 중단",
      timed,
      "immediate",
      "silence",
    );
  }
  frame("하나의 요청·공명 접수·상태별 행동 실행");
  const program = add("mode-program", "일반/절망 행동 선택 · 한 번 고정");
  link(mode, "mode", program, "mode");
  link(normal, "program", program, "normal");
  link(despair, "program", program, "active");
  const selection = add("target-selection", "스킬 입력 선택", {
    mode: bloom ? "player-tile" : "self",
  });
  if (bloom)
    link(
      add("range", "개화 타일 선택 범위 · 미정"),
      "range",
      selection,
      "range",
    );
  const request = add("request", "파편 시전 입력");
  link(selection, "selection", request, "selection");
  const use = b.admission(request, b.settings(), false, undefined, program);
  const run = add("run-action", "접수한 전체 행동 실행");
  link(use, "out", run, "in");
  link(use, "use", run, "use");
  link(program, "program", run, "program");
  if (bloom) {
    const self = add(
      "action-periodic-self-damage",
      "개화 절망 · 주기 최대 HP 피해",
      { percent: 10, damageType: "physical" },
    );
    link(run, "run", self, "run");
    link(despair, "program", self, "entry");
    link(seconds("유지 자기 피해 1초", 1), "value", self, "interval");
    const stop = add("action-stop-input", "개화 절망 직접 중단");
    link(run, "run", stop, "run");
  }
  frame("장착 유물 → 검 획득·상처·가호·행동을 각각 변경");
  const m = exampleModifiers(b);
  m.bind(
    "hasty-second-hand",
    "성급한 초침 · 검 생성 주기",
    "duration",
    b.doc.nodes.find((n) => n.title === "검 생성 주기 · 미정")!,
    "value",
  );
  m.bind(
    "bottomless-cup",
    "밑 빠진 잔 · 검 재고 상한만 해제",
    "capacity",
    swords,
    "bank",
  );
  m.bind(
    "layered-shell",
    "겹친 외피 · 긴급 보호막 새 생성량",
    "relation-shield",
    relation,
    "relation",
  );
  m.bind(
    "double-harvest",
    "두 겹의 수확 · 검 새 획득 1→2",
    "grant",
    timer,
    "grant",
    { factor: 2 },
  );
  m.bind(
    "unbroken-trance",
    "무아지경 · 진행 중 일반/절망 보호",
    "interruption",
    program,
    "program",
    {},
    "program",
  );
  const source = (node: GraphNode, port: string) =>
    b.doc.nodes.find(
      (n) =>
        n.id ===
        b.doc.edges.find(
          (e) => e.target.node === node.id && e.target.port === port,
        )?.source.node,
    )!;
  for (const node of [...b.doc.nodes]) {
    if (node.kind === "skill-hit-status" && node.values.effect === "stun")
      m.vulnerability(node, "hit");
    if (
      node.kind === "skill-hit-status" &&
      node.values.effect === "vulnerability"
    )
      m.bind(
        "deep-imprint",
        "깊은 각인 · 새 상처 스택 2배",
        "hit-count",
        node,
        "status",
        { factor: 2 },
      );
    if (node.kind === "skill-strike-plan" && node.values.target === "primary")
      m.bind(
        "widened-horizon",
        "넓어진 지평 · 주 대상 타일 피해",
        "primary-area",
        node,
        "plan",
      );
  }
  const ranges = new Set(
    b.doc.nodes
      .filter(
        (n) => n.kind === "skill-strike-plan" && n.values.target !== "primary",
      )
      .map((n) => source(n, "range")),
  );
  for (const r of ranges) m.range(r);
  for (const entry of bloom ? [normal] : [normal, despair]) {
    m.bind(
      "deep-wedge",
      "깊은 쐐기 · 같은 시전/대상 직접 적중",
      "multihit",
      source(entry, "plan"),
      "plan",
      { step: 2, cap: 20 },
    );
    if (!bloom)
      m.bind(
        "next",
        "다음. · 사망 위치 한 칸의 새 대상",
        "retarget",
        entry,
        "program",
        { tiles: 1 },
      );
  }
  return b.finish();
}
