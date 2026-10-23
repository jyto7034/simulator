import type { Field, FieldValue } from "../types";

// The canvas and inspector use the same parsing rules; the document validates ranges.
export function parseFieldValue(
  field: Field,
  value: string | boolean,
): FieldValue {
  if (field.valueType !== "number" && field.valueType !== "optional-number")
    return value;
  if (
    field.valueType === "optional-number" &&
    typeof value === "string" &&
    !value.trim()
  )
    return "";
  if (
    typeof value !== "string" ||
    !value.trim() ||
    !Number.isFinite(Number(value))
  )
    throw new Error(`${field.label}에는 유한한 숫자를 입력하세요.`);
  return Number(value);
}
