import { test, expect } from "@playwright/test";
import ELK from "elkjs/lib/elk.bundled.js";
import { exampleSkill } from "../../src/skill/examples";
import { createSkill, makeBlock } from "../../src/skill/model";
import {
  blockAncestors,
  descendantIds,
  projectSkillGraph,
} from "../../src/skill/graph-model";
import {
  layoutRequest,
  arrangeSkillGraphs,
} from "../../src/skill/graph-layout";
import { diagnoseSkill } from "../../src/skill/validation";

test("folding and revealing branches preserves the authored document and joins both paths before continuing", () => {
  const doc = exampleSkill("hood"),
    before = JSON.stringify(doc);
  const routine = doc.routines[2],
    branch = routine.blocks[1],
    next = routine.blocks[2];
  const folded = projectSkillGraph(doc, new Set(), diagnoseSkill(doc));
  expect(folded.items.some((n) => n.id === branch.children[0].id)).toBe(false);
  expect(
    folded.links.some((e) => e.source === branch.id && e.target === next.id),
  ).toBe(true);
  const expanded = projectSkillGraph(
    doc,
    new Set([branch.id]),
    diagnoseSkill(doc),
  );
  expect(
    expanded.links.filter((e) => e.source === branch.id).map((e) => e.label),
  ).toEqual(["참", "아니면"]);
  const join = `join:${branch.id}`;
  expect(
    expanded.links.filter((e) => e.target === join).map((e) => e.source),
  ).toEqual([branch.children[0].id, branch.otherwise[0].id]);
  expect(
    expanded.links.some((e) => e.source === join && e.target === next.id),
  ).toBe(true);
  expect(blockAncestors(doc, branch.otherwise[0].id)).toEqual([branch.id]);
  expect(JSON.stringify(doc)).toBe(before);
});

test("parallel lanes fan out independently and collapsed diagnostics include descendants", () => {
  const doc = createSkill();
  const container = makeBlock("parallel", {}, [
    makeBlock("wait"),
    makeBlock("damage"),
  ]);
  doc.routines[0].blocks = [container, makeBlock("control")];
  const diagnostics = diagnoseSkill(doc);
  const folded = projectSkillGraph(doc, new Set(), diagnostics);
  expect(folded.items.find((n) => n.id === container.id)?.issueCount).toBe(5);
  const expanded = projectSkillGraph(doc, new Set([container.id]), diagnostics);
  const first = container.children[0],
    second = container.children[1];
  expect(
    expanded.links.some((e) => e.source === first.id && e.target === second.id),
  ).toBe(false);
  expect(
    expanded.links
      .filter((e) => e.source === container.id)
      .map((e) => e.target),
  ).toEqual([first.id, second.id]);
  expect(descendantIds(container)).toEqual([container.id, first.id, second.id]);
  // Separate visual lanes still share the authored sibling order used by editing commands.
  expect(expanded.items.find((n) => n.id === first.id)).toMatchObject({
    canMoveUp: false,
    canMoveDown: true,
  });
  expect(expanded.items.find((n) => n.id === second.id)).toMatchObject({
    title: "2. 피해 적용",
    canMoveUp: true,
    canMoveDown: false,
  });
});

test("ELK gives every visible node a finite, non-overlapping position, preserving event rows", async () => {
  const doc = exampleSkill("hood");
  const graph = projectSkillGraph(
    doc,
    new Set([doc.routines[2].blocks[1].id]),
    diagnoseSkill(doc),
  );
  const requests = layoutRequest(graph);
  const elk = new ELK();
  // The bundled Node executor is in-process and does not create a terminable Web Worker.
  {
    const positions = await arrangeSkillGraphs(elk, requests);
    expect(Object.keys(positions)).toHaveLength(graph.items.length);
    const nodes = requests.flatMap((r) => r.children ?? []);
    for (const a of nodes) {
      expect(
        Number.isFinite(positions[a.id].x) &&
          Number.isFinite(positions[a.id].y),
      ).toBe(true);
      for (const b of nodes)
        if (a.id !== b.id) {
          const p = positions[a.id],
            q = positions[b.id];
          const overlap =
            p.x < q.x + b.width! &&
            p.x + a.width! > q.x &&
            p.y < q.y + b.height! &&
            p.y + a.height! > q.y;
          expect(overlap, `${a.id} overlaps ${b.id}`).toBe(false);
        }
    }
    expect(positions[doc.routines[0].id].y).toBeLessThan(
      positions[doc.routines[1].id].y,
    );
    expect(await arrangeSkillGraphs(elk, [])).toEqual({});
  }
});
