import {
  useEffect,
  useMemo,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import {
  ChevronDown,
  CircleHelp,
  ListTree,
  Plus,
  Search,
  Workflow,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ThemeMenu } from "@/components/theme-menu";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  blockDefinitions,
  createSkill,
  eventNames,
  editBlockSelection,
  id,
  insertBlock,
  makeBlock,
  makeRoutine,
  removeSelection,
  selectedRoutine,
  scopeNames,
  type BlockKind,
  type BlockEditAction,
  type SkillDocument,
  type Selection,
} from "./model";
import { exampleSkill } from "./examples";
import { SkillSession } from "./session";
import { SkillFiles } from "./files";
import { diagnoseSkill, type SkillDiagnostic } from "./validation";
import { SkillGraph } from "./SkillGraph";
import { SkillInspector } from "./SkillInspector";

type Confirmation = { title: string; description: string; action: () => void };
export function SkillEditor({ active }: { active: boolean }) {
  const [session] = useState(() => {
    const doc = exampleSkill("butterfly");
    const session = new SkillSession(doc);
    session.select({ type: "routine", id: doc.routines[2].id });
    return session;
  });
  const [files] = useState(() => new SkillFiles(session));
  const state = useSyncExternalStore(session.subscribe, session.getSnapshot);
  const [search, setSearch] = useState("");
  const [libraryOpen, setLibraryOpen] = useState(false);
  const [diagnosticsOpen, setDiagnosticsOpen] = useState(false);
  const [picker, setPicker] = useState<{
    selection: Selection;
    epoch: number;
    revision: number;
  } | null>(null);
  const pickerReturn = useRef<HTMLElement | null>(null);
  const [confirmation, setConfirmation] = useState<Confirmation | null>(null);
  const [exported, setExported] = useState<{
    doc: SkillDocument;
    epoch: number;
  } | null>(null);
  const [saving, setSaving] = useState(false);
  const fileInput = useRef<HTMLInputElement>(null);
  const root = useRef<HTMLDivElement>(null);
  const focusReturn = useRef<HTMLElement | null>(null);
  const diagnostics = useMemo(() => diagnoseSkill(state.doc), [state.doc]);
  const routine = selectedRoutine(state.doc, state.selection);
  const palette = Object.entries(blockDefinitions).filter(([, d]) =>
    `${d.label} ${d.group}`.includes(search.trim()),
  ) as [BlockKind, (typeof blockDefinitions)[BlockKind]][];
  const groups = [...new Set(palette.map(([, d]) => d.group))];
  const confirm = (title: string, description: string, action: () => void) => {
    focusReturn.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    const { epoch, revision } = session.getSnapshot();
    setConfirmation({
      title,
      description,
      action: () => {
        const current = session.getSnapshot();
        if (current.epoch !== epoch || current.revision !== revision) {
          session.message(
            "확인하는 동안 작업이 바뀌었습니다. 현재 작업을 보존했습니다.",
          );
          return;
        }
        action();
      },
    });
  };
  const replace = (action: () => void) => {
    const apply = () => {
      try {
        action();
        setExported(null);
        session.message("스킬 문서를 열었습니다. 설계를 이어가세요.");
      } catch (e) {
        session.message(`열기 실패: ${String(e)}`);
      }
    };
    if (session.isDirty())
      confirm(
        "스킬 문서를 교체할까요?",
        "현재 스킬에는 파일 확인 이후 변경이 있습니다. 보관하려면 취소 후 저장하세요. 맵 문서는 유지됩니다.",
        apply,
      );
    else apply();
  };
  const save = async () => {
    if (files.busy) return;
    setSaving(true);
    setExported(null);
    const output = await files.save();
    setSaving(false);
    if (output && output.epoch === session.getSnapshot().epoch)
      setExported(output);
  };
  const remove = (selection: Selection = session.getSnapshot().selection) => {
    if (!["block", "routine", "state"].includes(selection.type)) return;
    confirm(
      "선택 항목을 삭제할까요?",
      "하위 블록도 함께 삭제됩니다. 상태를 가리키는 참조는 보존하여 진단에 표시합니다. 실행 취소로 복원할 수 있습니다.",
      () =>
        session.commit((d) => {
          removeSelection(d, selection);
        }),
    );
  };
  const navigate = (diagnostic: SkillDiagnostic) => {
    setDiagnosticsOpen(true);
    session.select(diagnostic.selection);
    requestAnimationFrame(() => {
      const inspector = root.current?.querySelector(".skill-inspector");
      const field = [
        ...(inspector?.querySelectorAll<HTMLElement>("[data-field]") ?? []),
      ].find((el) => el.dataset.field === diagnostic.field);
      (
        field?.querySelector<HTMLElement>("input,select") ??
        inspector?.querySelector<HTMLElement>("input,select")
      )?.focus();
    });
  };
  useEffect(() => {
    const unload = (event: BeforeUnloadEvent) => {
      if (session.isDirty()) {
        event.preventDefault();
        event.returnValue = "";
      }
    };
    window.addEventListener("beforeunload", unload);
    return () => window.removeEventListener("beforeunload", unload);
  }, [session]);
  useEffect(() => {
    const key = (event: KeyboardEvent) => {
      if (
        !active ||
        confirmation ||
        picker ||
        event.isComposing ||
        (event.target instanceof Element &&
          event.target.closest(
            'input,textarea,select,[contenteditable="true"],[role="dialog"],[role="menu"]',
          ))
      )
        return;
      if (event.ctrlKey || event.metaKey) {
        const k = event.key.toLowerCase();
        if (["z", "y", "s", "o"].includes(k)) event.preventDefault();
        if (k === "z") session.travel(event.shiftKey ? "redo" : "undo");
        if (k === "y") session.travel("redo");
        if (k === "s") void save();
        if (k === "o") fileInput.current?.click();
      } else if (event.key === "Delete") {
        event.preventDefault();
        remove();
      }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  });
  const loadExample = (name: "butterfly" | "hood") =>
    replace(() => {
      const doc = exampleSkill(name);
      session.replace(doc, false);
      session.select({ type: "routine", id: doc.routines[2].id });
    });
  const select = (s: Selection) => {
    session.select(s);
    setLibraryOpen(false);
  };
  const addRoutine = () => {
    session.commit((d) => {
      const r = makeRoutine();
      d.routines.push(r);
      return { type: "routine", id: r.id };
    });
  };
  const add = (selection: Selection) => {
    pickerReturn.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    session.select(selection);
    setSearch("");
    const { epoch, revision } = session.getSnapshot();
    setPicker({ selection, epoch, revision });
  };
  const edit = (selection: Selection, action: BlockEditAction | "delete") => {
    if (action === "delete") remove(selection);
    else session.commit((d) => editBlockSelection(d, selection, action));
  };
  return (
    <div className="editor-shell skill-shell" ref={root}>
      <header className="editor-header">
        <div className="brand">
          <Workflow size={18} aria-hidden="true" />
          <h1>스킬 조립기</h1>
        </div>
        <div className="document-title" title={state.doc.name}>
          <span className="document-name">{state.doc.name || "이름 없음"}</span>
          <span className="dirty" data-testid="skill-dirty">
            {session.isDirty() ? " • 미저장" : " · 파일 확인됨"}
          </span>
        </div>
        <nav aria-label="스킬 문서 작업">
          <Button
            size="sm"
            variant="ghost"
            onClick={() => replace(() => session.replace(createSkill(), false))}
          >
            새로 만들기
          </Button>
          <DropdownMenu modal={false}>
            <DropdownMenuTrigger asChild>
              <Button size="sm" variant="ghost">
                예제
                <ChevronDown size={12} />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              <DropdownMenuItem onSelect={() => loadExample("butterfly")}>
                죽나장 예제
              </DropdownMenuItem>
              <DropdownMenuItem onSelect={() => loadExample("hood")}>
                두건 예제
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => fileInput.current?.click()}
          >
            열기
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={saving}
            onClick={() => void save()}
          >
            저장
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={!state.past.length}
            onClick={() => session.travel("undo")}
          >
            실행 취소
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={!state.future.length}
            onClick={() => session.travel("redo")}
          >
            다시 실행
          </Button>
          <ThemeMenu />
        </nav>
        <input
          ref={fileInput}
          type="file"
          accept=".json,application/json"
          className="sr-only"
          tabIndex={-1}
          aria-label="스킬 JSON 열기"
          onChange={async (e) => {
            const file = e.target.files?.[0];
            e.target.value = "";
            if (!file) return;
            try {
              const prepared = await files.read(file);
              replace(() => files.open(prepared));
            } catch (error) {
              session.message(`열기 실패: ${String(error)}`);
            }
          }}
        />
      </header>
      <main className="skill-main" data-diagnostics-open={diagnosticsOpen}>
        <aside
          hidden={!libraryOpen}
          className="skill-library scroll-panel"
          aria-label="스킬 구성과 블록"
        >
          <section className="skill-structure">
            <div className="skill-section-title">
              <h2>스킬 구성</h2>
              <span className="skill-muted">설계 초안</span>
            </div>
            <button
              className="skill-tree-item"
              aria-pressed={state.selection.type === "document"}
              onClick={() => select({ type: "document" })}
            >
              <ListTree size={14} aria-hidden="true" />
              기본 설정
            </button>
            <div className="skill-section-title">
              <h3>
                상태 <span>{state.doc.states.length}</span>
              </h3>
              <Button
                size="icon-xs"
                variant="ghost"
                aria-label="상태 추가"
                onClick={() =>
                  session.commit((d) => {
                    const key = id();
                    d.states.push({
                      id: key,
                      name: "새 상태",
                      scope: "undecided",
                      lifetime: "",
                      initial: "",
                    });
                    return { type: "state", id: key };
                  })
                }
              >
                <Plus />
              </Button>
            </div>
            {!state.doc.states.length && (
              <p className="hint">스택·자원 등 유지할 값을 선언하세요.</p>
            )}
            {state.doc.states.map((s) => (
              <button
                key={s.id}
                className="skill-tree-item"
                aria-pressed={
                  state.selection.type === "state" &&
                  state.selection.id === s.id
                }
                onClick={() => select({ type: "state", id: s.id })}
              >
                <span className="skill-state-dot" />
                <span>
                  {s.name || "이름 없는 상태"}
                  <small>{scopeNames[s.scope]}</small>
                </span>
              </button>
            ))}
            <div className="skill-section-title">
              <h3>
                동작 <span>{state.doc.routines.length}</span>
              </h3>
              <Button
                size="icon-xs"
                variant="ghost"
                aria-label="동작 추가"
                onClick={() =>
                  session.commit((d) => {
                    const r = makeRoutine();
                    d.routines.push(r);
                    return { type: "routine", id: r.id };
                  })
                }
              >
                <Plus />
              </Button>
            </div>
            {state.doc.routines.map((r) => (
              <button
                key={r.id}
                className="skill-tree-item"
                aria-pressed={routine?.id === r.id}
                onClick={() => select({ type: "routine", id: r.id })}
              >
                <Workflow size={13} aria-hidden="true" />
                <span>
                  {r.name || "이름 없는 동작"}
                  <small>{eventNames[r.event]}</small>
                </span>
              </button>
            ))}
          </section>
        </aside>
        <section className="skill-workarea" aria-label="스킬 조립 영역">
          <div className="skill-workarea-heading">
            <Button
              size="sm"
              variant="outline"
              aria-expanded={libraryOpen}
              onClick={() => setLibraryOpen(!libraryOpen)}
            >
              <ListTree size={14} />
              스킬 구성
            </Button>
            <h2>전체 흐름</h2>
            <span className="skill-muted">
              노드 선택 → 속성 편집 · 묶음 펼치기
            </span>
            <Button
              size="sm"
              variant="outline"
              aria-label="블록 추가"
              disabled={!routine}
              onClick={() => add(state.selection)}
            >
              <Plus size={14} />
              블록 추가
            </Button>
          </div>
          <SkillGraph
            key={state.epoch}
            doc={state.doc}
            selection={state.selection}
            diagnostics={diagnostics}
            active={active}
            select={select}
            add={add}
            navigate={navigate}
            addRoutine={addRoutine}
            edit={edit}
          />
        </section>
        <SkillInspector
          state={state}
          session={session}
          remove={() => remove()}
          add={add}
          edit={edit}
        />
        <section className="skill-diagnostics" aria-label="스킬 진단">
          <div className="skill-diagnostic-heading">
            <button
              aria-expanded={diagnosticsOpen}
              onClick={() => setDiagnosticsOpen(!diagnosticsOpen)}
            >
              작성 진단 <span>{diagnostics.length}</span>
              <ChevronDown size={12} />
            </button>
            <span>미지정·참조 확인 · 실행 검증 전</span>
          </div>
          <div
            className="skill-diagnostic-list scroll-panel"
            hidden={!diagnosticsOpen}
          >
            {diagnostics.map((d) => (
              <button
                key={`${JSON.stringify(d.selection)}-${d.field}-${d.code}`}
                onClick={() => navigate(d)}
              >
                <CircleHelp size={13} aria-hidden="true" />
                <span>{d.message}</span>
              </button>
            ))}
            {!diagnostics.length && (
              <p className="hint">
                미지정 항목이 없습니다. 수식·게임 규칙·Core 실행은 아직 검증하지
                않았습니다.
              </p>
            )}
          </div>
        </section>
      </main>
      <div className="message-bar">
        <span role="status" aria-label="스킬 문서 상태">
          {state.message}
        </span>
        {exported && exported.epoch === state.epoch && (
          <Button
            size="sm"
            variant="outline"
            onClick={() => {
              session.acknowledge(exported.doc, exported.epoch);
              setExported(null);
              session.message(
                "다운로드 파일 확인됨. 이후 변경은 별도로 저장하세요.",
              );
            }}
          >
            다운로드 확인
          </Button>
        )}
      </div>
      <footer>
        <span>설계 문서 · JSON 저장</span>
        <span>Core 미연동 · 전투 실행 및 RON 내보내기 미지원</span>
        <span>Ctrl+Z 실행 취소 · Ctrl+S 저장</span>
      </footer>
      <Dialog
        open={!!picker}
        onOpenChange={(open) => {
          if (!open) setPicker(null);
        }}
      >
        <DialogContent
          className="skill-picker"
          onCloseAutoFocus={(e) => {
            e.preventDefault();
            if (pickerReturn.current?.isConnected) pickerReturn.current.focus();
            else
              root.current
                ?.querySelector<HTMLElement>('button[aria-label="블록 추가"]')
                ?.focus();
          }}
        >
          <DialogTitle>블록 추가</DialogTitle>
          <DialogDescription>
            선택한 위치에 추가할 동작을 고르세요.
          </DialogDescription>
          <section className="skill-palette" aria-label="블록 목록">
            <div className="skill-section-title">
              <h2>블록 목록</h2>
              <Plus size={13} aria-hidden="true" />
            </div>
            <div className="skill-search">
              <Search size={13} aria-hidden="true" />
              <Input
                aria-label="블록 검색"
                placeholder="블록 찾기"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
            </div>
            <p className="hint">
              {routine
                ? "블록을 눌러 선택한 위치에 추가"
                : "먼저 동작을 선택하세요"}
            </p>
            {groups.map((group) => (
              <div className="skill-palette-group" key={group}>
                <h3>{group}</h3>
                {palette
                  .filter(([, d]) => d.group === group)
                  .map(([kind, def]) => (
                    <button
                      key={kind}
                      disabled={!routine}
                      data-group={group}
                      aria-label={`${def.label} 블록 추가`}
                      onClick={() => {
                        const current = session.getSnapshot();
                        if (
                          picker &&
                          picker.epoch === current.epoch &&
                          picker.revision === current.revision
                        ) {
                          session.commit((d) =>
                            insertBlock(d, picker.selection, makeBlock(kind)),
                          );
                        } else
                          session.message(
                            "추가 위치가 바뀌었습니다. 현재 작업을 보존했습니다.",
                          );
                        setPicker(null);
                      }}
                    >
                      <span>{def.label}</span>
                      <Plus size={12} aria-hidden="true" />
                    </button>
                  ))}
              </div>
            ))}
            {!palette.length && <p className="hint">검색 결과가 없습니다.</p>}
          </section>
        </DialogContent>
      </Dialog>
      <Dialog
        open={!!confirmation}
        onOpenChange={(open) => {
          if (!open) setConfirmation(null);
        }}
      >
        <DialogContent
          onCloseAutoFocus={(e) => {
            e.preventDefault();
            if (focusReturn.current?.isConnected) focusReturn.current.focus();
            else root.current?.querySelector<HTMLElement>("button")?.focus();
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
