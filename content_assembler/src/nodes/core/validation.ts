import type {
  Diagnostic,
  Endpoint,
  Fragment,
  FrameRect,
  GraphDocument,
  GraphNode,
  Point,
} from "../types";
import { definition } from "./catalog";
import {
  isLegacyArea,
  legacyAreaFields,
  readTilePattern,
} from "./tile-pattern";
import { skillDiagnostics } from "./skill-validation";
import { inputSources } from "./connections";
import { secondsToMilliseconds, timeValueMessage } from "./time-values";
import {
  byteLength,
  createDocument,
  GraphError,
  MAX_FILE_BYTES,
} from "./model";

function fail(code: string, message: string): never {
  throw new GraphError(code, message);
}
function shape(
  value: unknown,
  keys: string,
): asserts value is Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value))
    fail("parse", "노드 파일을 읽지 못했습니다: 객체가 필요합니다.");
  const expected = keys.split(" ");
  if (
    Object.keys(value).some((k) => !expected.includes(k)) ||
    expected.some((k) => !Object.hasOwn(value, k))
  )
    fail(
      "parse",
      `노드 파일을 읽지 못했습니다: 필드 구조가 올바르지 않습니다 (${keys}).`,
    );
}
function string(value: unknown): asserts value is string {
  if (typeof value !== "string")
    fail("parse", "노드 파일을 읽지 못했습니다: 문자열이 필요합니다.");
}
function text(value: string) {
  if (byteLength(value) > 16_384)
    fail("text-limit", "이름 또는 메모가 너무 깁니다.");
}
function list(value: unknown, max: number): asserts value is unknown[] {
  if (!Array.isArray(value))
    fail("parse", "노드 파일을 읽지 못했습니다: 목록이 필요합니다.");
  if (value.length > max)
    fail("limit", "지원 범위는 노드 2,000개·연결 8,000개·프레임 200개입니다.");
}
function point(value: unknown): asserts value is Point {
  shape(value, "x y");
  if (
    ![value.x, value.y].every(
      (v) =>
        typeof v === "number" && Number.isFinite(v) && Math.abs(v) <= 1_000_000,
    )
  )
    fail("position", "노드 위치가 유효 범위를 벗어났습니다.");
}
function endpoint(value: unknown): asserts value is Endpoint {
  shape(value, "node port");
  string(value.node);
  string(value.port);
}
function node(value: unknown): asserts value is GraphNode {
  shape(value, "id kind title position values note frame");
  string(value.id);
  string(value.kind);
  string(value.title);
  string(value.note);
  text(value.title);
  text(value.note);
  point(value.position);
  if (value.frame !== null) string(value.frame);
  const def = definition(value.kind);
  const fields = value.values;
  if (!fields || typeof fields !== "object" || Array.isArray(fields))
    fail("fields", "노드 속성 객체가 필요합니다.");
  const entries = Object.entries(fields);
  const fieldDefs = isLegacyArea({ kind: value.kind, values: fields })
    ? legacyAreaFields
    : value.kind === "skill-area-damage" && entries.length === 0
      ? [] // Historical area-damage had no damage type; preserve it as an unfinished draft.
      : def.fields;
  if (
    entries.length !== fieldDefs.length ||
    entries.some(([key]) => !fieldDefs.some((f) => f.id === key))
  )
    fail("fields", "노드 속성이 누락되었거나 지원하지 않는 속성이 있습니다.");
  for (const field of fieldDefs) {
    const entry = entries.find(([key]) => key === field.id);
    const v: unknown = entry?.[1];
    if (
      !(field.valueType === "optional-number"
        ? v === "" || (typeof v === "number" && Number.isFinite(v))
        : field.valueType === "number"
          ? typeof v === "number" && Number.isFinite(v)
          : field.valueType === "boolean"
            ? typeof v === "boolean"
            : typeof v === "string" && byteLength(v) <= 16_384)
    )
      fail(
        "field-type",
        `${def.label}: ${field.label} 값의 형식이 올바르지 않습니다.`,
      );
    if (field.valueType === "tile-pattern") readTilePattern(v as string);
    if (field.integer && typeof v === "number" && !Number.isSafeInteger(v))
      fail(
        "field-integer",
        `${field.label}: 소수 없이 안전한 정수를 입력하세요.`,
      );
    if (
      field.unit === "seconds" &&
      typeof v === "number" &&
      secondsToMilliseconds(v) === null
    )
      fail("field-time", `${field.label}: ${timeValueMessage}`);
    if (field.options && !field.options.some((option) => option.value === v))
      fail("field-choice", `${field.label}: 지원하는 항목을 선택하세요.`);
    if (
      typeof v === "number" &&
      ((field.min !== undefined && v < field.min) ||
        (field.max !== undefined && v > field.max))
    )
      fail(
        "field-range",
        `${field.label}: 허용 범위 ${field.min ?? "제한 없음"} ~ ${field.max ?? "제한 없음"}을 확인하세요.`,
      );
  }
}
export function endpointPort(
  nodes: ReadonlyMap<string, GraphNode>,
  end: Endpoint,
  output: boolean,
) {
  const n = nodes.get(end.node);
  if (!n) return fail("missing-node", "연결 대상 노드가 없습니다.");
  const def = definition(n.kind);
  const p = (output ? def.outputs : def.inputs).find((p) => p.id === end.port);
  if (!p)
    return fail(
      "missing-port",
      "연결 대상 포트 또는 방향이 올바르지 않습니다.",
    );
  return p;
}
const endpointKey = (end: Endpoint) => JSON.stringify([end.node, end.port]);

export function validateDocument(
  value: unknown,
): asserts value is GraphDocument {
  shape(value, "format version id name nodes edges frames");
  if (value.format !== "node-graph" || value.version !== 1)
    fail(
      "format",
      "노드 그래프 version 1 파일이 필요합니다. 기존 스킬 파일은 스킬 작업 공간에서 열어주세요.",
    );
  string(value.id);
  string(value.name);
  text(value.name);
  list(value.nodes, 2000);
  list(value.edges, 8000);
  list(value.frames, 200);
  const ids = new Set<string>();
  const unique = (key: unknown) => {
    string(key);
    if (!key || byteLength(key) > 128 || ids.has(key))
      fail("id", "ID가 비어 있거나 중복되거나 너무 깁니다.");
    ids.add(key);
  };
  unique(value.id);
  const frames = new Set<string>();
  for (const f of value.frames) {
    shape(f, "id title");
    string(f.id);
    string(f.title);
    text(f.title);
    unique(f.id);
    frames.add(f.id);
  }
  const nodes = new Map<string, GraphNode>();
  const usedFrames = new Set<string>();
  for (const n of value.nodes) {
    node(n);
    unique(n.id);
    nodes.set(n.id, n);
    if (n.frame !== null) {
      if (!frames.has(n.frame))
        fail("missing-frame", "노드의 프레임이 없습니다.");
      usedFrames.add(n.frame);
    }
  }
  if ([...frames].some((f) => !usedFrames.has(f)))
    fail("empty-frame", "프레임에 노드가 필요합니다.");
  const inputs = new Set<string>(),
    outputs = new Set<string>(),
    links = new Set<string>();
  for (const e of value.edges) {
    shape(e, "id source target");
    unique(e.id);
    endpoint(e.source);
    endpoint(e.target);
    const a = endpointPort(nodes, e.source, true),
      b = endpointPort(nodes, e.target, false);
    if (a.valueType !== b.valueType)
      fail("port-type", "같은 자료형의 출력과 입력만 연결할 수 있습니다.");
    const source = endpointKey(e.source),
      target = endpointKey(e.target),
      key = JSON.stringify([source, target]);
    if (links.has(key)) fail("duplicate-link", "동일한 연결이 중복됩니다.");
    links.add(key);
    if (
      (!a.multiple && outputs.has(source)) ||
      (!b.multiple && inputs.has(target))
    )
      fail("port-capacity", "단일 연결 포트에 여러 연결이 있습니다.");
    outputs.add(source);
    inputs.add(target);
  }
}

function parseJSON(content: string): unknown {
  if (byteLength(content) > MAX_FILE_BYTES)
    fail("file-size", "파일 크기 제한은 5MB입니다.");
  try {
    return JSON.parse(content);
  } catch {
    return fail(
      "parse",
      "노드 파일을 읽지 못했습니다: 올바른 JSON이 필요합니다.",
    );
  }
}
export function parseDocument(content: string): GraphDocument {
  const value = parseJSON(content);
  validateDocument(value);
  return value;
}
export function validateFragment(value: unknown): asserts value is Fragment {
  shape(value, "format version nodes edges frames");
  if (value.format !== "node-graph-fragment" || value.version !== 1)
    fail("clipboard", "노드 편집기에서 복사한 내용이 필요합니다.");
  validateDocument({
    ...createDocument(),
    nodes: value.nodes,
    edges: value.edges,
    frames: value.frames,
  });
}
export function parseFragment(content: string): Fragment {
  const value = parseJSON(content);
  validateFragment(value);
  return value;
}

export function diagnostics(doc: GraphDocument): Diagnostic[] {
  const out: Diagnostic[] = [];
  const source = inputSources(doc);
  const incoming = new Set(doc.edges.map((e) => endpointKey(e.target)));
  const nodes = new Map(doc.nodes.map((n) => [n.id, n]));
  for (const n of doc.nodes) {
    const def = definition(n.kind);
    if (n.kind === "delay") {
      const value = source(n, "seconds");
      if (
        value?.kind === "number" &&
        typeof value.values.value === "number" &&
        secondsToMilliseconds(value.values.value) === null
      )
        out.push({
          code: "time-precision",
          nodeId: n.id,
          portId: "seconds",
          message: `${n.title || def.label} · ${timeValueMessage}`,
        });
    }
    for (const p of def.inputs)
      if (p.required && !incoming.has(endpointKey({ node: n.id, port: p.id })))
        out.push({
          code: "unconnected-input",
          nodeId: n.id,
          portId: p.id,
          message: `${n.title || def.label} · ${p.label} 입력이 연결되지 않았습니다.`,
        });
  }
  for (const flow of [false, true]) {
    const degree = new Map(doc.nodes.map((n) => [n.id, 0]));
    const children = new Map<string, string[]>();
    for (const e of doc.edges)
      if ((endpointPort(nodes, e.source, true).valueType === "flow") === flow) {
        degree.set(e.target.node, degree.get(e.target.node)! + 1);
        const list = children.get(e.source.node) ?? [];
        list.push(e.target.node);
        children.set(e.source.node, list);
      }
    const queue = [...degree].filter(([, v]) => v === 0).map(([key]) => key);
    for (let i = 0; i < queue.length; i++)
      for (const child of children.get(queue[i]) ?? []) {
        const d = degree.get(child)! - 1;
        degree.set(child, d);
        if (!d) queue.push(child);
      }
    for (const n of doc.nodes)
      if (degree.get(n.id)! > 0)
        out.push({
          code: "cycle-dependency",
          nodeId: n.id,
          portId: null,
          message: `${flow ? "실행" : "데이터"} 연결이 순환하거나 순환에 의존합니다. 실행 규칙을 확인하세요.`,
        });
  }
  return [...out, ...skillDiagnostics(doc)];
}
export function frameRects(doc: GraphDocument): FrameRect[] {
  return doc.frames.map((f) => {
    const members = doc.nodes.filter((n) => n.frame === f.id);
    const x = Math.min(...members.map((n) => n.position.x)) - 24,
      y = Math.min(...members.map((n) => n.position.y)) - 46;
    const right =
      Math.max(...members.map((n) => n.position.x + definition(n.kind).width)) +
      24;
    const bottom =
      Math.max(
        ...members.map((n) => n.position.y + definition(n.kind).height),
      ) + 24;
    return { id: f.id, x, y, width: right - x, height: bottom - y };
  });
}
