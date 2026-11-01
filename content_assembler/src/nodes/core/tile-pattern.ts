import type { Field, FieldValue } from "../types";
import { GraphError } from "./model";

export type TileOffset = { x: number; y: number };

// Rows face up. @ is the origin; its inclusion is a separate, explicit value.
// Empty text is an unfinished authoring value, not an empty runtime footprint.
export function readTilePattern(value: string) {
  if (value === "") return { cells: [] as TileOffset[], assigned: false };
  const rows = value.split("\n");
  const width = rows[0].length;
  if (
    !width ||
    rows.some((row) => row.length !== width || /[^X.@]/.test(row)) ||
    [...value].filter((c) => c === "@").length !== 1
  )
    throw new GraphError(
      "field-tile-pattern",
      "타일 패턴은 같은 너비의 행, 기준점 @ 하나, X와 .으로 구성해야 합니다.",
    );
  const ay = rows.findIndex((row) => row.includes("@"));
  const ax = rows[ay].indexOf("@");
  const cells: TileOffset[] = [];
  rows.forEach((row, y) =>
    [...row].forEach((c, x) => {
      if (c === "X") cells.push({ x: x - ax, y: y - ay });
    }),
  );
  return { cells, assigned: true };
}

export function togglePatternTile(value: string, tile: TileOffset) {
  if (
    ![tile.x, tile.y].every(
      (v) => Number.isSafeInteger(v) && Math.abs(v) < 16_384,
    )
  )
    throw new GraphError(
      "field-tile-pattern",
      "편집 가능한 타일 좌표가 아닙니다.",
    );
  if (tile.x === 0 && tile.y === 0) return value || "@";
  const { cells } = readTilePattern(value);
  const exists = cells.some((p) => p.x === tile.x && p.y === tile.y);
  const next = exists
    ? cells.filter((p) => p.x !== tile.x || p.y !== tile.y)
    : [...cells, tile];
  const xmin = Math.min(0, ...next.map((p) => p.x));
  const xmax = Math.max(0, ...next.map((p) => p.x));
  const ymin = Math.min(0, ...next.map((p) => p.y));
  const ymax = Math.max(0, ...next.map((p) => p.y));
  // The existing field text limit also bounds the interactive editor.
  if ((xmax - xmin + 2) * (ymax - ymin + 1) - 1 > 16_384)
    throw new GraphError(
      "text-limit",
      "타일 패턴의 편집 크기 제한을 넘었습니다.",
    );
  const selected = new Set(next.map((p) => `${p.x},${p.y}`));
  return Array.from({ length: ymax - ymin + 1 }, (_, y) =>
    Array.from({ length: xmax - xmin + 1 }, (_, x) =>
      x + xmin === 0 && y + ymin === 0
        ? "@"
        : selected.has(`${x + xmin},${y + ymin}`)
          ? "X"
          : ".",
    ).join(""),
  ).join("\n");
}

// Read-only support for the exact former schema. Never rasterize it or discard
// it on open/save. An explicit tile-pattern edit replaces it in one transaction.
export const legacyAreaFields: Field[] = [
  {
    id: "shape",
    label: "이전 범위 형상",
    valueType: "text",
    default: "unresolved",
    options: ["unresolved", "sector", "rectangle"].map((value) => ({
      value,
      label: value,
    })),
  },
  {
    id: "facing",
    label: "방향 고정",
    valueType: "text",
    default: "unresolved",
    options: ["unresolved", "accepted", "release"].map((value) => ({
      value,
      label: value,
    })),
  },
  ...["length", "width", "angle"].map((id): Field => ({
    id,
    label: id,
    valueType: "optional-number",
    default: "",
    min: 0,
    ...(id === "angle" ? { max: 360 } : {}),
  })),
];

export function isLegacyArea(node: { kind: string; values: object }) {
  return node.kind === "skill-area" && Object.hasOwn(node.values, "shape");
}

export function areaEditorValues(node: {
  kind: string;
  values: Record<string, FieldValue>;
}) {
  return isLegacyArea(node)
    ? { pattern: "", includeAnchor: false, facing: node.values.facing }
    : node.values;
}
