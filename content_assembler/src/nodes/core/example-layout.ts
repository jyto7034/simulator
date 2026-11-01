import type { GraphDocument } from "../types";
import { definition } from "./catalog";

// Deterministic initial placement for generated examples. Interactive refinement
// uses ELK; creation must already keep connected dependencies and cards readable.
export function placeExample(doc: GraphDocument) {
  let top = 80;
  const frameTop = (id: string) =>
    Math.min(
      ...doc.nodes.filter((n) => n.frame === id).map((n) => n.position.y),
    );
  const frames = [...doc.frames].sort(
    (a, b) => frameTop(a.id) - frameTop(b.id),
  );
  for (const frame of [null, ...frames.map((f) => f.id)]) {
    const members = doc.nodes.filter((n) => n.frame === frame);
    if (!members.length) continue;
    const pending = new Set(members.map((n) => n.id));
    const levels = new Map<string, number>();
    const edges = doc.edges.filter(
      (e) => pending.has(e.source.node) && pending.has(e.target.node),
    );
    while (pending.size) {
      const ready = members.filter(
        (n) =>
          pending.has(n.id) &&
          !edges.some(
            (e) => e.target.node === n.id && pending.has(e.source.node),
          ),
      );
      // Invalid/cyclic drafts remain editable. Do not loop or change their edges.
      if (!ready.length) {
        const level = Math.max(-1, ...levels.values()) + 1;
        for (const id of pending) levels.set(id, level);
        break;
      }
      for (const n of ready) {
        levels.set(
          n.id,
          Math.max(
            -1,
            ...edges
              .filter((e) => e.target.node === n.id)
              .map((e) => levels.get(e.source.node) ?? -1),
          ) + 1,
        );
        pending.delete(n.id);
      }
    }
    const bottoms = new Map<number, number>();
    for (const n of members) {
      const level = levels.get(n.id) ?? 0;
      const d = definition(n.kind);
      n.position = { x: 40 + level * 440, y: bottoms.get(level) ?? top };
      bottoms.set(level, n.position.y + d.height + 88);
    }
    top = Math.max(...bottoms.values()) + 160;
  }
}
