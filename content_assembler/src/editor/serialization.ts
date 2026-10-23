import { assertMapDimension, type MapDocument } from "./model";
type Obj = Record<string, unknown>;
function object(v: unknown): asserts v is Obj {
  if (!v || typeof v !== "object" || Array.isArray(v))
    throw new Error("객체가 필요합니다.");
}
function shape(v: unknown, keys: string): asserts v is Obj {
  object(v);
  const expected = keys.split(" ");
  if (
    Object.keys(v).some((k) => !expected.includes(k)) ||
    expected.some((k) => !(k in v))
  )
    throw new Error(`필드 구조가 올바르지 않습니다: ${keys}`);
}
function str(v: unknown): asserts v is string {
  if (typeof v !== "string" || v.length > 10000)
    throw new Error("문자열 형식 또는 길이가 올바르지 않습니다.");
}
function num(v: unknown): asserts v is number {
  if (typeof v !== "number" || !Number.isFinite(v) || Math.abs(v) > 1e9)
    throw new Error("유한한 숫자가 필요합니다 (절댓값 ≤ 10억).");
}
function list(v: unknown): asserts v is unknown[] {
  if (!Array.isArray(v) || v.length > 20000)
    throw new Error("목록 형식 또는 크기가 올바르지 않습니다.");
}
function cells(v: unknown) {
  list(v);
  const seen = new Set();
  for (const k of v) {
    str(k);
    if (
      !/^\d{1,3},\d{1,3}$/.test(k) ||
      k.split(",").some((n) => Number(n) > 127 || String(Number(n)) !== n) ||
      seen.has(k)
    )
      throw new Error("셀 키가 잘못되었거나 중복되었습니다.");
    seen.add(k);
  }
}
export function validateDocument(v: unknown): asserts v is MapDocument {
  shape(
    v,
    "format version id name description width height valid terrain zones routes points slots waves",
  );
  if (v.format !== "battle-map" || v.version !== 1)
    throw new Error("지원하지 않는 문서 형식 또는 버전입니다.");
  const ids = new Set<string>();
  const id = (value: unknown) => {
    str(value);
    if (!value.trim() || ids.has(value))
      throw new Error("ID가 비어 있거나 중복되었습니다.");
    ids.add(value);
  };
  const named = (o: Obj) => {
    id(o.id);
    str(o.name);
  };
  named(v);
  str(v.description);
  for (const n of [v.width, v.height]) assertMapDimension(n);
  cells(v.valid);
  object(v.terrain);
  cells(Object.keys(v.terrain));
  for (const t of Object.values(v.terrain))
    if (t !== "floor" && t !== "platform" && t !== "obstacle")
      throw new Error("알 수 없는 지형입니다.");
  list(v.zones);
  for (const z of v.zones) {
    shape(z, "id name kind cells");
    named(z);
    if (z.kind !== "deploy" && z.kind !== "spawn")
      throw new Error("알 수 없는 구역 종류입니다.");
    cells(z.cells);
  }
  list(v.routes);
  for (const r of v.routes) {
    shape(r, "id name points");
    named(r);
    list(r.points);
    for (const p of r.points) {
      shape(p, "id x y");
      id(p.id);
      num(p.x);
      num(p.y);
      if (Math.abs(p.x) > 128 || Math.abs(p.y) > 128)
        throw new Error("경로 좌표의 지원 범위를 벗어났습니다.");
    }
  }
  list(v.points);
  for (const p of v.points) {
    shape(p, "id name x y");
    named(p);
    num(p.x);
    num(p.y);
  }
  list(v.slots);
  for (const s of v.slots) {
    shape(s, "id name");
    named(s);
  }
  list(v.waves);
  for (const w of v.waves) {
    shape(
      w,
      "id name enemies zoneId routeId trigger fallbackMs priority required",
    );
    named(w);
    str(w.zoneId);
    str(w.routeId);
    num(w.priority);
    if (typeof w.required !== "boolean")
      throw new Error("승리 필수 여부가 올바르지 않습니다.");
    if (w.fallbackMs !== null) num(w.fallbackMs);
    list(w.enemies);
    for (const e of w.enemies) {
      shape(e, "id contentId count intervalMs");
      id(e.id);
      str(e.contentId);
      num(e.count);
      num(e.intervalMs);
    }
    object(w.trigger);
    const t = w.trigger;
    switch (t.kind) {
      case "time":
        shape(t, "kind timeMs");
        num(t.timeMs);
        break;
      case "clear":
        shape(t, "kind waveId");
        str(t.waveId);
        break;
      case "hp":
        shape(t, "kind slotId percent");
        str(t.slotId);
        num(t.percent);
        break;
      case "death":
        shape(t, "kind slotId");
        str(t.slotId);
        break;
      case "reach":
        shape(t, "kind slotId pointId radius");
        str(t.slotId);
        str(t.pointId);
        num(t.radius);
        break;
      default:
        throw new Error("지원하지 않는 발동 조건입니다.");
    }
  }
}
function checkFileSize(text: string) {
  if (
    text.length > 5_000_000 ||
    new TextEncoder().encode(text).byteLength > 5_000_000
  )
    throw new Error("파일 크기 제한은 UTF-8 기준 5MB입니다.");
}
export function parseDocument(text: string): MapDocument {
  checkFileSize(text);
  const value: unknown = JSON.parse(text);
  validateDocument(value);
  return value;
}
export function serializeDocument(doc: MapDocument): string {
  validateDocument(doc);
  const text = JSON.stringify(doc, null, 2);
  checkFileSize(text);
  return text;
}
