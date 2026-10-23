import type { Definition, Field, Port } from "../types";

export const input = (id: string, label: string, valueType: string): Port => ({
  id,
  label,
  valueType,
  multiple: valueType === "flow",
  required: true,
});
export const output = (id: string, label: string, valueType: string): Port => ({
  id,
  label,
  valueType,
  multiple: valueType !== "flow",
  required: false,
});
export const number = (id: string, label: string, min = 0): Field => ({
  id,
  label,
  valueType: "optional-number",
  default: "",
  min,
});
export const choice = (
  id: string,
  label: string,
  options: [string, string][],
  value = "unresolved",
): Field => ({
  id,
  label,
  valueType: "text",
  default: value,
  options: options.map(([value, label]) => ({ value, label })),
});
export const unresolved: [string, string] = ["unresolved", "미지정"];
const roles: Partial<Record<string, string>> = {
  "스킬 상태": "상태 선언",
  "스킬 사건": "▶ 시작 사건",
  "스킬 동작": "실행 동작",
  "스킬 조건": "조건 확인",
  "스킬 값": "설정값",
  연출: "연출",
};
export function define(
  kind: string,
  label: string,
  category: string,
  description: string,
  inputs: Port[],
  outputs: Port[],
  fields: Field[] = [],
): Definition {
  const role = kind === "time-power" ? "값 계산" : roles[category];
  return {
    kind: `skill-${kind}`,
    label,
    category,
    role,
    description,
    inputs,
    outputs,
    fields,
    width: 340,
    height:
      Math.max(
        86,
        62 + Math.max(inputs.length, outputs.length) * 44 + fields.length * 36,
      ) + (role ? 24 : 0),
  };
}
export const state = () => input("state", "상태 참조", "skill-state");
export const activation = () =>
  input("activation", "이번 활성", "skill-activation");
export const incoming = () => input("in", "실행", "flow");
export const next = () => output("out", "다음", "flow");
export const cue = (label: string) =>
  output("cue", label, "presentation-event");

export const optionalInput = (
  id: string,
  label: string,
  type: string,
): Port => ({
  ...input(id, label, type),
  required: false,
});
export const multipleInput = (
  id: string,
  label: string,
  type: string,
  required = false,
): Port => ({
  ...input(id, label, type),
  multiple: true,
  required,
});
export const integer = (id: string, label: string, min = 0): Field => ({
  ...number(id, label, min),
  integer: true,
});
