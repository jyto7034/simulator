import { object, type Json, type Path } from "./source";

export type DefinitionView = {
  id: string;
  name: string;
  group: string;
  path: Path;
  value: Json;
};
export type ReferenceView = {
  from: string;
  to: string;
  path: Path;
  kind: string;
};
export function projection(value: Json) {
  const definitions: DefinitionView[] = [];
  if (!object(value)) return { definitions, references: [] as ReferenceView[] };
  for (const [key, group] of [
    ["resources", "자원"],
    ["values", "공유 값"],
    ["definitions", "공통 동작"],
    ["predicates", "공통 조건"],
    ["skills", "스킬"],
    ["equipment_attacks", "장비 공격"],
    ["equipment_defenses", "장비 방어"],
    ["equipment_supports", "장비 지원"],
    ["equipment_states", "장비 상태"],
  ]) {
    const items = value[key];
    if (!Array.isArray(items)) continue;
    items.forEach((item, index) => {
      const identity = object(item)
        ? (item.name ?? item.id ?? item.equipment)
        : undefined;
      definitions.push({
        id: `${key}:${index}`,
        name: typeof identity === "string" ? identity : `미완성 ${index + 1}`,
        group,
        path: [key, index],
        value: item,
      });
    });
  }
  const references: ReferenceView[] = [];
  function walk(value: Json, path: Path, from: string) {
    if (Array.isArray(value))
      value.forEach((child, index) => walk(child, [...path, index], from));
    else if (object(value)) {
      if (
        (path.at(-1) === "bounty" && path.at(-2) === "cohort") ||
        path.at(-1) === "victory_forms"
      ) {
        for (const field of ["pending", "confirmed"]) {
          const target = value[field];
          const to = definitions.find(
            (item) => item.group === "자원" && item.name === target,
          );
          if (typeof target === "string" && to)
            references.push({
              from,
              to: to.id,
              path: [...path, field],
              kind: "Resource",
            });
        }
      }
      for (const [kind, target, group] of [
        ["Shared", value.Shared, "공유 값"],
        ["Resource", value.resource, "자원"],
        [
          "Predicate",
          object(value.Predicate) ? value.Predicate.name : null,
          "공통 조건",
        ],
        ["Predicate", object(value.Call) ? value.Call.name : null, "공통 조건"],
        [
          "Call",
          object(value.Call) ? value.Call.definition : null,
          "공통 동작",
        ],
      ]) {
        if (typeof target !== "string") continue;
        const to = definitions.find(
          (item) => item.group === group && item.name === target,
        );
        if (to) references.push({ from, to: to.id, path, kind: String(kind) });
      }
      for (const [key, child] of Object.entries(value))
        walk(child, [...path, key], from);
    }
  }
  definitions.forEach((item) => walk(item.value, item.path, item.id));
  return { definitions, references };
}
export function leaves(
  value: Json,
  path: Path,
): { path: Path; value: null | boolean | number | string }[] {
  if (Array.isArray(value))
    return value.flatMap((item, index) => leaves(item, [...path, index]));
  if (object(value))
    return Object.entries(value).flatMap(([key, item]) =>
      leaves(item, [...path, key]),
    );
  return [{ path, value }];
}
