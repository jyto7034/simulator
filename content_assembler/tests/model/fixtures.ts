import {
  createDocument,
  createWave,
  type MapDocument,
} from "../../src/editor/model";
export function fixture(
  kind: "small" | "irregular" | "many" | "empty" | "incomplete" = "small",
): MapDocument {
  const d = createDocument();
  d.id = "map-fixture";
  d.name = "제삼 격리 시설 · 북동쪽 방어 전장";
  d.description = "고정된 프로토타입 검증 자료";
  d.width = 8;
  d.height = 6;
  d.valid = Array.from(
    { length: 48 },
    (_, i) => `${i % 8},${Math.floor(i / 8)}`,
  ).sort();
  if (kind === "empty") {
    d.valid = [];
    return d;
  }
  d.zones = [
    {
      id: "zone-deploy",
      name: "아군 배치 구역",
      kind: "deploy",
      cells: ["6,2", "6,3"],
    },
    {
      id: "zone-spawn",
      name: "적 등장 구역",
      kind: "spawn",
      cells: ["0,2", "0,3"],
    },
  ];
  d.terrain = { "6,2": "platform", "4,4": "obstacle" };
  d.routes = [
    {
      id: "route-main",
      name: "주 이동 경로",
      points: [
        { id: "route-point-a", x: 0.5, y: 2.5 },
        { id: "route-point-b", x: 6.5, y: 2.5 },
      ],
    },
  ];
  d.points = [{ id: "landmark-a", name: "중앙 관측 지점", x: 3.5, y: 2.5 }];
  d.slots = [{ id: "slot-boss", name: "주 보스 슬롯" }];
  const w = createWave();
  w.id = "wave-first";
  w.name = "첫 번째 웨이브";
  w.enemies[0] = {
    id: "enemy-first",
    contentId: "enemy.prototype",
    count: 4,
    intervalMs: 1250,
  };
  w.zoneId = "zone-spawn";
  w.routeId = "route-main";
  d.waves = [w];
  if (kind === "irregular" || kind === "many") {
    d.valid = d.valid.filter(
      (k) => !["0,0", "1,0", "0,1", "3,3", "4,3"].includes(k),
    );
    d.terrain["3,1"] = "platform";
  }
  if (kind === "incomplete") {
    d.waves[0].zoneId = "missing-zone";
    d.waves[0].trigger = {
      kind: "reach",
      slotId: "",
      pointId: "missing-point",
      radius: -1,
    };
    d.routes[0].points = [];
  }
  if (kind === "many") {
    d.name =
      "매우 긴 한글 전장 이름 · 격리 시설 최하층의 붕괴된 중앙 통제실과 외곽 방어선";
    for (let i = 1; i <= 16; i++) {
      d.routes.push({
        id: `route-${i}`,
        name: `보조 경로 ${i} · 긴 이름 확인`,
        points: [
          { id: `p-${i}-a`, x: 0.5, y: 1.5 },
          { id: `p-${i}-b`, x: 6.5, y: 4.5 },
        ],
      });
      const next = structuredClone(w);
      next.id = `wave-${i}`;
      next.name = `추가 웨이브 ${i} · 긴 한글 이름`;
      next.enemies[0].id = `enemy-${i}`;
      next.trigger =
        i % 4 === 0
          ? { kind: "clear", waveId: "wave-first" }
          : i % 4 === 1
            ? { kind: "hp", slotId: "slot-boss", percent: 50 }
            : i % 4 === 2
              ? { kind: "death", slotId: "slot-boss" }
              : {
                  kind: "reach",
                  slotId: "slot-boss",
                  pointId: "landmark-a",
                  radius: 1,
                };
      next.fallbackMs = 45000;
      d.waves.push(next);
    }
  }
  return d;
}
