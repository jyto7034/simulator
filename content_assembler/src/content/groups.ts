export type GraphGroup = { id: string; name: string; files: string[] };
export type GraphGroups = {
  format: "graph-groups";
  version: 1;
  groups: GraphGroup[];
};
export type GroupSnapshot = { document: GraphGroups; revision: string | null };
export const emptyGroups = (): GraphGroups => ({
  format: "graph-groups",
  version: 1,
  groups: [],
});
export const groupsFilename = ".graph-groups.json";

export function contentName(
  value: unknown,
  suffix: ".node-graph.json" | ".skill.yaml" = ".node-graph.json",
): string {
  if (
    typeof value !== "string" ||
    !value.endsWith(suffix) ||
    value.length <= suffix.length ||
    new TextEncoder().encode(value).length > 180 ||
    value.startsWith(".") ||
    /[<>:"/\\|?*]/.test(value) ||
    [...value].some((c) => c.charCodeAt(0) < 32) ||
    /^(con|prn|aux|nul|com[0-9]|lpt[0-9])\./i.test(value)
  )
    throw new Error(
      `파일 이름을 확인하세요. 폴더 경로 없이 ${suffix} 이름을 사용하세요.`,
    );
  return value;
}

export function parseGroups(value: unknown): GraphGroups {
  const invalid = () =>
    new Error(
      "그룹 정보가 올바르지 않습니다. 이름 중복·빈 이름·파일 중복을 확인하세요.",
    );
  if (
    !value ||
    typeof value !== "object" ||
    !("format" in value) ||
    value.format !== "graph-groups" ||
    !("version" in value) ||
    value.version !== 1 ||
    !("groups" in value) ||
    !Array.isArray(value.groups) ||
    value.groups.length > 200
  )
    throw invalid();
  const ids = new Set<string>(),
    names = new Set<string>(),
    files = new Set<string>();
  const groups = value.groups.map((entry: unknown) => {
    if (
      !entry ||
      typeof entry !== "object" ||
      !("id" in entry) ||
      typeof entry.id !== "string" ||
      !/^[a-zA-Z0-9_-]{1,80}$/.test(entry.id) ||
      ids.has(entry.id) ||
      !("name" in entry) ||
      typeof entry.name !== "string" ||
      !entry.name.trim() ||
      entry.name !== entry.name.trim() ||
      [...entry.name].length > 80 ||
      [...entry.name].some((c) => c.charCodeAt(0) < 32) ||
      names.has(entry.name) ||
      !("files" in entry) ||
      !Array.isArray(entry.files)
    )
      throw invalid();
    ids.add(entry.id);
    names.add(entry.name);
    return {
      id: entry.id,
      name: entry.name,
      files: entry.files.map((file: unknown) => {
        const name = contentName(file);
        if (files.has(name)) throw invalid();
        files.add(name);
        return name;
      }),
    };
  });
  return { format: "graph-groups", version: 1, groups };
}

export function moveToGroup(
  document: GraphGroups,
  file: string,
  groupId: string,
): GraphGroups {
  if (groupId && !document.groups.some((group) => group.id === groupId))
    throw new Error("그룹을 찾을 수 없습니다.");
  return {
    ...document,
    groups: document.groups.map((group) => ({
      ...group,
      files: [
        ...group.files.filter((name) => name !== file),
        ...(group.id === groupId ? [file] : []),
      ],
    })),
  };
}
