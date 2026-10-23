import { useRef } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { moveToGroup, type GraphGroups } from "@/content/groups";

export function ContentGroupEditor({
  document,
  files,
  disabled,
  onChange,
}: {
  document: GraphGroups;
  files: string[];
  disabled: boolean;
  onChange(document: GraphGroups): void;
}) {
  const addButton = useRef<HTMLButtonElement>(null);
  return (
    <div role="group" className="ng-group-editor" aria-label="그룹 편집 내용">
      <p className="ng-content-hint">
        이름과 소속을 바꾼 뒤 적용하세요. 그룹 해제 시 그래프는 미분류로
        이동합니다.
      </p>
      <Button
        ref={addButton}
        variant="outline"
        disabled={disabled || document.groups.length >= 200}
        onClick={() => {
          let name = "새 그룹",
            index = 2;
          while (document.groups.some((group) => group.name === name))
            name = `새 그룹 ${index++}`;
          onChange({
            ...document,
            groups: [
              ...document.groups,
              { id: crypto.randomUUID(), name, files: [] },
            ],
          });
        }}
      >
        그룹 추가
      </Button>
      <div className="ng-group-edit-scroll">
        {document.groups.map((group, index) => (
          <div className="ng-group-name-row" key={group.id}>
            <Input
              aria-label={`그룹 이름 ${index + 1}`}
              value={group.name}
              disabled={disabled}
              maxLength={80}
              onChange={(event) =>
                onChange({
                  ...document,
                  groups: document.groups.map((item) =>
                    item.id === group.id
                      ? { ...item, name: event.target.value }
                      : item,
                  ),
                })
              }
            />
            <Button
              variant="outline"
              disabled={disabled}
              aria-label={`${group.name || "이름 없는 그룹"} 그룹 해제`}
              onClick={() => {
                onChange({
                  ...document,
                  groups: document.groups.filter(
                    (item) => item.id !== group.id,
                  ),
                });
                addButton.current?.focus();
              }}
            >
              그룹 해제
            </Button>
          </div>
        ))}
        <div
          role="group"
          className="ng-group-assignments"
          aria-label="그래프 소속"
        >
          {files.map((file) => (
            <label className="ng-group-assignment" key={file}>
              <span>{file}</span>
              <select
                aria-label={`${file} 소속 그룹`}
                title={
                  document.groups.find((group) => group.files.includes(file))
                    ?.name ?? "미분류"
                }
                disabled={disabled}
                value={
                  document.groups.find((group) => group.files.includes(file))
                    ?.id ?? ""
                }
                onChange={(event) =>
                  onChange(moveToGroup(document, file, event.target.value))
                }
              >
                <option value="">미분류</option>
                {document.groups.map((group) => (
                  <option key={group.id} value={group.id}>
                    {group.name || "이름 없는 그룹"}
                  </option>
                ))}
              </select>
            </label>
          ))}
          {!files.length && (
            <p>저장된 그래프가 없습니다. 빈 그룹을 미리 만들 수 있습니다.</p>
          )}
        </div>
      </div>
    </div>
  );
}

export function ContentGroupList({
  document,
  files,
  disabled,
  onOpen,
}: {
  document: GraphGroups;
  files: string[];
  disabled: boolean;
  onOpen(file: string): void;
}) {
  const assigned = new Set(document.groups.flatMap((group) => group.files));
  const ungrouped = files.filter((file) => !assigned.has(file));
  const buttons = (names: string[]) =>
    names.map((file) => (
      <button
        className="ng-content-file"
        key={file}
        disabled={disabled}
        onClick={() => onOpen(file)}
      >
        {file}
      </button>
    ));
  return (
    <>
      {document.groups.map((group) => {
        const members = files.filter((file) => group.files.includes(file));
        return (
          <details className="ng-content-group" key={group.id}>
            <summary aria-label={`${group.name} · ${members.length}개 그래프`}>
              <span>{group.name}</span>
              <small>{members.length}개</small>
            </summary>
            <div className="ng-content-group-files">
              {members.length ? (
                buttons(members)
              ) : (
                <p>이 그룹에는 그래프가 없습니다.</p>
              )}
            </div>
          </details>
        );
      })}
      {ungrouped.length > 0 && (
        <div className="ng-content-ungrouped">
          {document.groups.length > 0 && (
            <p className="ng-content-hint">미분류 · {ungrouped.length}</p>
          )}
          {buttons(ungrouped)}
        </div>
      )}
    </>
  );
}
