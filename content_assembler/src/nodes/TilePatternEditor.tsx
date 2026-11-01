import { useState } from "react";
import { Button } from "@/components/ui/button";
import { readTilePattern, togglePatternTile } from "./core/tile-pattern";

type Props = {
  pattern: string;
  includeAnchor: boolean;
  disabled: boolean;
  change(pattern: string, includeAnchor: boolean): void;
  message(message: string): void;
};

export function TilePatternEditor({
  pattern,
  includeAnchor,
  disabled,
  change,
  message,
}: Props) {
  const [view, setView] = useState({ x: 0, y: 0 });
  const { cells, assigned } = readTilePattern(pattern);
  const selected = new Set(cells.map((p) => `${p.x},${p.y}`));
  return (
    <div className="ng-tile-editor" role="group" aria-label="효과 타일 패턴">
      <p className="ng-muted">
        ↑ 패턴 위쪽 · 기준 타일은 ◎입니다. 타일을 눌러 포함/제외합니다.
      </p>
      <div className="ng-tile-pan" role="group" aria-label="패턴 보기 이동">
        {[
          ["왼쪽", -3, 0],
          ["위", 0, -3],
          ["아래", 0, 3],
          ["오른쪽", 3, 0],
        ].map(([label, dx, dy]) => (
          <Button
            key={label}
            type="button"
            size="sm"
            variant="outline"
            aria-label={`타일 보기 ${label}`}
            onClick={() =>
              setView((p) => ({ x: p.x + Number(dx), y: p.y + Number(dy) }))
            }
          >
            {label}
          </Button>
        ))}
      </div>
      <div className="ng-tile-grid">
        {Array.from({ length: 49 }, (_, i) => {
          const x = view.x + (i % 7) - 3;
          const y = view.y + Math.floor(i / 7) - 3;
          const anchor = x === 0 && y === 0;
          const on = anchor
            ? assigned && includeAnchor
            : selected.has(`${x},${y}`);
          return (
            <button
              key={`${x},${y}`}
              type="button"
              className={anchor ? "ng-tile-anchor" : undefined}
              aria-label={`타일 (${x}, ${y})${anchor ? " 기준" : ""}`}
              aria-pressed={on}
              disabled={disabled}
              onClick={() => {
                try {
                  change(
                    togglePatternTile(pattern, { x, y }),
                    anchor ? !includeAnchor : includeAnchor,
                  );
                } catch (e) {
                  message(e instanceof Error ? e.message : String(e));
                }
              }}
            >
              {anchor ? "◎" : on ? "●" : ""}
            </button>
          );
        })}
      </div>
      <p className="ng-muted" aria-live="polite">
        {assigned
          ? `선택 ${cells.length + Number(includeAnchor)}타일`
          : "범위 미정"}
        {` · 보기 중심 (${view.x}, ${view.y})`}
      </p>
      <div className="ng-tile-pan">
        <Button
          type="button"
          size="sm"
          variant="outline"
          onClick={() => setView({ x: 0, y: 0 })}
        >
          기준점 보기
        </Button>
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={disabled || !assigned}
          onClick={() => change("", false)}
        >
          선택 지우기
        </Button>
      </div>
    </div>
  );
}
