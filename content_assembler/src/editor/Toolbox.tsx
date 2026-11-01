import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Choice } from "./fields";
import { toolDefinitions, type Tool } from "./tools";
import type { EditorSession, EditorState } from "./history";
import { uid, type Collection } from "./model";
export function Toolbox({
  session,
  state,
  tool,
  setTool,
  shape,
  setShape,
  subdivisions,
  setSubdivisions,
}: {
  session: EditorSession;
  state: EditorState;
  tool: Tool;
  setTool: (t: Tool) => void;
  shape: string;
  setShape: (v: "brush" | "rectangle") => void;
  subdivisions: number | null;
  setSubdivisions: (n: number | null) => void;
}) {
  const { doc, selection, locked, hidden } = state;
  const add = (
    kind: "zones" | "routes" | "slots",
    zoneKind: "deploy" | "spawn" = "deploy",
  ) => {
    const id = uid();
    if (
      session.commit((d) => {
        if (kind === "zones")
          d.zones.push({
            id,
            name: `${zoneKind === "deploy" ? "배치" : "등장"} 구역 ${d.zones.length + 1}`,
            kind: zoneKind,
            cells: [],
          });
        if (kind === "routes")
          d.routes.push({
            id,
            name: `경로 ${d.routes.length + 1}`,
            points: [],
          });
        if (kind === "slots")
          d.slots.push({ id, name: `개체 슬롯 ${d.slots.length + 1}` });
      })
    ) {
      session.select({ kind, id });
      if (kind === "zones") setTool("zone-add");
      if (kind === "routes") setTool("route");
    }
  };
  const list = (kind: Collection) =>
    !hidden.has(kind) && (
      <div className="object-list">
        {doc[kind].map((o) => (
          <Button
            key={o.id}
            size="sm"
            variant={
              selection?.kind === kind && selection.id === o.id
                ? "secondary"
                : "ghost"
            }
            className="object-button"
            title={o.name}
            onClick={() => session.select({ kind, id: o.id })}
          >
            {o.name || "(이름 없음)"}
          </Button>
        ))}
        {!doc[kind].length && <p className="hint">아직 없습니다.</p>}
      </div>
    );
  return (
    <aside className="toolbox scroll-panel" aria-label="도구와 대상 목록">
      <h2>편집 도구</h2>
      <div className="tool-list">
        {toolDefinitions
          .filter((t) => !t.radialOnly)
          .map((t) => (
            <Tooltip key={t.id}>
              <TooltipTrigger asChild>
                <Button
                  size="sm"
                  variant={tool === t.id ? "secondary" : "ghost"}
                  aria-pressed={tool === t.id}
                  onClick={() => setTool(t.id)}
                >
                  <span aria-hidden="true" className="tool-glyph">
                    {t.glyph}
                  </span>
                  <span>{t.label}</span>
                  <kbd>{t.key}</kbd>
                </Button>
              </TooltipTrigger>
              <TooltipContent side="right">
                {t.label} · {t.key}
              </TooltipContent>
            </Tooltip>
          ))}
      </div>
      <Choice
        label="범위 방식"
        value={shape}
        onChange={(v) => setShape(v as "brush" | "rectangle")}
      >
        <option value="brush">브러시</option>
        <option value="rectangle">사각형</option>
      </Choice>
      {tool === "remove" && (
        <p className="hint">
          제거한 셀의 지형·구역·경로는 보존하고 진단합니다.
        </p>
      )}
      <Choice
        label="경로·지점 입력 스냅"
        value={subdivisions === null ? "center" : String(subdivisions)}
        onChange={(v) => setSubdivisions(v === "center" ? null : Number(v))}
      >
        <option value="center">기본 · 타일 중심</option>
        <option value="2">고급 · 2 × 2</option>
        <option value="4">고급 · 4 × 4</option>
        <option value="8">고급 · 8 × 8</option>
      </Choice>
      <p className="hint">
        고급: 셀 내부 분할선 교점.
        <br />
        중심 포함 · 기존 좌표 유지.
      </p>
      <details open>
        <summary>구역</summary>
        <div className="two-fields">
          <Button
            size="sm"
            variant="outline"
            disabled={locked.has("zones")}
            onClick={() => add("zones", "deploy")}
          >
            + 배치 구역
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={locked.has("zones")}
            onClick={() => add("zones", "spawn")}
          >
            + 등장 구역
          </Button>
        </div>
        {list("zones")}
      </details>
      <details open>
        <summary>경로 / 지점</summary>
        <Button
          size="sm"
          variant="outline"
          disabled={locked.has("routes")}
          onClick={() => add("routes")}
        >
          + 경로 만들기
        </Button>
        {list("routes")}
        {list("points")}
      </details>
      <details>
        <summary>임시 개체 슬롯</summary>
        <Button
          size="sm"
          variant="outline"
          disabled={locked.has("slots")}
          onClick={() => add("slots")}
        >
          + 개체 슬롯
        </Button>
        {list("slots")}
      </details>
    </aside>
  );
}
