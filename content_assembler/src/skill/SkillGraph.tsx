import { useEffect, useMemo, useRef, useState } from "react";
import {
  Background,
  Controls,
  Handle,
  MarkerType,
  MiniMap,
  Position,
  ReactFlow,
  ReactFlowProvider,
  useNodesInitialized,
  useReactFlow,
  type Node,
  type NodeProps,
} from "@xyflow/react";
import ELK from "elkjs/lib/elk-api";
import workerUrl from "elkjs/lib/elk-worker.min.js?url";
import { ChevronDown, ChevronRight, Focus, Plus, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useUiStore } from "@/stores/ui-store";
import {
  blockAncestors,
  graphSize,
  projectSkillGraph,
  type GraphItem,
} from "./graph-model";
import { arrangeSkillGraphs, layoutRequest } from "./graph-layout";
import type { Selection, SkillDocument } from "./model";
import type { SkillDiagnostic } from "./validation";

import { SkillNodeMenu, type SkillNodeActions } from "./SkillNodeMenu";

type SkillNode = Node<GraphItem & SkillNodeActions, "skill">;
function SkillNodeView({ data: n, selected }: NodeProps<SkillNode>) {
  return (
    <SkillNodeMenu node={n}>
      <div
        className={`skill-graph-node ${n.kind}`}
        data-group={n.category}
        data-selected={selected}
        data-item-id={n.id}
      >
        {n.kind !== "event" && (
          <Handle
            type="target"
            position={Position.Left}
            isConnectable={false}
          />
        )}
        <Handle type="source" position={Position.Right} isConnectable={false} />
        {n.kind === "join" ? (
          <span>{n.title}</span>
        ) : (
          <>
            <button
              className="skill-node-content"
              title="드래그하여 위치 이동"
              onClick={() => n.select(n.selection)}
              aria-label={`${n.title} 선택`}
              aria-pressed={selected}
            >
              <span className="skill-node-category">{n.category}</span>
              <strong>{n.title}</strong>
              <span className="skill-node-summary" title={n.summary}>
                {n.summary}
              </span>
            </button>
            <div
              className="skill-node-tools nodrag"
              onClick={(e) => e.stopPropagation()}
            >
              {n.container && (
                <button
                  aria-label={`${n.title} ${n.expanded ? "접기" : "펼치기"}`}
                  aria-expanded={n.expanded}
                  onClick={() => n.toggle(n.id)}
                >
                  {n.expanded ? (
                    <ChevronDown size={12} />
                  ) : (
                    <ChevronRight size={12} />
                  )}
                  {n.childCount}개
                </button>
              )}
              {n.issueCount > 0 && (
                <button
                  className="skill-node-issues"
                  aria-label={`${n.title} 진단 ${n.issueCount}개`}
                  onClick={() => n.diagnose(n.id)}
                >
                  미정 {n.issueCount}
                </button>
              )}
              <button
                className="skill-node-add"
                aria-label={`${n.title} ${n.kind === "event" ? "끝에" : "뒤에"} 블록 추가`}
                onClick={() => n.add(n.selection)}
              >
                <Plus size={13} />
              </button>
            </div>
          </>
        )}
      </div>
    </SkillNodeMenu>
  );
}
const nodeTypes = { skill: SkillNodeView };
type Props = {
  doc: SkillDocument;
  selection: Selection;
  diagnostics: SkillDiagnostic[];
  active: boolean;
  select: (s: Selection) => void;
  add: (s: Selection) => void;
  navigate: (d: SkillDiagnostic) => void;
  addRoutine: () => void;
  edit: SkillNodeActions["edit"];
};
function GraphCanvas({
  doc,
  selection,
  diagnostics,
  active,
  select,
  add,
  navigate,
  addRoutine,
  edit,
}: Props) {
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set());
  const [query, setQuery] = useState("");
  const [searchOpen, setSearchOpen] = useState(false);
  const [minimapOpen, setMinimapOpen] = useState(false);
  const [retry, setRetry] = useState(0);
  // Manual positions belong to this open document's view, not its execution order/history.
  const [moved, setMoved] = useState<Record<string, { x: number; y: number }>>(
    {},
  );
  const [layout, setLayout] = useState<{
    signature: string;
    positions: Record<string, { x: number; y: number }>;
    error?: string;
  }>();
  const theme = useUiStore((s) => s.theme);
  const rf = useReactFlow<SkillNode>();
  const initialized = useNodesInitialized();
  const fitted = useRef("");
  const focused = useRef("");
  const selectedId =
    "id" in selection
      ? selection.id
      : selection.type === "slot"
        ? selection.parentId
        : "";
  const open = useMemo(
    () => new Set([...expanded, ...blockAncestors(doc, selectedId)]),
    [expanded, doc, selectedId],
  );
  const model = useMemo(
    () => projectSkillGraph(doc, open, diagnostics),
    [doc, open, diagnostics],
  );
  const request = useMemo(() => layoutRequest(model), [model]);
  const signature = JSON.stringify(request);
  const ready = layout?.signature === signature && !layout.error;
  useEffect(() => {
    if (layout?.signature === signature) return;
    let live = true;
    const worker = new Worker(workerUrl);
    const elk = new ELK({ workerFactory: () => worker });
    const workerFailure = new Promise<never>((_, reject) => {
      worker.addEventListener(
        "error",
        (event) =>
          reject(
            new Error(event.message || "자동 배치 작업을 시작하지 못했습니다."),
          ),
        { once: true },
      );
    });
    Promise.race([arrangeSkillGraphs(elk, request), workerFailure])
      .then((positions) => {
        if (live) setLayout({ signature, positions });
      })
      .catch((error: unknown) => {
        if (live) setLayout({ signature, positions: {}, error: String(error) });
      })
      .finally(() => elk.terminateWorker());
    return () => {
      live = false;
      elk.terminateWorker();
    };
  }, [signature, request, layout?.signature, retry]);

  const fit = () =>
    void rf.fitView({ padding: 0.08, minZoom: 0.12, maxZoom: 1 });
  useEffect(() => {
    if (!active || !ready || !initialized || !rf.viewportInitialized) return;
    if (fitted.current !== signature) {
      fitted.current = signature;
      focused.current = selectedId;
      fit();
    }
  });
  useEffect(() => {
    if (
      !active ||
      !ready ||
      !initialized ||
      !rf.viewportInitialized ||
      !selectedId ||
      focused.current === selectedId
    )
      return;
    focused.current = selectedId;
    const node = model.items.find((n) => n.id === selectedId);
    const p = moved[selectedId] ?? layout.positions[selectedId];
    if (!node || !p) return;
    // Keep an already visible node in place; reveal offscreen selections from search/diagnostics.
    const v = rf.getViewport();
    const bounds = root.current?.getBoundingClientRect();
    if (
      bounds &&
      (p.x * v.zoom + v.x < 0 ||
        (p.x + graphSize.width) * v.zoom + v.x > bounds.width ||
        p.y * v.zoom + v.y < 0 ||
        (p.y + graphSize.height) * v.zoom + v.y > bounds.height)
    )
      void rf.setCenter(p.x + graphSize.width / 2, p.y + graphSize.height / 2, {
        zoom: Math.max(v.zoom, 0.8),
      });
  });
  const root = useRef<HTMLDivElement>(null);
  const toggle = (id: string) => {
    const item = model.items.find((n) => n.id === id);
    if (item) select(item.selection);
    setExpanded((previous) => {
      const next = new Set(previous);
      if (open.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };
  const diagnose = (id: string) => {
    const d = diagnostics.find(
      (d) =>
        "id" in d.selection &&
        (d.selection.id === id ||
          blockAncestors(doc, d.selection.id).includes(id)),
    );
    if (d) navigate(d);
  };
  const nodes: SkillNode[] = ready
    ? model.items.map((n) => ({
        id: n.id,
        type: "skill",
        position: moved[n.id] ?? layout.positions[n.id],
        measured: {
          width: n.kind === "join" ? graphSize.joinWidth : graphSize.width,
          height: n.kind === "join" ? graphSize.joinHeight : graphSize.height,
        },
        width: n.kind === "join" ? graphSize.joinWidth : graphSize.width,
        height: n.kind === "join" ? graphSize.joinHeight : graphSize.height,
        selected: n.kind !== "join" && n.id === selectedId,
        selectable: false,
        focusable: false,
        data: { ...n, select, add, toggle, diagnose, edit },
      }))
    : [];
  const edges = model.links.map((e) => ({
    ...e,
    type: "smoothstep",
    markerEnd: {
      type: MarkerType.ArrowClosed,
      width: 16,
      height: 16,
      color: "var(--muted-foreground)",
    },
    selectable: false,
    focusable: false,
  }));
  const searchItems = model.items.filter(
    (n) =>
      n.kind !== "join" && `${n.title} ${n.summary}`.includes(query.trim()),
  );
  return (
    <div className="skill-graph-shell">
      <div className="skill-graph-toolbar">
        <span className="skill-graph-caption">
          사건별 흐름 <b>{doc.routines.length}</b>
        </span>
        <Button
          size="sm"
          variant="ghost"
          aria-pressed={minimapOpen}
          onClick={() => setMinimapOpen(!minimapOpen)}
        >
          미니맵
        </Button>
        <Controls
          aria-label=""
          showInteractive={false}
          showFitView={false}
          orientation="horizontal"
        />
        <Button
          size="sm"
          variant="ghost"
          disabled={!ready || !doc.routines.length}
          title="옮긴 위치를 초기화하고 흐름 순서대로 정렬"
          onClick={() => {
            fitted.current = "";
            setMoved({});
          }}
        >
          자동 정렬
        </Button>
        <Button
          size="sm"
          variant="ghost"
          onClick={fit}
          disabled={!ready || !doc.routines.length}
        >
          <Focus size={14} />
          전체 보기
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={!ready || !selectedId}
          onClick={() => {
            const item = model.items.find((n) => n.id === selectedId);
            if (item)
              void rf.fitView({
                nodes: model.items
                  .filter((n) => n.routineId === item.routineId)
                  .map((n) => ({ id: n.id })),
                padding: 0.15,
                minZoom: 0.2,
                maxZoom: 1,
              });
          }}
        >
          선택 동작 보기
        </Button>
        <Button
          size="sm"
          variant="ghost"
          aria-label="노드 찾기"
          aria-expanded={searchOpen}
          onClick={() => setSearchOpen(!searchOpen)}
        >
          <Search size={14} />
        </Button>
        <Button size="sm" variant="outline" onClick={addRoutine}>
          <Plus size={14} />
          동작 추가
        </Button>
      </div>
      {searchOpen && (
        <div className="skill-node-search">
          <Input
            aria-label="노드 검색"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="펼쳐진 노드의 이름·내용 검색"
          />
          <div className="skill-node-results scroll-panel">
            {searchItems.map((n) => (
              <button
                key={n.id}
                onClick={() => {
                  focused.current = "";
                  select(n.selection);
                  setSearchOpen(false);
                  const p = moved[n.id] ?? layout?.positions[n.id];
                  if (p)
                    void rf.setCenter(
                      p.x + graphSize.width / 2,
                      p.y + graphSize.height / 2,
                      { zoom: 1 },
                    );
                }}
              >
                {n.title}
                <small>{n.summary}</small>
              </button>
            ))}
            {!searchItems.length && <p>검색 결과가 없습니다.</p>}
          </div>
        </div>
      )}
      <div
        className="skill-graph-canvas"
        ref={root}
        data-testid="skill-graph"
        data-layout={ready ? "ready" : layout?.error ? "error" : "loading"}
      >
        <ReactFlow<SkillNode>
          nodes={nodes}
          edges={ready ? edges : []}
          nodeTypes={nodeTypes}
          onNodeClick={(_, node) => select(node.data.selection)}
          onNodeDragStart={(_, node) => {
            focused.current = node.id;
            select(node.data.selection);
          }}
          onNodesChange={(changes) => {
            const positions = changes.filter(
              (change) => change.type === "position" && change.position,
            );
            if (!positions.length) return;
            setMoved((previous) => {
              const next = { ...previous };
              for (const change of positions) {
                if (change.type === "position" && change.position)
                  next[change.id] = change.position;
              }
              return next;
            });
          }}
          nodesDraggable
          selectNodesOnDrag={false}
          nodesConnectable={false}
          edgesReconnectable={false}
          nodesFocusable={false}
          edgesFocusable={false}
          deleteKeyCode={null}
          selectionKeyCode={null}
          minZoom={0.12}
          maxZoom={1.8}
          colorMode={theme}
          preventScrolling={active}
          ariaLabelConfig={{
            "controls.zoomIn.ariaLabel": "그래프 확대",
            "controls.zoomOut.ariaLabel": "그래프 축소",
            "controls.fitView.ariaLabel": "그래프 화면 맞춤",
            "minimap.ariaLabel": "스킬 그래프 미니맵",
          }}
        >
          <Background gap={20} size={1} />
          {minimapOpen && (
            <MiniMap
              pannable
              zoomable
              nodeColor="var(--muted-foreground)"
              maskColor="var(--graph-minimap-mask)"
            />
          )}
        </ReactFlow>
        {!ready && (
          <div className="skill-graph-status" role="status">
            {layout?.signature === signature && layout.error ? (
              <>
                <p>자동 배치 실패: {layout.error}</p>
                <Button
                  onClick={() => {
                    setLayout(undefined);
                    setRetry((n) => n + 1);
                  }}
                >
                  다시 배치
                </Button>
              </>
            ) : (
              "흐름을 배치하고 있습니다…"
            )}
          </div>
        )}
        {ready && !doc.routines.length && (
          <div className="skill-graph-status">
            <p>시작 사건을 추가해 스킬 흐름을 만드세요.</p>
            <Button onClick={addRoutine}>첫 동작 추가</Button>
          </div>
        )}
      </div>
    </div>
  );
}
export function SkillGraph(props: Props) {
  return (
    <ReactFlowProvider>
      <GraphCanvas {...props} />
    </ReactFlowProvider>
  );
}
