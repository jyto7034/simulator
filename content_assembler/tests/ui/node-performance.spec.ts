import { makeNode } from "../../src/nodes/core/catalog";
import {
  instrumentNodeStore,
  readNodeState,
  readCommitTimes,
} from "./node-observer";
import { test, expect, openWorkspace, capture } from "./fixtures";

test("measure a 500-node graph with one local commit per drag", async ({
  page,
}, info) => {
  await instrumentNodeStore(page);
  await openWorkspace(page, info);
  await page
    .getByRole("navigation", { name: "작업 공간" })
    .getByRole("button", { name: "노드", exact: true })
    .click();
  await expect(page.getByTestId("node-canvas")).toBeVisible();
  await expect
    .poll(async () => (await readNodeState(page)).doc.nodes.length)
    .toBe(0);
  const initial = await readNodeState(page);
  const template = makeNode("add", { x: 0, y: 0 });
  const nodes = Array.from({ length: 500 }, (_, i) => ({
    ...template,
    id: `measure-${i}`,
    title: `측정용 계산 노드 ${i}`,
    position: { x: (i % 20) * 300, y: Math.floor(i / 20) * 220 },
  }));
  const edges = nodes.slice(1).map((n, i) => ({
    id: `link-${i}`,
    source: { node: nodes[i].id, port: "value" },
    target: { node: n.id, port: "a" },
  }));
  const doc = {
    ...initial.doc,
    name: "500개 노드 측정",
    nodes,
    edges,
    frames: [],
  };
  await page.getByLabel("노드 그래프 파일 열기").setInputFiles({
    name: "measure.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(doc)),
  });
  await page.getByRole("button", { name: "교체", exact: true }).click();
  await expect
    .poll(async () => (await readNodeState(page)).doc.nodes.length)
    .toBe(500);
  const node = page.getByTestId("rf__node-measure-250");
  await expect(node).toBeInViewport();
  const box = (await node.locator(".ng-drag-handle").boundingBox())!;
  await page.evaluate(() => {
    const perf = { frames: [] as number[], running: true };
    Object.assign(window, { __nodePerf: perf });
    function tick(time: number) {
      perf.frames.push(time);
      if (perf.running) requestAnimationFrame(tick);
    }
    requestAnimationFrame(tick);
  });
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(
    box.x + box.width / 2 + 75,
    box.y + box.height / 2 + 30,
    { steps: 30 },
  );
  expect(await readCommitTimes(page)).toHaveLength(0);
  await page.mouse.up();
  await expect.poll(async () => (await readNodeState(page)).undoCount).toBe(1);
  const state = await readNodeState(page);
  const timings = await readCommitTimes(page);
  expect(timings).toHaveLength(1);
  expect(
    state.doc.nodes.find((n) => n.id === "measure-250")!.position,
  ).not.toEqual(nodes[250].position);
  const frames = await page.evaluate(() => {
    const perf = (
      window as Window & { __nodePerf?: { running: boolean; frames: number[] } }
    ).__nodePerf;
    if (!perf) throw new Error("Frame measurement was not started");
    perf.running = false;
    return perf.frames.slice(1).map((time, i) => time - perf.frames[i]);
  });
  frames.sort((a, b) => a - b);

  await info.attach("node-performance", {
    contentType: "application/json",
    body: JSON.stringify(
      {
        nodes: 500,
        edges: 499,
        dragSteps: 30,
        commits: timings.length,
        frameSamples: frames.length,
        frameMedianMs: frames[Math.floor(frames.length / 2)],
        frameP95Ms: frames[Math.floor(frames.length * 0.95)],
        localCommitMs: timings[0],
        browser: info.project.name,
      },
      null,
      2,
    ),
  });
  await capture(page, info, "nodes-500");
});
