import { Button } from "@/components/ui/button";
import { Field, Choice } from "./fields";
import { cellPosition } from "./coordinates";
import { moveRoutePoint, paint, resizeMap, setZoneKind } from "./operations";
import {
  layers,
  layerNames,
  type Collection,
  type MapDocument,
  type Terrain,
} from "./model";
import type { EditorSession, EditorState } from "./history";
export type ConfirmAction = (
  title: string,
  description: string,
  action: () => void,
) => void;
export function Inspector({
  state,
  session,
  confirm,
  remove,
}: {
  state: EditorState;
  session: EditorSession;
  confirm: ConfirmAction;
  remove: () => void;
}) {
  const { doc, selection: s, locked, hidden } = state;
  const kind: Collection | null = s && s.kind !== "cell" ? s.kind : null;
  const selected =
    kind && s && "id" in s ? doc[kind].find((o) => o.id === s.id) : undefined;
  const route =
    s?.kind === "routes" ? doc.routes.find((o) => o.id === s.id) : undefined;
  const point =
    s?.kind === "points"
      ? doc.points.find((o) => o.id === s.id)
      : s?.kind === "routes"
        ? route?.points.find((p) => p.id === s.pointId)
        : undefined;
  const zone =
    s?.kind === "zones" ? doc.zones.find((o) => o.id === s.id) : undefined;
  const edit = (fn: (d: MapDocument) => void) => {
    session.commit(fn);
  };
  const resize = (axis: "width" | "height", value: string) => {
    const n = Number(value);
    const action = () => edit((d) => resizeMap(d, axis, n));
    if (n < doc[axis])
      confirm(
        "맵 크기 축소",
        "범위 밖 셀·지형·구역·경로·지점은 보존하고 진단합니다. 자동으로 옮기거나 경로를 다시 작성하지 않습니다.",
        action,
      );
    else action();
  };
  return (
    <aside className="inspector scroll-panel" aria-label="속성과 레이어">
      <div className="panel-heading">
        <h2>속성</h2>
        <Button size="sm" variant="ghost" onClick={() => session.select(null)}>
          맵 정보
        </Button>
      </div>
      <div className="properties" key={`${state.epoch}:${JSON.stringify(s)}`}>
        {!selected && s?.kind !== "cell" && (
          <>
            <Field
              label="맵 이름"
              value={doc.name}
              onCommit={(v) =>
                edit((d) => {
                  d.name = v;
                })
              }
            />
            <Field
              label="설명"
              value={doc.description}
              onCommit={(v) =>
                edit((d) => {
                  d.description = v;
                })
              }
            />
            <div className="two-fields">
              <Field
                label="가로 (셀)"
                value={doc.width}
                numeric
                disabled={locked.has("area")}
                onCommit={(v) => resize("width", v)}
              />
              <Field
                label="세로 (셀)"
                value={doc.height}
                numeric
                disabled={locked.has("area")}
                onCommit={(v) => resize("height", v)}
              />
            </div>
            <p className="hint">
              1~128셀 · 좌측 상단 (0, 0)
              <br />
              오른쪽 +X / 아래쪽 +Y
              <br />
              늘어난 범위는 바로 칠할 수 있습니다. 기존 범위의 빈 영역은
              유지합니다.
            </p>
            <code className="object-id">ID {doc.id}</code>
            <Button
              size="sm"
              variant="outline"
              disabled={locked.has("area")}
              onClick={() =>
                confirm(
                  "범위 밖 유효 셀 정리",
                  "현재 사각 범위 밖 유효 셀만 제거합니다. 지형·구역·경로는 보존되어 진단할 수 있습니다.",
                  () =>
                    edit((d) => {
                      d.valid = d.valid.filter((k) => {
                        const p = cellPosition(k);
                        return p.x < d.width && p.y < d.height;
                      });
                    }),
                )
              }
            >
              범위 밖 유효 셀 정리
            </Button>
          </>
        )}
        {selected && kind && (
          <>
            <h3>{layerNames[kind]}</h3>
            <code className="object-id">ID {selected.id}</code>
            <fieldset disabled={locked.has(kind)}>
              <Field
                label="이름"
                value={selected.name}
                onCommit={(v) =>
                  edit((d) => {
                    const target = d[kind].find((o) => o.id === selected.id);
                    if (target) target.name = v;
                  })
                }
              />
              {zone && (
                <>
                  <Choice
                    label="구역 종류"
                    value={zone.kind}
                    onChange={(v) =>
                      edit((d) => {
                        setZoneKind(d, zone.id, v as "deploy" | "spawn");
                      })
                    }
                  >
                    <option value="deploy">아군 배치 가능</option>
                    <option value="spawn">적 등장</option>
                  </Choice>
                  <p className="hint">
                    {zone.cells.length}셀 · 반대 종류의 구역과 겹칠 수 없습니다.
                    칠하기·종류 변경 시 겹친 셀은 이 구역 종류로 바뀝니다.
                  </p>
                </>
              )}
              {route && (
                <>
                  <p className="hint">
                    경로 도구: 선택한 점 다음에 삽입.
                    <br />
                    선택 도구: 점을 끌어 이동. 선분 선택 후에는 끝에 추가합니다.
                  </p>
                  <div
                    className="object-list"
                    role="group"
                    aria-label="경로 점 목록"
                  >
                    {route.points.map((p, i) => (
                      <Button
                        key={p.id}
                        variant={
                          s?.kind === "routes" && s.pointId === p.id
                            ? "secondary"
                            : "ghost"
                        }
                        size="sm"
                        onClick={() =>
                          session.select({
                            kind: "routes",
                            id: route.id,
                            pointId: p.id,
                          })
                        }
                      >
                        {i + 1}.{" "}
                        {i === 0
                          ? "시작"
                          : i === route.points.length - 1
                            ? "도착"
                            : "경유"}{" "}
                        ({p.x}, {p.y})
                      </Button>
                    ))}
                  </div>
                </>
              )}
              {point && (
                <div className="two-fields">
                  {(["x", "y"] as const).map((axis) => (
                    <Field
                      key={axis}
                      label={`${axis.toUpperCase()} 좌표`}
                      value={point[axis]}
                      numeric
                      onCommit={(v) =>
                        edit((d) => {
                          if (route)
                            moveRoutePoint(d, route.id, point.id, {
                              x: point.x,
                              y: point.y,
                              [axis]: Number(v),
                            });
                          else {
                            const target = d.points.find(
                              (p) => p.id === point.id,
                            );
                            if (target) target[axis] = Number(v);
                          }
                        })
                      }
                    />
                  ))}
                </div>
              )}
              {kind === "slots" && (
                <p className="hint">
                  조건이 참조하는 임시 보스 개체 슬롯입니다. 적 종류·수량과
                  독립적이며 실제 바인딩은 후속 작업입니다.
                </p>
              )}
              {kind === "waves" && (
                <p className="hint">
                  아래 웨이브 패널에서 구성과 발동 조건을 편집합니다.
                </p>
              )}
              <Button variant="outline" size="sm" onClick={remove}>
                {s?.kind === "routes" && s.pointId
                  ? "선택한 경로 점 삭제"
                  : "선택 대상 삭제"}
              </Button>
            </fieldset>
            {locked.has(kind) && (
              <p className="hint">
                잠긴 레이어 · 잠금을 해제하면 수정할 수 있습니다.
              </p>
            )}
          </>
        )}
        {s?.kind === "cell" && (
          <>
            <h3>셀 ({s.key})</h3>
            <p className="hint">
              {doc.valid.includes(s.key) ? "유효 영역" : "영역 없음"} · 지형과
              구역은 별도 정보입니다.
            </p>
            <Choice
              label="셀 지형"
              value={doc.terrain[s.key] ?? "floor"}
              disabled={locked.has("terrain") || !doc.valid.includes(s.key)}
              onChange={(v) => edit((d) => paint(d, [s.key], v as Terrain))}
            >
              <option value="floor">일반 바닥</option>
              <option value="platform">고지대 / 플랫폼</option>
              <option value="obstacle">장애물</option>
            </Choice>
            <Button
              size="sm"
              variant="outline"
              disabled={locked.has("terrain")}
              onClick={() =>
                edit((d) => {
                  delete d.terrain[s.key];
                })
              }
            >
              이 셀의 지형 정보 지우기
            </Button>
          </>
        )}
      </div>
      <details open className="layers">
        <summary>레이어 · 표시 / 잠금</summary>
        <p className="hint">
          숨김은 표시만, 잠금은 모든 편집과 이력 적용을 제한합니다.
        </p>
        {layers.map((l) => (
          <div className="layer-row" key={l}>
            <span>{layerNames[l]}</span>
            <label>
              <input
                type="checkbox"
                aria-label={`${layerNames[l]} 표시`}
                checked={!hidden.has(l)}
                onChange={() => session.toggleLayer(l, "hidden")}
              />
              표시
            </label>
            <label>
              <input
                type="checkbox"
                aria-label={`${layerNames[l]} 잠금`}
                checked={locked.has(l)}
                onChange={() => session.toggleLayer(l, "locked")}
              />
              잠금
            </label>
          </div>
        ))}
      </details>
    </aside>
  );
}
