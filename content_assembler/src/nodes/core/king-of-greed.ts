import { exampleModifiers } from "./example-modifiers";
import { skillExampleBuilder } from "./skill-example-builder";

export function kingOfGreed() {
  const b = skillExampleBuilder("탐욕의 왕 · 황금의 지평선");
  const { add, link, seconds, frame } = b;
  frame("파편 사본 원장 · 전투 지급·대여·다음 전투 정산");
  const ledger = add("credit-ledger", "기본 지급 10 · 자동 대여 한도 20", {
    grant: 10,
    creditLimit: 20,
  });
  const mark = add("damage-mark", "자기 출처 회수 상태 · 비중첩");
  link(seconds("회수 상태 5~10초 · 정확한 값 미정"), "value", mark, "duration");
  const recovery = add("damage-recovery", "모든 귀속 피해 → 상태 보유 적 사망");
  link(ledger, "bank", recovery, "bank");
  link(mark, "mark", recovery, "mark");
  const credit = add("resource-credit", "사망 한 번당 재고 +1 · 빚 우선 정산", {
    amount: 1,
  });
  link(recovery, "out", credit, "in");
  link(recovery, "grant", credit, "grant");
  link(ledger, "bank", credit, "bank");
  frame("후반 가속 전개 · 한 번의 상한·기한·오래된 포탈 붕괴");
  const policy = add("portal-policy", "전개 속도와 독립된 발사 기한");
  for (const [port, label] of [
    ["first", "첫 포탈 생성 간격"],
    ["last", "마지막 포탈 생성 간격"],
    ["hold", "최대 전개 뒤 유지 유예"],
    ["deadline", "전개 시작부터 발사 기한"],
    ["decay", "오래된 포탈 붕괴 간격"],
  ])
    link(seconds(`${label} · 미정`), "value", policy, port);
  const program = add("portal-program", "포탈 전개·사출 시작 설정");
  link(ledger, "bank", program, "bank");
  link(policy, "policy", program, "policy");
  const selection = add("target-selection", "차징 시작은 자기 자신", {
    mode: "self",
  });
  const request = add("request", "황금의 지평선 차징 입력");
  link(selection, "selection", request, "selection");
  const use = b.admission(request, b.settings(), false, undefined, program);
  const charge = add("run-portals", "전개부터 사출까지 하나의 집중");
  link(use, "out", charge, "in");
  link(use, "use", charge, "use");
  link(program, "program", charge, "program");
  frame("재입력 · 남은 발수 고정 · 착탄 완료 후 다음 선택");
  const release = add("portal-release-request", "추가 공명 없는 사출 재입력");
  link(charge, "charge", release, "charge");
  const freeze = add("portal-freeze", "추가 전개·붕괴 중지 후 남은 발수 고정");
  link(release, "out", freeze, "in");
  link(release, "right", freeze, "right");
  const selector = add("enemy-selector", "자기 상태 없음 → 루트 진행도 우선", {
    order: "unmarked-route",
    air: "include",
  });
  link(mark, "mark", selector, "mark");
  link(add("range", "보구 표적 선택 범위 · 미정"), "range", selector, "range");
  const profile = add(
    "projectile-profile",
    "보구 한 발 · 실제 투사체 착탄 범위",
    {
      delivery: "impact",
      tracking: "homing",
      damageType: "physical",
      air: "include",
      obstacle: "stop",
    },
  );
  link(
    add("damage-amount", "보구 한 발 위력 · 미정"),
    "power",
    profile,
    "power",
  );
  link(add("range", "보구 착탄 효과 범위 · 미정"), "range", profile, "area");
  link(seconds("보구 최대 비행 시간 · 미정"), "value", profile, "lifetime");
  const serial = add(
    "serial-projectiles",
    "한 발 정산 완료까지 대기 → 다음 표적",
  );
  link(freeze, "out", serial, "in");
  link(freeze, "volley", serial, "volley");
  link(profile, "profile", serial, "profile");
  link(selector, "selector", serial, "selector");
  const stop = add("portal-stop-input", "차징·사출 직접 종료 · 미방출 무환불");
  link(charge, "charge", stop, "charge");
  const results = add("projectile-results", "적중·빗나감 모두 개별 정산");
  link(serial, "batch", results, "batch");
  frame("장착 유물 → 회수와 사출의 명시된 지점만 변경");
  const m = exampleModifiers(b);
  m.duration(
    b.doc.nodes.find((n) => n.title === "회수 상태 5~10초 · 정확한 값 미정")!,
  );
  m.range(b.doc.nodes.find((n) => n.title === "보구 착탄 효과 범위 · 미정")!);
  m.bind(
    "double-harvest",
    "두 겹의 수확 · 사망 회수 1→2",
    "grant",
    recovery,
    "grant",
    { factor: 2 },
  );
  m.bind(
    "unbroken-trance",
    "무아지경 · 전개와 사출 보호",
    "interruption",
    program,
    "program",
    {},
    "program",
  );
  m.bind(
    "deep-wedge",
    "깊은 쐐기 · 같은 시전/대상 적중",
    "projectile-multihit",
    profile,
    "profile",
    { step: 2, cap: 20 },
  );
  const stun = add("hit-status", "왕의 위압 · 보구 범위의 기절", {
    effect: "stun",
  });
  link(seconds("왕의 위압 · 기절 지속 시간 미정"), "value", stun, "duration");
  const attached = m.bind(
    "king-of-greed-stunning-treasures",
    "왕의 위압 · 적중 후 기절",
    "projectile-status",
    profile,
    "profile",
  );
  link(stun, "status", attached, "status");
  const endless = m.bind(
    "king-of-greed-endless-treasury",
    "끝없는 금고 · HP 비용 사출",
    "continuous-volley",
    program,
    "program",
    { percent: 5 },
  );
  link(
    seconds("끝없는 금고 · 정산 뒤 사출 간격 미정"),
    "value",
    endless,
    "interval",
  );
  link(
    seconds("끝없는 금고 · HP 비용 매초", 1),
    "value",
    endless,
    "costInterval",
  );
  return b.finish();
}
