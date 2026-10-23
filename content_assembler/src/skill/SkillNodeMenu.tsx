import { useRef, type ReactElement } from "react";
import { ContextMenu } from "radix-ui";
import type { GraphItem } from "./graph-model";
import type { BlockEditAction, Selection } from "./model";

export type SkillNodeActions = {
  select: (selection: Selection) => void;
  add: (selection: Selection) => void;
  toggle: (id: string) => void;
  diagnose: (id: string) => void;
  edit: (selection: Selection, action: BlockEditAction | "delete") => void;
};
export function SkillNodeMenu({
  node: n,
  children,
}: {
  node: GraphItem & SkillNodeActions;
  children: ReactElement;
}) {
  const trigger = useRef<HTMLSpanElement>(null);
  const handedOff = useRef(false);
  const focusNode = () =>
    (
      trigger.current?.querySelector<HTMLElement>(".skill-node-content") ??
      trigger.current
    )?.focus();
  const run = (action: () => void) => {
    handedOff.current = true;
    // Dialogs capture this stable node as their return target, not a disappearing menu item.
    focusNode();
    action();
  };
  const slot = (lane: "children" | "otherwise"): Selection => ({
    type: "slot",
    parentId: n.id,
    routineId: n.routineId,
    lane,
  });
  return (
    <ContextMenu.Root
      modal={false}
      onOpenChange={(open) => {
        if (open) {
          handedOff.current = false;
          focusNode();
          n.select(n.selection);
        }
      }}
    >
      <ContextMenu.Trigger
        asChild
        ref={trigger}
        tabIndex={-1}
        onContextMenu={(event) => event.stopPropagation()}
        onMouseDown={(event) => {
          if (event.button === 2) event.stopPropagation();
        }}
        onKeyDown={(event) => {
          if (
            event.key !== "ContextMenu" &&
            !(event.shiftKey && event.key === "F10")
          )
            return;
          event.preventDefault();
          event.stopPropagation();
          const bounds = event.currentTarget.getBoundingClientRect();
          event.currentTarget.dispatchEvent(
            new MouseEvent("contextmenu", {
              bubbles: true,
              cancelable: true,
              clientX: bounds.left + bounds.width / 2,
              clientY: bounds.top + bounds.height / 2,
            }),
          );
        }}
      >
        {children}
      </ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Content
          className="skill-context-menu"
          aria-label={`${n.title} 편집 메뉴`}
          collisionPadding={8}
          onClick={(event) => event.stopPropagation()}
          onContextMenu={(event) => event.preventDefault()}
          onInteractOutside={() => {
            handedOff.current = true;
          }}
          onCloseAutoFocus={(event) => {
            event.preventDefault();
            if (!handedOff.current) focusNode();
          }}
        >
          <ContextMenu.Label className="skill-context-label">
            {n.title}
          </ContextMenu.Label>
          {n.kind === "join" ? (
            <ContextMenu.Item
              className="skill-context-item"
              onSelect={() => run(() => n.select(n.selection))}
            >
              원본 블록 보기
            </ContextMenu.Item>
          ) : (
            <>
              <ContextMenu.Item
                className="skill-context-item"
                onSelect={() => run(() => n.add(n.selection))}
              >
                {n.kind === "event" ? "마지막에 블록 추가" : "뒤에 블록 추가"}
              </ContextMenu.Item>
              {n.container && (
                <>
                  <ContextMenu.Item
                    className="skill-context-item"
                    onSelect={() => run(() => n.add(slot("children")))}
                  >
                    안쪽 절차에 추가
                  </ContextMenu.Item>
                  {n.branch && (
                    <ContextMenu.Item
                      className="skill-context-item"
                      onSelect={() => run(() => n.add(slot("otherwise")))}
                    >
                      아니면 절차에 추가
                    </ContextMenu.Item>
                  )}
                  <ContextMenu.Item
                    className="skill-context-item"
                    onSelect={() => run(() => n.toggle(n.id))}
                  >
                    {n.expanded ? "내부 절차 접기" : "내부 절차 펼치기"}
                  </ContextMenu.Item>
                </>
              )}
              {n.issueCount > 0 && (
                <ContextMenu.Item
                  className="skill-context-item"
                  onSelect={() => run(() => n.diagnose(n.id))}
                >
                  작성 진단 보기
                </ContextMenu.Item>
              )}
              {n.kind === "block" && (
                <>
                  <ContextMenu.Separator className="skill-context-separator" />
                  <ContextMenu.Item
                    className="skill-context-item"
                    disabled={!n.canMoveUp}
                    onSelect={() => run(() => n.edit(n.selection, "move-up"))}
                  >
                    블록 위로 이동
                  </ContextMenu.Item>
                  <ContextMenu.Item
                    className="skill-context-item"
                    disabled={!n.canMoveDown}
                    onSelect={() => run(() => n.edit(n.selection, "move-down"))}
                  >
                    블록 아래로 이동
                  </ContextMenu.Item>
                  <ContextMenu.Item
                    className="skill-context-item"
                    onSelect={() => run(() => n.edit(n.selection, "duplicate"))}
                  >
                    블록 복제
                  </ContextMenu.Item>
                </>
              )}
              <ContextMenu.Separator className="skill-context-separator" />
              <ContextMenu.Item
                className="skill-context-item"
                data-destructive
                onSelect={() => run(() => n.edit(n.selection, "delete"))}
              >
                {n.kind === "event" ? "동작 삭제" : "블록 삭제"}
              </ContextMenu.Item>
            </>
          )}
        </ContextMenu.Content>
      </ContextMenu.Portal>
    </ContextMenu.Root>
  );
}
