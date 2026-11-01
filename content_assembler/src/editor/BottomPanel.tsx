import { Tabs } from "radix-ui";
import { Button } from "@/components/ui/button";
import { WavePanel } from "./WavePanel";
import type { EditorSession, EditorState } from "./history";
import type { Diagnostic } from "./diagnostics";
export function BottomPanel({
  state,
  session,
  tab,
  setTab,
  expanded,
  setExpanded,
  diagnostics,
  navigate,
  remove,
}: {
  state: EditorState;
  session: EditorSession;
  tab: "waves" | "diagnostics";
  setTab: (tab: "waves" | "diagnostics") => void;
  expanded: boolean;
  setExpanded: (expanded: boolean) => void;
  diagnostics: Diagnostic[];
  navigate: (d: Diagnostic) => void;
  remove: () => void;
}) {
  return (
    <Tabs.Root
      className="bottom-panel"
      role="region"
      aria-label="웨이브와 진단"
      value={tab}
      onValueChange={(v) => setTab(v as "waves" | "diagnostics")}
    >
      <div className="bottom-tabs">
        <Tabs.List aria-label="하단 패널">
          <Tabs.Trigger value="waves">
            웨이브 <span>{state.doc.waves.length}</span>
          </Tabs.Trigger>
          <Tabs.Trigger value="diagnostics">
            진단{" "}
            <span>
              {diagnostics.filter((d) => d.severity !== "info").length}
            </span>
          </Tabs.Trigger>
        </Tabs.List>
        <Button
          size="sm"
          variant="ghost"
          onClick={() => setExpanded(!expanded)}
        >
          {expanded ? "패널 축소" : "패널 확장"}
        </Button>
        <span className="hint">
          편집기 프로토타입 · 전투 실행 / 코어 내보내기 미포함
        </span>
      </div>
      <Tabs.Content value="waves">
        <WavePanel session={session} state={state} remove={remove} />
      </Tabs.Content>
      <Tabs.Content
        value="diagnostics"
        className="diagnostic-list scroll-panel"
      >
        {diagnostics.map((d, i) => (
          <button
            className={`diagnostic ${d.severity}`}
            key={`${d.code}:${i}`}
            onClick={() => navigate(d)}
          >
            <span>
              {d.severity === "error"
                ? "오류"
                : d.severity === "warning"
                  ? "주의"
                  : "안내"}
            </span>
            <code>{d.code}</code>
            <span>{d.description}</span>
            <small>
              {d.field} · {d.targetId}
            </small>
          </button>
        ))}
      </Tabs.Content>
    </Tabs.Root>
  );
}
