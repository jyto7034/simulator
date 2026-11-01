import { Button } from "@/components/ui/button";
import type { EditorSession, EditorState } from "./history";
import { toolDefinitions, type Tool } from "./tools";

export function CanvasTargets({
  tool,
  state,
  session,
  setTool,
  temporary,
}: {
  tool: Tool;
  state: EditorState;
  session: EditorSession;
  setTool: (tool: Tool) => void;
  temporary: boolean;
}) {
  const zones =
    tool === "deploy" || tool === "spawn" || tool.startsWith("zone-");
  const kind = zones ? "zones" : tool === "route" ? "routes" : null;
  const options =
    kind === "zones"
      ? state.doc.zones.filter(
          (zone) =>
            (tool !== "deploy" && tool !== "spawn") || zone.kind === tool,
        )
      : kind === "routes"
        ? state.doc.routes
        : [];
  const selected = state.selection;
  const value =
    selected?.kind === kind &&
    "id" in selected &&
    options.some((option) => option.id === selected.id)
      ? selected.id
      : "";
  const label =
    tool === "deploy"
      ? "배치 구역"
      : tool === "spawn"
        ? "등장 구역"
        : zones
          ? "구역"
          : "경로";
  const blocked = !!kind && (state.hidden.has(kind) || state.locked.has(kind));
  return (
    <div className="canvas-tools">
      <span className="current-tool">
        {toolDefinitions.find((entry) => entry.id === tool)?.label}
        {temporary && " · 임시"}
      </span>
      {kind ? (
        <>
          <select
            aria-label={`편집할 ${label}`}
            value={value}
            disabled={blocked}
            onChange={(event) =>
              session.select(
                event.target.value ? { kind, id: event.target.value } : null,
              )
            }
          >
            <option value="">
              {tool === "zone-add" || tool === "zone-remove"
                ? "대상 선택"
                : `새 ${label} · 첫 편집으로 생성`}
            </option>
            {options.map((option) => (
              <option key={option.id} value={option.id}>
                {option.name || "(이름 없음)"}
              </option>
            ))}
          </select>
          {tool !== "zone-remove" && (
            <Button
              size="sm"
              variant="outline"
              disabled={blocked}
              onClick={() => {
                if (tool === "zone-add") setTool("deploy");
                session.select(null);
              }}
            >
              + 새 {label}
            </Button>
          )}
          {blocked && <span className="hint">표시·잠금 확인</span>}
        </>
      ) : (
        <span
          className="hint"
          title="Q 도구 · 좌클릭 적용 · 우클릭 제거 · Shift 사각형 · Space 이동 · D 임시 지우기"
        >
          Q 도구 · 우클릭 제거 · Shift 사각형
        </span>
      )}
    </div>
  );
}
