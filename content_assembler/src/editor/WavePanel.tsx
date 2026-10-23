import { Button } from "@/components/ui/button";
import { Field, Choice, Reference } from "./fields";
import {
  createWave,
  defaultTrigger,
  uid,
  type Trigger,
  type Wave,
} from "./model";
import { duplicateWave } from "./operations";
import type { EditorSession, EditorState } from "./history";
export function WavePanel({
  state,
  session,
  remove,
}: {
  state: EditorState;
  session: EditorSession;
  remove: () => void;
}) {
  const { doc, selection, locked, hidden } = state;
  const wave =
    selection?.kind === "waves"
      ? doc.waves.find((w) => w.id === selection.id)
      : undefined;
  const update = (fn: (w: Wave) => void) =>
    session.commit((d) => {
      const w = d.waves.find((w) => w.id === wave?.id);
      if (w) fn(w);
    });
  const add = () => {
    const w = createWave();
    if (
      session.commit((d) => {
        d.waves.push(w);
      })
    )
      session.select({ kind: "waves", id: w.id });
  };
  const move = (delta: number) => {
    if (!wave) return;
    session.commit((d) => {
      const index = d.waves.findIndex((w) => w.id === wave.id);
      if (index + delta >= 0 && index + delta < d.waves.length) {
        d.waves.splice(index, 1);
        d.waves.splice(index + delta, 0, wave);
      }
    });
  };
  return (
    <div className="wave-layout">
      <div className="wave-list scroll-panel">
        <Button
          size="sm"
          variant="outline"
          disabled={locked.has("waves")}
          onClick={add}
        >
          + 웨이브 추가
        </Button>
        <p className="hint">목록 순서 ≠ 발동 순서</p>
        {!hidden.has("waves") &&
          doc.waves.map((w, i) => (
            <Button
              key={w.id}
              variant={wave?.id === w.id ? "secondary" : "ghost"}
              size="sm"
              className="object-button"
              onClick={() => session.select({ kind: "waves", id: w.id })}
            >
              {i + 1}. {w.name || "(이름 없음)"}
            </Button>
          ))}
        {!doc.waves.length && <p className="hint">웨이브가 없습니다.</p>}
      </div>
      <div
        className="wave-properties scroll-panel"
        key={`${state.epoch}:${wave?.id}`}
      >
        {!wave ? (
          <p className="hint">
            웨이브를 추가하거나 목록에서 선택하세요. 다섯 가지 기본 조건과
            선택적인 대체 발동 시각을 작성할 수 있습니다.
          </p>
        ) : (
          <fieldset disabled={locked.has("waves")}>
            <div className="wave-actions">
              <strong>{wave.name || "이름 없음"}</strong>
              <code className="object-id">{wave.id}</code>
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  let id: string | null = null;
                  if (
                    session.commit((d) => {
                      id = duplicateWave(d, wave.id);
                    }) &&
                    id
                  )
                    session.select({ kind: "waves", id });
                }}
              >
                복제
              </Button>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => move(-1)}
                disabled={doc.waves[0].id === wave.id}
              >
                위로
              </Button>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => move(1)}
                disabled={doc.waves.at(-1)?.id === wave.id}
              >
                아래로
              </Button>
              <Button size="sm" variant="ghost" onClick={remove}>
                삭제
              </Button>
            </div>
            <div className="wave-form">
              <section>
                <h3>구성</h3>
                <Field
                  label="웨이브 이름"
                  value={wave.name}
                  onCommit={(v) =>
                    update((w) => {
                      w.name = v;
                    })
                  }
                />
                <Reference
                  label="등장 구역"
                  value={wave.zoneId}
                  options={doc.zones.filter((z) => z.kind === "spawn")}
                  onChange={(v) =>
                    update((w) => {
                      w.zoneId = v;
                    })
                  }
                />
                <Reference
                  label="이동 경로"
                  value={wave.routeId}
                  options={doc.routes}
                  onChange={(v) =>
                    update((w) => {
                      w.routeId = v;
                    })
                  }
                />
                <Field
                  label="우선순위"
                  value={wave.priority}
                  numeric
                  onCommit={(v) =>
                    update((w) => {
                      w.priority = Number(v);
                    })
                  }
                />
                <label className="check-label">
                  <input
                    type="checkbox"
                    checked={wave.required}
                    onChange={(e) =>
                      update((w) => {
                        w.required = e.target.checked;
                      })
                    }
                  />
                  승리 판정에 필요한 웨이브
                </label>
              </section>
              <section data-field="enemies">
                <h3>적 구성 · 참조 ID 직접 입력</h3>
                {wave.enemies.map((enemy, i) => (
                  <div className="enemy-entry" key={enemy.id}>
                    <Field
                      label={`적 ${i + 1} 콘텐츠 ID`}
                      value={enemy.contentId}
                      onCommit={(v) =>
                        update((w) => {
                          w.enemies[i].contentId = v;
                        })
                      }
                    />
                    <div className="two-fields">
                      <Field
                        label={`적 ${i + 1} 수량`}
                        value={enemy.count}
                        numeric
                        onCommit={(v) =>
                          update((w) => {
                            w.enemies[i].count = Number(v);
                          })
                        }
                      />
                      <Field
                        label={`적 ${i + 1} 간격 (초)`}
                        value={enemy.intervalMs}
                        scale={1000}
                        numeric
                        onCommit={(v) =>
                          update((w) => {
                            w.enemies[i].intervalMs = Number(v);
                          })
                        }
                      />
                    </div>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() =>
                        update((w) => {
                          w.enemies.splice(i, 1);
                        })
                      }
                    >
                      적 {i + 1} 제거
                    </Button>
                  </div>
                ))}
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() =>
                    update((w) => {
                      w.enemies.push({
                        id: uid(),
                        contentId: "",
                        count: 1,
                        intervalMs: 1000,
                      });
                    })
                  }
                >
                  적 구성 추가
                </Button>
                <p className="hint">콘텐츠 ID의 존재 여부는 미검증입니다.</p>
              </section>
              <section data-field="trigger">
                <h3>발동 조건</h3>
                <Choice
                  label="기본 조건"
                  value={wave.trigger.kind}
                  onChange={(v) =>
                    update((w) => {
                      w.trigger = defaultTrigger(v as Trigger["kind"]);
                      if (v === "time") w.fallbackMs = null;
                    })
                  }
                >
                  <option value="time">전투 시간</option>
                  <option value="clear">선행 웨이브 정리</option>
                  <option value="hp">지정 개체 HP</option>
                  <option value="death">지정 개체 사망</option>
                  <option value="reach">지정 개체의 지점 도달</option>
                </Choice>
                {wave.trigger.kind === "time" && (
                  <Field
                    label="전투 시작부터 (초)"
                    value={wave.trigger.timeMs}
                    numeric
                    scale={1000}
                    onCommit={(v) =>
                      update((w) => {
                        if (w.trigger.kind === "time")
                          w.trigger.timeMs = Number(v);
                      })
                    }
                  />
                )}
                {wave.trigger.kind === "clear" && (
                  <>
                    <Reference
                      label="선행 웨이브"
                      value={wave.trigger.waveId}
                      options={doc.waves}
                      onChange={(v) =>
                        update((w) => {
                          if (w.trigger.kind === "clear") w.trigger.waveId = v;
                        })
                      }
                    />
                    <p className="hint">
                      구성원이 등장한 뒤 모두 비활성 상태. 사망 외 제거도
                      포함합니다.
                    </p>
                  </>
                )}
                {"slotId" in wave.trigger && (
                  <Reference
                    label="임시 개체 슬롯"
                    value={wave.trigger.slotId}
                    options={doc.slots}
                    onChange={(v) =>
                      update((w) => {
                        if ("slotId" in w.trigger) w.trigger.slotId = v;
                      })
                    }
                  />
                )}
                {wave.trigger.kind === "hp" && (
                  <Field
                    label="HP 이하 (%)"
                    value={wave.trigger.percent}
                    numeric
                    onCommit={(v) =>
                      update((w) => {
                        if (w.trigger.kind === "hp")
                          w.trigger.percent = Number(v);
                      })
                    }
                  />
                )}
                {wave.trigger.kind === "death" && (
                  <p className="hint">
                    개체의 사망을 뜻합니다. 단순 퇴장과 구분합니다.
                  </p>
                )}
                {wave.trigger.kind === "reach" && (
                  <>
                    <Reference
                      label="도달 지점"
                      value={wave.trigger.pointId}
                      options={doc.points}
                      onChange={(v) =>
                        update((w) => {
                          if (w.trigger.kind === "reach") w.trigger.pointId = v;
                        })
                      }
                    />
                    <Field
                      label="도달 반경 (셀)"
                      value={wave.trigger.radius}
                      numeric
                      onCommit={(v) =>
                        update((w) => {
                          if (w.trigger.kind === "reach")
                            w.trigger.radius = Number(v);
                        })
                      }
                    />
                  </>
                )}
                {wave.trigger.kind !== "time" && (
                  <>
                    <label className="check-label">
                      <input
                        type="checkbox"
                        checked={wave.fallbackMs !== null}
                        onChange={(e) =>
                          update((w) => {
                            w.fallbackMs = e.target.checked ? 45000 : null;
                          })
                        }
                      />
                      대체 발동 시각 사용
                    </label>
                    {wave.fallbackMs !== null && (
                      <Field
                        label="대체 발동 시각 (초)"
                        value={wave.fallbackMs}
                        scale={1000}
                        numeric
                        onCommit={(v) =>
                          update((w) => {
                            w.fallbackMs = Number(v);
                          })
                        }
                      />
                    )}
                  </>
                )}
                <p className="hint">
                  전투 시작 기준 절대 시각. 기본 조건 또는 대체 시각으로 한 번
                  발동합니다. 동시 충족 시 기본 조건 우선. 대체 시각은
                  만료·취소가 아닙니다.
                </p>
              </section>
            </div>
          </fieldset>
        )}
      </div>
    </div>
  );
}
