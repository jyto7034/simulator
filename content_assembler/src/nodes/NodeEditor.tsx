import { NodeCompiler } from "./compiler/NodeCompiler";
import ELK, { type ELK as ElkEngine } from "elkjs/lib/elk-api";
import workerUrl from "elkjs/lib/elk-worker.min.js?url";
import { arrangeGraph } from "./core/layout";
import {
  includePrivateConstants,
  privateConstants,
} from "./core/constant-presentation";
import { NodeFieldDialog, type FieldEdit } from "./NodeFieldDialog";
import {
  useCallback,
  useEffect,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import {
  Braces,
  Copy,
  Download,
  FolderOpen,
  Frame,
  Plus,
  Redo2,
  Search,
  Save,
  Trash2,
  Undo2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogTitle,
} from "@/components/ui/dialog";
import { ThemeMenu } from "@/components/theme-menu";
import { downloadJson } from "@/editor/files";
import { NodeStore, type EditResult } from "./store";
import { GraphCanvas, type CanvasApi, type MenuAction } from "./GraphCanvas";
import { NodeInspector, type InspectorApi } from "./NodeInspector";
import { parseFragment } from "./core/validation";
import { ContentDialog, type ContentDialogRequest } from "./ContentDialog";
import type { ContentFile } from "@/content/files";
import type { Anchor, Command, Fragment, Point, Version } from "./types";
import "./nodes.css";

type Picker = { position: Point; anchor?: Anchor; version: Version };
type Confirmation = {
  title: string;
  description: string;
  action(): Promise<void>;
};
export function NodeEditor({ active }: { active: boolean }) {
  const [store] = useState(() => new NodeStore());
  const view = useSyncExternalStore(store.subscribe, store.getSnapshot);
  const { state, catalog } = view;
  const [selected, setSelected] = useState<string[]>([]);
  const [search, setSearch] = useState("");
  const [pickerSearch, setPickerSearch] = useState("");
  const [picker, setPicker] = useState<Picker | null>(null);
  const [anchor, setAnchor] = useState<Anchor | null>(null);
  const [connectionTarget, setConnectionTarget] = useState("");
  const [confirmation, setConfirmation] = useState<Confirmation | null>(null);
  const [draft, setDraft] = useState(false);
  const [fileBusy, setFileBusy] = useState(false);
  const [contentDialog, setContentDialog] = useState<
    (ContentDialogRequest & { version: Version; token?: string }) | null
  >(null);
  const [contentSource, setContentSource] = useState<Pick<
    ContentFile,
    "name" | "revision"
  > | null>(null);
  const [exported, setExported] = useState<{
    token: string;
    epoch: number;
  } | null>(null);
  const [diagnosticsOpen, setDiagnosticsOpen] = useState(false);
  const canvas = useRef<CanvasApi>(null);
  const inspector = useRef<InspectorApi>(null);
  const fileInput = useRef<HTMLInputElement>(null);
  const focusReturn = useRef<HTMLElement | null>(null);
  const clipboard = useRef<Fragment | null>(null);
  const saving = useRef(false);
  const [arranging, setArranging] = useState(false);
  const [fieldEdit, setFieldEdit] = useState<FieldEdit | null>(null);
  const [flowOnly, setFlowOnly] = useState(false);
  const [foldConstants, setFoldConstants] = useState(false);
  const layoutWorker = useRef<Worker | null>(null);
  useEffect(() => () => layoutWorker.current?.terminate(), []);
  const busy = fileBusy || arranging;
  const blocked = busy || draft;
  const reconcile = useCallback((result: EditResult | undefined) => {
    if (!result?.state) return false;
    const doc = result.state.doc;
    const ids = new Set(
      [...doc.nodes, ...doc.edges, ...doc.frames].map((item) => item.id),
    );
    setSelected((current) =>
      result.selected.length
        ? result.selected
        : current.filter((id) => ids.has(id)),
    );
    setDraft(false);
    return true;
  }, []);
  const edit = useCallback(
    async (command: Command, version?: Version) => {
      const result = await store.edit(command, version);
      if (!reconcile(result)) return false;
      store.message("편집을 적용했습니다.");
      return true;
    },
    [store, reconcile],
  );
  const arrange = async () => {
    if (!state || blocked) return;
    const version = { epoch: state.epoch, revision: state.revision };
    setArranging(true);
    let elk: ElkEngine | undefined;
    try {
      const worker = new Worker(workerUrl);
      layoutWorker.current = worker;
      elk = new ELK({ workerFactory: () => worker });
      const failure = new Promise<never>((_, reject) =>
        worker.addEventListener(
          "error",
          (e) =>
            reject(new Error(e.message || "배치 작업을 시작하지 못했습니다.")),
          { once: true },
        ),
      );
      const positions = await Promise.race([
        arrangeGraph(state.doc, elk),
        failure,
      ]);
      if (await edit({ type: "move", positions }, version)) {
        store.message(
          "연결 방향에 따라 배치했습니다. 실행 취소로 이전 배치를 복원할 수 있습니다.",
        );
        requestAnimationFrame(() => canvas.current?.fit());
      }
    } catch (error) {
      store.message(error instanceof Error ? error.message : String(error));
    } finally {
      elk?.terminateWorker();
      layoutWorker.current?.terminate();
      layoutWorker.current = null;
      setArranging(false);
    }
  };
  const rememberFocus = () => {
    focusReturn.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
  };
  const restoreFocus = (e: Event) => {
    e.preventDefault();
    focusReturn.current?.focus();
  };
  const showAdd = useCallback(
    (position: Point, anchor?: Anchor) => {
      if (fileBusy || draft) return;
      focusReturn.current =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
      setPickerSearch("");
      setPicker({ position, anchor, version: store.version() });
    },
    [store, draft, fileBusy],
  );
  const choosePort = useCallback((value: Anchor) => {
    focusReturn.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    setConnectionTarget("");
    setAnchor(value);
  }, []);
  const travel = async (redo: boolean) => {
    if (draft) {
      inspector.current?.cancel();
      store.message("미적용 속성 변경을 취소했습니다.");
      return;
    }
    canvas.current?.cancel();
    const result = await store.travel(redo);
    if (reconcile(result))
      store.message(redo ? "다시 실행했습니다." : "실행 취소했습니다.");
  };
  const withConstants = (ids: string[]) =>
    foldConstants && state
      ? includePrivateConstants(
          ids,
          privateConstants(state.doc, new Map(catalog.map((d) => [d.kind, d]))),
        )
      : ids;
  const copy = async (ids = selected) => {
    const result = await store.copy(withConstants(ids));
    if (!result?.fragment) return;
    clipboard.current = result.fragment;
    try {
      await navigator.clipboard.writeText(JSON.stringify(result.fragment));
      store.message(`${result.fragment.nodes.length}개 노드를 복사했습니다.`);
    } catch {
      store.message(
        "앱 안에 노드를 복사했습니다. 시스템 클립보드에 쓰지 못해 이 창에서만 붙여넣을 수 있습니다.",
      );
    }
  };
  const paste = async () => {
    const version = store.version();
    try {
      let fragment: Fragment | undefined;
      let text: string | undefined;
      try {
        text = await navigator.clipboard.readText();
      } catch (error) {
        if (clipboard.current) fragment = clipboard.current;
        else throw error;
      }
      if (text !== undefined) fragment = parseFragment(text);
      if (!fragment) throw new Error("복사한 노드가 없습니다.");
      await edit(
        {
          type: "paste",
          fragment,
          position: canvas.current?.center() ?? { x: 40, y: 40 },
        },
        version,
      );
    } catch (error) {
      store.message(
        error instanceof Error ? error.message : String(error),
        true,
      );
    }
  };
  const menu = (action: MenuAction, ids = selected) => {
    if (blocked || ids.length === 0) return;
    if (action === "copy") {
      void copy(ids);
      return;
    }
    ids = withConstants(ids);
    if (action === "group") {
      void edit({ type: "group", ids, title: "새 프레임" });
      return;
    }
    void edit({ type: action, ids });
  };
  const newGraph = async () => {
    rememberFocus();
    const version = store.version();
    const action = async () => {
      const result = await store.create(version);
      if (reconcile(result)) {
        setSelected([]);
        setExported(null);
        setContentSource(null);
        store.message("빈 노드 그래프를 만들었습니다.");
      }
    };
    if (state?.dirty)
      setConfirmation({
        title: "노드 그래프를 교체할까요?",
        description:
          "현재 그래프에 저장을 확인하지 않은 변경이 있습니다. 보관하려면 취소 후 저장하세요.",
        action,
      });
    else await action();
  };
  const exportDownload = async () => {
    if (saving.current || fileBusy) return;
    saving.current = true;
    try {
      if (draft && !(await inspector.current?.apply())) return;
      setFileBusy(true);
      setExported(null);
      const result = await store.export();
      if (!result?.export || !result.state) return;
      await downloadJson(result.export.json, result.export.filename);
      setExported({ token: result.export.token, epoch: result.state.epoch });
      store.message(
        "다운로드를 요청했습니다. 파일을 확인한 뒤 ‘다운로드 확인’을 누르세요.",
      );
    } catch (error) {
      store.message(
        `저장 실패: ${error instanceof Error ? error.message : String(error)}`,
        true,
      );
    } finally {
      saving.current = false;
      setFileBusy(false);
    }
  };
  const save = async () => {
    if (saving.current || fileBusy) return;
    saving.current = true;
    try {
      if (draft && !(await inspector.current?.apply())) return;
      const result = store.export();
      if (!result?.export) return;
      setContentDialog({
        mode: "save",
        version: store.version(),
        token: result.export.token,
        json: result.export.json,
        name: contentSource?.name ?? result.export.filename,
        previous: contentSource,
      });
    } finally {
      saving.current = false;
    }
  };
  const openJson = async (
    json: string,
    name: string,
    version: Version,
    source: Pick<ContentFile, "name" | "revision"> | null,
  ) => {
    const action = async () => {
      const result = store.load(json, version);
      if (reconcile(result)) {
        setSelected([]);
        setExported(null);
        setContentSource(source);
        store.message(`${name} 파일을 열었습니다.`);
      }
    };
    if (store.getSnapshot().state?.dirty) {
      if (!source) rememberFocus();
      setConfirmation({
        title: "파일로 그래프를 교체할까요?",
        description:
          "현재 그래프에 저장하지 않은 변경이 있습니다. 보관하려면 취소 후 저장하세요.",
        action,
      });
    } else await action();
  };
  const openFile = async (file: File) => {
    const version = store.version();
    setFileBusy(true);
    try {
      if (file.size > 5_000_000) throw new Error("파일 크기 제한은 5MB입니다.");
      const json = await file.text();
      const current = store.version();
      if (
        current.epoch !== version.epoch ||
        current.revision !== version.revision
      )
        throw new Error(
          "파일을 읽는 동안 문서가 바뀌었습니다. 현재 작업을 보존했습니다.",
        );
      await openJson(json, file.name, version, null);
    } catch (error) {
      store.message(
        error instanceof Error ? error.message : String(error),
        true,
      );
    } finally {
      setFileBusy(false);
    }
  };
  useEffect(() => {
    const unload = (event: BeforeUnloadEvent) => {
      if (store.getSnapshot().state?.dirty || draft) {
        event.preventDefault();
        event.returnValue = "";
      }
    };
    window.addEventListener("beforeunload", unload);
    return () => window.removeEventListener("beforeunload", unload);
  }, [store, draft]);
  useEffect(() => {
    const key = (event: KeyboardEvent) => {
      if (
        !active ||
        !state ||
        busy ||
        fieldEdit ||
        confirmation ||
        picker ||
        anchor ||
        contentDialog ||
        event.isComposing ||
        (event.target instanceof Element &&
          event.target.closest(
            'input,textarea,select,[contenteditable="true"],[role="dialog"],[role="menu"]',
          ))
      )
        return;
      const k = event.key.toLowerCase();
      if (event.ctrlKey || event.metaKey) {
        if (["z", "y", "s", "o", "c", "v", "d", "g", "a"].includes(k))
          event.preventDefault();
        if (k === "z") void travel(event.shiftKey);
        else if (k === "y") void travel(true);
        else if (k === "s") void save();
        else if (!blocked) {
          if (k === "o") {
            rememberFocus();
            setContentDialog({ mode: "open", version: store.version() });
          } else if (k === "c") void copy();
          else if (k === "v") void paste();
          else if (k === "d") menu("duplicate");
          else if (k === "g") menu(event.shiftKey ? "ungroup" : "group");
          else if (k === "a") setSelected(state.doc.nodes.map((n) => n.id));
        }
      } else if (!blocked && event.key === "Delete") {
        event.preventDefault();
        menu("delete");
      } else if (!blocked && event.shiftKey && k === "a") {
        event.preventDefault();
        showAdd(canvas.current?.center() ?? { x: 40, y: 40 });
      }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  });
  const portType = (value: Anchor) => {
    const n = state?.doc.nodes.find((n) => n.id === value.endpoint.node);
    const d = catalog.find((d) => d.kind === n?.kind);
    return (value.output ? d?.outputs : d?.inputs)?.find(
      (p) => p.id === value.endpoint.port,
    )?.valueType;
  };
  const available = (query: string, link?: Anchor) =>
    catalog.filter(
      (d) =>
        `${d.label} ${d.category}`.includes(query.trim()) &&
        (!link ||
          (link.output ? d.inputs : d.outputs).some(
            (p) => p.valueType === portType(link),
          )),
    );
  const targets = anchor
    ? (state?.doc.nodes.flatMap((n) => {
        const d = catalog.find((d) => d.kind === n.kind);
        return (
          (anchor.output ? d?.inputs : d?.outputs)
            ?.filter((p) => p.valueType === portType(anchor))
            .map((p) => ({
              key: JSON.stringify({ node: n.id, port: p.id }),
              endpoint: { node: n.id, port: p.id },
              label: `${n.title || d?.label} · ${p.label}`,
            })) ?? []
        );
      }) ?? [])
    : [];
  const palette = available(search);
  const selectionKey = selected.join("|");
  return (
    <div className="ng-shell" data-testid="node-editor">
      <header className="ng-header">
        <Braces size={18} aria-hidden="true" />
        <div className="ng-document-title">
          <strong>{state?.doc.name ?? "노드 편집기"}</strong>
          <span>
            {state?.dirty ? "저장하지 않은 변경" : "저장 확인됨"} · 노드 그래프
          </span>
        </div>
        <div className="ng-header-actions">
          <Button
            size="sm"
            variant="outline"
            disabled={blocked || !state}
            onClick={() => void newGraph()}
          >
            <Plus />새 그래프
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={blocked || !state}
            title="저장 폴더의 그래프 목록 (Ctrl+O)"
            onClick={() => {
              rememberFocus();
              setContentDialog({ mode: "open", version: store.version() });
            }}
          >
            <FolderOpen />
            그래프 목록
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={busy || !state}
            onClick={() => void save()}
          >
            <Save />
            저장
          </Button>
          {state && (
            <NodeCompiler
              doc={state.doc}
              version={{ epoch: state.epoch, revision: state.revision }}
              getVersion={() => store.version()}
              disabled={blocked}
              navigate={(id) => {
                setSelected([id]);
                canvas.current?.focus(id);
              }}
            />
          )}
          <Button
            size="icon-sm"
            variant="outline"
            aria-label="JSON 내보내기"
            title="JSON 다운로드로 내보내기"
            disabled={busy || !state}
            onClick={() => void exportDownload()}
          >
            <Download />
          </Button>
          {exported && exported.epoch === state?.epoch && (
            <Button
              size="sm"
              variant="outline"
              disabled={blocked}
              onClick={() => {
                const result = store.acknowledge(exported.token);
                if (result) {
                  setExported(null);
                  store.message("다운로드 확인을 반영했습니다.");
                }
              }}
            >
              다운로드 확인
            </Button>
          )}
          <ThemeMenu />
        </div>
        <input
          ref={fileInput}
          className="sr-only"
          aria-label="노드 그래프 파일 열기"
          type="file"
          accept=".json,application/json"
          onChange={(e) => {
            const file = e.target.files?.[0];
            e.target.value = "";
            if (file) void openFile(file);
          }}
        />
      </header>
      <div className="ng-toolbar" role="toolbar" aria-label="노드 편집 도구">
        <Button
          size="icon-sm"
          variant="ghost"
          aria-label="노드 실행 취소"
          title="실행 취소 (Ctrl Z)"
          disabled={busy || (!draft && !state?.canUndo)}
          onClick={() => void travel(false)}
        >
          <Undo2 />
        </Button>
        <Button
          size="icon-sm"
          variant="ghost"
          aria-label="노드 다시 실행"
          title="다시 실행 (Ctrl Shift Z)"
          disabled={blocked || !state?.canRedo}
          onClick={() => void travel(true)}
        >
          <Redo2 />
        </Button>
        <span className="ng-toolbar-separator" />
        <Button
          size="sm"
          variant="ghost"
          disabled={blocked || !state}
          onClick={() => showAdd(canvas.current?.center() ?? { x: 40, y: 40 })}
        >
          <Plus />
          추가
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={blocked || !selected.length}
          onClick={() => menu("duplicate")}
        >
          <Copy />
          복제
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={blocked || !selected.length}
          onClick={() => menu("group")}
        >
          <Frame />
          묶기
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={blocked || !selected.length}
          onClick={() => menu("ungroup")}
        >
          풀기
        </Button>
        <Button
          size="icon-sm"
          variant="ghost"
          aria-label="선택 노드 삭제"
          title="선택 삭제 (Delete)"
          disabled={blocked || !selected.length}
          onClick={() => menu("delete")}
        >
          <Trash2 />
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={!state}
          onClick={() => canvas.current?.fit()}
        >
          전체 보기
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={blocked || !state?.doc.nodes.length}
          onClick={() => void arrange()}
        >
          {arranging ? "배치 중…" : "흐름 정렬"}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          aria-pressed={flowOnly}
          title="실행 연결과 선택 노드의 연결만 표시합니다."
          onClick={() => setFlowOnly((v) => !v)}
        >
          실행 중심
        </Button>
        <Button
          size="sm"
          variant="ghost"
          aria-pressed={foldConstants}
          title="한 입력에서만 쓰는 고정값은 입력 옆에서 편집합니다. 공유값·변경 규칙의 값은 펼쳐 둡니다."
          onClick={() => setFoldConstants((v) => !v)}
        >
          고정값 접기
        </Button>
        <span className="ng-toolbar-count">
          {state?.doc.nodes.length ?? 0} 노드 · {state?.doc.edges.length ?? 0}{" "}
          연결
        </span>
        <button
          className="ng-diagnostics-toggle"
          aria-expanded={diagnosticsOpen}
          disabled={!state}
          onClick={() => setDiagnosticsOpen((v) => !v)}
        >
          진단 {state?.diagnostics.length ?? 0}
        </button>
      </div>
      {state ? (
        <div className="ng-body">
          <aside className="ng-palette" aria-label="노드 목록">
            {state.doc.nodes.length > 0 && (
              <label className="ng-navigator">
                그래프 탐색
                <select
                  aria-label="노드 또는 구간으로 이동"
                  value={
                    selected.length === 1 &&
                    [...state.doc.nodes, ...state.doc.frames].some(
                      (n) => n.id === selected[0],
                    )
                      ? selected[0]
                      : ""
                  }
                  disabled={blocked}
                  onChange={(e) => {
                    if (!e.target.value) return;
                    setSelected([e.target.value]);
                    canvas.current?.focus(e.target.value);
                  }}
                >
                  <option value="">노드·구간 선택…</option>
                  {state.doc.frames.length > 0 && (
                    <optgroup label="구간">
                      {state.doc.frames.map((f) => (
                        <option key={f.id} value={f.id}>
                          {f.title}
                        </option>
                      ))}
                    </optgroup>
                  )}
                  <optgroup label="노드">
                    {[...state.doc.nodes]
                      .sort(
                        (a, b) =>
                          a.position.y - b.position.y ||
                          a.position.x - b.position.x,
                      )
                      .map((n) => (
                        <option key={n.id} value={n.id}>
                          {n.title ||
                            catalog.find((d) => d.kind === n.kind)?.label}
                        </option>
                      ))}
                  </optgroup>
                </select>
              </label>
            )}
            <div className="ng-panel-heading">
              <strong>노드 추가</strong>
              <small>Shift A</small>
            </div>
            <div className="ng-search">
              <Search size={13} />
              <Input
                aria-label="노드 검색"
                placeholder="노드 검색…"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
            </div>
            {[...new Set(palette.map((d) => d.category))].map((category) => (
              <section key={category}>
                <h3>{category}</h3>
                {palette
                  .filter((d) => d.category === category)
                  .map((d) => (
                    <button
                      key={d.kind}
                      disabled={blocked}
                      data-category={d.category}
                      title={d.description}
                      onClick={() =>
                        void edit({
                          type: "add",
                          kind: d.kind,
                          position: canvas.current?.center() ?? {
                            x: 40,
                            y: 40,
                          },
                        })
                      }
                    >
                      {d.label}
                      <Plus size={12} />
                    </button>
                  ))}
              </section>
            ))}
            {palette.length === 0 && (
              <p className="ng-muted">일치하는 노드가 없습니다.</p>
            )}
          </aside>
          <main className="ng-main">
            <GraphCanvas
              key={state.epoch}
              state={state}
              catalog={catalog}
              flowOnly={flowOnly}
              foldConstants={foldConstants}
              field={(node, field) => {
                if (!blocked) {
                  rememberFocus();
                  setFieldEdit({
                    node,
                    field,
                    version: { epoch: state.epoch, revision: state.revision },
                  });
                }
              }}
              navigate={(id) => {
                if (!blocked) {
                  setSelected([id]);
                  canvas.current?.focus(id);
                }
              }}
              selected={selected}
              setSelected={setSelected}
              busy={blocked}
              active={active}
              edit={edit}
              port={choosePort}
              menu={menu}
              add={showAdd}
              api={canvas}
            />
            {diagnosticsOpen && (
              <section className="ng-diagnostics" aria-label="노드 작성 진단">
                <strong>작성 진단 {state.diagnostics.length}</strong>
                {state.diagnostics.length === 0 ? (
                  <p>현재 그래프에 작성 진단이 없습니다.</p>
                ) : (
                  state.diagnostics.map((d, index) => (
                    <button
                      key={`${d.code}:${d.nodeId}:${d.portId}:${index}`}
                      disabled={blocked}
                      onClick={() => {
                        setSelected([d.nodeId]);
                        canvas.current?.focus(d.nodeId);
                      }}
                    >
                      {d.message}
                    </button>
                  ))
                )}
              </section>
            )}
          </main>
          <NodeInspector
            key={`${state.epoch}:${state.revision}:${selectionKey}`}
            state={state}
            catalog={catalog}
            selected={selected}
            busy={busy}
            edit={edit}
            navigate={(id) => {
              if (blocked) return;
              setSelected([id]);
              canvas.current?.focus(id);
              requestAnimationFrame(() => inspector.current?.focusTitle());
            }}
            pending={setDraft}
            message={(message) => store.message(message, true)}
            api={inspector}
          />
        </div>
      ) : (
        <div className="ng-loading" role="status">
          노드 문서를 준비하고 있습니다…
        </div>
      )}
      <footer className="ng-status">
        <span role={view.error ? "alert" : "status"} data-error={view.error}>
          {view.message}
        </span>
        <small>노드 그래프 편집</small>
      </footer>
      {contentDialog && (
        <ContentDialog
          request={contentDialog}
          onClose={() => setContentDialog(null)}
          onImport={() => {
            setContentDialog(null);
            fileInput.current?.click();
          }}
          onOpen={(file) => {
            const version = contentDialog.version;
            setContentDialog(null);
            void openJson(file.json, file.name, version, {
              name: file.name,
              revision: file.revision,
            });
          }}
          onSaved={(file) => {
            const { version, token } = contentDialog;
            setContentDialog(null);
            if (store.version().epoch !== version.epoch || !token) return;
            if (store.acknowledge(token)) {
              setContentSource({ name: file.name, revision: file.revision });
              setExported(null);
              store.message(`콘텐츠 폴더에 ${file.name} 파일을 저장했습니다.`);
            }
          }}
        />
      )}
      <Dialog
        open={!!picker}
        onOpenChange={(open) => {
          if (!open) setPicker(null);
        }}
      >
        <DialogContent
          className="ng-picker-dialog"
          onCloseAutoFocus={restoreFocus}
        >
          <DialogTitle>
            {picker?.anchor ? "연결할 노드 추가" : "노드 추가"}
          </DialogTitle>
          <DialogDescription>
            {picker?.anchor
              ? "이 포트와 연결할 수 있는 노드를 선택하세요."
              : "추가할 노드를 검색하고 선택하세요."}
          </DialogDescription>
          <Input
            aria-label="추가할 노드 검색"
            value={pickerSearch}
            onChange={(e) => setPickerSearch(e.target.value)}
          />
          <div className="ng-picker-list">
            {available(pickerSearch, picker?.anchor).map((d) => (
              <button
                key={d.kind}
                onClick={() => {
                  if (!picker) return;
                  const value = picker;
                  setPicker(null);
                  void edit(
                    {
                      type: "add",
                      kind: d.kind,
                      position: value.position,
                      link: value.anchor,
                    },
                    value.version,
                  );
                }}
              >
                <span>{d.label}</span>
                <small>
                  {d.category} · {d.inputs.length} 입력 / {d.outputs.length}{" "}
                  출력
                </small>
              </button>
            ))}
            {available(pickerSearch, picker?.anchor).length === 0 && (
              <p>일치하는 노드가 없습니다.</p>
            )}
          </div>
        </DialogContent>
      </Dialog>
      <Dialog
        open={!!anchor}
        onOpenChange={(open) => {
          if (!open) setAnchor(null);
        }}
      >
        <DialogContent onCloseAutoFocus={restoreFocus}>
          <DialogTitle>포트 연결</DialogTitle>
          <DialogDescription>
            연결할 {anchor?.output ? "입력" : "출력"}을 선택하세요. 단일 포트의
            기존 연결은 함께 교체됩니다.
          </DialogDescription>
          <select
            className="ng-target-select"
            aria-label="연결할 포트"
            value={connectionTarget}
            onChange={(e) => setConnectionTarget(e.target.value)}
          >
            <option value="">대상 포트 선택</option>
            {targets.map((p) => (
              <option key={p.key} value={p.key}>
                {p.label}
              </option>
            ))}
          </select>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAnchor(null)}>
              취소
            </Button>
            <Button
              disabled={!connectionTarget}
              onClick={() => {
                const target = targets.find((p) => p.key === connectionTarget);
                if (!anchor || !target) return;
                const command: Command = {
                  type: "connect",
                  source: anchor.output ? anchor.endpoint : target.endpoint,
                  target: anchor.output ? target.endpoint : anchor.endpoint,
                };
                setAnchor(null);
                void edit(command);
              }}
            >
              연결
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <Dialog
        open={!!confirmation}
        onOpenChange={(open) => {
          if (!open) setConfirmation(null);
        }}
      >
        <DialogContent onCloseAutoFocus={restoreFocus}>
          <DialogTitle>{confirmation?.title}</DialogTitle>
          <DialogDescription>{confirmation?.description}</DialogDescription>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmation(null)}>
              취소
            </Button>
            <Button
              onClick={() => {
                const action = confirmation?.action;
                setConfirmation(null);
                if (action) void action();
              }}
            >
              교체
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      {fieldEdit && (
        <NodeFieldDialog
          request={fieldEdit}
          close={() => setFieldEdit(null)}
          restoreFocus={() => focusReturn.current?.focus()}
          apply={(value, version) =>
            edit(
              {
                type: "setNode",
                id: fieldEdit.node.id,
                values: { [fieldEdit.field.id]: value },
              },
              version,
            )
          }
        />
      )}
    </div>
  );
}
