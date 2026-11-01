import { exampleModifiers } from "./example-modifiers";
import { skillExampleBuilder } from "./skill-example-builder";

export function laetitia(bloom = false) {
  const b = skillExampleBuilder(
    `래티씨아 · ${bloom ? "개화형" : "기본형"} 친구 배치·합체`,
  );
  b.frame("같은 소유자·사본·타일의 누적 수 · 이동·감지 재사용");
  const small = b.add("mobile-effect", "작은 친구 · 비저지 함정 · 1칸 감지", {
    motion: "wait-leap",
    damageType: "magic",
  });
  const detection = b.add("range", "설치 위치 1칸 · 상하좌우와 중심", {
    pattern: ".X.\nX@X\n.X.",
    includeAnchor: true,
  });
  const smallRange = b.add("range", "작은 친구 폭발 범위 · 미정");
  const basePower = b.add("weapon-power", "작은 친구 무기 Attack 계수 · 미정");
  b.link(detection, "range", small, "detection");
  b.link(smallRange, "range", small, "range");
  b.link(basePower, "power", small, "power");
  b.link(b.seconds("도약·폭발 지연 · 미정"), "value", small, "delay");
  b.link(b.seconds("대기 수명 · 미정"), "value", small, "lifetime");
  const large = b.add(
    "mobile-variant",
    "큰 친구 · 같은 감지·이동 · 위력·범위 증가",
  );
  const largeRange = b.add("range", "큰 친구 폭발 범위 · 미정");
  const largePower = b.add("damage-bonus", "큰 친구 추가 위력 · 미정");
  b.link(basePower, "power", largePower, "base");
  b.link(small, "profile", large, "base");
  b.link(largePower, "power", large, "power");
  b.link(largeRange, "range", large, "range");
  const special = b.add("mobile-variant", "특별한 친구 · 실제 누적 수 보존");
  const specialPower = b.add(
    "fusion-count-power",
    "3마리분 이상 실제 누적 수 비례",
    { baseline: 2 },
  );
  b.link(largePower, "power", specialPower, "base");
  const specialRange = b.add(
    "scaled-range",
    "큰 친구 폭발 범위 길이 2배 · 한 번",
    { factor: 2 },
  );
  b.link(largeRange, "range", specialRange, "base");
  b.link(large, "profile", special, "base");
  b.link(specialPower, "power", special, "power");
  b.link(specialRange, "range", special, "range");
  const family = b.add("fusion-family", "같은 타일 1 / 2 / 3+ 단위 합체");
  [small, large, special].forEach((profile, i) => {
    const tier = b.add("fusion-tier", `${i + 1}마리분 이상`, {
      minimum: i + 1,
    });
    b.link(profile, "profile", tier, "profile");
    b.link(tier, "tier", family, "tiers");
  });
  b.frame("선택 타일 + 인접 한 곳 · 공간 부족은 합산");
  const place = b.add(
    "place-fusion-plan",
    bloom ? "큰 친구 1 + 작은 친구 1" : "작은 친구 1 + 작은 친구 1",
    { primary: bloom ? 2 : 1, adjacent: 1 },
  );
  b.link(family, "family", place, "family");
  b.action(place, "player-tile");
  b.frame("아직 남은 선물 장착 조건 · 직접 시전과 별도 사망 생성");
  const seed = b.add(
    "fusion-death-seed",
    "유물이 있을 때만 선물 → 사망 위치 1단위",
    { relic: "red-thread" },
  );
  b.link(family, "family", seed, "family");
  b.link(b.seconds("선물 지속 시간", 10), "value", seed, "duration");
  b.frame("장착 유물 → 폭발·선물 변경 · 감지/대기/도약 유지");
  const m = exampleModifiers(b);
  m.range(smallRange);
  m.range(largeRange);
  m.range(specialRange);
  // Apply the bonus at the final mobile profile, not its shared ancestor formula.
  for (const profile of [small, large, special])
    m.bind(
      "explosion-vulnerability",
      "압축 화약 · 최종 친구 폭발 피해",
      "mobile-power",
      profile,
      "profile",
    );
  m.summonVulnerability(small);
  m.duration(b.doc.nodes.find((n) => n.title === "선물 지속 시간")!);
  return b.finish();
}
export function snowQueen() {
  const b = skillExampleBuilder("눈의 여왕 · 빙설의 파도");
  b.frame("자기 피해의 냉기 / 혹한 중 억제 / 타격 전 상태 변환");
  const cold = b.add("reactive-status", "냉기 · 이동 둔화 · 시간 갱신", {
    effect: "slow",
    percent: 30,
    refresh: "refresh",
  });
  const frozen = b.add("reactive-status", "혹한 · 빙결 · 시간 유지", {
    effect: "freeze",
    refresh: "preserve",
  });
  b.link(b.seconds("냉기 시간", 3), "value", cold, "duration");
  b.link(b.seconds("혹한 시간", 2), "value", frozen, "duration");
  b.link(frozen, "status", cold, "blockedBy");
  b.link(
    cold,
    "status",
    b.add("owned-damage-status", "자기 소유 피해 뒤 냉기 · 혹한 중 제외"),
    "status",
  );
  const strike = b.add("strike-plan", "지정 방향 T자 타일의 Magic 피해", {
    target: "selected-direction-area",
    damageType: "magic",
  });
  b.link(b.add("range", "정확한 T자 타일 마스크"), "range", strike, "range");
  b.link(b.add("weapon-power", "T자 공격 계수"), "power", strike, "power");
  const convert = b.add("convert-status-plan", "타격 직전 냉기만 소비 → 혹한");
  b.link(strike, "plan", convert, "body");
  b.link(cold, "status", convert, "from");
  b.link(frozen, "status", convert, "to");
  b.frame("비저지 잔영 · 가까운 적 추격 · 지연 뒤 현재 위치 폭발");
  const remnant = b.add(
    "mobile-effect",
    "빙결 잔영 · 자기 계보 추가 생성 금지",
    { motion: "chase-delay", damageType: "magic" },
  );
  b.link(b.add("range", "잔영 탐색 범위"), "range", remnant, "detection");
  b.link(b.add("range", "잔영 폭발 범위"), "range", remnant, "range");
  b.link(b.add("weapon-power", "잔영 폭발 계수"), "power", remnant, "power");
  b.link(b.seconds("잔영 폭발 지연"), "value", remnant, "delay");
  const limit = b.add("count-positive", "기본·유물 잔영의 공통 동시 상한");
  const spawn = b.add(
    "spawn-mobile-plan",
    "액티브 유효 적중 뒤 시전당 잔영 1개",
  );
  b.link(remnant, "profile", spawn, "profile");
  b.link(limit, "count", spawn, "limit");
  const onHit = b.add("on-hit-plan", "변환 타격 완료 뒤 잔영 생성");
  b.link(convert, "plan", onHit, "body");
  b.link(spawn, "plan", onHit, "after");
  b.action(onHit, "player-direction");
  b.frame("겨울의 유언 장착 조건 · 자기 상태 사망 1회 · 잔영 계보 제외");
  const death = b.add(
    "status-death-mobile",
    "유물이 있을 때만 사망 위치 잔영",
    { relic: "snow-queen-remnant" },
  );
  b.link(cold, "status", death, "states");
  b.link(frozen, "status", death, "states");
  b.link(remnant, "profile", death, "profile");
  b.link(limit, "count", death, "limit");
  b.frame("장착 유물 → 냉기/혹한과 직접/잔영 효과 분리");
  const m = exampleModifiers(b);
  m.duration(b.doc.nodes.find((n) => n.title === "냉기 시간")!);
  m.duration(b.doc.nodes.find((n) => n.title === "혹한 시간")!);
  m.vulnerability(frozen, "reactive");
  m.range(b.doc.nodes.find((n) => n.title === "정확한 T자 타일 마스크")!);
  m.range(b.doc.nodes.find((n) => n.title === "잔영 폭발 범위")!);
  m.bind(
    "explosion-vulnerability",
    "압축 화약 · 잔영 폭발만 증가",
    "mobile-power",
    remnant,
    "profile",
  );
  m.summonVulnerability(remnant);
  return b.finish();
}
