import type { RefObject } from "react";
import { radialTools, toolDefinitions, type Tool } from "./tools";
import { RADIAL_RADIUS, type RadialState } from "./useCanvasTools";

const polar = (angle: number, radius: number) => ({
  x: RADIAL_RADIUS + Math.sin(angle) * radius,
  y: RADIAL_RADIUS - Math.cos(angle) * radius,
});
function sector(index: number) {
  const angle = (index * Math.PI) / 4;
  const start = angle - Math.PI / 8;
  const end = angle + Math.PI / 8;
  const a = polar(start, 148),
    b = polar(end, 148);
  const c = polar(end, 49),
    d = polar(start, 49);
  return `M${a.x},${a.y} A148,148 0 0 1 ${b.x},${b.y} L${c.x},${c.y} A49,49 0 0 0 ${d.x},${d.y} Z`;
}
export function RadialTools({
  menu,
  menuRef,
  choose,
  cancel,
  highlight,
}: {
  menu: RadialState;
  menuRef: RefObject<HTMLDivElement | null>;
  choose: (tool: Tool) => void;
  cancel: () => void;
  highlight: (index: number) => void;
}) {
  return (
    <div
      className="radial-backdrop"
      onPointerDown={(event) => {
        if (event.target === event.currentTarget) cancel();
      }}
      onContextMenu={(event) => {
        event.preventDefault();
        cancel();
      }}
    >
      <div
        ref={menuRef}
        className="radial-tools"
        role="menu"
        aria-label="빠른 편집 도구"
        aria-describedby="radial-help"
        tabIndex={-1}
        style={{
          left: menu.center.x - RADIAL_RADIUS,
          top: menu.center.y - RADIAL_RADIUS,
        }}
      >
        <svg viewBox="0 0 300 300" aria-hidden="true">
          {radialTools.map((entry, index) => (
            <path
              key={entry.id}
              d={sector(index)}
              className={menu.active === index ? "active" : ""}
            />
          ))}
        </svg>
        {radialTools.map((entry, index) => {
          const at = polar((index * Math.PI) / 4, 101);
          return (
            <button
              key={entry.id}
              type="button"
              role="menuitem"
              tabIndex={menu.active === index ? 0 : -1}
              className="radial-item"
              data-active={menu.active === index}
              data-tool={entry.id}
              style={{ left: at.x, top: at.y }}
              onFocus={() => highlight(index)}
              onClick={() => choose(entry.id)}
            >
              <span aria-hidden="true">
                {toolDefinitions.find((tool) => tool.id === entry.id)?.glyph}
              </span>
              {entry.label}
            </button>
          );
        })}
        <div className="radial-center" aria-hidden="true">
          <kbd>Q</kbd>
          <span>
            {menu.active === null ? "취소" : radialTools[menu.active].label}
          </span>
        </div>
        <p id="radial-help" className="sr-only">
          Q를 누른 채 방향을 고르고 놓으면 선택합니다. 중앙과 Esc는 취소입니다.
          방향키로 이동하고 Enter로 선택할 수도 있습니다.
        </p>
      </div>
    </div>
  );
}
