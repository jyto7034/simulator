import { exampleModifiers } from "./example-modifiers";
import type { GraphNode } from "../types";
import { skillExampleBuilder } from "./skill-example-builder";

export function queenOfHatred(bloom = false) {
  const b = skillExampleBuilder(`증오의 여왕 · ${bloom ? "개화형" : "기본형"}`);
  const { add, link, seconds } = b;
  b.frame("사본별 하트 · 배치 중 주기 획득 · 조준 기억");
  const settings = b.settings();
  const counter = add("battle-counter", "하트 · 사본별 상한 6", {
    capacity: 6,
  });
  const memory = add("aim-memory", "마지막 일반 광선의 접수 타일");
  link(counter, "counter", memory, "counter");
  const interval = seconds("배치 중 하트 획득 주기", 15);
  const periodic = add(
    "counter-periodic",
    "일반 시전 중 진행 · 히스테릭 예약부터 정지",
  );
  link(counter, "counter", periodic, "counter");
  link(interval, "value", periodic, "interval");
  const one = add("number", "시간 획득 하트 1개", { value: 1 });
  const gain = add("counter-gain", "주기마다 하트 1개 · 상한 초과 버림");
  link(counter, "counter", gain, "counter");
  link(one, "value", gain, "amount");
  link(periodic, "out", gain, "in");

  const manual = (entry: string, minimum: number, spend: string) => {
    const selectionRange = add("range", "지정 타일 선택 범위 · 미정");
    const selection = add("target-selection", "빈 유효 타일도 지정 가능", {
      mode: "player-tile",
    });
    link(selectionRange, "range", selection, "range");
    const request = add(
      "menu-request",
      entry === "beam" ? "일반 광선 입력" : "수동 블랙홀 입력",
      { entry },
    );
    link(selection, "selection", request, "selection");
    const terms = add(
      "counter-terms",
      `하트 ${minimum}~5 · ${spend === "none" ? "소비 없음" : "접수 시 전량 소비"}`,
      { minimum, maximum: 5, spend },
    );
    link(counter, "counter", terms, "counter");
    return b.admission(request, settings, false, terms);
  };
  const channel = (use: GraphNode, predecessor: GraphNode, title: string) => {
    const owner = add("sequence-channel", title, {
      basicAttack: "pause",
      interruption: "standard",
    });
    link(use, "use", owner, "use");
    link(predecessor, "out", owner, "in");
    return owner;
  };
  const placement = (title: string, origin: string, pattern?: GraphNode) => {
    const range = pattern ?? add("range", `${title} 효과 타일 · 미정`);
    const p = add("range-placement", title, { origin });
    link(range, "range", p, "range");
    return p;
  };
  const phase = (
    owner: GraphNode,
    title: string,
    duration: number | "",
    area: GraphNode,
  ) => {
    const time = seconds(`${title} 유지 시간`, duration);
    const p = add("channel-phase", title);
    link(time, "value", p, "duration");
    link(area, "area", p, "area");
    const instance = add("phase-area", `${title} · 활성 중에만 존재`);
    link(owner, "channel", instance, "channel");
    link(p, "phase", instance, "phase");
    const interval = seconds(`${title} 판정 주기`, 0.5);
    const pulse = add("ground-periodic", `${title} 즉시 첫 판정`, {
      first: "immediate",
    });
    link(instance, "area", pulse, "area");
    link(interval, "value", pulse, "interval");
    return { definition: p, area: instance, pulse };
  };
  type Phase = ReturnType<typeof phase>;
  const query = (
    p: Phase,
    predecessor: GraphNode,
    flow: string,
    side: string,
    title: string,
  ) => {
    const filter =
      side === "corrupted"
        ? add("corrupted-filter", "흡인 · 침식체 적만")
        : add("ground-filter", title, {
            side,
            caster: side === "enemies" ? "exclude" : "include",
          });
    const q = add("ground-targets", title);
    link(p.area, "area", q, "area");
    link(filter, "filter", q, "filter");
    link(predecessor, flow, q, "in");
    return q;
  };
  const pull = (p: Phase) => {
    const q = query(
      p,
      p.pulse,
      "out",
      "corrupted",
      "흡인할 침식체 현재 위치 조회",
    );
    const effect = add("ground-pull", "한 주기 흡인 거리 · 미정");
    link(q, "out", effect, "in");
    link(q, "member", effect, "member");
    return q;
  };
  const effect = (
    p: Phase,
    previous: GraphNode,
    flow: string,
    healing: boolean,
    all = false,
    count?: GraphNode,
  ) => {
    const q = query(
      p,
      previous,
      flow,
      healing ? "allies" : all ? "all" : "enemies",
      healing
        ? "현재 범위의 아군 · 시전자 포함"
        : all
          ? "현재 범위의 모든 편 · 시전자 포함"
          : "현재 범위의 모든 적",
    );
    let power = add(
      healing ? "healing-amount" : "damage-amount",
      `${p.definition.title} · ${healing ? "기본 회복 요청량" : "기본 피해 위력"} 미정`,
    );
    if (count) {
      const scaled = add(
        healing ? "count-healing" : "count-damage",
        `${p.definition.title} · 확정 소비 하트 비례 ${healing ? "회복" : "피해"} · 증가율 미정`,
      );
      link(power, "power", scaled, "base");
      link(count, "count", scaled, "count");
      power = scaled;
    }
    const e = add(
      healing ? "ground-heal" : "ground-damage",
      healing
        ? "아군 실제 HP 회복 결과"
        : all
          ? "히스테릭 · 아군과 적군 마법 피해"
          : "적 실제 HP 피해 결과",
      healing ? {} : { damageType: "magic" },
    );
    link(q, "out", e, "in");
    link(q, "member", e, "member");
    link(power, "power", e, "power");
    return { query: q, effect: e };
  };
  b.frame("일반 광선 · 2초 집중 · 실제 적 피해와 아군 회복 · 완료 보상");
  const beamUse = manual("beam", 0, "none");
  const remember = add("remember-aim", "접수한 일반 광선 타일 기억");
  link(beamUse, "out", remember, "in");
  link(beamUse, "use", remember, "use");
  link(memory, "memory", remember, "memory");
  const beamOwner = channel(
    beamUse,
    remember,
    "일반 광선 · 중단에는 하트 보상 없음",
  );
  const beam = phase(
    beamOwner,
    "일반 광선",
    2,
    placement("접수 위치에서 지정 타일 방향", "selected-direction"),
  );
  link(beam.definition, "phase", beamOwner, "first");
  const damage = effect(beam, beam.pulse, "out", false);
  const healing = effect(beam, damage.query, "completed", true);
  const summary = add("cast-outcomes", "같은 광선의 실제 피해·회복 성공 집계");
  link(beamOwner, "channel", summary, "channel");
  link(damage.effect, "result", summary, "damage");
  link(healing.effect, "result", summary, "healing");
  const award = add("completion-award", "완주 1개 + 피해·회복 모두 성공 1개", {
    base: 1,
    bonus: 1,
  });
  link(summary, "summary", award, "summary");
  link(beamOwner, "completed", award, "in");
  const reward = add("counter-gain", "완료 보상 전체 확정 뒤 상한 판단");
  link(counter, "counter", reward, "counter");
  link(award, "out", reward, "in");
  link(award, "amount", reward, "amount");

  b.frame(
    `수동 블랙홀 · 하트 ${bloom ? "0" : "3"}~5 전량 소비${bloom ? " · 빛기둥 후속" : " · 적 피해 없음"}`,
  );
  const blackUse = manual("black-hole", bloom ? 0 : 3, "admission-all");
  const receipt = add("counter-receipt", "이번 접수에서 실제 소비한 하트");
  link(blackUse, "out", receipt, "in");
  link(blackUse, "counterReceipt", receipt, "receipt");
  const blackOwner = channel(
    blackUse,
    receipt,
    "블랙홀 연속 집중 · 하트 생성 없음",
  );
  const ranges = add("count-range", "소비 하트별 흡인·회복 범위");
  link(receipt, "count", ranges, "count");
  for (let count = bloom ? 0 : 3; count <= 5; count++) {
    const r = add("range", `하트 ${count}개 효과 타일 · 미정`),
      tier = add("count-range-tier", `소비 ${count}개`, { count });
    link(r, "range", tier, "range");
    link(tier, "tier", ranges, "tiers");
  }
  const black = phase(
    blackOwner,
    "수동 블랙홀",
    2,
    placement("지정 타일 중심 · 소비량별 범위", "selected-tile", ranges),
  );
  link(black.definition, "phase", blackOwner, "first");
  effect(black, pull(black), "completed", true, false, receipt);
  if (bloom) {
    const pillar = phase(
      blackOwner,
      "개화 빛기둥",
      3,
      placement("블랙홀 중심의 고정 빛기둥", "selected-tile"),
    );
    link(pillar.definition, "phase", black.definition, "next");
    const damage = effect(pillar, pillar.pulse, "out", false, false, receipt);
    effect(pillar, damage.query, "completed", true, false, receipt);
  }

  b.frame("하트 6 · 예약 자동 시전 · 공명 추가 소비 없음 · 종료 시 하트 소진");
  const auto = add(
    "counter-auto-request",
    "히스테릭 예약 · 현재 시전 정리 뒤 자동 시작",
  );
  link(counter, "counter", auto, "counter");
  link(memory, "memory", auto, "memory");
  const terms = add("counter-terms", "상한 6 전용 · 완료/중단 뒤 전부 소진", {
    minimum: 6,
    maximum: 6,
    spend: "end-all",
  });
  link(counter, "counter", terms, "counter");
  const autoUse = b.admission(auto, settings, true, terms);
  const autoOwner = channel(
    autoUse,
    autoUse,
    "히스테릭 · 수동 취소 불가 · 외부 중단 시 정상 복귀",
  );
  const autoBlack = phase(
    autoOwner,
    "히스테릭 블랙홀",
    2,
    placement("기억한 고정 타일 · 없으면 시전자 타일", "selected-tile"),
  );
  link(autoBlack.definition, "phase", autoOwner, "first");
  effect(autoBlack, pull(autoBlack), "completed", false, true);
  const last = phase(
    autoOwner,
    bloom ? "히스테릭 빛기둥" : "히스테릭 광선",
    bloom ? 3 : "",
    placement(
      bloom ? "블랙홀 중심의 빛기둥" : "시전자에서 블랙홀 중심 방향",
      bloom ? "selected-tile" : "selected-direction",
    ),
  );
  link(last.definition, "phase", autoBlack.definition, "next");
  effect(last, last.pulse, "out", false, true);
  const warning = add("cue", "예정 타일·위험 범위·하트 5 타이머 강조", {
    description:
      "기억한 예정 타일을 계속 표시합니다. 일반 광선 조준으로 6개에 도달할 수 있으면 자동 공격 위험 범위를 표시하고 하트 5개에서는 다음 자동 획득까지 남은 시간을 강조합니다. 표시 자체는 피해·회복·발동 시간을 바꾸지 않습니다.",
  });
  link(auto, "cue", warning, "event");
  b.frame("장착 유물 → 회복·효과 범위·하트 획득·진행 집중");
  const m = exampleModifiers(b);
  m.bind(
    "hasty-second-hand",
    "성급한 초침 · 자동 하트 획득 주기만",
    "duration",
    interval,
    "value",
  );
  for (const n of [...b.doc.nodes]) {
    if (n.kind === "skill-ground-heal") {
      const source = b.doc.edges.find(
        (e) => e.target.node === n.id && e.target.port === "power",
      )!;
      const power = b.doc.nodes.find((p) => p.id === source.source.node)!;
      m.bind(
        "healing-increase",
        "온기의 성배 · 실제 회복 요청량 증가",
        "healing",
        power,
        "power",
      );
      m.healingBuff(power);
    }
    if (
      n.kind === "skill-range" &&
      b.doc.edges.some(
        (e) =>
          e.source.node === n.id &&
          ["skill-range-placement", "skill-count-range-tier"].includes(
            b.doc.nodes.find((p) => p.id === e.target.node)!.kind,
          ),
      )
    )
      m.range(n);
    if (n.kind === "skill-sequence-channel")
      m.bind(
        "unbroken-trance",
        "무아지경 · 진행 중 집중 보호",
        "interruption",
        n,
        "channel",
        {},
        "channel",
      );
  }
  m.bind(
    "double-harvest",
    "두 겹의 수확 · 시간 획득 하트 2배",
    "number",
    one,
    "value",
    { factor: 2 },
  );
  m.bind(
    "double-harvest",
    "두 겹의 수확 · 완주/동시 성공 보상 2배",
    "number",
    award,
    "amount",
    { factor: 2 },
  );
  // Layered Shell has no fragment shield site; keep that explicit source deferral.
  return b.finish();
}
