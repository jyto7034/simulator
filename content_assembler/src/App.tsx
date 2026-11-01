import { lazy, Suspense, useState } from "react";
import { Braces, Map, Workflow } from "lucide-react";
import MapEditor from "./MapEditor";
const SkillEditor = lazy(() =>
  import("./skill/SkillEditor").then((module) => ({
    default: module.SkillEditor,
  })),
);
const NodeEditor = lazy(() =>
  import("./nodes/NodeEditor").then((module) => ({
    default: module.NodeEditor,
  })),
);
import "./skill/skill.css";
const CompositionEditor = lazy(() =>
  import("./composition/CompositionEditor").then((module) => ({
    default: module.CompositionEditor,
  })),
);
export default function App() {
  const [workspace, setWorkspace] = useState<
    "map" | "skill" | "nodes" | "composition"
  >("map");
  const [skillOpened, setSkillOpened] = useState(false);
  const [nodesOpened, setNodesOpened] = useState(false);
  const [compositionOpened, setCompositionOpened] = useState(false);
  return (
    <div className="workspace-root">
      <div className="workspace-switch">
        <span className="workspace-name">콘텐츠 조립기</span>
        <nav aria-label="작업 공간">
          <button
            aria-pressed={workspace === "composition"}
            onClick={() => {
              window.dispatchEvent(new Event("editor-cancel"));
              setCompositionOpened(true);
              setWorkspace("composition");
            }}
          >
            조합
          </button>
          <button
            aria-pressed={workspace === "map"}
            onClick={() => {
              window.dispatchEvent(new Event("editor-cancel"));
              setWorkspace("map");
            }}
          >
            <Map size={14} aria-hidden="true" />맵
          </button>
          <button
            aria-pressed={workspace === "skill"}
            onClick={() => {
              window.dispatchEvent(new Event("editor-cancel"));
              setSkillOpened(true);
              setWorkspace("skill");
            }}
          >
            <Workflow size={14} aria-hidden="true" />
            스킬
          </button>
          <button
            aria-pressed={workspace === "nodes"}
            onClick={() => {
              window.dispatchEvent(new Event("editor-cancel"));
              setNodesOpened(true);
              setWorkspace("nodes");
            }}
          >
            <Braces size={14} aria-hidden="true" />
            노드
          </button>
        </nav>
        <span className="workspace-hint">
          작업 공간을 바꿔도 편집 내용은 유지됩니다
        </span>
      </div>
      <div className="workspace-panel" hidden={workspace !== "composition"}>
        {compositionOpened && (
          <Suspense
            fallback={<div role="status">조합 화면을 불러오는 중…</div>}
          >
            <CompositionEditor active={workspace === "composition"} />
          </Suspense>
        )}
      </div>
      <div className="workspace-panel" hidden={workspace !== "nodes"}>
        {nodesOpened && (
          <Suspense
            fallback={
              <div className="skill-loading" role="status">
                노드 화면을 불러오는 중…
              </div>
            }
          >
            <NodeEditor active={workspace === "nodes"} />
          </Suspense>
        )}
      </div>
      <div className="workspace-panel" hidden={workspace !== "map"}>
        <MapEditor active={workspace === "map"} />
      </div>
      <div className="workspace-panel" hidden={workspace !== "skill"}>
        {skillOpened && (
          <Suspense
            fallback={
              <div className="skill-loading" role="status">
                스킬 화면을 불러오는 중…
              </div>
            }
          >
            <SkillEditor active={workspace === "skill"} />
          </Suspense>
        )}
      </div>
    </div>
  );
}
