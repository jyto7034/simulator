import { useMemo, useRef, useState, useSyncExternalStore } from "react";
import {
  ReactFlow,
  Background,
  Controls,
  Handle,
  Position,
  useViewport,
  BaseEdge,
  getBezierPath,
  type NodeProps,
  type EdgeProps,
  type Node,
  type Edge,
} from "@xyflow/react";
import { Button } from "../components/ui/button";
import { useUiStore } from "../stores/ui-store";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "../components/ui/dialog";
import { contentFile, listContent } from "../content/files";
import { contentName } from "../content/groups";
import { compileComposition } from "./client";
import { CompositionTrial } from "./CompositionTrial";
import { CompositionSession } from "./session";
import { modelLocation, type Span } from "./source";
import { leaves, projection } from "./view";
import example from "../../../core_enhanced/tests/fixtures/skill_composition/woodsman_monk.skill.yaml?raw";
import "./composition.css";

function DefinitionNode({ data }: NodeProps<Node<{ label: string }>>) {
  const { zoom } = useViewport();
  return (
    <>
      <Handle type="target" position={Position.Left} />
      <span title={data.label} style={{ fontSize: 12 / Math.min(1, zoom) }}>
        {data.label}
      </span>
      <Handle type="source" position={Position.Right} />
    </>
  );
}
function ReferenceEdge(props: EdgeProps) {
  const { zoom } = useViewport();
  const [path, labelX, labelY] = getBezierPath(props);
  return (
    <BaseEdge
      id={props.id}
      path={path}
      label={zoom < 0.9 ? undefined : props.label}
      labelX={labelX}
      labelY={labelY}
      labelStyle={{ fontSize: 12 / zoom }}
    />
  );
}
const nodeTypes = { definition: DefinitionNode };
const edgeTypes = { reference: ReferenceEdge };
const fitViewOptions = {
  padding: { top: "12px", right: "18px", bottom: "44px", left: "18px" },
} as const;

export function CompositionEditor({ active }: { active: boolean }) {
  const theme = useUiStore((store) => store.theme);
  const [session] = useState(() => new CompositionSession(example));
  const state = useSyncExternalStore(session.subscribe, session.getSnapshot);
  const [selected, setSelected] = useState("skills:1");
  const [name, setName] = useState("나무꾼과 풍운승려.skill.yaml");
  const [files, setFiles] = useState<string[]>([]);
  const [opening, setOpening] = useState(false);
  const [fileBusy, setFileBusy] = useState(false);
  const [discard, setDiscard] = useState(false);
  const text = useRef<HTMLTextAreaElement>(null);
  const openButton = useRef<HTMLButtonElement>(null);
  const view = useMemo(
    () =>
      state.parsed.ok
        ? projection(state.parsed.value)
        : { definitions: [], references: [] },
    [state.parsed],
  );
  const selection = view.definitions.find((item) => item.id === selected);
  const fields = selection ? leaves(selection.value, selection.path) : [];
  const graph = useMemo(() => {
    const neighbors = new Set([selected]);
    const references = view.references.filter(
      (ref) => ref.from === selected || ref.to === selected,
    );
    references.forEach((ref) => {
      neighbors.add(ref.from);
      neighbors.add(ref.to);
    });
    const rows = new Map<string, number>();
    const nodes: Node[] = view.definitions
      .filter((item) => neighbors.has(item.id))
      .map((item) => {
        const row = rows.get(item.group) ?? 0;
        rows.set(item.group, row + 1);
        return {
          id: item.id,
          type: "definition",
          position: {
            x:
              ["스킬", "공통 동작", "공통 조건", "공유 값", "자원"].indexOf(
                item.group,
              ) * 310,
            y: row * 55,
          },
          data: { label: `${item.group} · ${item.name}` },
          selected: item.id === selected,
          style: { width: 210, height: 40 },
        };
      });
    const connections = new Map<
      string,
      { from: string; to: string; kind: string; count: number }
    >();
    for (const ref of references) {
      const key = JSON.stringify([ref.from, ref.to, ref.kind]);
      const connection = connections.get(key);
      if (connection) connection.count++;
      else connections.set(key, { ...ref, count: 1 });
    }
    const edges: Edge[] = [...connections.values()].map((ref, index) => ({
      id: `ref-${index}`,
      type: "reference",
      source: ref.from,
      target: ref.to,
      label: ref.count > 1 ? `${ref.kind} ×${ref.count}` : ref.kind,
      ariaLabel: `${view.definitions.find((item) => item.id === ref.from)?.name} → ${view.definitions.find((item) => item.id === ref.to)?.name}: ${ref.kind} ${ref.count}곳`,
    }));
    return { nodes, edges };
  }, [view, selected]);
  const attempt = (action: () => void) => {
    try {
      action();
      return true;
    } catch (error) {
      session.message(error instanceof Error ? error.message : String(error));
      return false;
    }
  };
  const locate = (span: Span | null) => {
    if (!span || !text.current) return;
    const source = text.current;
    source.focus();
    source.setSelectionRange(span.start, span.end);
    source.scrollTop = Math.max(0, (span.line - 3) * 19);
    const lineStart = source.value.lastIndexOf("\n", span.start - 1) + 1;
    const prefix = source.value.slice(lineStart, span.start);
    const context = document.createElement("canvas").getContext("2d");
    if (context) {
      const style = getComputedStyle(source);
      context.font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
      // YAML indentation uses spaces; account for tabs in unfinished source too.
      const expanded = prefix.replace(
        /\t/g,
        " ".repeat(Number(style.tabSize) || 4),
      );
      const width = context.measureText(expanded).width;
      source.scrollLeft = Math.max(0, width - source.clientWidth / 2);
    }
  };
  async function save() {
    let ticket;
    try {
      contentName(name, ".skill.yaml");
      ticket = session.beginSave(name);
      const file = await contentFile(
        {
          operation: "save",
          name,
          json: ticket.source,
          expected: ticket.expected,
        },
        "composition",
      );
      if (file.json !== ticket.source || file.name !== ticket.name)
        throw new Error("저장 응답이 요청한 원본과 다릅니다.");
      session.finishSave(ticket, file.revision);
    } catch (error) {
      const reason = error instanceof Error ? error.message : String(error);
      if (ticket) session.failSave(ticket, reason);
      else session.message(reason);
    }
  }
  async function list() {
    setOpening(true);
    setDiscard(false);
    setFileBusy(true);
    try {
      setFiles((await listContent("composition")).files);
    } catch (error) {
      session.message(error instanceof Error ? error.message : String(error));
    } finally {
      setFileBusy(false);
    }
  }
  async function open(fileName: string | null) {
    if (state.dirty && !discard) return;
    const version = session.getSnapshot();
    setFileBusy(true);
    try {
      if (fileName) {
        const file = await contentFile(
          { operation: "read", name: fileName },
          "composition",
        );
        session.replace(
          file.json,
          { name: file.name, revision: file.revision, savedSource: file.json },
          version,
        );
        setName(file.name);
      } else {
        session.replace(example, null, version);
        setName("나무꾼과 풍운승려.skill.yaml");
      }
      setOpening(false);
      setSelected("skills:1");
    } catch (error) {
      session.message(error instanceof Error ? error.message : String(error));
    } finally {
      setFileBusy(false);
    }
  }
  async function compile() {
    const ticket = session.beginCompile();
    try {
      session.finishCompile(ticket, await compileComposition(ticket));
    } catch (error) {
      session.failCompile(
        ticket,
        error instanceof Error ? error.message : String(error),
      );
    }
  }
  return (
    <section className="composition-shell" aria-label="스킬 조합 작성">
      <header className="composition-toolbar">
        <label>
          <span>원본 파일</span>
          <input
            aria-label="조합 파일 이름"
            value={name}
            onChange={(event) => setName(event.target.value)}
          />
        </label>
        <Button
          ref={openButton}
          size="sm"
          variant="outline"
          onClick={() => void list()}
        >
          열기
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={state.saving}
          onClick={() => void save()}
        >
          저장{state.dirty ? " *" : ""}
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={!state.canUndo}
          onClick={() => session.travel(false)}
        >
          실행 취소
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={!state.canRedo}
          onClick={() => session.travel(true)}
        >
          다시 실행
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={state.compilation === "running"}
          onClick={() => void compile()}
        >
          Core 컴파일
        </Button>
      </header>
      <div className="composition-execution-toolbar">
        <CompositionTrial
          currentSource={state.source}
          definition={
            state.compilation === "compiled" ? state.definition : null
          }
        />
      </div>
      <div className="composition-version">
        원본 {state.epoch}:{state.revision} ·{" "}
        {state.compilation === "compiled"
          ? "현재 원본 컴파일 완료"
          : state.compilation === "running"
            ? "검증 중"
            : "실행 검증 필요"}
        {state.definition && state.compilation !== "compiled"
          ? ` · 이전 정의 ${state.definition.ticket.epoch}:${state.definition.ticket.revision} 보존`
          : ""}
      </div>
      <div className="composition-body">
        <div className="composition-source">
          <h2>
            작성 원본 <span>YAML 1.2 · 미완성 내용도 저장 가능</span>
          </h2>
          <textarea
            ref={text}
            aria-label="스킬 조합 원본"
            spellCheck={false}
            value={state.source}
            onChange={(event) =>
              attempt(() => session.edit(event.target.value))
            }
          />
        </div>
        <div className="composition-structure">
          <h2>정의와 공유 관계</h2>
          <div className="composition-graph">
            {active && (
              <ReactFlow
                key={selected}
                colorMode={theme}
                nodeTypes={nodeTypes}
                edgeTypes={edgeTypes}
                nodes={graph.nodes}
                edges={graph.edges}
                nodesDraggable={false}
                nodesConnectable={false}
                elementsSelectable
                fitView
                fitViewOptions={fitViewOptions}
                minZoom={0.2}
                onNodeClick={(_, node) => setSelected(node.id)}
                aria-label="조합 정의 참조 그래프"
              >
                <Background />
                <Controls
                  orientation="horizontal"
                  position="bottom-left"
                  showInteractive={false}
                  fitViewOptions={fitViewOptions}
                  aria-label=""
                />
              </ReactFlow>
            )}
          </div>
          <div
            className="composition-definition-list"
            role="group"
            aria-label="정의 선택"
          >
            {view.definitions.map((item) => (
              <button
                key={item.id}
                aria-pressed={selected === item.id}
                onClick={() => setSelected(item.id)}
              >
                {item.group} · {item.name}
              </button>
            ))}
            {!view.definitions.length && (
              <p>
                {state.parsed.ok
                  ? "정의가 없습니다. 원본에 자원·공유 값·공통 조건·공통 동작·스킬을 작성하세요."
                  : "문법을 수정하면 정의와 참조를 표시합니다."}
              </p>
            )}
          </div>
          <h2>속성 · {selection?.name ?? "정의를 선택하세요"}</h2>
          <div className="composition-properties">
            {selection && (
              <p>
                직접 참조하는 정의{" "}
                {
                  new Set(
                    view.references
                      .filter((ref) => ref.to === selection.id)
                      .map((ref) => ref.from),
                  ).size
                }
                개 · 변경하면 이 원본의 모든 참조에 적용됩니다.
              </p>
            )}
            {fields.map((field) => {
              const key = JSON.stringify(field.path);
              const label = field.path.slice(2).join(".");
              return (
                <label
                  key={`${key}:${String(field.value)}:${state.epoch}`}
                  className="composition-property"
                >
                  <button
                    type="button"
                    onClick={() =>
                      locate(
                        state.parsed.ok
                          ? (state.parsed.spans[key] ?? null)
                          : null,
                      )
                    }
                    title="원본 위치로 이동"
                  >
                    {label}
                  </button>
                  <input
                    aria-label={`${selection?.name} ${label}`}
                    defaultValue={
                      field.value === null ? "null" : String(field.value)
                    }
                    onBlur={(event) => {
                      const raw = event.target.value;
                      const committed = attempt(() => {
                        const value: unknown =
                          typeof field.value === "number"
                            ? raw.trim()
                              ? Number(raw)
                              : NaN
                            : typeof field.value === "boolean"
                              ? raw === "true"
                                ? true
                                : raw === "false"
                                  ? false
                                  : (() => {
                                      throw new Error(
                                        "true 또는 false를 입력하세요.",
                                      );
                                    })()
                              : field.value === null
                                ? JSON.parse(raw)
                                : raw;
                        if (
                          value !== null &&
                          typeof value !== "string" &&
                          typeof value !== "number" &&
                          typeof value !== "boolean"
                        )
                          throw new Error(
                            "구조 변경은 작성 원본에서 편집하세요.",
                          );
                        session.scalar(field.path, value, state);
                      });
                      if (!committed)
                        event.target.value =
                          field.value === null ? "null" : String(field.value);
                    }}
                  />
                </label>
              );
            })}
          </div>
        </div>
      </div>
      <div
        className="composition-diagnostics"
        role="group"
        aria-label="조합 진단"
      >
        {!state.parsed.ok &&
          state.parsed.issues.slice(0, 3).map((issue, index) => (
            <button key={index} onClick={() => locate(issue.span)}>
              {issue.span.line}:{issue.span.column} · {issue.reason}
            </button>
          ))}
        {state.diagnostic && (
          <button
            onClick={() =>
              locate(
                modelLocation(
                  state.parsed,
                  state.diagnostic!.source.definition,
                ),
              )
            }
          >
            {state.diagnostic.code} · {state.diagnostic.reason}
          </button>
        )}
        <span role="status">{state.message}</span>
      </div>
      <Dialog open={opening} onOpenChange={setOpening}>
        <DialogContent
          className="composition-files"
          onCloseAutoFocus={(event) => {
            event.preventDefault();
            openButton.current?.focus();
          }}
        >
          <DialogHeader>
            <DialogTitle>조합 원본 열기</DialogTitle>
            <DialogDescription>
              저장한 YAML 원본과 검증 예제를 엽니다.
            </DialogDescription>
          </DialogHeader>
          {state.dirty && (
            <label>
              <input
                type="checkbox"
                checked={discard}
                onChange={(event) => setDiscard(event.target.checked)}
              />{" "}
              미저장 변경을 버리고 열기
            </label>
          )}
          {fileBusy ? (
            <p role="status">파일 목록을 읽는 중…</p>
          ) : (
            <div className="composition-file-list">
              {files.map((file) => (
                <Button
                  key={file}
                  variant="outline"
                  disabled={state.dirty && !discard}
                  onClick={() => void open(file)}
                >
                  {file}
                </Button>
              ))}
              {!files.length && <p>저장된 조합 원본이 없습니다.</p>}
              <Button
                variant="outline"
                disabled={state.dirty && !discard}
                onClick={() => void open(null)}
              >
                E01/E09 검증 예제 열기
              </Button>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </section>
  );
}
