import { exampleModifiers } from "./example-modifiers";
import { skillExampleBuilder } from "./skill-example-builder";

export function littleRed() {
  const b = skillExampleBuilder("붉은 두건 · 창 배치와 전체 동시 사격");
  const { add, link, seconds, frame } = b;
  frame("전투 사본 · 일반 창과 강화 창");
  const normal = add("stock", "일반 창 · 독립 재고", {
    initial: 0,
    lifetime: "battle",
    limit: "bounded",
  });
  const enhanced = add("stock", "강화 창 · 독립 재고", {
    initial: 0,
    lifetime: "battle",
    limit: "bounded",
  });
  const timer = add("stock-periodic", "장착 중 일반 창 획득", {
    scope: "equipped",
  });
  link(normal, "bank", timer, "bank");
  link(seconds("일반 창 획득 주기 · 미정"), "value", timer, "interval");
  const gain = add("resource-credit", "시간으로 일반 창 1개", { amount: 1 });
  link(normal, "bank", gain, "bank");
  link(timer, "grant", gain, "grant");
  link(timer, "out", gain, "in");
  frame("실제 적 늑대 · 주기 출현·아군 공격·처치 보상");
  const wolf = add("hostile-profile", "아군을 공격하는 늑대", {
    block: 1,
    damageType: "physical",
  });
  link(add("range", "늑대 기본 공격 범위 · 미정"), "range", wolf, "range");
  link(seconds("늑대 공격 주기 · 미정"), "value", wolf, "interval");
  const weakness = add(
    "hostile-family-weakness",
    "두건 파편·무기 출처 피해 +50%",
    { family: "little-red", percent: 50 },
  );
  link(wolf, "profile", weakness, "base");
  const spawner = add("hostile-spawner", "파편 장착 중 늑대 주기 출현");
  link(weakness, "profile", spawner, "profile");
  link(seconds("늑대 출현 주기 · 미정"), "value", spawner, "interval");
  const killed = add("cohort-killed", "이 사본의 늑대 사망 보상");
  link(spawner, "cohort", killed, "cohort");
  link(enhanced, "bank", killed, "bank");
  const reward = add("resource-credit", "늑대 처치 강화 창 1개", { amount: 1 });
  link(killed, "out", reward, "in");
  link(killed, "grant", reward, "grant");
  link(enhanced, "bank", reward, "bank");
  frame("정지 중 배치 · 두 조준 방식 · 하나의 전체 계획");
  const memory = add("layout-memory", "마지막 성공 배치만 재사용");
  link(normal, "bank", memory, "normal");
  link(enhanced, "bank", memory, "enhanced");
  const settings = b.settings();
  const requirement = add(
    "resonance-requirement",
    "준비와 확정 · 같은 공명 만충 조건",
    { mode: "full" },
  );
  const prepare = add("plan-prepare", "Core 전투 정지 · 미소비 준비");
  link(memory, "memory", prepare, "memory");
  link(add("range", "창 배치 가능 범위 · 미정"), "range", prepare, "range");
  link(settings, "settings", prepare, "settings");
  link(requirement, "requirement", prepare, "requirement");
  const layout = add("plan-layout", "종류·수량·순서·타일 배치", {
    duplicates: "reject",
  });
  link(prepare, "out", layout, "in");
  link(prepare, "preparation", layout, "preparation");
  const mode = add("plan-aim-mode", "메뉴 · 정밀/집중 사격");
  link(layout, "out", mode, "in");
  link(layout, "layout", mode, "layout");
  const precise = add("plan-precise-aim", "정밀 · 각 창 방향 저장만");
  link(mode, "precise", precise, "in");
  link(layout, "layout", precise, "layout");
  const focused = add("plan-focused-aim", "집중 · 하나의 타일로 방향 계산");
  link(mode, "focused", focused, "in");
  link(layout, "layout", focused, "layout");
  const merge = add("plan-merge", "실제로 선택한 계획만 합류");
  link(precise, "out", merge, "in");
  link(precise, "plan", merge, "precise");
  link(focused, "out", merge, "in");
  link(focused, "plan", merge, "focused");
  frame("일반탄·강화탄 정의 · 서로 다른 개별 적중");
  const power = add("damage-amount", "일반탄 피해 위력 · 미정");
  const enhancedPower = add("damage-bonus", "강화탄 · 일반탄보다 강한 위력");
  link(power, "power", enhancedPower, "base");
  const root = add("hit-status", "강화탄 적중 · 2초 속박", { effect: "root" });
  link(seconds("강화탄 속박 2초", 2), "value", root, "duration");
  const profiles = [power, enhancedPower].map((damage, i) => {
    const p = add(
      "projectile-profile",
      i ? "강화탄 · 관통 피해와 속박" : "일반탄 · 관통 피해",
      {
        delivery: "directional",
        tracking: "fixed",
        damageType: "physical",
        pierce: "count",
        air: "include",
        obstacle: "stop",
      },
    );
    link(damage, "power", p, "power");
    link(
      seconds(
        i ? "강화탄 최대 비행 시간 · 미정" : "일반탄 최대 비행 시간 · 미정",
      ),
      "value",
      p,
      "lifetime",
    );
    if (i) link(root, "status", p, "after");
    return p;
  });
  frame("최종 검증 → 비용·전체 방출·기억·재개 일괄 확정");
  const read = add("resonance-read", "완성한 요청의 공명 재조회");
  link(prepare, "request", read, "request");
  link(settings, "settings", read, "settings");
  const cost = add("resonance-cost", "확정 때 공명 전량 소비", { mode: "all" });
  link(read, "snapshot", cost, "snapshot");
  const qualify = add("resonance-qualify", "전체 계획과 최종 공명 자격");
  link(merge, "out", qualify, "in");
  link(merge, "plan", qualify, "plan");
  link(prepare, "request", qualify, "request");
  link(read, "snapshot", qualify, "snapshot");
  link(requirement, "requirement", qualify, "requirement");
  link(cost, "cost", qualify, "cost");
  const commit = add("plan-commit", "소비·모든 탄환 동시 방출·전투 재개");
  link(qualify, "out", commit, "in");
  link(qualify, "permit", commit, "permit");
  link(merge, "plan", commit, "plan");
  link(profiles[0], "profile", commit, "normal");
  link(profiles[1], "profile", commit, "enhanced");
  const results = add("projectile-results", "동시 방출 뒤 개별 탄환 정산");
  link(commit, "batch", results, "batch");
  frame("장착 유물 → 늑대 처치 강화 창만 증폭");
  const m = exampleModifiers(b);
  m.bind(
    "double-harvest",
    "두 겹의 수확 · 늑대 처치 1→2",
    "grant",
    killed,
    "grant",
    { factor: 2 },
  );
  frame("전용 탄환 유물 · 직접 피해와 관통 폭발 분리");
  const wound = add(
    "modifier-projectile-wound",
    "관통상 · 기존 스택으로 직접 탄환 증폭",
    { amount: 1, percent: 30 },
  );
  link(
    m.condition("little-red-puncture-wound", "관통상 · 장착 여부"),
    "condition",
    wound,
    "condition",
  );
  link(
    seconds("관통상 · 전체 스택 유지 시간 미정"),
    "value",
    wound,
    "duration",
  );
  for (const p of profiles) link(p, "profile", wound, "profiles");
  m.bind(
    "deep-imprint",
    "깊은 각인 · 관통상 신규 부여",
    "wound-grant",
    wound,
    "wound",
    { factor: 2 },
  );
  link(
    m.condition("little-red-puncture-wound", "관통상 · 장착 여부"),
    "condition",
    m.condition("deep-imprint", "깊은 각인 · 장착 여부"),
    "requires",
  );
  const stun = add("hit-status", "슬러그탄 · 직접 적중 후 기절", {
    effect: "stun",
  });
  link(seconds("슬러그탄 · 기절 지속 시간 미정"), "value", stun, "duration");
  m.vulnerability(stun, "hit");
  link(
    m.condition("little-red-slug-round", "슬러그탄 · 장착 여부"),
    "condition",
    m.condition("broken-balance", "깨진 균형 · 장착 여부"),
    "requires",
  );
  const explosion = add("explosion-rule", "소이탄 · 별도 폭발 피해 유형 미정", {
    dedup: "blast",
    bonus: 0,
  });
  const blastPower = add("damage-amount", "소이탄 · 폭발 기준 위력 미정");
  const blastRange = add("range", "소이탄 · 폭발 효과 타일 미정");
  for (const p of profiles) {
    const status = m.bind(
      "little-red-slug-round",
      "슬러그탄 · 기절",
      "projectile-status",
      p,
      "profile",
    );
    link(stun, "status", status, "status");
    m.bind(
      "little-red-slug-round",
      "슬러그탄 · 밀어내기",
      "projectile-push",
      p,
      "profile",
    );
    const blast = m.bind(
      "little-red-incendiary-round",
      "소이탄 · 실제 관통 폭발",
      "projectile-explosion",
      p,
      "profile",
    );
    link(explosion, "explosion", blast, "explosion");
    link(blastPower, "power", blast, "power");
    link(blastRange, "range", blast, "range");
  }
  return b.finish();
}
