import { laetitia, snowQueen } from "./mobile-effect-examples";
import {
  warmHeartedWoodsman,
  cloudedMonk,
  rudolta,
  blackSwan,
} from "./offensive-examples";
import {
  porccubus,
  schadenfreude,
  scarecrow,
  queenBee,
  namelessFetus,
} from "./protection-examples";
import { meltingLove, funeralButterflies } from "./mark-examples";
import { knightOfDespair } from "./knight-of-despair";
import { littleRed } from "./little-red";
import { kingOfGreed } from "./king-of-greed";
import { definition, makeNode } from "./catalog";
import { canonicalize, createDocument, GraphError, id } from "./model";
import { spatialExample } from "./spatial-examples";
import { groundExample } from "./ground-examples";
import { helloGoodbye } from "./hello-goodbye";
import { burrowingHeaven } from "./burrowing-heaven";
import { mountainOfSmilingBodies } from "./mountain-of-smiling-bodies";

import { queenOfHatred } from "./queen-of-hatred";
import { yinYang } from "./yin-yang";

export function example(name = "empty") {
  if (name === "laetitia") return laetitia();
  if (name === "laetitia-bloom") return laetitia(true);
  if (name === "snow-queen") return snowQueen();
  if (name === "warm-hearted-woodsman") return warmHeartedWoodsman();
  if (name === "clouded-monk") return cloudedMonk();
  if (name === "rudolta") return rudolta();
  if (name === "dream-of-black-swan") return blackSwan();
  if (name === "porccubus") return porccubus();
  if (name === "schadenfreude") return schadenfreude();
  if (name === "scarecrow-searching-for-wisdom") return scarecrow();
  if (name === "queen-bee") return queenBee();
  if (name === "nameless-fetus") return namelessFetus();
  if (name === "melting-love") return meltingLove();
  if (name === "funeral-of-dying-butterflies") return funeralButterflies();
  if (name === "funeral-of-dying-butterflies-bloom")
    return funeralButterflies(true);
  if (name === "funeral-of-dying-butterflies-orb")
    return funeralButterflies(true, true);
  if (name === "knight-of-despair") return knightOfDespair();
  if (name === "knight-of-despair-bloom") return knightOfDespair(true);
  if (name === "little-red") return littleRed();
  if (name === "king-of-greed") return kingOfGreed();
  if (name === "queen-of-hatred") return queenOfHatred();
  if (name === "queen-of-hatred-bloom") return queenOfHatred(true);
  if (name === "yin-yang") return yinYang();
  if (name === "mountain-of-smiling-bodies") return mountainOfSmilingBodies();
  if (name === "burrowing-heaven") return burrowingHeaven();
  if (
    name === "ground-damage" ||
    name === "ground-heal" ||
    name === "ground-status"
  )
    return groundExample(name.slice(7) as "damage" | "heal" | "status");
  if (name === "front-strike" || name === "selected-ground")
    return spatialExample(name);
  if (name === "hello-goodbye") return helloGoodbye();
  const doc = createDocument();
  let layout: [string, number, number][] = [];
  let links: [number, string, number, string][] = [];
  if (name === "values") {
    doc.name = "값 연결 예제";
    const middle = 40 + definition("number").width + 100;
    const last = middle + definition("add").width + 100;
    layout = [
      ["number", 40, 60],
      ["number", 40, 260],
      ["add", middle, 60],
      ["multiply", last, 60],
    ];
    links = [
      [0, "value", 2, "a"],
      [1, "value", 2, "b"],
      [2, "value", 3, "a"],
    ];
  } else if (name === "flow") {
    doc.name = "실행 연결 예제";
    const middle = 40 + definition("event").width + 100;
    const last = middle + definition("branch").width + 100;
    layout = [
      ["event", 40, 60],
      ["boolean", 40, 260],
      ["branch", middle, 60],
      ["action", last, 20],
      ["delay", last, 300],
    ];
    links = [
      [0, "out", 2, "in"],
      [1, "value", 2, "condition"],
      [2, "true", 3, "in"],
      [2, "false", 4, "in"],
    ];
  } else if (name !== "empty")
    throw new GraphError("example", "지원하지 않는 예제입니다.");
  doc.nodes = layout.map(([kind, x, y]) => makeNode(kind, { x, y }));
  if (name === "values") {
    doc.nodes[0].values.value = 10;
    doc.nodes[1].values.value = 5;
    doc.nodes[3].values.b = 2;
  }
  doc.edges = links.map(([a, ap, b, bp]) => ({
    id: id(),
    source: { node: doc.nodes[a].id, port: ap },
    target: { node: doc.nodes[b].id, port: bp },
  }));
  canonicalize(doc);
  return doc;
}
