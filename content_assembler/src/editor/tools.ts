import type { Paint } from "./operations";
export type Tool =
  Paint | "select" | "hand" | "route" | "point" | "deploy" | "spawn";
export const toolDefinitions: {
  id: Tool;
  label: string;
  key: string;
  glyph: string;
  radialOnly?: boolean;
}[] = [
  { id: "select", label: "선택 / 점 이동", key: "V", glyph: "↖" },
  { id: "hand", label: "화면 이동", key: "H", glyph: "✥" },
  { id: "add", label: "유효 영역 추가", key: "A", glyph: "▧" },
  { id: "remove", label: "유효 영역 제거", key: "X", glyph: "▱" },
  { id: "floor", label: "일반 바닥", key: "B", glyph: "□" },
  { id: "platform", label: "고지대 / 플랫폼", key: "P", glyph: "▤" },
  { id: "obstacle", label: "장애물", key: "O", glyph: "▨" },
  { id: "erase", label: "지형 지우기", key: "E", glyph: "◇" },
  { id: "zone-add", label: "구역 칠하기", key: "Z", glyph: "▦" },
  { id: "zone-remove", label: "구역 셀 지우기", key: "C", glyph: "⊟" },
  { id: "route", label: "경로 점 추가", key: "R", glyph: "↗" },
  { id: "point", label: "이름 있는 지점", key: "T", glyph: "⊙" },
  { id: "deploy", label: "배치 구역", key: "", glyph: "▦", radialOnly: true },
  { id: "spawn", label: "등장 구역", key: "", glyph: "▦", radialOnly: true },
];

// Clockwise from twelve o'clock; keep directions stable as the document changes.
export const radialTools = [
  { id: "select", label: "선택 / 이동" },
  { id: "floor", label: "일반 바닥" },
  { id: "platform", label: "고지대" },
  { id: "obstacle", label: "장애물" },
  { id: "deploy", label: "배치 구역" },
  { id: "spawn", label: "등장 구역" },
  { id: "route", label: "경로" },
  { id: "erase", label: "지형 지우기" },
] as const satisfies ReadonlyArray<{ id: Tool; label: string }>;
