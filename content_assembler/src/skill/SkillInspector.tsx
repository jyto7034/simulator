import { ArrowDown, ArrowUp, Copy, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Field, Choice, Reference } from "../editor/fields";
import {
  blockDefinitions,
  eventNames,
  scopeNames,
  findBlock,
  selectedRoutine,
  isContainer,
  type Selection,
  type BlockEditAction,
  type SkillRoutine,
  type SkillState,
} from "./model";
import type { SkillEditorState, SkillSession } from "./session";
export function SkillInspector({
  state,
  session,
  remove,
  add,
  edit,
}: {
  state: SkillEditorState;
  session: SkillSession;
  remove: () => void;
  add: (selection: Selection) => void;
  edit: (selection: Selection, action: BlockEditAction) => void;
}) {
  const { selection, doc } = state;
  const routine = selectedRoutine(doc, selection);
  const found =
    selection.type === "block" && routine
      ? findBlock(routine.blocks, selection.id)
      : undefined;
  const variable =
    selection.type === "state"
      ? doc.states.find((s) => s.id === selection.id)
      : undefined;
  const changeDoc = (key: "name" | "description", value: string) =>
    session.commit((d) => {
      d[key] = value;
    });
  const changeRoutine = (key: "name" | "condition" | "event", value: string) =>
    session.commit((d) => {
      const r = d.routines.find((r) => r.id === routine?.id);
      if (r) {
        if (key === "event") r.event = value as SkillRoutine["event"];
        else r[key] = value;
      }
    });
  const changeState = (
    key: "name" | "scope" | "initial" | "lifetime",
    value: string,
  ) =>
    session.commit((d) => {
      const s = d.states.find((s) => s.id === variable?.id);
      if (s) {
        if (key === "scope") s.scope = value as SkillState["scope"];
        else s[key] = value;
      }
    });
  const editBlock = (edit: (b: NonNullable<typeof found>["block"]) => void) =>
    session.commit((d) => {
      const r = selectedRoutine(d, selection);
      if (r && selection.type === "block") {
        const b = findBlock(r.blocks, selection.id);
        if (b) edit(b.block);
      }
    });
  return (
    <aside className="skill-inspector scroll-panel" aria-label="스킬 속성">
      <div className="panel-heading">
        <h2>속성</h2>
        <span className="skill-muted">선택한 구성 요소</span>
      </div>
      <div
        className="properties"
        key={`${state.epoch}-${state.revision}-${JSON.stringify(selection)}`}
      >
        {selection.type === "document" && (
          <>
            <h3>스킬 기본 설정</h3>
            <Field
              label="스킬 이름"
              value={doc.name}
              onCommit={(v) => changeDoc("name", v)}
            />
            <Field
              label="스킬 설명"
              value={doc.description}
              onCommit={(v) => changeDoc("description", v)}
            />
            <p className="hint">
              설계 문서입니다. 값과 수식은 작성 의도를 기록하며 아직 실행하거나
              계산하지 않습니다.
            </p>
          </>
        )}
        {variable && (
          <>
            <h3>상태 선언</h3>
            <Field
              label="상태 이름"
              value={variable.name}
              onCommit={(v) => changeState("name", v)}
            />
            <Choice
              label="상태가 붙을 대상"
              value={variable.scope}
              onChange={(v) => changeState("scope", v)}
            >
              {Object.entries(scopeNames).map(([k, v]) => (
                <option value={k} key={k}>
                  {v}
                </option>
              ))}
            </Choice>
            <Field
              label="초기값"
              value={variable.initial}
              onCommit={(v) => changeState("initial", v)}
            />
            <Field
              label="수명·종료 규칙"
              value={variable.lifetime}
              onCommit={(v) => changeState("lifetime", v)}
            />
            <p className="hint">
              상태의 대상과 발생 출처는 별개입니다. 여러 사본의 합산·소비 정책은
              설명과 동작에서 명시하세요.
            </p>
          </>
        )}
        {selection.type === "routine" && routine && (
          <>
            <h3>시작 사건</h3>
            <Field
              label="동작 이름"
              value={routine.name}
              onCommit={(v) => changeRoutine("name", v)}
            />
            <Choice
              label="시작 사건"
              value={routine.event}
              onChange={(v) => changeRoutine("event", v)}
            >
              {Object.entries(eventNames).map(([k, v]) => (
                <option value={k} key={k}>
                  {v}
                </option>
              ))}
            </Choice>
            <Field
              label="시작 조건·출처"
              value={routine.condition}
              onCommit={(v) => changeRoutine("condition", v)}
            />
            <Button size="sm" variant="outline" onClick={() => add(selection)}>
              마지막에 블록 추가
            </Button>
          </>
        )}
        {found && (
          <>
            <h3>{blockDefinitions[found.block.kind].label}</h3>
            <div className="skill-block-actions">
              <Button
                size="icon-sm"
                variant="outline"
                aria-label="블록 위로 이동"
                disabled={found.index === 0}
                onClick={() => edit(selection, "move-up")}
              >
                <ArrowUp />
              </Button>
              <Button
                size="icon-sm"
                variant="outline"
                aria-label="블록 아래로 이동"
                disabled={found.index === found.siblings.length - 1}
                onClick={() => edit(selection, "move-down")}
              >
                <ArrowDown />
              </Button>
              <Button
                size="icon-sm"
                variant="outline"
                aria-label="블록 복제"
                onClick={() => edit(selection, "duplicate")}
              >
                <Copy />
              </Button>
            </div>
            {Object.entries(blockDefinitions[found.block.kind].fields).map(
              ([key, label]) => (
                <div key={key} data-field={key}>
                  {["state", "destination"].includes(key) ? (
                    <Reference
                      label={label}
                      value={found.block.fields[key]}
                      options={doc.states}
                      onChange={(v) =>
                        editBlock((b) => {
                          b.fields[key] = v;
                        })
                      }
                    />
                  ) : (
                    <Field
                      label={label}
                      value={found.block.fields[key]}
                      onCommit={(v) =>
                        editBlock((b) => {
                          b.fields[key] = v;
                        })
                      }
                    />
                  )}
                </div>
              ),
            )}
            <Field
              label="설계 메모"
              value={found.block.note}
              onCommit={(v) =>
                editBlock((b) => {
                  b.note = v;
                })
              }
            />
            <div className="skill-insert-actions">
              <Button
                size="sm"
                variant="outline"
                onClick={() => add(selection)}
              >
                뒤에 블록 추가
              </Button>
              {isContainer(found.block.kind) && routine && (
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() =>
                    add({
                      type: "slot",
                      parentId: found.block.id,
                      routineId: routine.id,
                      lane: "children",
                    })
                  }
                >
                  안쪽 절차에 추가
                </Button>
              )}
              {found.block.kind === "branch" && routine && (
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() =>
                    add({
                      type: "slot",
                      parentId: found.block.id,
                      routineId: routine.id,
                      lane: "otherwise",
                    })
                  }
                >
                  아니면 절차에 추가
                </Button>
              )}
            </div>
          </>
        )}
        {selection.type === "slot" && (
          <>
            <h3>블록 추가 위치</h3>
            <p className="hint">
              {selection.lane === "otherwise" ? "아니면" : "안쪽"} 절차를
              선택했습니다. 블록 추가 버튼으로 동작을 고르세요.
            </p>
          </>
        )}
        {["state", "routine", "block"].includes(selection.type) && (
          <Button
            className="skill-delete"
            size="sm"
            variant="outline"
            onClick={remove}
          >
            <Trash2 size={14} />
            선택 항목 삭제
          </Button>
        )}
      </div>
    </aside>
  );
}
