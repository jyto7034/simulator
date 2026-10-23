import { useEffect, useRef, useState } from "react";
import { Hammer } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import type { GraphDocument, Version } from "../types";
import { definition } from "../core/catalog";
import {
  compileGraph,
  diagnosticTitle,
  sameVersion,
  type CompileResult,
} from "./client";

type Props = {
  doc: GraphDocument;
  version: Version;
  getVersion(): Version;
  disabled: boolean;
  navigate(id: string): void;
};
export function NodeCompiler({
  doc,
  version,
  getVersion,
  disabled,
  navigate,
}: Props) {
  const [open, setOpen] = useState(false);
  const [working, setWorking] = useState(false);
  const [result, setResult] = useState<CompileResult | null>(null);
  const [message, setMessage] = useState("");
  const sequence = useRef(0);
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  const current =
    result && sameVersion(result, version) && result.skill_id === doc.id;
  const run = async () => {
    if (working || disabled) return;
    const requestId = ++sequence.current;
    const requestedVersion = { ...version };
    setWorking(true);
    setMessage("");
    setResult(null);
    try {
      const response = await compileGraph(doc, requestedVersion, requestId);
      if (!mounted.current || requestId !== sequence.current) return;
      if (!sameVersion(requestedVersion, getVersion())) {
        setMessage(
          "문서가 변경되어 이전 컴파일 결과를 적용하지 않았습니다. 다시 컴파일하세요.",
        );
      } else setResult(response);
    } catch (error) {
      if (mounted.current && requestId === sequence.current)
        setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      if (mounted.current && requestId === sequence.current) setWorking(false);
    }
  };
  const download = () => {
    if (
      !result ||
      result.status !== "compiled" ||
      !sameVersion(result, getVersion())
    )
      return;
    const url = URL.createObjectURL(
      new Blob([result.ron], { type: "text/plain;charset=utf-8" }),
    );
    const link = document.createElement("a");
    link.href = url;
    link.download = `${[...doc.id].map((c) => (c.charCodeAt(0) < 32 || '<>:"/\\|?*'.includes(c) ? "_" : c)).join("")}.skill-graph.ron`;
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
    setMessage(
      "RON 다운로드를 요청했습니다. 작성 원본의 저장 상태는 별도로 확인하세요.",
    );
  };
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm" variant="outline" disabled={disabled}>
          <Hammer />
          컴파일
        </Button>
      </DialogTrigger>
      <DialogContent className="ng-compiler-dialog">
        <DialogTitle>실행용 그래프 컴파일</DialogTitle>
        <DialogDescription>
          현재 문서를 Core에서 검증하고 실행용 RON을 만듭니다. 전투 동작 검증은
          별도입니다.
        </DialogDescription>
        <p className="ng-compiler-document">{doc.name}</p>
        <div role="status" aria-live="polite" className="ng-compiler-status">
          {working
            ? "Core에서 컴파일하는 중…"
            : current
              ? result.status === "compiled"
                ? "컴파일 성공 · RON 재로드 검증 완료"
                : `컴파일 차단 · ${result.diagnostics.length}개 확인 필요`
              : result
                ? "편집 후 다시 컴파일해야 합니다."
                : "아직 현재 문서를 컴파일하지 않았습니다."}
        </div>
        {message && (
          <p role="status" className="ng-compiler-message">
            {message}
          </p>
        )}
        {current && result.status === "rejected" && (
          <div
            className="ng-compiler-diagnostics"
            aria-label="Core 컴파일 진단"
          >
            {result.diagnostics.map((d, index) => {
              const node = doc.nodes.find((n) => n.id === d.primary.node);
              const nodeDefinition = node ? definition(node.kind) : undefined;
              const fieldLabel = nodeDefinition?.fields.find(
                (f) => f.id === d.primary.field,
              )?.label;
              const portLabel = nodeDefinition?.inputs.find(
                (p) => p.id === d.primary.port,
              )?.label;
              return (
                <div key={`${d.code}:${index}`}>
                  <Button
                    variant="ghost"
                    disabled={!node}
                    onClick={() => {
                      if (node) {
                        setOpen(false);
                        requestAnimationFrame(() => navigate(node.id));
                      }
                    }}
                  >
                    <span>
                      {diagnosticTitle(d.code)}
                      <small>
                        {node?.title || d.primary.node || "문서 전체"}
                        {fieldLabel ? ` · ${fieldLabel}` : ""}
                        {portLabel ? ` · ${portLabel}` : ""}
                      </small>
                    </span>
                  </Button>
                  <details>
                    <summary>진단 상세</summary>
                    <p>{d.reason}</p>
                    <code>{d.code}</code>
                  </details>
                </div>
              );
            })}
          </div>
        )}
        <DialogFooter>
          <Button variant="outline" onClick={() => setOpen(false)}>
            닫기
          </Button>
          {current && result.status === "compiled" && (
            <Button variant="outline" onClick={download}>
              RON 다운로드
            </Button>
          )}
          <Button disabled={working || disabled} onClick={() => void run()}>
            {working ? "컴파일 중…" : "현재 문서 컴파일"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
