import type { Command, Exported, Fragment, Snapshot, Version } from "./types";
import { catalog } from "./core/catalog";
import { createDocument, freeze } from "./core/model";
import { GraphSession } from "./core/session";

export type EditResult = {
  state: Snapshot;
  selected: string[];
  fragment?: Fragment;
  export?: Exported;
};
type View = {
  state: Snapshot;
  catalog: typeof catalog;
  message: string;
  error: boolean;
};

// React subscribes to this local store. Editing itself stays synchronous and independent of React.
export class NodeStore {
  private readonly session: GraphSession;
  private view: View;
  private listeners = new Set<() => void>();
  constructor(session = new GraphSession(createDocument())) {
    this.session = session;
    this.view = freeze({
      state: session.getSnapshot(),
      catalog,
      message:
        "포트를 연결하고 노드를 배치하세요. 이동도 실행 취소할 수 있습니다.",
      error: false,
    });
  }
  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };
  getSnapshot = () => this.view;
  private publish(patch: Partial<View>) {
    this.view = freeze({ ...this.view, ...patch });
    // Observer failure happens after commit and must not turn a successful edit into a failed edit.
    for (const listener of [...this.listeners]) {
      try {
        listener();
      } catch (error) {
        console.error("확정된 노드 편집의 화면 알림 실패", error);
      }
    }
  }
  message(message: string, error = false) {
    this.publish({ message, error });
  }
  version(): Version {
    const { epoch, revision } = this.view.state;
    return { epoch, revision };
  }
  private run(
    action: () => Partial<Omit<EditResult, "state">> | void,
  ): EditResult | undefined {
    let result: EditResult;
    try {
      result = { selected: [], ...action(), state: this.session.getSnapshot() };
    } catch (error) {
      this.message(
        error instanceof Error ? error.message : String(error),
        true,
      );
      return;
    }
    this.publish({ state: result.state, error: false });
    return result;
  }
  edit(command: Command, version = this.version()) {
    return this.run(() => ({ selected: this.session.edit(command, version) }));
  }
  travel(redo: boolean) {
    return this.run(() => this.session.travel(redo));
  }
  create(version = this.version()) {
    return this.run(() =>
      this.session.replace(createDocument(), false, version),
    );
  }
  load(json: string, version: Version) {
    return this.run(() => this.session.load(json, version));
  }
  copy(ids: string[]) {
    return this.run(() => ({ fragment: this.session.copy(ids) }));
  }
  export() {
    return this.run(() => ({ export: this.session.export() }));
  }
  acknowledge(token: string) {
    return this.run(() => this.session.acknowledge(token));
  }
}
