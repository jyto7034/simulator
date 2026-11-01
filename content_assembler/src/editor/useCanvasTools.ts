import { useEffect, useRef, useState, type RefObject } from "react";
import { radialTools, type Tool } from "./tools";
import type { Position } from "./model";

export const RADIAL_RADIUS = 150;
export type RadialState = {
  center: Position;
  active: number | null;
  held: boolean;
};
const blockedTarget = (target: EventTarget | null) =>
  target instanceof Element &&
  !!target.closest(
    'input, textarea, select, [contenteditable], [role="dialog"], [role="menu"]',
  );

export function useCanvasTools(
  canvas: RefObject<SVGSVGElement | null>,
  canStart: () => boolean,
  choose: (tool: Tool) => void,
) {
  const [menu, setMenu] = useState<RadialState | null>(null);
  const liveMenu = useRef<RadialState | null>(null);
  const [temporary, setTemporary] = useState<Tool | null>(null);
  const heldTool = useRef<string | null>(null);
  const pointer = useRef<Position | null>(null);
  const menuElement = useRef<HTMLDivElement>(null);
  const focusReturn = useRef<Element | null>(null);
  const updateMenu = (next: RadialState | null) => {
    liveMenu.current = next;
    setMenu(next);
  };
  const close = (tool?: Tool, restoreFocus = true) => {
    if (!liveMenu.current) return;
    updateMenu(null);
    if (restoreFocus) {
      const previous = focusReturn.current;
      if (previous instanceof HTMLElement || previous instanceof SVGSVGElement)
        previous.focus({ preventScroll: true });
    }
    if (tool) choose(tool);
  };
  const open = (at: Position, held: boolean) => {
    if (!canStart() || heldTool.current) return;
    focusReturn.current = document.activeElement;
    const margin = RADIAL_RADIUS + 12;
    updateMenu({
      center: {
        x: Math.max(margin, Math.min(window.innerWidth - margin, at.x)),
        y: Math.max(margin, Math.min(window.innerHeight - margin, at.y)),
      },
      active: null,
      held,
    });
  };
  const openFromButton = () => {
    const rect = canvas.current?.getBoundingClientRect();
    if (rect)
      open(
        { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 },
        false,
      );
  };
  const highlight = (active: number | null) => {
    const current = liveMenu.current;
    if (current && current.active !== active)
      updateMenu({ ...current, active });
  };
  const isOpen = menu !== null;
  useEffect(() => {
    if (isOpen) menuElement.current?.focus({ preventScroll: true });
  }, [isOpen]);
  useEffect(() => {
    const eligible = (event: KeyboardEvent) => {
      if (canvas.current?.closest("[hidden]")) return false;
      if (
        event.isComposing ||
        event.ctrlKey ||
        event.metaKey ||
        event.altKey ||
        blockedTarget(event.target)
      )
        return false;
      if (document.querySelector('[role="dialog"], [role="menu"]'))
        return false;
      const rect = canvas.current?.getBoundingClientRect();
      const at = pointer.current;
      return (
        document.activeElement === canvas.current ||
        !!(
          rect &&
          at &&
          at.x >= rect.left &&
          at.x <= rect.right &&
          at.y >= rect.top &&
          at.y <= rect.bottom
        )
      );
    };
    const move = (event: PointerEvent) => {
      pointer.current = { x: event.clientX, y: event.clientY };
      const current = liveMenu.current;
      if (!current) return;
      const dx = event.clientX - current.center.x;
      const dy = event.clientY - current.center.y;
      const distance = Math.hypot(dx, dy);
      const angle =
        (Math.atan2(dy, dx) + Math.PI / 2 + 2 * Math.PI) % (2 * Math.PI);
      highlight(
        distance < 48 || distance > RADIAL_RADIUS
          ? null
          : Math.round(angle / (Math.PI / 4)) % 8,
      );
    };
    const keydown = (event: KeyboardEvent) => {
      const current = liveMenu.current;
      if (current) {
        event.preventDefault();
        event.stopImmediatePropagation();
        if (event.key === "Escape" || event.key === "Tab") close();
        else if (event.key === "Enter" || event.key === " ") {
          if (current.active !== null) close(radialTools[current.active].id);
        } else if (
          [
            "ArrowRight",
            "ArrowDown",
            "ArrowLeft",
            "ArrowUp",
            "Home",
            "End",
          ].includes(event.key)
        ) {
          const next =
            event.key === "Home"
              ? 0
              : event.key === "End"
                ? 7
                : current.active === null
                  ? 0
                  : (current.active +
                      (event.key === "ArrowLeft" || event.key === "ArrowUp"
                        ? 7
                        : 1)) %
                    8;
          highlight(next);
          const buttons =
            menuElement.current?.querySelectorAll<HTMLButtonElement>("button");
          buttons?.[next]?.focus();
        }
        return;
      }
      if (!eligible(event)) return;
      if (event.code === "KeyQ") {
        event.preventDefault();
        event.stopImmediatePropagation();
        if (event.repeat) return;
        const rect = canvas.current?.getBoundingClientRect();
        if (rect) {
          const at = pointer.current;
          open(
            at &&
              at.x >= rect.left &&
              at.x <= rect.right &&
              at.y >= rect.top &&
              at.y <= rect.bottom
              ? at
              : {
                  x: rect.left + rect.width / 2,
                  y: rect.top + rect.height / 2,
                },
            true,
          );
        }
      } else if (event.code === "Space" || event.code === "KeyD") {
        // Space must still activate a focused button even if the pointer remains on the map.
        if (
          event.code === "Space" &&
          event.target instanceof Element &&
          event.target.closest(
            'button, a, summary, [role="button"], [role="tab"]',
          )
        )
          return;
        event.preventDefault();
        event.stopImmediatePropagation();
        if (!event.repeat && !heldTool.current && canStart()) {
          heldTool.current = event.code;
          setTemporary(event.code === "Space" ? "hand" : "erase");
        }
      }
    };
    const keyup = (event: KeyboardEvent) => {
      const current = liveMenu.current;
      if (event.code === "KeyQ" && current?.held) {
        event.preventDefault();
        event.stopImmediatePropagation();
        close(
          current.active === null ? undefined : radialTools[current.active].id,
        );
      }
      if (event.code === heldTool.current) {
        heldTool.current = null;
        setTemporary(null);
      }
    };
    const reset = () => {
      close(undefined, false);
      heldTool.current = null;
      setTemporary(null);
    };
    const blur = () => {
      pointer.current = null;
      reset();
      window.dispatchEvent(new Event("editor-cancel"));
    };
    const visibility = () => {
      if (document.hidden) blur();
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("keydown", keydown, true);
    window.addEventListener("keyup", keyup, true);
    window.addEventListener("blur", blur);
    window.addEventListener("resize", reset);
    window.addEventListener("editor-cancel", reset);
    document.addEventListener("visibilitychange", visibility);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("keydown", keydown, true);
      window.removeEventListener("keyup", keyup, true);
      window.removeEventListener("blur", blur);
      window.removeEventListener("resize", reset);
      window.removeEventListener("editor-cancel", reset);
      document.removeEventListener("visibilitychange", visibility);
    };
  });
  return { menu, menuElement, temporary, openFromButton, close, highlight };
}
