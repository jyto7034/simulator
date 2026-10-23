import type { GraphNode, Point } from "../types";
import { makeNode } from "./catalog";
import { canonicalize, createDocument, id } from "./model";
import { placeExample } from "./example-layout";

// This is an editable authoring example, not an executable skill or a balance preset.
export function helloGoodbye() {
  const doc = createDocument();
  doc.name = "아무 것도 없는 · Hello / Goodbye";
  const groups = [
    "1 · Hello 시작과 기본 공격",
    "2 · 재입력과 Goodbye",
    "3 · 만료와 외부 종료",
  ].map((title) => ({ id: id(), title }));
  doc.frames = groups;
  const byKey = new Map<string, GraphNode>();
  const add = (
    key: string,
    kind: string,
    title: string,
    position: Point,
    group: number,
    values: GraphNode["values"] = {},
    note = "",
  ) => {
    const node = makeNode(`skill-${kind}`, position);
    node.title = title;
    node.frame = groups[group].id;
    node.values = { ...node.values, ...values };
    node.note = note;
    byKey.set(key, node);
    doc.nodes.push(node);
  };
  const link = (from: string, output: string, to: string, input: string) => {
    doc.edges.push({
      id: id(),
      source: { node: byKey.get(from)!.id, port: output },
      target: { node: byKey.get(to)!.id, port: input },
    });
  };
  add(
    "state",
    "state",
    "Hello 상태 선언",
    { x: 360, y: 440 },
    0,
    {},
    "이 선언은 상태의 종류를 식별합니다. 직원·장착 스킬별 활성은 상태 시작에서 생성되며, 각 활성에 타이머와 마무리 권리가 속합니다.",
  );
  add("duration", "seconds", "Hello 유지 시간", { x: 680, y: 440 }, 0);
  add(
    "profile",
    "attack-profile",
    "Hello 관통 공격",
    { x: 1000, y: 440 },
    0,
    {},
    "미미크리 장착 시 실제 HP 피해를 공격별 합산해 무기 흡혈 상한을 한 번 적용합니다. 관통 유물 이력은 다음 공격에서 초기화합니다.",
  );
  add(
    "selection",
    "target-selection",
    "Hello 자기 사용",
    { x: 40, y: 440 },
    0,
    { mode: "self" },
  );
  add(
    "piercing-condition",
    "equipment-condition",
    "꿰뚫는 송곳 · 장착 여부",
    { x: 1960, y: 800 },
    0,
    {
      definition: "piercing-damage-increase",
      slot: "relic",
      variant: "normal",
      copies: "once",
    },
  );
  add(
    "piercing",
    "modifier-piercing",
    "꿰뚫는 송곳 → Hello 관통 공격",
    { x: 2280, y: 800 },
    0,
  );
  add("use", "request", "Hello / Goodbye 입력", { x: 40, y: 80 }, 0);
  add("lookup", "lookup", "Hello 활성 조회", { x: 360, y: 80 }, 0);
  add(
    "qualify",
    "resonance-qualify",
    "Hello 첫 사용 자격",
    { x: 680, y: 80 },
    0,
  );
  add(
    "overlay",
    "resonance-install",
    "Hello 적용",
    { x: 1000, y: 80 },
    0,
    {},
    "시간제 공격 교체 동안 공명 사용 잠금을 유지합니다. Goodbye로 교체를 끝내도 남은 마무리 동작의 사용 의무를 함께 확인합니다.",
  );
  add("transform", "cue", "포신 변형 표시", { x: 1320, y: 80 }, 0, {
    description:
      "같은 직원의 팔을 포신처럼 표시합니다. 실제 위치·저지·무기 종류를 바꾸지 않습니다.",
  });
  add("basic", "basic-event", "Hello의 개별 기본 공격", { x: 40, y: 760 }, 0);
  add("shot", "cue", "관통 공격 표시", { x: 360, y: 760 }, 0, {
    description:
      "실제 기본 공격 사건에 맞춰 직선 관통 공격을 표시합니다. 별도 추가 타격이나 물리 투사체를 생성하는 지시가 아닙니다.",
  });

  add(
    "permission",
    "permission",
    "Goodbye 재입력 허용",
    { x: 40, y: 1120 },
    1,
    {
      permission: "manual-followup",
    },
  );
  add("reinput", "claim", "Goodbye 마무리 권리", { x: 360, y: 1120 }, 1, {
    expiryOrder: "expiry-first",
  });
  add("capture", "capture-time", "포기할 시간 고정", { x: 680, y: 1120 }, 1, {
    moment: "accepted",
  });
  add("manual-end", "finish", "Hello 조기 종료", { x: 1000, y: 1120 }, 1);
  add("power", "time-power", "Goodbye 위력", { x: 680, y: 1500 }, 1);
  add("area", "area", "Goodbye 전방 범위", { x: 1000, y: 1500 }, 1, {
    facing: "accepted",
  });
  add("damage", "area-damage", "Goodbye 일격", { x: 1320, y: 1120 }, 1, {
    damageType: "physical",
  });
  add("blade", "cue", "생체 칼날 표시", { x: 1640, y: 1120 }, 1, {
    description:
      "전방 일격의 실제 사건에 맞춰 팔을 생체 칼날로 표시합니다. Hello 관통 공격의 이력·흡혈은 적용하지 않습니다.",
  });

  add("expiry", "expiry", "Hello 자연 만료", { x: 40, y: 1920 }, 2);
  add("expiry-end", "finish", "만료 후 복귀", { x: 360, y: 1920 }, 2, {
    reason: "expiry",
  });
  add("exit", "owner-exit", "Hello 외부 종료", { x: 40, y: 2240 }, 2, {
    policy: "end-active",
  });
  add("exit-end", "finish", "이탈 상태 정리", { x: 360, y: 2240 }, 2, {
    reason: "owner-exit",
  });
  add("restore", "cue", "정상 모습 복귀", { x: 680, y: 2080 }, 2, {
    description:
      "일격 종료·자연 만료·이탈 정리 후 정상 외형을 표시합니다. 자동 Goodbye나 부활을 발생시키지 않습니다.",
  });

  add(
    "capacity",
    "resonance-amount",
    "Hello 공명 저장 상한",
    { x: 40, y: 800 },
    0,
  );
  add(
    "settings",
    "resonance-settings",
    "Hello 공명 설정",
    { x: 360, y: 800 },
    0,
  );
  add("resonance", "resonance-read", "Hello 공명 조회", { x: 680, y: 440 }, 0);
  add(
    "requirement",
    "resonance-requirement",
    "Hello 공명 조건",
    { x: 1000, y: 440 },
    0,
    { mode: "full" },
  );
  add("cost", "resonance-cost", "Hello 공명 비용", { x: 1320, y: 440 }, 0, {
    mode: "all",
  });
  add("admit", "resonance-admit", "Hello 접수와 지불", { x: 1320, y: 80 }, 0);
  const positions: Record<string, Point> = {
    qualify: { x: 1000, y: 80 },
    overlay: { x: 1640, y: 80 },
    transform: { x: 1960, y: 80 },
    duration: { x: 1640, y: 440 },
    profile: { x: 1960, y: 440 },
    basic: { x: 40, y: 1040 },
    shot: { x: 360, y: 1040 },
  };
  for (const [key, position] of Object.entries(positions))
    byKey.get(key)!.position = position;
  for (const node of doc.nodes)
    if (node.frame !== groups[0].id) node.position.y += 400;
  link("selection", "selection", "use", "selection");
  link("piercing-condition", "condition", "piercing", "condition");
  link("profile", "profile", "piercing", "target");
  link("capacity", "value", "settings", "capacity");
  link("settings", "settings", "resonance", "settings");
  link("use", "request", "resonance", "request");
  link("use", "request", "qualify", "request");
  link("resonance", "snapshot", "cost", "snapshot");
  link("resonance", "snapshot", "qualify", "snapshot");
  link("requirement", "requirement", "qualify", "requirement");
  link("cost", "cost", "qualify", "cost");

  link("state", "state", "overlay", "state");
  link("duration", "value", "overlay", "duration");
  link("profile", "profile", "overlay", "profile");
  link("use", "out", "lookup", "in");
  link("use", "request", "lookup", "request");
  link("lookup", "absent", "qualify", "in");
  link("lookup", "vacancy", "qualify", "vacancy");
  link("qualify", "out", "admit", "in");
  link("qualify", "permit", "admit", "permit");
  link("admit", "out", "overlay", "in");
  link("admit", "use", "overlay", "use");
  link("lookup", "present", "permission", "in");
  link("lookup", "activation", "permission", "activation");
  link("use", "request", "permission", "request");
  link("permission", "out", "reinput", "in");
  link("permission", "permit", "reinput", "permit");
  link("overlay", "cue", "transform", "event");
  for (const consumer of ["basic", "lookup", "expiry", "exit"])
    link("state", "state", consumer, "state");
  link("basic", "cue", "shot", "event");
  link("reinput", "out", "capture", "in");
  link("reinput", "activation", "capture", "activation");
  link("capture", "out", "manual-end", "in");
  link("reinput", "activation", "manual-end", "activation");
  link("capture", "seconds", "power", "seconds");
  link("manual-end", "out", "damage", "in");
  link("power", "power", "damage", "power");
  link("area", "area", "damage", "area");
  link("damage", "cue", "blade", "event");
  link("damage", "completed", "restore", "event");
  for (const [event, finish] of [
    ["expiry", "expiry-end"],
    ["exit", "exit-end"],
  ]) {
    link(event, "out", finish, "in");
    link(event, "activation", finish, "activation");
    link(finish, "cue", "restore", "event");
  }
  placeExample(doc);
  canonicalize(doc);
  return doc;
}
