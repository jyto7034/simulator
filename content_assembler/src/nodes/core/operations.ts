import { isLegacyArea } from "./tile-pattern";
import type {
  Command,
  Endpoint,
  Fragment,
  GraphDocument,
  Point,
} from "../types";
import { definition, makeNode } from "./catalog";
import { canonicalize, GraphError, id } from "./model";
import { endpointPort, validateFragment } from "./validation";

const missing = () =>
  new GraphError(
    "selection",
    "편집할 항목이 없습니다. 현재 선택을 확인하세요.",
  );
function selectedNodes(doc: GraphDocument, ids: string[]) {
  const selected = new Set(ids);
  return new Set(
    doc.nodes
      .filter(
        (n) =>
          selected.has(n.id) || (n.frame !== null && selected.has(n.frame)),
      )
      .map((n) => n.id),
  );
}
function pruneFrames(doc: GraphDocument) {
  const used = new Set(doc.nodes.map((n) => n.frame));
  doc.frames = doc.frames.filter((f) => used.has(f.id));
}
export function copy(doc: GraphDocument, ids: string[]): Fragment {
  const selected = selectedNodes(doc, ids);
  if (!selected.size) throw missing();
  const frames = doc.frames.filter((f) => ids.includes(f.id));
  return structuredClone({
    format: "node-graph-fragment",
    version: 1,
    frames,
    nodes: doc.nodes
      .filter((n) => selected.has(n.id))
      .map((n) => ({
        ...n,
        frame: frames.some((f) => f.id === n.frame) ? n.frame : null,
      })),
    edges: doc.edges.filter(
      (e) => selected.has(e.source.node) && selected.has(e.target.node),
    ),
  });
}
function paste(
  doc: GraphDocument,
  fragment: Fragment,
  position: Point,
): string[] {
  validateFragment(fragment);
  if (!fragment.nodes.length) throw missing();
  const x = Math.min(...fragment.nodes.map((n) => n.position.x)),
    y = Math.min(...fragment.nodes.map((n) => n.position.y));
  const mapping = new Map(
    [...fragment.nodes, ...fragment.frames].map((n) => [n.id, id()]),
  );
  const nodes = fragment.nodes.map((n) => ({
    ...structuredClone(n),
    id: mapping.get(n.id)!,
    position: {
      x: position.x + n.position.x - x,
      y: position.y + n.position.y - y,
    },
    frame: n.frame === null ? null : mapping.get(n.frame)!,
  }));
  doc.nodes.push(...nodes);
  doc.frames.push(
    ...fragment.frames.map((f) => ({ id: mapping.get(f.id)!, title: f.title })),
  );
  doc.edges.push(
    ...fragment.edges.map((e) => ({
      id: id(),
      source: { node: mapping.get(e.source.node)!, port: e.source.port },
      target: { node: mapping.get(e.target.node)!, port: e.target.port },
    })),
  );
  return nodes.map((n) => n.id);
}
const sameEndpoint = (a: Endpoint, b: Endpoint) =>
  a.node === b.node && a.port === b.port;
function connect(
  doc: GraphDocument,
  source: Endpoint,
  target: Endpoint,
  replace?: string,
) {
  const nodes = new Map(doc.nodes.map((n) => [n.id, n]));
  const a = endpointPort(nodes, source, true),
    b = endpointPort(nodes, target, false);
  if (a.valueType !== b.valueType)
    throw new GraphError(
      "port-type",
      "같은 자료형의 출력과 입력만 연결할 수 있습니다.",
    );
  if (replace !== undefined && !doc.edges.some((e) => e.id === replace))
    throw missing();
  if (
    doc.edges.some(
      (e) =>
        sameEndpoint(e.source, source) &&
        sameEndpoint(e.target, target) &&
        (replace === undefined || e.id === replace),
    )
  )
    return;
  doc.edges = doc.edges.filter(
    (e) =>
      e.id !== replace &&
      (b.multiple || !sameEndpoint(e.target, target)) &&
      (a.multiple || !sameEndpoint(e.source, source)),
  );
  doc.edges.push({ id: id(), source: { ...source }, target: { ...target } });
}

// Only the detached draft is mutated. Session validates and commits the entire result.
export function apply(doc: GraphDocument, command: Command): string[] {
  let selected: string[] = [];
  switch (command.type) {
    case "add": {
      const n = makeNode(command.kind, command.position);
      doc.nodes.push(n);
      selected = [n.id];
      if (command.link) {
        const anchor = command.link;
        const old = endpointPort(
          new Map(doc.nodes.map((n) => [n.id, n])),
          anchor.endpoint,
          anchor.output,
        );
        const def = definition(command.kind);
        const p = (anchor.output ? def.inputs : def.outputs).find(
          (p) => p.valueType === old.valueType,
        );
        if (!p)
          throw new GraphError(
            "port-type",
            "새 노드에 호환되는 포트가 없습니다.",
          );
        const end = { node: n.id, port: p.id };
        connect(
          doc,
          anchor.output ? anchor.endpoint : end,
          anchor.output ? end : anchor.endpoint,
        );
      }
      break;
    }
    case "move": {
      const seen = new Set<string>(),
        nodes = new Map(doc.nodes.map((n) => [n.id, n]));
      for (const p of command.positions) {
        if (seen.has(p.id))
          throw new GraphError(
            "position",
            "같은 노드에 여러 위치가 지정되었습니다.",
          );
        seen.add(p.id);
        const n = nodes.get(p.id);
        if (!n) throw missing();
        n.position = { ...p.position };
      }
      break;
    }
    case "setNode": {
      const n = doc.nodes.find((n) => n.id === command.id);
      if (!n) throw missing();
      if (command.title !== undefined) n.title = command.title;
      if (command.note !== undefined) n.note = command.note;
      if (command.values !== undefined) {
        if (isLegacyArea(n) && Object.hasOwn(command.values, "pattern"))
          n.values = { ...command.values };
        else n.values = { ...n.values, ...command.values };
      }
      break;
    }
    case "rename":
      doc.name = command.name;
      break;
    case "delete": {
      const nodes = selectedNodes(doc, command.ids);
      doc.nodes = doc.nodes.filter((n) => !nodes.has(n.id));
      doc.edges = doc.edges.filter(
        (e) =>
          !nodes.has(e.source.node) &&
          !nodes.has(e.target.node) &&
          !command.ids.includes(e.id),
      );
      pruneFrames(doc);
      break;
    }
    case "connect":
      connect(doc, command.source, command.target, command.replaceEdge);
      break;
    case "disconnect":
      doc.edges = doc.edges.filter((e) => !command.ids.includes(e.id));
      break;
    case "duplicate": {
      const fragment = copy(doc, command.ids);
      selected = paste(doc, fragment, {
        x: Math.min(...fragment.nodes.map((n) => n.position.x)) + 40,
        y: Math.min(...fragment.nodes.map((n) => n.position.y)) + 40,
      });
      break;
    }
    case "paste":
      selected = paste(doc, command.fragment, command.position);
      break;
    case "group": {
      const nodes = selectedNodes(doc, command.ids);
      if (!nodes.size) throw missing();
      const frame = { id: id(), title: command.title };
      for (const n of doc.nodes) if (nodes.has(n.id)) n.frame = frame.id;
      doc.frames.push(frame);
      selected = [frame.id];
      pruneFrames(doc);
      break;
    }
    case "ungroup": {
      const frames = new Set([
        ...command.ids,
        ...doc.nodes
          .filter((n) => command.ids.includes(n.id))
          .map((n) => n.frame),
      ]);
      for (const n of doc.nodes)
        if (n.frame !== null && frames.has(n.frame)) {
          n.frame = null;
          selected.push(n.id);
        }
      pruneFrames(doc);
      break;
    }
    case "renameFrame": {
      const frame = doc.frames.find((f) => f.id === command.id);
      if (!frame) throw missing();
      frame.title = command.title;
      break;
    }
    case "reroute": {
      const edge = doc.edges.find((e) => e.id === command.edgeId);
      if (!edge) throw missing();
      const p = endpointPort(
        new Map(doc.nodes.map((n) => [n.id, n])),
        edge.source,
        true,
      );
      const kind = `reroute-${p.valueType}`;
      let position = command.position;
      if (!position) {
        const from = doc.nodes.find((node) => node.id === edge.source.node)!;
        const to = doc.nodes.find((node) => node.id === edge.target.node)!;
        const size = definition(kind);
        position = {
          x:
            (from.position.x +
              definition(from.kind).width +
              to.position.x -
              size.width) /
            2,
          y: (from.position.y + to.position.y) / 2,
        };
        // Automatic insertion keeps existing layout intact. Move down past
        // intersecting cards until the new card has a clear margin. Explicit
        // double-click positions remain under the author's control.
        const occupied = doc.nodes.map((node) => ({
          ...node.position,
          ...definition(node.kind),
        }));
        for (;;) {
          const blockers = occupied.filter(
            (rect) =>
              position!.x < rect.x + rect.width + 40 &&
              position!.x + size.width + 40 > rect.x &&
              position!.y < rect.y + rect.height + 40 &&
              position!.y + size.height + 40 > rect.y,
          );
          if (!blockers.length) break;
          position.y = Math.max(
            ...blockers.map((rect) => rect.y + rect.height + 40),
          );
        }
      }
      const n = makeNode(kind, position);
      doc.nodes.push(n);
      doc.edges = doc.edges.filter((e) => e.id !== edge.id);
      connect(doc, edge.source, { node: n.id, port: "in" });
      connect(doc, { node: n.id, port: "out" }, edge.target);
      selected = [n.id];
      break;
    }
  }
  canonicalize(doc);
  return selected;
}
