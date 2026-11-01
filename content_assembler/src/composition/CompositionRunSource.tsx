import { useEffect, useMemo, useRef, useState } from "react";
import { Button } from "../components/ui/button";
import { modelLocation, parseSource } from "./source";
import type { SourceTrace } from "./execution";

/** Navigation always belongs to the frozen run source, including after edits. */
export function CompositionRunSource({
  source,
  currentSource,
  origin,
}: {
  source: string;
  currentSource: string;
  origin: SourceTrace;
}) {
  const parsed = useMemo(() => parseSource(source), [source]);
  const [selected, setSelected] = useState<{ path: string } | null>(null);
  const input = useRef<HTMLTextAreaElement>(null);
  useEffect(() => {
    if (!selected || !input.current) return;
    const span = modelLocation(parsed, selected.path);
    if (!span) return;
    input.current.focus();
    input.current.setSelectionRange(span.start, span.end);
    const lineHeight =
      Number.parseFloat(getComputedStyle(input.current).lineHeight) || 18;
    input.current.scrollTop = Math.max(
      0,
      (span.line - 1) * lineHeight - input.current.clientHeight / 2,
    );
    input.current.scrollIntoView({ block: "nearest" });
  }, [selected, parsed]);
  const locations = [
    { label: "정의", path: origin.definition },
    ...origin.calls.map((path, i) => ({ label: `호출 ${i + 1}`, path })),
    ...origin.inputs.map((path, i) => ({ label: `입력 ${i + 1}`, path })),
  ];
  return (
    <details className="composition-run-source">
      <summary>실행 원본의 정의·호출 위치</summary>
      <p>
        {source === currentSource
          ? "현재 편집기와 같은 원본입니다. 아래 원본은 읽기 전용입니다."
          : "현재 편집기와 다른 원본입니다. 실행 당시 원본을 읽기 전용으로 표시합니다."}
      </p>
      <div className="composition-run-locations">
        {locations.map(({ label, path }, index) => (
          <Button
            key={`${index}:${path}`}
            size="sm"
            variant="outline"
            onClick={() => setSelected({ path })}
            title={path}
          >
            {label} 보기
          </Button>
        ))}
      </div>
      {selected && (
        <>
          <p>{selected.path}</p>
          <textarea
            ref={input}
            readOnly
            spellCheck={false}
            value={source}
            rows={8}
            aria-label="실행 당시 스킬 조합 원본"
          />
        </>
      )}
    </details>
  );
}
