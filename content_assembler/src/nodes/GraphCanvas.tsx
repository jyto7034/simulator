import { cardPresentation } from "./core/card-presentation";
import { privateConstants } from "./core/constant-presentation";
import { inputSources } from "./core/connections";
import { p4FieldActive } from "./core/p4-fields";
import { spatialFieldActive } from "./core/spatial-catalog";
import { groundFieldActive } from "./core/ground-catalog";
import {
  memo,
  useCallback,
  useEffect,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
  type Dispatch,
  type Ref,
  type SetStateAction,
} from "react";
import {
  Background,
  BackgroundVariant,
  Controls,
  Handle,
  Position,
  ReactFlow,
  ReactFlowProvider,
  SelectionMode,
  useReactFlow,
  useStore,
  useUpdateNodeInternals,
  type Edge,
  type Node,
  type NodeProps,
  type Connection,
  type NodeChange,
} from "@xyflow/react";
import { ContextMenu } from "radix-ui";
import type {
  Anchor,
  Command,
  Definition,
  Field,
  GraphNode,
  Point,
  Snapshot,
  Version,
} from "./types";
import "@xyflow/react/dist/style.css";
import { resonanceFieldActive } from "./core/resonance-catalog";

export type CanvasApi = {
  center(): Point;
  fit(): void;
  focus(id: string): void;
  cancel(): void;
};
export type MenuAction = "duplicate" | "copy" | "group" | "ungroup" | "delete";
type NodeData = {
  model?: GraphNode;
  definition?: Definition;
  title: string;
  issues: number;
  optionalCount?: number;
  expanded?: boolean;
  expand?(): void;
  sources?: Record<string, { node: GraphNode; field?: Field; shared: boolean }>;
  field(node: GraphNode, field: Field): void;
  navigate(id: string): void;
  select(): void;
  port(anchor: Anchor): void;
  menu(action: MenuAction): void;
  busy: boolean;
  [key: string]: unknown;
};
type CanvasNode = Node<NodeData>;
const GraphCard = memo(function GraphCard({
  data,
  id,
  selected,
}: NodeProps<CanvasNode>) {
  const trigger = useRef<HTMLDivElement>(null);
  const definition = data.definition;
  const updateInternals = useUpdateNodeInternals();
  useEffect(() => {
    updateInternals(id);
  }, [id, definition, updateInternals]);
  const isFrame = !definition;
  const label = data.title;
  return (
    <ContextMenu.Root
      modal={false}
      onOpenChange={(open) => {
        if (open && !data.busy) {
          data.select();
          trigger.current?.focus();
        }
      }}
    >
      <ContextMenu.Trigger
        asChild
        disabled={data.busy}
        onContextMenu={(e) => e.stopPropagation()}
      >
        <div
          ref={trigger}
          tabIndex={0}
          role="group"
          className={isFrame ? "ng-frame-title" : "ng-card"}
          data-selected={selected}
          data-category={definition?.category}
          data-role={definition?.role}
          data-entry={definition?.category === "스킬 사건" || undefined}
          aria-label={`${isFrame ? "프레임" : "노드"} ${label}`}
          onKeyDown={(e) => {
            if (
              e.target === e.currentTarget &&
              (e.key === "Enter" || e.key === " ") &&
              !data.busy
            ) {
              e.preventDefault();
              data.select();
            }
            if (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10")) {
              e.preventDefault();
              e.stopPropagation();
              const r = e.currentTarget.getBoundingClientRect();
              e.currentTarget.dispatchEvent(
                new MouseEvent("contextmenu", {
                  bubbles: true,
                  cancelable: true,
                  clientX: r.left + 20,
                  clientY: r.top + 20,
                }),
              );
            }
          }}
        >
          {definition?.role && (
            <div className="ng-role-label">{definition.role}</div>
          )}
          <div className="ng-drag-handle" title={label}>
            <span>{label}</span>
            {data.issues > 0 && <small title="작성 진단">{data.issues}</small>}
          </div>
          {definition && (
            <>
              <div className="ng-ports">
                {Array.from(
                  {
                    length: Math.max(
                      definition.inputs.length,
                      definition.outputs.length,
                    ),
                  },
                  (_, index) => (
                    <div className="ng-port-row" key={index}>
                      {([false, true] as const).map((output) => {
                        const p = (
                          output ? definition.outputs : definition.inputs
                        )[index];
                        return (
                          <div
                            className={output ? "ng-output" : "ng-input"}
                            key={String(output)}
                          >
                            {p && (
                              <>
                                <Handle
                                  type={output ? "source" : "target"}
                                  position={
                                    output ? Position.Right : Position.Left
                                  }
                                  id={p.id}
                                  className={`ng-handle ng-type-${p.valueType}`}
                                  isConnectable={!data.busy}
                                  role="img"
                                  aria-label={`${label} ${p.label} ${output ? "출력" : "입력"}`}
                                />
                                <button
                                  className="nodrag nopan"
                                  disabled={data.busy}
                                  title={`${p.label} · ${p.valueType} · ${output ? "출력" : "입력"}`}
                                  aria-label={`${label} ${p.label} ${output ? "출력" : "입력"} 연결`}
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    data.port({
                                      endpoint: { node: id, port: p.id },
                                      output,
                                    });
                                  }}
                                >
                                  {p.label}
                                </button>
                                {!output &&
                                  data.sources?.[p.id] &&
                                  (() => {
                                    const source = data.sources[p.id];
                                    const value = source.field
                                      ? source.node.values[source.field.id]
                                      : undefined;
                                    return (
                                      <button
                                        className="ng-input-source nodrag nopan"
                                        disabled={data.busy}
                                        title={`${source.shared ? "공유" : "연결"} · ${source.node.title} · ${source.field ? "값 수정" : "노드로 이동"}`}
                                        aria-label={`${label} ${p.label} 연결값 ${source.field ? "수정" : "이동"}`}
                                        onClick={(e) => {
                                          e.stopPropagation();
                                          if (source.field)
                                            data.field(
                                              source.node,
                                              source.field,
                                            );
                                          else data.navigate(source.node.id);
                                        }}
                                      >
                                        {source.field
                                          ? `${source.shared ? "공유" : "값"}: ${value === "" ? "미정" : String(value)}${source.field.unit === "seconds" && value !== "" ? "초" : ""}`
                                          : `↗ ${source.node.title}`}
                                      </button>
                                    );
                                  })()}
                              </>
                            )}
                          </div>
                        );
                      })}
                    </div>
                  ),
                )}
              </div>
              {Boolean(data.optionalCount) && (
                <button
                  className="ng-optional-inputs nodrag nopan"
                  disabled={data.busy}
                  aria-expanded={data.expanded}
                  aria-label={`${label} 추가 입력 ${data.optionalCount}개`}
                  onClick={(event) => {
                    event.stopPropagation();
                    data.expand?.();
                  }}
                >
                  {data.expanded
                    ? "추가 입력 접기"
                    : `추가 입력 ${data.optionalCount}개 펼치기`}
                </button>
              )}
              <div className="ng-node-values">
                {definition.fields
                  .filter(
                    (field) =>
                      !data.model ||
                      (resonanceFieldActive(data.model, field) &&
                        spatialFieldActive(data.model, field) &&
                        groundFieldActive(data.model, field) &&
                        p4FieldActive(data.model, field)),
                  )
                  .map((field) => (
                    <div key={field.id}>
                      <span>{field.label}</span>
                      <button
                        className="ng-field-value nodrag nopan"
                        disabled={data.busy}
                        aria-label={`${label} ${field.label} 수정`}
                        title={`${String(data.model?.values[field.id] ?? "")} · 클릭하여 수정`}
                        onClick={(e) => {
                          e.stopPropagation();
                          if (!data.model) return;
                          if (field.valueType === "tile-pattern")
                            data.navigate(id);
                          else data.field(data.model, field);
                        }}
                      >
                        {field.valueType === "tile-pattern"
                          ? data.model?.values.pattern
                            ? `${String(data.model.values.pattern).split("X").length - 1 + Number(data.model.values.includeAnchor)}타일`
                            : data.model?.values.shape
                              ? "타일 재지정 필요"
                              : "미정"
                          : field.options
                            ? field.options.find(
                                (o) => o.value === data.model?.values[field.id],
                              )?.label
                            : typeof data.model?.values[field.id] === "boolean"
                              ? data.model.values[field.id]
                                ? "참"
                                : "거짓"
                              : String(data.model?.values[field.id] ?? "") ||
                                (field.valueType === "optional-number"
                                  ? "미정"
                                  : "—")}
                      </button>
                    </div>
                  ))}
              </div>
            </>
          )}
        </div>
      </ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Content
          className="skill-context-menu"
          aria-label={`${label} 노드 메뉴`}
          collisionPadding={8}
          onCloseAutoFocus={(e) => {
            e.preventDefault();
            trigger.current?.focus();
          }}
        >
          <ContextMenu.Label className="skill-context-label">
            {label}
          </ContextMenu.Label>
          {(
            [
              ["duplicate", "복제"],
              ["copy", "복사"],
              ["group", "프레임으로 묶기"],
              ["ungroup", "프레임 풀기"],
              ["delete", "삭제"],
            ] as const
          ).map(([action, text]) => (
            <ContextMenu.Item
              key={action}
              className="skill-context-item"
              disabled={data.busy}
              data-destructive={action === "delete" || undefined}
              onSelect={() => data.menu(action)}
            >
              {text}
            </ContextMenu.Item>
          ))}
        </ContextMenu.Content>
      </ContextMenu.Portal>
    </ContextMenu.Root>
  );
});
const nodeTypes = { graph: GraphCard, frame: GraphCard };
type Props = {
  state: Snapshot;
  catalog: Definition[];
  selected: string[];
  setSelected: Dispatch<SetStateAction<string[]>>;
  busy: boolean;
  active: boolean;
  flowOnly: boolean;
  foldConstants: boolean;
  field(node: GraphNode, field: Field): void;
  navigate(id: string): void;
  edit(command: Command, version?: Version): Promise<boolean>;
  port(anchor: Anchor): void;
  menu(action: MenuAction, ids: string[]): void;
  add(position: Point, anchor?: Anchor): void;
  api: Ref<CanvasApi>;
};
function Canvas({
  state,
  catalog,
  selected,
  setSelected,
  busy,
  active,
  flowOnly,
  foldConstants,
  field,
  navigate,
  edit,
  port,
  menu,
  add,
  api,
}: Props) {
  const flow = useReactFlow<CanvasNode>();
  const root = useRef<HTMLDivElement>(null);
  const drag = useRef<{ version: Version } | null>(null);
  const cancelled = useRef(false);
  const reconnecting = useRef(false);
  const previewRef = useRef<Record<string, Point>>({});
  const [expanded, setExpanded] = useState<ReadonlySet<string>>(new Set());
  const [preview, setPreview] = useState<Record<string, Point>>({});
  // The canvas remounts for each document epoch; later edits keep the user's view.
  const [initialEntry] = useState(
    () =>
      state.doc.nodes
        .filter((node) => ["skill-request", "skill-use"].includes(node.kind))
        .sort(
          (a, b) => a.position.y - b.position.y || a.position.x - b.position.x,
        )[0],
  );
  const resetPreview = useCallback(() => {
    previewRef.current = {};
    setPreview({});
  }, []);
  const cancel = useCallback(() => {
    cancelled.current = true;
    drag.current = null;
    resetPreview();
  }, [resetPreview]);
  useImperativeHandle(
    api,
    () => ({
      center() {
        const rect = root.current?.getBoundingClientRect();
        return rect
          ? flow.screenToFlowPosition({
              x: rect.left + rect.width / 2,
              y: rect.top + rect.height / 2,
            })
          : { x: 40, y: 40 };
      },
      fit() {
        void flow.fitView({ padding: 0.18, duration: 0 });
      },
      focus(id) {
        const node = flow.getNode(id);
        const rect = root.current?.getBoundingClientRect();
        if (node?.type === "graph" && rect) {
          // Keep text at reading size. Tall cards start at their heading and can
          // be panned; fitting the whole card would make every label too small.
          void flow.setViewport({
            x:
              Math.max(24, (rect.width - (node.width ?? 340)) / 2) -
              node.position.x,
            y:
              Math.max(32, (rect.height - (node.height ?? 140)) / 2) -
              node.position.y,
            zoom: 1,
          });
        } else if (node)
          void flow.fitView({
            nodes: [node],
            padding: 0.6,
            maxZoom: 1,
            duration: 0,
          });
      },
      cancel,
    }),
    [flow, cancel],
  );
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") cancel();
    };
    window.addEventListener("keydown", key, true);
    window.addEventListener("pointercancel", cancel);
    window.addEventListener("blur", cancel);
    window.addEventListener("editor-cancel", cancel);
    return () => {
      window.removeEventListener("keydown", key, true);
      window.removeEventListener("pointercancel", cancel);
      window.removeEventListener("blur", cancel);
      window.removeEventListener("editor-cancel", cancel);
    };
  }, [cancel]);
  const frameById = useMemo(
    () => new Map(state.frameRects.map((f) => [f.id, f])),
    [state.frameRects],
  );
  const nodeById = useMemo(
    () => new Map(state.doc.nodes.map((n) => [n.id, n])),
    [state.doc.nodes],
  );
  const positionFor = useCallback(
    (node: GraphNode, positions: Record<string, Point>) => {
      if (positions[node.id]) return positions[node.id];
      const f = node.frame ? frameById.get(node.frame) : undefined;
      const p = node.frame ? positions[node.frame] : undefined;
      return f && p
        ? { x: node.position.x + p.x - f.x, y: node.position.y + p.y - f.y }
        : node.position;
    },
    [frameById],
  );
  const definitions = useMemo(
    () => new Map(catalog.map((d) => [d.kind, d])),
    [catalog],
  );
  const folded = useMemo(() => {
    if (!foldConstants) return new Set<string>();
    return new Set(
      [...privateConstants(state.doc, definitions)]
        .filter(
          ([id, edge]) => !selected.includes(id) && !selected.includes(edge.id),
        )
        .map(([id]) => id),
    );
  }, [foldConstants, state.doc, definitions, selected]);
  const baseNodes = useMemo<CanvasNode[]>(() => {
    const issueCounts = new Map<string, number>();
    for (const d of state.diagnostics)
      issueCounts.set(d.nodeId, (issueCounts.get(d.nodeId) ?? 0) + 1);
    const frameTitles = new Map(state.doc.frames.map((f) => [f.id, f.title]));
    const actionData = (
      id: string,
    ): Pick<
      NodeData,
      "select" | "port" | "menu" | "busy" | "field" | "navigate"
    > => ({
      select: () => {
        if (!busy)
          setSelected((current) => (current.includes(id) ? current : [id]));
      },
      port,
      field,
      navigate,
      menu: (action) => menu(action, selected.includes(id) ? selected : [id]),
      busy,
    });
    const source = inputSources(state.doc);
    const uses = new Map<string, number>();
    for (const edge of state.doc.edges)
      uses.set(edge.source.node, (uses.get(edge.source.node) ?? 0) + 1);
    return [
      ...state.frameRects.map((rect) => ({
        id: rect.id,
        type: "frame",
        position: { x: rect.x, y: rect.y },
        data: {
          title: frameTitles.get(rect.id) || "프레임",
          issues: 0,
          ...actionData(rect.id),
        },
        selected: selected.includes(rect.id),
        width: rect.width,
        height: rect.height,
        measured: { width: rect.width, height: rect.height },
        style: { width: rect.width, height: rect.height },
        zIndex: 0,
        dragHandle: ".ng-drag-handle",
        className: "ng-frame",
      })),
      ...state.doc.nodes.map((n) => {
        const fullDefinition = definitions.get(n.kind);
        const presentation = fullDefinition
          ? cardPresentation(
              n,
              fullDefinition,
              new Set(
                state.doc.edges
                  .filter((e) => e.target.node === n.id)
                  .map((e) => e.target.port),
              ),
              expanded.has(n.id),
            )
          : undefined;
        const d = presentation?.definition;
        const width = d?.width ?? 232;
        const height = d?.height ?? 140;
        return {
          id: n.id,
          type: "graph",
          hidden: folded.has(n.id),
          position: n.position,
          data: {
            model: n,
            definition: d,
            optionalCount: presentation?.optionalCount,
            expanded: expanded.has(n.id),
            expand: () =>
              setExpanded((current) => {
                const next = new Set(current);
                if (next.has(n.id)) next.delete(n.id);
                else next.add(n.id);
                return next;
              }),
            title: n.title || d?.label || n.kind,
            issues: issueCounts.get(n.id) ?? 0,
            sources: Object.fromEntries(
              (d?.inputs ?? [])
                .filter((p) => p.valueType !== "flow" && !p.multiple)
                .flatMap((p) => {
                  const node = source(n, p.id);
                  if (!node) return [];
                  const def = definitions.get(node.kind);
                  const literal =
                    def &&
                    def.inputs.length === 0 &&
                    def.fields.length === 1 &&
                    def.fields[0].id === "value" &&
                    def.fields[0].valueType !== "tile-pattern";
                  return [
                    [
                      p.id,
                      {
                        node,
                        field: literal ? def.fields[0] : undefined,
                        shared: (uses.get(node.id) ?? 0) > 1,
                      },
                    ],
                  ];
                }),
            ),
            ...actionData(n.id),
          },
          selected: selected.includes(n.id),
          width,
          height,
          measured: { width, height },
          style: { width, height },
          zIndex: 2,
          dragHandle: ".ng-drag-handle",
        };
      }),
    ];
  }, [
    expanded,
    state.doc,
    state.frameRects,
    state.diagnostics,
    definitions,
    selected,
    setSelected,
    port,
    menu,
    busy,
    field,
    navigate,
    folded,
  ]);
  const nodes = useMemo(
    () =>
      baseNodes.map((n) => {
        const position = n.data.model
          ? positionFor(n.data.model, preview)
          : (preview[n.id] ?? n.position);
        return position === n.position ? n : { ...n, position };
      }),
    [baseNodes, positionFor, preview],
  );
  // React Flow measures newly exposed handles on the next animation frame.
  // Derive render readiness from those measurements; the document and history
  // already contain the complete edit, including its connected input ports.
  const measuredEdgeIds = useStore((flowState) =>
    JSON.stringify(
      state.doc.edges
        .filter((edge) =>
          (["source", "target"] as const).every((end) =>
            flowState.nodeLookup
              .get(edge[end].node)
              ?.internals.handleBounds?.[end]?.some(
                (handle) => handle.id === edge[end].port,
              ),
          ),
        )
        .map((edge) => edge.id),
    ),
  );
  const edges = useMemo<Edge[]>(() => {
    const measured = new Set<string>(JSON.parse(measuredEdgeIds));
    const selection = new Set(selected);
    return state.doc.edges
      .filter((e) => measured.has(e.id))
      .filter((e) => !folded.has(e.source.node) && !folded.has(e.target.node))
      .filter(
        (e) =>
          !flowOnly ||
          selection.has(e.id) ||
          selection.has(e.source.node) ||
          selection.has(e.target.node) ||
          definitions
            .get(nodeById.get(e.source.node)?.kind ?? "")
            ?.outputs.find((p) => p.id === e.source.port)?.valueType === "flow",
      )
      .map((e) => ({
        id: e.id,
        source: e.source.node,
        sourceHandle: e.source.port,
        target: e.target.node,
        targetHandle: e.target.port,
        selected: selection.has(e.id),
        type: "default",
        // Keep edges clickable above frame backgrounds and below node cards.
        zIndex: 1,
        className: `ng-edge ng-type-${definitions.get(nodeById.get(e.source.node)?.kind ?? "")?.outputs.find((p) => p.id === e.source.port)?.valueType ?? "number"}${selection.has(e.source.node) || selection.has(e.target.node) ? " ng-edge-connected" : ""}`,
        ariaLabel: `연결 ${e.id}`,
        interactionWidth: 20,
      }));
  }, [
    state.doc.edges,
    selected,
    definitions,
    nodeById,
    flowOnly,
    folded,
    measuredEdgeIds,
  ]);
  const start = () => {
    cancelled.current = false;
    drag.current = {
      version: { epoch: state.epoch, revision: state.revision },
    };
  };
  const changes = (changes: NodeChange<CanvasNode>[]) => {
    const selection = changes.filter((c) => c.type === "select");
    if (selection.length && !busy)
      setSelected((current) => {
        const next = new Set(current);
        for (const c of selection) {
          if (c.selected) next.add(c.id);
          else next.delete(c.id);
        }
        return [...next];
      });
    if (cancelled.current || !drag.current) return;
    const positions = changes.filter(
      (c) => c.type === "position" && c.position,
    );
    if (positions.length) {
      const next = { ...previewRef.current };
      for (const c of positions)
        if (c.type === "position" && c.position) next[c.id] = c.position;
      previewRef.current = next;
      setPreview(next);
    }
  };
  const finish = async () => {
    const gesture = drag.current;
    drag.current = null;
    if (!gesture || cancelled.current) {
      resetPreview();
      return;
    }
    const positions = state.doc.nodes
      .map((n) => ({ id: n.id, position: positionFor(n, previewRef.current) }))
      .filter((p) => {
        const n = nodeById.get(p.id);
        return (
          n && (n.position.x !== p.position.x || n.position.y !== p.position.y)
        );
      });
    if (positions.length)
      await edit({ type: "move", positions }, gesture.version);
    resetPreview();
  };
  const connect = (connection: Connection, replaceEdge?: string) => {
    if (
      cancelled.current ||
      !connection.sourceHandle ||
      !connection.targetHandle
    )
      return;
    void edit({
      type: "connect",
      source: { node: connection.source, port: connection.sourceHandle },
      target: { node: connection.target, port: connection.targetHandle },
      replaceEdge,
    });
  };
  return (
    <div
      ref={root}
      role="region"
      className="ng-canvas"
      data-testid="node-canvas"
      aria-label="노드 캔버스"
      onFocusCapture={(event) => {
        const target = event.target;
        if (
          !(target instanceof HTMLElement) ||
          !target.matches(".ng-card button")
        )
          return;
        // Keep keyboard editing at reading zoom, including fields below the
        // heading of a tall card. Panning is view state, never a document edit.
        const bounds = event.currentTarget.getBoundingClientRect();
        const row = target.closest(".ng-node-values > div") ?? target;
        const rect = row.getBoundingClientRect();
        const dx =
          rect.left < bounds.left + 16
            ? bounds.left + 16 - rect.left
            : Math.min(0, bounds.right - 16 - rect.right);
        const dy =
          rect.top < bounds.top + 16
            ? bounds.top + 16 - rect.top
            : Math.min(0, bounds.bottom - 16 - rect.bottom);
        if (dx || dy) {
          const viewport = flow.getViewport();
          void flow.setViewport({
            ...viewport,
            x: viewport.x + dx,
            y: viewport.y + dy,
          });
        }
      }}
    >
      <ReactFlow<CanvasNode>
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onNodesChange={changes}
        onNodeClick={(event, node) => {
          if (!busy && !event.ctrlKey && !event.metaKey && !event.shiftKey)
            setSelected([node.id]);
        }}
        onEdgeClick={(event, edge) => {
          if (!busy && !event.ctrlKey && !event.metaKey && !event.shiftKey)
            setSelected([edge.id]);
        }}
        onEdgesChange={(changes) => {
          const selection = changes.filter((c) => c.type === "select");
          if (selection.length && !busy)
            setSelected((current) => {
              const next = new Set(current);
              for (const c of selection) {
                if (c.selected) next.add(c.id);
                else next.delete(c.id);
              }
              return [...next];
            });
        }}
        onNodeDragStart={start}
        onSelectionDragStart={start}
        onNodeDragStop={() => void finish()}
        onSelectionDragStop={() => void finish()}
        onConnectStart={() => {
          cancelled.current = false;
        }}
        onConnect={(c) => connect(c)}
        onReconnectStart={() => {
          cancelled.current = false;
          reconnecting.current = true;
        }}
        onReconnect={(edge, c) => connect(c, edge.id)}
        onReconnectEnd={() => {
          reconnecting.current = false;
        }}
        onEdgeDoubleClick={(event, edge) => {
          if (!busy)
            void edit({
              type: "reroute",
              edgeId: edge.id,
              position: flow.screenToFlowPosition({
                x: event.clientX,
                y: event.clientY,
              }),
            });
        }}
        onConnectEnd={(event, connection) => {
          if (
            reconnecting.current ||
            cancelled.current ||
            connection.isValid ||
            connection.toHandle ||
            !connection.fromNode ||
            !connection.fromHandle ||
            !(event.target instanceof Element) ||
            !event.target.closest(".react-flow__pane")
          )
            return;
          const p = "changedTouches" in event ? event.changedTouches[0] : event;
          if (p)
            add(flow.screenToFlowPosition({ x: p.clientX, y: p.clientY }), {
              endpoint: {
                node: connection.fromNode.id,
                port: connection.fromHandle.id ?? "",
              },
              output: connection.fromHandle.type === "source",
            });
        }}
        onPaneContextMenu={(event) => {
          event.preventDefault();
          add(
            flow.screenToFlowPosition({ x: event.clientX, y: event.clientY }),
          );
        }}
        onPaneClick={() => {
          if (!busy) setSelected([]);
        }}
        nodesDraggable={!busy && active}
        nodesConnectable={!busy && active}
        edgesReconnectable={!busy && active}
        elementsSelectable={!busy && active}
        elevateNodesOnSelect={false}
        deleteKeyCode={null}
        disableKeyboardA11y
        selectionOnDrag
        selectionMode={SelectionMode.Partial}
        multiSelectionKeyCode={["Control", "Meta"]}
        panOnDrag={[1, 2]}
        panOnScroll={false}
        zoomOnScroll
        minZoom={0.05}
        maxZoom={1.5}
        defaultViewport={
          initialEntry
            ? {
                x: 32 - initialEntry.position.x * 0.85,
                y: 64 - initialEntry.position.y * 0.85,
                zoom: 0.85,
              }
            : undefined
        }
        fitView={!initialEntry}
        fitViewOptions={{ padding: 0.18 }}
        proOptions={{ hideAttribution: false }}
      >
        <Background variant={BackgroundVariant.Dots} gap={20} size={1} />
        <Controls showInteractive={false} aria-label="" />
      </ReactFlow>
      {state.doc.nodes.length === 0 && (
        <div className="ng-empty">
          <strong>첫 노드를 추가하세요</strong>
          <span>왼쪽 목록 또는 캔버스 우클릭으로 시작합니다.</span>
        </div>
      )}
    </div>
  );
}
export function GraphCanvas(props: Props) {
  return (
    <ReactFlowProvider>
      <Canvas {...props} />
    </ReactFlowProvider>
  );
}
