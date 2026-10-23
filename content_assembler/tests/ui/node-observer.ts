import type { Page } from "@playwright/test";
import type { Snapshot } from "../../src/nodes/types";

// Test-only module instrumentation uses the store's public snapshot/subscription API.
// No test globals, alternate state or observability branches ship in product code.
export async function instrumentNodeStore(page: Page) {
  await page.route("**/src/nodes/store.ts", async (route) => {
    const response = await route.fetch();
    const source = await response.text();
    await route.fulfill({
      response,
      body:
        source +
        `
const OriginalNodeStore = NodeStore;
NodeStore = class extends OriginalNodeStore {
  constructor(...args) {
    super(...args);
    window.__nodeCommitTimes = [];
    const notify = () => window.__nodeSnapshot?.(this.getSnapshot().state);
    const subscribe = this.subscribe;
    subscribe(notify);
    // StrictMode may discard a constructed store. Capture the one React subscribes to.
    this.subscribe = (listener) => {
      window.__nodeTestStore = this;
      return subscribe(listener);
    };
    notify();
  }
  edit(...args) {
    const start = performance.now();
    const before = this.getSnapshot().state;
    const result = super.edit(...args);
    if (this.getSnapshot().state.revision !== before.revision)
      window.__nodeCommitTimes.push(performance.now() - start);
    return result;
  }
};
`,
    });
  });
}

export async function observeNodes(page: Page) {
  let latest: Snapshot | undefined;
  const edits: number[] = [];
  await page.exposeFunction("__nodeSnapshot", (state: Snapshot) => {
    if (
      latest &&
      latest.epoch === state.epoch &&
      latest.revision !== state.revision
    )
      edits.push(state.revision);
    latest = state;
  });
  await instrumentNodeStore(page);
  return { state: () => latest!, edits };
}

export function readNodeState(page: Page) {
  return page.evaluate(() => {
    const store = (
      window as Window & {
        __nodeTestStore?: { getSnapshot(): { state: Snapshot } };
      }
    ).__nodeTestStore;
    if (!store) throw new Error("Node store has not mounted");
    return store.getSnapshot().state;
  });
}
export function readCommitTimes(page: Page) {
  return page.evaluate(
    () =>
      (window as Window & { __nodeCommitTimes?: number[] }).__nodeCommitTimes ??
      [],
  );
}
