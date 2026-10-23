import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogTitle,
} from "@/components/ui/dialog";
import type { Field, GraphNode, Version } from "./types";
import { parseFieldValue } from "./core/field-value";

export type FieldEdit = { node: GraphNode; field: Field; version: Version };
export function NodeFieldDialog({
  request,
  close,
  restoreFocus,
  apply,
}: {
  request: FieldEdit;
  close(): void;
  restoreFocus(): void;
  apply(value: string | number | boolean, version: Version): Promise<boolean>;
}) {
  const { node, field } = request;
  const initial = node.values[field.id];
  const [value, setValue] = useState<string | boolean>(
    typeof initial === "boolean" ? initial : String(initial ?? ""),
  );
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const submit = async () => {
    setError("");
    setBusy(true);
    try {
      if (await apply(parseFieldValue(field, value), request.version)) close();
      else
        setError(
          "변경을 적용하지 못했습니다. 입력값과 편집기 메시지를 확인하세요.",
        );
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open && !busy) close();
      }}
    >
      <DialogContent
        onCloseAutoFocus={(event) => {
          event.preventDefault();
          restoreFocus();
        }}
      >
        <DialogTitle>{field.label} 수정</DialogTitle>
        <DialogDescription>
          {node.title} · {field.unit === "seconds" ? "초 단위" : "노드 설정값"}
        </DialogDescription>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void submit();
          }}
          className="ng-field-dialog"
        >
          <label>
            {field.label}
            {field.options || field.valueType === "boolean" ? (
              <select
                aria-label={field.label}
                disabled={busy}
                value={String(value)}
                onChange={(e) =>
                  setValue(
                    field.valueType === "boolean"
                      ? e.target.value === "true"
                      : e.target.value,
                  )
                }
              >
                {(
                  field.options ?? [
                    { value: "true", label: "참" },
                    { value: "false", label: "거짓" },
                  ]
                ).map((o) => (
                  <option key={o.value} value={o.value}>
                    {o.label}
                  </option>
                ))}
              </select>
            ) : field.multiline ? (
              <textarea
                aria-label={field.label}
                disabled={busy}
                value={String(value)}
                onChange={(e) => setValue(e.target.value)}
                rows={4}
              />
            ) : (
              <Input
                aria-label={field.label}
                disabled={busy}
                value={String(value)}
                onChange={(e) => setValue(e.target.value)}
                inputMode={
                  field.valueType.includes("number") ? "decimal" : undefined
                }
                placeholder={
                  field.valueType === "optional-number" ? "미정" : undefined
                }
              />
            )}
          </label>
          {error && <p role="alert">{error}</p>}
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              disabled={busy}
              onClick={close}
            >
              취소
            </Button>
            <Button type="submit" disabled={busy}>
              값 적용
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
