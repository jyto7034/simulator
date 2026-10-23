import {
  useEffect,
  useMemo,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import { Layers3 } from "lucide-react";
import { ThemeMenu } from "@/components/theme-menu";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogTitle,
} from "@/components/ui/dialog";
import { EditorSession } from "./editor/history";
import { DocumentFiles } from "./editor/files";
import { createDocument, type MapDocument } from "./editor/model";
import { diagnose, type Diagnostic } from "./editor/diagnostics";
import { removeObject } from "./editor/operations";
import { MapCanvas } from "./editor/MapCanvas";
import { Toolbox } from "./editor/Toolbox";
import { Inspector, type ConfirmAction } from "./editor/Inspector";
import { BottomPanel } from "./editor/BottomPanel";
import { toolDefinitions, type Tool } from "./editor/tools";
import "./editor/editor.css";
type Confirmation = { title: string; description: string; action: () => void };
export default function MapEditor({ active }: { active: boolean }) {
  const [session] = useState(() => new EditorSession());
  const [files] = useState(() => new DocumentFiles(session));
  const state = useSyncExternalStore(session.subscribe, session.getSnapshot);
  const [tool, setTool] = useState<Tool>("select");
  const [shape, setShape] = useState<"brush" | "rectangle">("brush");
  const [subdivisions, setSubdivisions] = useState<number | null>(null);
  const [status, setStatus] = useState("휠: 확대 · 가운데 버튼: 화면 이동");
  const [tab, setTab] = useState<"waves" | "diagnostics">("waves");
  const [confirmation, setConfirmation] = useState<Confirmation | null>(null);
  const [exported, setExported] = useState<{
    doc: MapDocument;
    epoch: number;
  } | null>(null);
  const [saving, setSaving] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const fileInput = useRef<HTMLInputElement>(null);
  const newButton = useRef<HTMLButtonElement>(null);
  const focusReturn = useRef<HTMLElement | null>(null);
  const diagnostics = useMemo(() => diagnose(state.doc), [state.doc]);
  const dirty = session.isDirty();
  const confirm: ConfirmAction = (title, description, action) => {
    focusReturn.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    setConfirmation({ title, description, action });
  };
  const save = async () => {
    if (saving) return;
    setSaving(true);
    setExported(null);
    const result = await files.save();
    setSaving(false);
    if (result && result.epoch === session.getSnapshot().epoch)
      setExported(result);
  };
  const replace = (apply: () => void, opened: boolean) => {
    const { epoch, revision } = session.getSnapshot();
    const action = () => {
      try {
        const current = session.getSnapshot();
        if (epoch !== current.epoch || revision !== current.revision)
          throw new Error(
            "확인하는 동안 작업이 바뀌었습니다. 현재 작업을 보존했습니다. 다시 시도하세요.",
          );
        apply();
      } catch (error) {
        session.message(
          `교체하지 않았습니다: ${error instanceof Error ? error.message : String(error)}`,
        );
        return;
      }
      setExported(null);
      setTool("select");
      session.message(
        opened
          ? "파일을 열었습니다. 편집을 이어갈 수 있습니다."
          : "새 전투 맵을 만들었습니다.",
      );
    };
    if (session.isDirty())
      confirm(
        "미저장 작업을 교체할까요?",
        "현재 작업에 파일 확인 이후 변경이 있습니다. 보관하려면 취소 후 저장하고 다운로드 파일을 확인하세요. 교체하면 현재 이력이 사라집니다.",
        action,
      );
    else action();
  };
  const remove = () => {
    const s = session.getSnapshot().selection;
    if (!s || s.kind === "cell") return;
    confirm(
      "선택 대상 삭제",
      s.kind === "routes" && s.pointId
        ? "선택한 경로 점을 삭제합니다. 다른 경로 점은 이동하지 않습니다."
        : "이 대상을 가리키는 참조는 그대로 보존되어 진단에 표시됩니다. 실행 취소하면 같은 ID로 복원됩니다.",
      () =>
        session.commit((d) => {
          if (s.kind === "routes" && s.pointId) {
            const r = d.routes.find((r) => r.id === s.id);
            if (r) r.points = r.points.filter((p) => p.id !== s.pointId);
          } else removeObject(d, s.kind, s.id);
        }),
    );
  };
  const cancelGesture = () => window.dispatchEvent(new Event("editor-cancel"));
  const chooseTool = (next: Tool) => {
    cancelGesture();
    if (
      next !== "select" &&
      next !== "hand" &&
      session.getSnapshot().selection?.kind === "cell"
    )
      session.select(null);
    setTool(next);
  };
  const travel = (direction: "undo" | "redo") => {
    cancelGesture();
    session.travel(direction);
  };
  useEffect(() => {
    const unload = (e: BeforeUnloadEvent) => {
      if (session.isDirty() || document.activeElement?.tagName === "INPUT") {
        e.preventDefault();
        e.returnValue = "";
      }
    };
    window.addEventListener("beforeunload", unload);
    return () => window.removeEventListener("beforeunload", unload);
  }, [session]);
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (!active) return;
      if (
        e.isComposing ||
        (e.target instanceof Element &&
          e.target.closest(
            'input, textarea, select, [contenteditable="true"], [role="dialog"], [role="menu"]',
          ))
      )
        return;
      if (confirmation) return;
      if (e.ctrlKey || e.metaKey) {
        if (e.key.toLowerCase() === "z") {
          e.preventDefault();
          travel(e.shiftKey ? "redo" : "undo");
        }
        if (e.key.toLowerCase() === "y") {
          e.preventDefault();
          travel("redo");
        }
        if (e.key.toLowerCase() === "s") {
          e.preventDefault();
          void save();
        }
        if (e.key.toLowerCase() === "o") {
          e.preventDefault();
          fileInput.current?.click();
        }
        return;
      }
      if (e.key === "Delete") {
        e.preventDefault();
        remove();
      }
      const entry = toolDefinitions.find((t) => t.key === e.key.toUpperCase());
      if (entry) {
        e.preventDefault();
        chooseTool(entry.id);
      }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  });
  const navigateDiagnostic = (item: Diagnostic) => {
    session.select(item.selection);
    if (item.selection?.kind === "waves") setTab("waves");
    session.message(`${item.code} · ${item.field} · ${item.description}`);
    // Wait for the selected panel to mount before focusing its relevant field.
    requestAnimationFrame(() => {
      const labels: Record<string, string> = {
        name: "맵 이름",
        size: "가로 (셀)",
        zoneId: "등장 구역",
        routeId: "이동 경로",
        priority: "우선순위",
        fallbackMs: "대체 발동 시각 (초)",
        trigger: "기본 조건",
        position: "X 좌표",
        terrain: "셀 지형",
      };
      const root = document.querySelector(
        item.selection?.kind === "waves" ? ".wave-properties" : ".properties",
      );
      const label = [...(root?.querySelectorAll("label") ?? [])].find(
        (l) =>
          labels[item.field] && l.textContent?.startsWith(labels[item.field]),
      );
      const input =
        root?.querySelector<HTMLElement>(
          `[data-field="${item.field}"] input, [data-field="${item.field}"] select`,
        ) ??
        label?.querySelector<HTMLElement>("input, select") ??
        root?.querySelector<HTMLElement>("input, select, button");
      input?.focus();
      input?.scrollIntoView({ block: "nearest" });
    });
  };
  return (
    <div className="editor-shell">
      <header className="editor-header">
        <div className="brand">
          <Layers3 size={18} aria-hidden="true" />
          <h1>전투 맵 조립기</h1>
        </div>
        <div className="document-title" title={state.doc.name}>
          <span className="document-name">{state.doc.name || "이름 없음"}</span>
          <span className="dirty" data-testid="dirty">
            {dirty ? " • 미저장" : " · 파일 확인됨"}
          </span>
        </div>
        <nav aria-label="문서 작업">
          <Button
            ref={newButton}
            size="sm"
            variant="ghost"
            onClick={() =>
              replace(() => session.replace(createDocument(), false), false)
            }
          >
            새로 만들기
          </Button>
          <Button
            size="sm"
            variant="ghost"
            title="열기 (Ctrl+O)"
            onClick={() => fileInput.current?.click()}
          >
            열기
          </Button>
          <Button
            size="sm"
            variant="outline"
            title="JSON 다운로드 (Ctrl+S)"
            disabled={saving}
            onClick={() => void save()}
          >
            저장
          </Button>
          <Button
            size="sm"
            variant="ghost"
            title="실행 취소 (Ctrl+Z)"
            disabled={!state.past.length}
            onClick={() => travel("undo")}
          >
            실행 취소
          </Button>
          <Button
            size="sm"
            variant="ghost"
            title="다시 실행 (Ctrl+Shift+Z)"
            disabled={!state.future.length}
            onClick={() => travel("redo")}
          >
            다시 실행
          </Button>
          <ThemeMenu />
        </nav>
        <input
          ref={fileInput}
          className="sr-only"
          tabIndex={-1}
          aria-label="전투 맵 JSON 열기"
          type="file"
          accept=".json,application/json"
          onChange={async (e) => {
            const file = e.target.files?.[0];
            e.target.value = "";
            if (!file) return;
            try {
              session.message("파일을 읽는 중…");
              const prepared = await files.read(file);
              replace(() => files.open(prepared), true);
            } catch (error) {
              session.message(
                `열기 실패: ${error instanceof Error ? error.message : String(error)}`,
              );
            }
          }}
        />
      </header>
      <main className="editor-main" data-expanded={expanded}>
        <Toolbox
          session={session}
          state={state}
          tool={tool}
          setTool={chooseTool}
          shape={shape}
          setShape={setShape}
          subdivisions={subdivisions}
          setSubdivisions={setSubdivisions}
        />
        <MapCanvas
          key={state.epoch}
          session={session}
          state={state}
          tool={tool}
          shape={shape}
          setTool={chooseTool}
          subdivisions={subdivisions}
          onStatus={setStatus}
        />
        <Inspector
          state={state}
          session={session}
          confirm={confirm}
          remove={remove}
        />
        <BottomPanel
          state={state}
          session={session}
          tab={tab}
          setTab={setTab}
          expanded={expanded}
          setExpanded={setExpanded}
          diagnostics={diagnostics}
          navigate={navigateDiagnostic}
          remove={remove}
        />
      </main>
      <div className="message-bar">
        <span role="status" aria-label="문서 상태">
          {state.message}
        </span>
        {exported && exported.epoch === state.epoch && (
          <Button
            size="sm"
            variant="outline"
            onClick={() => {
              session.acknowledgeExport(exported.doc, exported.epoch);
              setExported(null);
              session.message(
                "사용자가 다운로드 파일을 확인했습니다. 이후 변경은 별도로 저장하세요.",
              );
            }}
          >
            다운로드 확인
          </Button>
        )}
      </div>
      <footer>
        <span>{toolDefinitions.find((t) => t.id === tool)?.label}</span>
        <span>{status}</span>
        <span>Esc: 미리보기 취소 · 폼 Enter/포커스 이동: 확정</span>
      </footer>
      <Dialog
        open={!!confirmation}
        onOpenChange={(open) => {
          if (!open) setConfirmation(null);
        }}
      >
        <DialogContent
          onCloseAutoFocus={(e) => {
            e.preventDefault();
            const target = focusReturn.current;
            if (target?.isConnected) target.focus();
            else newButton.current?.focus();
          }}
        >
          <DialogTitle>{confirmation?.title}</DialogTitle>
          <DialogDescription>{confirmation?.description}</DialogDescription>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmation(null)}>
              취소
            </Button>
            <Button
              onClick={() => {
                confirmation?.action();
                setConfirmation(null);
              }}
            >
              계속
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
