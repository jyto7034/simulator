import type { Definition, GraphDocument, GraphNode } from "./types";
import { stateConnections } from "./core/connections";

export function StateConnections({
  doc,
  stateId,
  catalog,
  disabled,
  navigate,
}: {
  doc: GraphDocument;
  stateId: string;
  catalog: Definition[];
  disabled: boolean;
  navigate(id: string): void;
}) {
  const { applications, events } = stateConnections(doc, stateId);
  const link = (node: GraphNode) => {
    const title =
      node.title ||
      catalog.find((d) => d.kind === node.kind)?.label ||
      node.kind;
    return (
      <button
        type="button"
        disabled={disabled}
        onClick={() => navigate(node.id)}
        aria-label={`${title} 노드로 이동`}
      >
        {title} <span aria-hidden="true">↗</span>
      </button>
    );
  };
  return (
    <section className="ng-state-connections" aria-label="연결된 상태 구성">
      <strong>연결된 상태 구성</strong>
      <p className="ng-muted">
        그래프에 작성된 연결입니다. 실행 중 남은 시간은 표시하지 않습니다.
      </p>
      <h3>적용 동작 · {applications.length}</h3>
      {applications.length === 0 && (
        <p className="ng-muted">
          이 상태를 적용하는 동작이 연결되지 않았습니다.
        </p>
      )}
      {applications.length > 1 && (
        <p className="ng-muted">
          여러 적용 동작이 연결되어 있습니다. 활성 소유 범위를 확인하세요.
        </p>
      )}
      {applications.map(({ node, duration, profile }) => (
        <div className="ng-state-application" key={node.id}>
          {link(node)}
          <dl>
            <dt>유지 시간</dt>
            <dd>
              {duration ? (
                <>
                  {link(duration)}
                  {duration.kind === "skill-seconds" && (
                    <span className="ng-state-value">
                      {duration.values.value === ""
                        ? "값 미정"
                        : `${duration.values.value}초`}
                    </span>
                  )}
                </>
              ) : (
                "연결 없음"
              )}
            </dd>
            <dt>공격 설정</dt>
            <dd>{profile ? link(profile) : "연결 없음"}</dd>
          </dl>
        </div>
      ))}
      <h3>이 상태를 조회·관찰하는 노드 · {events.length}</h3>
      {events.length ? (
        <ul>
          {events.map((node) => (
            <li key={node.id}>{link(node)}</li>
          ))}
        </ul>
      ) : (
        <p className="ng-muted">연결된 조회·관찰 노드가 없습니다.</p>
      )}
    </section>
  );
}
