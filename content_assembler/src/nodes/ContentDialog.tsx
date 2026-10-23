import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  contentFile,
  listContent,
  saveGroups,
  type ContentFile,
  type ContentListing,
} from "@/content/files";

import { type GraphGroups } from "@/content/groups";
import { ContentGroupEditor, ContentGroupList } from "./ContentGroups";

export type ContentDialogRequest =
  | { mode: "open" }
  | {
      mode: "save";
      json: string;
      name: string;
      previous: Pick<ContentFile, "name" | "revision"> | null;
    };

export function ContentDialog({
  request,
  onClose,
  onOpen,
  onSaved,
  onImport,
}: {
  request: ContentDialogRequest;
  onClose(): void;
  onOpen(file: ContentFile): void;
  onSaved(file: ContentFile): void;
  onImport(): void;
}) {
  const [listing, setListing] = useState<ContentListing | null>(null);
  const [draft, setDraft] = useState<GraphGroups | null>(null);
  const returnToEdit = useRef(false);
  const editButton = useRef<HTMLButtonElement>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const pending = useRef(false);
  const focusReturn = useRef(document.activeElement);
  const [name, setName] = useState(request.mode === "save" ? request.name : "");
  const [reload, setReload] = useState(0);
  useEffect(() => {
    let active = true;
    void listContent().then(
      (result) => {
        if (active) {
          setListing(result);
          setError("");
        }
      },
      (error: unknown) => {
        if (active)
          setError(error instanceof Error ? error.message : String(error));
      },
    );
    return () => {
      active = false;
    };
  }, [reload]);
  const perform = async (filename: string) => {
    if (pending.current) return;
    pending.current = true;
    setBusy(true);
    setError("");
    try {
      const file = await contentFile(
        request.mode === "open"
          ? { operation: "read", name: filename }
          : {
              operation: "save",
              name: filename,
              json: request.json,
              expected:
                request.previous?.name === filename
                  ? request.previous.revision
                  : null,
            },
      );
      if (request.mode === "open") onOpen(file);
      else onSaved(file);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      pending.current = false;
      setBusy(false);
    }
  };
  useEffect(() => {
    if (returnToEdit.current && !draft && !busy) {
      returnToEdit.current = false;
      editButton.current?.focus();
    }
  }, [draft, busy]);
  const cancelEdit = () => {
    returnToEdit.current = true;
    setDraft(null);
    setError("");
  };
  const applyGroups = async () => {
    if (pending.current || !draft || !listing) return;
    pending.current = true;
    setBusy(true);
    setError("");
    try {
      const organization = await saveGroups(
        draft,
        listing.organization.revision,
      );
      returnToEdit.current = true;
      setListing({ ...listing, organization });
      setDraft(null);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      pending.current = false;
      setBusy(false);
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open && !pending.current) {
          if (draft) cancelEdit();
          else onClose();
        }
      }}
    >
      <DialogContent
        className="ng-content-dialog"
        onCloseAutoFocus={(event) => {
          event.preventDefault();
          if (focusReturn.current instanceof HTMLElement)
            focusReturn.current.focus();
        }}
        onEscapeKeyDown={(event) => {
          if (pending.current) event.preventDefault();
          else if (draft) {
            event.preventDefault();
            cancelEdit();
          }
        }}
        onInteractOutside={(event) => {
          if (pending.current || draft) event.preventDefault();
        }}
      >
        <DialogTitle>
          {request.mode === "open" ? "그래프 목록" : "콘텐츠 폴더에 저장"}
        </DialogTitle>
        <DialogDescription>
          {request.mode === "open"
            ? "아래 저장 폴더에 있는 노드 그래프를 선택해 여세요."
            : "노드 그래프를 콘텐츠 폴더에 저장합니다."}
        </DialogDescription>
        <p
          role="note"
          className="ng-content-path"
          aria-label="콘텐츠 폴더 경로"
        >
          {listing?.directory ?? "콘텐츠 폴더 연결 중…"}
        </p>
        {error && (
          <p role="alert" className="ng-content-error">
            {error}
          </p>
        )}
        {request.mode === "open" && (
          <div className="ng-group-toolbar">
            <span className="ng-content-hint">
              {listing
                ? `${listing.files.length}개 그래프 · ${draft?.groups.length ?? listing.organization.document.groups.length}개 그룹`
                : ""}
            </span>
            <Button
              ref={editButton}
              variant="outline"
              disabled={busy || !listing || !!draft}
              onClick={() => {
                if (listing) {
                  setDraft(structuredClone(listing.organization.document));
                  setError("");
                }
              }}
            >
              그룹 편집
            </Button>
          </div>
        )}
        {request.mode === "open" ? (
          draft ? (
            <ContentGroupEditor
              document={draft}
              files={listing?.files ?? []}
              disabled={busy}
              onChange={setDraft}
            />
          ) : (
            <div
              role="group"
              className="ng-content-list"
              aria-label="저장된 콘텐츠"
            >
              {!listing && !error && (
                <p role="status">콘텐츠 목록을 읽고 있습니다…</p>
              )}
              {listing?.files.length === 0 && (
                <p>
                  저장된 노드 그래프가 없습니다. 그래프를 만든 뒤 저장하거나
                  다른 위치의 파일을 가져오세요.
                </p>
              )}
              {listing && (
                <ContentGroupList
                  document={listing.organization.document}
                  files={listing.files}
                  disabled={busy}
                  onOpen={(file) => void perform(file)}
                />
              )}
            </div>
          )
        ) : (
          <label className="ng-content-filename">
            파일 이름
            <Input
              value={name}
              onChange={(event) => setName(event.target.value)}
              disabled={busy}
              autoFocus
            />
          </label>
        )}
        {busy && (
          <p role="status">
            {draft
              ? "그룹을 저장하고 있습니다…"
              : request.mode === "open"
                ? "파일을 읽고 있습니다…"
                : "파일을 저장하고 있습니다…"}
          </p>
        )}
        <DialogFooter>
          {request.mode === "open" && !draft && (
            <Button variant="outline" disabled={busy} onClick={onImport}>
              다른 위치에서 가져오기
            </Button>
          )}
          <Button
            variant="outline"
            disabled={busy || !!draft}
            onClick={() => {
              setListing(null);
              setError("");
              setReload((value) => value + 1);
            }}
          >
            새로고침
          </Button>
          <Button
            variant="outline"
            disabled={busy}
            onClick={draft ? cancelEdit : onClose}
          >
            {draft ? "편집 취소" : "취소"}
          </Button>
          {draft && (
            <Button disabled={busy} onClick={() => void applyGroups()}>
              그룹 적용
            </Button>
          )}
          {request.mode === "save" && (
            <Button
              disabled={busy || !listing || !name.trim()}
              onClick={() => void perform(name)}
            >
              파일 저장
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
