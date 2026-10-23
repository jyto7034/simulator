import type {
  Definition,
  Field,
  FieldValue,
  GraphNode,
  Point,
  Port,
} from "../types";
import { freeze, GraphError, id } from "./model";
import { skillCatalog } from "./skill-catalog";

const port = (
  id: string,
  label: string,
  valueType: string,
  output: boolean,
): Port => ({
  id,
  label,
  valueType,
  multiple: valueType === "flow" ? !output : output,
  required: !output && valueType === "flow",
});
const field = (
  id: string,
  label: string,
  valueType: string,
  value: FieldValue,
): Field => ({ id, label, valueType, default: value });
const def = (
  kind: string,
  label: string,
  category: string,
  inputs: Port[],
  outputs: Port[],
  fields: Field[],
): Definition => ({
  kind,
  label,
  category,
  description: `${label} 노드 · ${inputs.length} 입력 / ${outputs.length} 출력`,
  inputs,
  outputs,
  fields,
  width: 340,
  height: Math.max(
    86,
    62 + Math.max(inputs.length, outputs.length) * 44 + fields.length * 36,
  ),
});
export const catalog: Definition[] = freeze([
  def(
    "number",
    "숫자",
    "값",
    [],
    [port("value", "숫자", "number", true)],
    [field("value", "값", "number", 1)],
  ),
  def(
    "boolean",
    "참 / 거짓",
    "값",
    [],
    [port("value", "조건", "boolean", true)],
    [field("value", "값", "boolean", true)],
  ),
  def(
    "text",
    "문자열",
    "값",
    [],
    [port("value", "문자열", "text", true)],
    [field("value", "값", "text", "")],
  ),
  ...(
    [
      ["add", "더하기", 0],
      ["multiply", "곱하기", 1],
      ["greater", "보다 큼", 0],
    ] as const
  ).map(([kind, label, value]) =>
    def(
      kind,
      label,
      "계산",
      [port("a", "A", "number", false), port("b", "B", "number", false)],
      [
        port(
          "value",
          kind === "greater" ? "A > B" : "결과",
          kind === "greater" ? "boolean" : "number",
          true,
        ),
      ],
      [
        field("a", "A 기본값", "number", value),
        field("b", "B 기본값", "number", value),
      ],
    ),
  ),
  def(
    "event",
    "시작 사건",
    "실행",
    [],
    [port("out", "실행", "flow", true)],
    [field("event", "사건 이름", "text", "시작")],
  ),
  def(
    "branch",
    "조건 분기",
    "실행",
    [
      port("in", "실행", "flow", false),
      port("condition", "조건", "boolean", false),
    ],
    [port("true", "참", "flow", true), port("false", "거짓", "flow", true)],
    [field("condition", "조건 기본값", "boolean", true)],
  ),
  def(
    "action",
    "동작",
    "실행",
    [port("in", "실행", "flow", false), port("text", "내용", "text", false)],
    [port("out", "다음", "flow", true)],
    [field("text", "동작 내용", "text", "새 동작")],
  ),
  def(
    "delay",
    "대기",
    "실행",
    [port("in", "실행", "flow", false), port("seconds", "초", "number", false)],
    [port("out", "다음", "flow", true)],
    [
      {
        ...field("seconds", "초 기본값", "number", 1),
        min: 0,
        unit: "seconds",
      },
    ],
  ),
  def(
    "note",
    "메모",
    "정리",
    [],
    [],
    [field("text", "메모", "text", "설명을 입력하세요")],
  ),
  ...(
    [
      ["number", "숫자"],
      ["boolean", "조건"],
      ["text", "문자열"],
      ["flow", "실행"],
    ] as const
  ).map(([type, label]) =>
    def(
      `reroute-${type}`,
      `${label} 경유`,
      "정리",
      [{ ...port("in", "입력", type, false), required: true }],
      [port("out", "출력", type, true)],
      [],
    ),
  ),
  ...skillCatalog,
]);
const definitions = new Map(catalog.map((d) => [d.kind, d]));
export function definition(kind: string): Definition {
  const found = definitions.get(kind);
  if (!found)
    throw new GraphError("unknown-node", `지원하지 않는 노드 종류: ${kind}`);
  return found;
}
export function makeNode(kind: string, position: Point): GraphNode {
  return {
    id: id(),
    kind,
    title: "",
    position: { ...position },
    values: Object.fromEntries(
      definition(kind).fields.map((f) => [f.id, f.default]),
    ),
    note: "",
    frame: null,
  };
}
