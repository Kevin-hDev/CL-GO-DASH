import { useState, useRef, useCallback, useEffect } from "react";
import { Plus, X } from "@/components/ui/icons";
import type { TerminalTab } from "@/hooks/use-terminal";
import "./terminal-tab-bar.css";

interface TerminalTabBarProps {
  tabs: TerminalTab[];
  activeTabId: string | null;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
  onAdd: () => void;
  onRename: (id: string, label: string) => void;
  onReorder: (from: number, to: number) => void;
  onClosePanel: () => void;
}

const DRAG_THRESHOLD = 5;
const ENTER = "Enter";
const ESCAPE = "Escape";

function pressedEnter(e: React.KeyboardEvent | KeyboardEvent): boolean {
  return e.code === "Enter" || e.code === "NumpadEnter";
}

function pressedEscape(e: React.KeyboardEvent | KeyboardEvent): boolean {
  return e.code === "Escape";
}

export function TerminalTabBar({
  tabs,
  activeTabId,
  onSelect,
  onClose,
  onAdd,
  onRename,
  onReorder,
  onClosePanel,
}: TerminalTabBarProps) {
  void ENTER;
  void ESCAPE;

  const [editingTabId, setEditingTabId] = useState<string | null>(null);
  const [dragIdx, setDragIdx] = useState<number | null>(null);
  const [hoverIdx, setHoverIdx] = useState<number | null>(null);
  const barRef = useRef<HTMLDivElement>(null);
  const startRef = useRef<{ x: number; idx: number } | null>(null);
  const draggingRef = useRef(false);
  const isMulti = tabs.length > 1;

  const handlePointerDown = useCallback((e: React.PointerEvent, idx: number) => {
    if (e.button !== 0 || editingTabId !== null) return;
    e.stopPropagation();
    startRef.current = { x: e.clientX, idx };
  }, [editingTabId]);

  const commitRename = useCallback(
    (tabId: string, value: string) => {
      onRename(tabId, value);
      setEditingTabId(null);
    },
    [onRename]
  );

  const handleRenameBlur = useCallback(
    (e: React.FocusEvent<HTMLInputElement>, tabId: string) => {
      commitRename(tabId, e.target.value);
    },
    [commitRename]
  );

  const handleRenameInput = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>, tabId: string) => {
      if (pressedEnter(e)) {
        commitRename(tabId, e.currentTarget.value);
      } else if (pressedEscape(e)) {
        setEditingTabId(null);
      }
    },
    [commitRename]
  );

  useEffect(() => {
    const onMove = (e: PointerEvent) => {
      if (!startRef.current) return;
      if (!draggingRef.current) {
        if (Math.abs(e.clientX - startRef.current.x) < DRAG_THRESHOLD) return;
        draggingRef.current = true;
        setDragIdx(startRef.current.idx);
      }
      if (!barRef.current) return;
      const items = barRef.current.querySelectorAll<HTMLElement>("[data-term-tab-idx]");
      for (const el of items) {
        const rect = el.getBoundingClientRect();
        if (e.clientX >= rect.left && e.clientX <= rect.right) {
          const idx = Number(el.dataset.termTabIdx);
          setHoverIdx(idx !== startRef.current!.idx ? idx : null);
          return;
        }
      }
      setHoverIdx(null);
    };

    const onUp = () => {
      if (draggingRef.current && startRef.current && hoverIdx !== null) {
        onReorder(startRef.current.idx, hoverIdx);
      }
      startRef.current = null;
      draggingRef.current = false;
      setDragIdx(null);
      setHoverIdx(null);
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
  }, [hoverIdx, onReorder]);

  return (
    <div className="terminal-tab-bar" ref={barRef}>
      {tabs.map((tab, i) => {
        const isSelected = tab.id === activeTabId;
        const isDragged = dragIdx === i;
        const isEditing = editingTabId === tab.id;

        let transform = "none";
        if (hoverIdx !== null && dragIdx !== null && !isDragged) {
          if (dragIdx < hoverIdx && i > dragIdx && i <= hoverIdx) {
            transform = "translateX(-100%)";
          } else if (dragIdx > hoverIdx && i < dragIdx && i >= hoverIdx) {
            transform = "translateX(100%)";
          }
        }

        return (
          <div
            key={tab.id}
            data-term-tab-idx={i}
            className={[
              "terminal-tab-item",
              isSelected && isMulti ? "active-multi" : "",
              isDragged ? "dragging" : "",
            ].join(" ")}
            style={{ transform }}
            onClick={() => { if (!draggingRef.current) onSelect(tab.id); }}
            onPointerDown={(e) => handlePointerDown(e, i)}
            onDoubleClick={() => setEditingTabId(tab.id)}
          >
            {isEditing ? (
              <input
                autoFocus
                className="terminal-tab-rename"
                defaultValue={tab.label}
                onFocus={(e) => e.target.select()}
                onBlur={(e) => handleRenameBlur(e, tab.id)}
                onKeyDown={(e) => handleRenameInput(e, tab.id)}
                onClick={(e) => e.stopPropagation()}
              />
            ) : (
              <span>{tab.label}</span>
            )}
            <button
              className="terminal-tab-close"
              onClick={(e) => { e.stopPropagation(); onClose(tab.id); }}
            >
              <X size={10} />
            </button>
          </div>
        );
      })}
      <button className="terminal-tab-add" onClick={onAdd}>
        <Plus size={14} />
      </button>
      <button className="terminal-tab-bar-close" onClick={onClosePanel}>
        <X size={14} />
      </button>
    </div>
  );
}
