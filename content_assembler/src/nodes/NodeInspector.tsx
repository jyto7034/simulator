import { parseFieldValue } from "./core/field-value";
import { useImperativeHandle, useRef, useState, type Ref } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { resonanceFieldActive } from "./core/resonance-catalog";
import { StateConnections } from "./StateConnections";
import { TilePatternEditor } from "./TilePatternEditor";
import { spatialFieldActive } from "./core/spatial-catalog";
import { groundFieldActive } from "./core/ground-catalog";
import { p4FieldActive } from "./core/p4-fields";
import { areaEditorValues, isLegacyArea } from "./core/tile-pattern";
import type {
  Command,
  Definition,
  Endpoint,
  FieldValue,
  Snapshot,
} from "./types";

export type InspectorApi = {
  apply(): Promise<boolean>;
  cancel(): void;
  focusTitle(): void;
};
type Props = {
  state: Snapshot;
  catalog: Definition[];
  selected: string[];
  busy: boolean;
  navigate(id: string): void;
  edit(command: Command): Promise<boolean>;
  pending(value: boolean): void;
  message(message: string): void;
  api: Ref<InspectorApi>;
};
export function NodeInspector({
  state,
  catalog,
  selected,
  busy,
  navigate,
  edit,
  pending,
  message,
  api,
}: Props) {
  const node = state.doc.nodes.find((n) => selected.includes(n.id));
  const frame = state.doc.frames.find((n) => selected.includes(n.id));
  const edge = state.doc.edges.find((n) => selected.includes(n.id));
  const definition = catalog.find((d) => d.kind === node?.kind);
  const titleInput = useRef<HTMLInputElement>(null);
  const [title, setTitle] = useState(
    node?.title ?? frame?.title ?? state.doc.name,
  );
  const [note, setNote] = useState(node?.note ?? "");
  const [values, setValues] = useState<Record<string, string | boolean>>(() =>
    Object.fromEntries(
      Object.entries(node ? areaEditorValues(node) : {}).map(([key, value]) => [
        key,
        typeof value === "boolean" ? value : String(value),
      ]),
    ),
  );
  const [x, setX] = useState(String(node?.position.x ?? 0));
  const [y, setY] = useState(String(node?.position.y ?? 0));
  const [positionDirty, setPositionDirty] = useState(false);
  const [dirty, setDirty] = useState(false);
  const [source, setSource] = useState(edge ? JSON.stringify(edge.source) : "");
  const [target, setTarget] = useState(edge ? JSON.stringify(edge.target) : "");
  const changed = () => {
    setDirty(true);
    pending(true);
  };
  const cancel = () => {
    setTitle(node?.title ?? frame?.title ?? state.doc.name);
    setNote(node?.note ?? "");
    setValues(
      Object.fromEntries(
        Object.entries(node ? areaEditorValues(node) : {}).map(
          ([key, value]) => [
            key,
            typeof value === "boolean" ? value : String(value),
          ],
        ),
      ),
    );
    setX(String(node?.position.x ?? 0));
    setY(String(node?.position.y ?? 0));
    setPositionDirty(false);
    setDirty(false);
    pending(false);
    setSource(edge ? JSON.stringify(edge.source) : "");
    setTarget(edge ? JSON.stringify(edge.target) : "");
  };
  const ports = (output: boolean) =>
    state.doc.nodes.flatMap((n) => {
      const d = catalog.find((d) => d.kind === n.kind);
      return (
        (output ? d?.outputs : d?.inputs)?.map((p) => ({
          key: JSON.stringify({ node: n.id, port: p.id }),
          endpoint: { node: n.id, port: p.id },
          label: `${n.title || d?.label} · ${p.label}`,
          type: p.valueType,
        })) ?? []
      );
    });
  const apply = async () => {
    if (!dirty) return true;
    let command: Command;
    if (edge) {
      const from = ports(true).find((p) => p.key === source);
      const to = ports(false).find((p) => p.key === target);
      if (!from || !to) {
        message("연결할 출력과 입력을 선택하세요.");
        return false;
      }
      command = {
        type: "connect",
        source: from.endpoint,
        target: to.endpoint,
        replaceEdge: edge.id,
      };
    } else if (node && positionDirty) {
      if (
        !x.trim() ||
        !y.trim() ||
        !Number.isFinite(Number(x)) ||
        !Number.isFinite(Number(y))
      ) {
        message("위치에는 유한한 숫자를 입력하세요.");
        return false;
      }
      command = {
        type: "move",
        positions: [{ id: node.id, position: { x: Number(x), y: Number(y) } }],
      };
    } else if (node) {
      const parsed: Record<string, FieldValue> = {};
      try {
        for (const field of definition?.fields ?? [])
          parsed[field.id] = parseFieldValue(field, values[field.id]);
      } catch (error) {
        message(error instanceof Error ? error.message : String(error));
        return false;
      }
      command = {
        type: "setNode",
        id: node.id,
        title,
        note,
        values:
          isLegacyArea(node) && parsed.pattern === ""
            ? { facing: parsed.facing }
            : parsed,
      };
    } else if (frame) command = { type: "renameFrame", id: frame.id, title };
    else command = { type: "rename", name: title };
    const ok = await edit(command);
    if (ok) {
      setDirty(false);
      pending(false);
    }
    return ok;
  };
  useImperativeHandle(api, () => ({
    apply,
    cancel,
    focusTitle: () => titleInput.current?.focus(),
  }));
  const titleFor = (id: string) => {
    const n = state.doc.nodes.find((n) => n.id === id);
    return n?.title || catalog.find((d) => d.kind === n?.kind)?.label || id;
  };
  const portLabel = (endpoint: Endpoint, output: boolean) => {
    const n = state.doc.nodes.find((n) => n.id === endpoint.node);
    const d = catalog.find((d) => d.kind === n?.kind);
    return (
      (output ? d?.outputs : d?.inputs)?.find((p) => p.id === endpoint.port)
        ?.label ?? endpoint.port
    );
  };
  const related = node
    ? state.doc.edges.filter(
        (e) => e.source.node === node.id || e.target.node === node.id,
      )
    : [];
  return (
    <aside className="ng-inspector" aria-label="노드 속성">
      <div className="ng-panel-heading">
        <strong>
          {selected.length > 1
            ? `${selected.length}개 선택`
            : node
              ? "노드 속성"
              : frame
                ? "프레임 속성"
                : edge
                  ? "연결 속성"
                  : "그래프 속성"}
        </strong>
        {definition && <small>{definition.role ?? definition.category}</small>}
      </div>
      {selected.length > 1 ? (
        <p className="ng-muted">
          제목을 끌어 함께 이동하세요. 선택 항목을 복제하거나 프레임으로 묶을 수
          있습니다.
        </p>
      ) : (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void apply();
          }}
        >
          {edge ? (
            <>
              <label>
                출력
                <select
                  aria-label="연결 출력"
                  value={source}
                  disabled={busy}
                  onChange={(e) => {
                    setSource(e.target.value);
                    changed();
                  }}
                >
                  {ports(true).map((p) => (
                    <option key={p.key} value={p.key}>
                      {p.label} ({p.type})
                    </option>
                  ))}
                </select>
              </label>
              <label>
                입력
                <select
                  aria-label="연결 입력"
                  value={target}
                  disabled={busy}
                  onChange={(e) => {
                    setTarget(e.target.value);
                    changed();
                  }}
                >
                  {ports(false).map((p) => (
                    <option key={p.key} value={p.key}>
                      {p.label} ({p.type})
                    </option>
                  ))}
                </select>
              </label>
              <p className="ng-muted">
                연결 끝을 끌거나 여기서 대상을 변경하세요. 한 번의 실행 취소로
                복원됩니다.
              </p>
            </>
          ) : (
            <>
              {node && <p className="ng-node-kind">{definition?.label}</p>}
              {definition?.kind.startsWith("skill-") && (
                <p className="ng-muted">{definition.description}</p>
              )}
              {node?.kind === "skill-state" && (
                <StateConnections
                  doc={state.doc}
                  stateId={node.id}
                  catalog={catalog}
                  disabled={busy || dirty}
                  navigate={navigate}
                />
              )}
              <label>
                {node ? "표시 이름" : frame ? "프레임 이름" : "그래프 이름"}
                <Input
                  ref={titleInput}
                  aria-label={
                    node
                      ? "노드 표시 이름"
                      : frame
                        ? "프레임 이름"
                        : "그래프 이름"
                  }
                  value={title}
                  placeholder={definition?.label}
                  disabled={busy || positionDirty}
                  onChange={(e) => {
                    setTitle(e.target.value);
                    changed();
                  }}
                />
              </label>
              {node && isLegacyArea(node) && (
                <p className="ng-muted" role="status">
                  이전 도형 설정입니다. 타일 범위를 다시 지정하고 변경을
                  적용하세요. 적용 전까지 원래 도형 값은 보존됩니다.
                </p>
              )}
              {node && ["skill-area", "skill-range"].includes(node.kind) && (
                <TilePatternEditor
                  pattern={String(values.pattern ?? "")}
                  includeAnchor={values.includeAnchor === true}
                  disabled={busy || positionDirty}
                  message={message}
                  change={(pattern, includeAnchor) => {
                    setValues((v) => ({ ...v, pattern, includeAnchor }));
                    changed();
                  }}
                />
              )}
              {definition?.fields
                .filter(
                  (field) =>
                    !node ||
                    !["skill-area", "skill-range"].includes(node.kind) ||
                    !["pattern", "includeAnchor"].includes(field.id),
                )
                .filter(
                  (field) =>
                    p4FieldActive({ kind: node?.kind ?? "", values }, field) &&
                    spatialFieldActive(
                      { kind: node?.kind ?? "", values },
                      field,
                    ) &&
                    groundFieldActive(
                      { kind: node?.kind ?? "", values },
                      field,
                    ),
                )
                .filter((field) =>
                  resonanceFieldActive(
                    { kind: node?.kind ?? "", values },
                    field,
                  ),
                )
                .map((field) => (
                  <label key={field.id}>
                    {field.label}
                    {field.options ? (
                      <select
                        aria-label={field.label}
                        value={String(values[field.id])}
                        disabled={busy || positionDirty}
                        onChange={(e) => {
                          setValues((v) => ({
                            ...v,
                            [field.id]: e.target.value,
                          }));
                          changed();
                        }}
                      >
                        {field.options.map((o) => (
                          <option key={o.value} value={o.value}>
                            {o.label}
                          </option>
                        ))}
                      </select>
                    ) : field.valueType === "boolean" ? (
                      <select
                        aria-label={field.label}
                        value={String(values[field.id])}
                        disabled={busy || positionDirty}
                        onChange={(e) => {
                          setValues((v) => ({
                            ...v,
                            [field.id]: e.target.value === "true",
                          }));
                          changed();
                        }}
                      >
                        <option value="true">참</option>
                        <option value="false">거짓</option>
                      </select>
                    ) : field.multiline ? (
                      <textarea
                        aria-label={field.label}
                        value={String(values[field.id] ?? "")}
                        disabled={busy || positionDirty}
                        rows={4}
                        onChange={(e) => {
                          setValues((v) => ({
                            ...v,
                            [field.id]: e.target.value,
                          }));
                          changed();
                        }}
                      />
                    ) : (
                      <Input
                        aria-label={field.label}
                        value={String(values[field.id] ?? "")}
                        inputMode={
                          field.valueType === "number" ||
                          field.valueType === "optional-number"
                            ? field.integer
                              ? "numeric"
                              : "decimal"
                            : undefined
                        }
                        placeholder={
                          field.valueType === "optional-number"
                            ? "미정"
                            : undefined
                        }
                        disabled={busy || positionDirty}
                        onChange={(e) => {
                          setValues((v) => ({
                            ...v,
                            [field.id]: e.target.value,
                          }));
                          changed();
                        }}
                      />
                    )}
                  </label>
                ))}
              {node && (
                <>
                  <label>
                    메모
                    <textarea
                      aria-label="노드 메모"
                      value={note}
                      disabled={busy || positionDirty}
                      onChange={(e) => {
                        setNote(e.target.value);
                        changed();
                      }}
                      rows={3}
                    />
                  </label>
                  <div className="ng-coordinate-fields">
                    <label>
                      X
                      <Input
                        aria-label="노드 X 위치"
                        inputMode="decimal"
                        value={x}
                        disabled={busy || (dirty && !positionDirty)}
                        onChange={(e) => {
                          setX(e.target.value);
                          setPositionDirty(true);
                          changed();
                        }}
                      />
                    </label>
                    <label>
                      Y
                      <Input
                        aria-label="노드 Y 위치"
                        inputMode="decimal"
                        value={y}
                        disabled={busy || (dirty && !positionDirty)}
                        onChange={(e) => {
                          setY(e.target.value);
                          setPositionDirty(true);
                          changed();
                        }}
                      />
                    </label>
                  </div>
                </>
              )}
            </>
          )}
          <div className="ng-form-actions">
            <Button type="submit" size="sm" disabled={busy || !dirty}>
              변경 적용
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={busy || !dirty}
              onClick={cancel}
            >
              취소
            </Button>
          </div>
          {dirty && (
            <p className="ng-draft-hint" role="status">
              변경을 적용하거나 취소하세요.
            </p>
          )}
        </form>
      )}
      {selected.length === 1 && edge && (
        <div className="ng-form-actions">
          <Button
            size="sm"
            variant="outline"
            disabled={busy || dirty}
            onClick={() => void edit({ type: "reroute", edgeId: edge.id })}
          >
            경유 노드 삽입
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={busy || dirty}
            onClick={() => void edit({ type: "disconnect", ids: [edge.id] })}
          >
            연결 해제
          </Button>
        </div>
      )}
      {selected.length === 1 && node && (
        <section className="ng-related">
          <h3>연결 {related.length}</h3>
          {related.length === 0 ? (
            <p className="ng-muted">연결된 포트가 없습니다.</p>
          ) : (
            related.map((e) => (
              <div key={e.id}>
                <span
                  title={`${titleFor(e.source.node)} · ${portLabel(e.source, true)} → ${titleFor(e.target.node)} · ${portLabel(e.target, false)}`}
                >
                  {titleFor(e.source.node)}{" "}
                  <small>{portLabel(e.source, true)}</small>
                  <br />→ {titleFor(e.target.node)}{" "}
                  <small>{portLabel(e.target, false)}</small>
                </span>
                <button
                  disabled={busy || dirty}
                  aria-label={`${titleFor(e.source.node)} ${e.source.port} 연결 해제`}
                  onClick={() => void edit({ type: "disconnect", ids: [e.id] })}
                >
                  해제
                </button>
              </div>
            ))
          )}
        </section>
      )}
      {selected.length === 0 && (
        <section className="ng-related">
          <h3>편집 안내</h3>
          <p className="ng-muted">
            포트 사이를 끌어 연결합니다.
            <br />
            Ctrl + 클릭으로 여러 항목을 선택합니다.
            <br />빈 공간을 끌어 영역 선택합니다.
            <br />
            가운데 버튼으로 화면을 이동합니다.
          </p>
          <p className="ng-muted">
            Ctrl Z 실행 취소
            <br />
            Ctrl Shift Z 다시 실행
            <br />
            Ctrl D 복제 · Ctrl G 프레임
            <br />
            Shift A 노드 추가
          </p>
        </section>
      )}
    </aside>
  );
}
