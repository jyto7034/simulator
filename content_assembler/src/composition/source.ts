import { isAlias, isMap, isScalar, isSeq, parseDocument } from "yaml";

export type Json =
  null | boolean | number | string | Json[] | { [key: string]: Json };
export type Path = (string | number)[];
export type Span = { start: number; end: number; line: number; column: number };
export type SourceIssue = {
  code: string;
  reason: string;
  path: Path;
  span: Span;
};
export type ParsedSource =
  | { ok: true; value: Json; spans: Readonly<Record<string, Span>> }
  | { ok: false; issues: SourceIssue[] };
export const sourceLimit = 2_000_000;
const pathKey = (path: Path) => JSON.stringify(path);
export const object = (
  value: Json | undefined,
): value is { [key: string]: Json } =>
  !!value && typeof value === "object" && !Array.isArray(value);
export const at = (value: Json, path: Path): Json | undefined =>
  path.reduce<Json | undefined>((node, part) => {
    if (Array.isArray(node) && typeof part === "number") return node[part];
    if (object(node) && typeof part === "string") return node[part];
    return undefined;
  }, value);

export function parseSource(source: string): ParsedSource {
  const lines = [0];
  for (let i = 0; i < source.length; i++)
    if (source[i] === "\n") lines.push(i + 1);
  const span = (start = 0, end = start): Span => {
    let low = 0,
      high = lines.length;
    while (low + 1 < high) {
      const mid = Math.floor((low + high) / 2);
      if (lines[mid] <= start) low = mid;
      else high = mid;
    }
    return { start, end, line: low + 1, column: start - lines[low] + 1 };
  };
  const issue = (
    reason: string,
    path: Path = [],
    start = 0,
    end = start,
    code = "source.parse",
  ): SourceIssue => ({ code, reason, path, span: span(start, end) });
  if (new TextEncoder().encode(source).length > sourceLimit)
    return {
      ok: false,
      issues: [issue("원본 크기 제한은 2MB입니다.", [], 0, 0, "source.limit")],
    };
  if (source.split("\n").some((line) => line.startsWith("%")))
    return {
      ok: false,
      issues: [
        issue(
          "YAML 지시자는 지원하지 않습니다. 고정된 YAML 1.2 문법을 사용하세요.",
        ),
      ],
    };
  let doc;
  try {
    doc = parseDocument(source, {
      version: "1.2",
      uniqueKeys: true,
      keepSourceTokens: true,
    });
  } catch (error) {
    return {
      ok: false,
      issues: [issue(error instanceof Error ? error.message : String(error))],
    };
  }
  if (doc.errors.length || doc.warnings.length)
    return {
      ok: false,
      issues: [...doc.errors, ...doc.warnings].map((error) =>
        issue(error.message, [], error.pos?.[0], error.pos?.[1]),
      ),
    };
  const spans: Record<string, Span> = Object.create(null) as Record<
    string,
    Span
  >;
  let count = 0;
  const issues: SourceIssue[] = [];
  function walk(node: unknown, path: Path, depth: number): Json {
    if (
      !node ||
      (!isMap(node) && !isSeq(node) && !isScalar(node) && !isAlias(node))
    ) {
      issues.push(
        issue("값을 입력하세요. null은 명시적으로 작성해야 합니다.", path),
      );
      return null;
    }
    const range = node.range ?? [0, 0];
    spans[pathKey(path)] = span(range[0], range[1]);
    const fail = (reason: string) => {
      issues.push(issue(reason, path, range[0], range[1]));
      return null;
    };
    count++;
    if (count > 100_000 || ((isMap(node) || isSeq(node)) && depth > 64))
      return fail("원본의 값 개수 또는 중첩 제한을 초과했습니다.");
    if (isAlias(node))
      return fail(
        "YAML 별칭을 지원하지 않습니다. 명시적인 이름 참조를 사용하세요.",
      );
    if (node.anchor || node.tag)
      return fail("YAML 앵커·태그를 지원하지 않습니다.");
    if (isMap(node)) {
      const result: { [key: string]: Json } = Object.create(null) as {
        [key: string]: Json;
      };
      for (const pair of node.items) {
        count++;
        if (
          !isScalar(pair.key) ||
          typeof pair.key.value !== "string" ||
          pair.key.value === "<<" ||
          pair.key.anchor ||
          pair.key.tag
        )
          return fail(
            "키는 태그·앵커가 없는 문자열이어야 합니다. merge는 지원하지 않습니다.",
          );
        result[pair.key.value] = walk(
          pair.value,
          [...path, pair.key.value],
          depth + 1,
        );
      }
      return result;
    }
    if (isSeq(node))
      return node.items.map((child, i) => walk(child, [...path, i], depth + 1));
    const value: unknown = node.value;
    if (value === null)
      return node.source === ""
        ? fail("빈 값은 실행할 수 없습니다. null을 명시하거나 값을 입력하세요.")
        : null;
    if (typeof value === "string" || typeof value === "boolean") return value;
    if (
      typeof value === "number" &&
      Number.isFinite(value) &&
      (!Number.isInteger(value) || Number.isSafeInteger(value))
    )
      return value === 0 ? 0 : value;
    return fail("유한한 수치와 정확히 표현할 수 있는 정수만 지원합니다.");
  }
  const value = walk(doc.contents, [], 1);
  return issues.length ? { ok: false, issues } : { ok: true, value, spans };
}

export function scalarPatch(
  source: string,
  path: Path,
  value: null | boolean | number | string,
): string {
  const parsed = parseSource(source);
  if (!parsed.ok) throw new Error("먼저 원본 문법 오류를 수정하세요.");
  const previous = at(parsed.value, path);
  if (
    previous === undefined ||
    (previous !== null && typeof previous === "object")
  )
    throw new Error("선택한 값이 바뀌었습니다. 다시 선택하세요.");
  if (
    typeof value === "number" &&
    (!Number.isFinite(value) ||
      (Number.isInteger(value) && !Number.isSafeInteger(value)))
  )
    throw new Error("수치 범위를 확인하세요.");
  const location = parsed.spans[pathKey(path)];
  if (!location) throw new Error("원본 위치를 찾지 못했습니다.");
  const candidate =
    source.slice(0, location.start) +
    JSON.stringify(value) +
    source.slice(location.end);
  const checked = parseSource(candidate);
  if (!checked.ok || at(checked.value, path) !== value)
    throw new Error("원본 의미를 보존할 수 없어 편집을 취소했습니다.");
  return candidate;
}

// Compiler paths omit externally tagged enum names after semantic lowering.
// Walk the same source, transparently crossing singleton enum wrappers.
export function modelLocation(
  parsed: ParsedSource,
  modelPath: string,
): Span | null {
  if (!parsed.ok) return null;
  const parts: Path = Array.from(
    modelPath.matchAll(/([^.[\]]+)|\[(\d+)\]/g),
    (match) => (match[2] === undefined ? match[1] : Number(match[2])),
  );
  const path: Path = [];
  let current: Json | undefined = parsed.value;
  for (const part of parts) {
    while (
      object(current) &&
      !(typeof part === "string" && Object.hasOwn(current, part)) &&
      Object.keys(current).length === 1
    ) {
      const key: string = Object.keys(current)[0];
      path.push(key);
      current = current[key];
    }
    const child: Json | undefined =
      current === undefined ? undefined : at(current, [part]);
    if (child === undefined) break;
    path.push(part);
    current = child;
  }
  return parsed.spans[pathKey(path)] ?? null;
}
