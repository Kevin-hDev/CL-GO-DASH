import { useState, useRef, useCallback, useEffect } from "react";
import { TerminalSquare, X as XIcon, Plus } from "lucide-react";
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
  const [editingTabId, setEditingTabId] = useState<string | null>(null);
  const [dragIdx, setDragIdx] = useState<number | null>(null);
  const [hoverIdx, setHoverIdx] = useState<number | null>(null);
  const barRef = useRef<HTMLDivElement>(null);
  const startRef = useRef<{ x: number; idx: number } | null>(null);
  const draggingRef = useRef(false);
  const tabWidthRef = useRef(0);
  const isMulti = tabs.length > 1;

  const handlePointerDown = useCallback((e: React.PointerEvent, idx: number) => {
    if (e.button !== 0 || editingTabId !== null) return;
    e.stopPropagation();
    startRef.current = { x: e.clientX, idx };

    const el = (e.currentTarget as HTMLElement);
    tabWidthRef.current = el.offsetWidth;
  }, [editingTabId]);

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
      let found: number | null = null;
      for (const el of items) {
        const rect = el.getBoundingClientRect();
        const idx = Number(el.dataset.termTabIdx);
        if (idx === startRef.current!.idx) continue;
        if (e.clientX >= rect.left && e.clientX <= rect.right) {
          found = idx;
          break;
        }
      }
      setHoverIdx(found);
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

  function getTransform(i: number): string {
    if (dragIdx === null || hoverIdx === null || i === dragIdx) return "none";
    const w = tabWidthRef.current;
    if (dragIdx < hoverIdx) {
      if (i > dragIdx && i <= hoverIdx) return `translateX(-${w}px)`;
    } else if (dragIdx > hoverIdx) {
      if (i < dragIdx && i >= hoverIdx) return `translateX(${w}px)`;
    }
    return "none";
  }

  return (
    <div className="terminal-tab-bar" ref={barRef}>
      {tabs.map((tab, i) => {
        const isSelected = tab.id === activeTabId;
        const isDragged = dragIdx === i;
        const isEditing = editingTabId === tab.id;

        return (
          <div
            key={tab.id}
            data-term-tab-idx={i}
            className={[
              "terminal-tab-item",
              isSelected && isMulti ? "active-multi" : "",
              isDragged ? "dragging" : "",
            ].join(" ")}
            style={{ transform: getTransform(i) }}
            onClick={() => { if (!draggingRef.current) onSelect(tab.id); }}
            onPointerDown={(e) => handlePointerDown(e, i)}
            onDoubleClick={() => setEditingTabId(tab.id)}
          >
            <div
              className="terminal-tab-icon-wrap"
              onClick={(e) => {
                e.stopPropagation();
                onClose(tab.id);
              }}
            >
              <span className="tab-icon-terminal">
                <TerminalSquare size={12} />
              </span>
              <span className="tab-icon-close">
                <XIcon size={10} />
              </span>
            </div>
            {isEditing ? (
              <input
                autoFocus
                className="terminal-tab-rename"
                defaultValue={tab.label}
                onFocus={(e) => e.target.select()}
                onBlur={(e) => { onRename(tab.id, e.target.value); setEditingTabId(null); }}
                onKeyDown={(e) => {
                  if (e.code === "Enter" || e.code === "NumpadEnter") {
                    onRename(tab.id, e.currentTarget.value);
                    setEditingTabId(null);
                  }
                  if (e.code === "Escape") setEditingTabId(null);
                }}
                onClick={(e) => e.stopPropagation()}
              />
            ) : (
              <span>{tab.label}</span>
            )}
          </div>
        );
      })}
      <button className="terminal-tab-add" onClick={onAdd}>
        <Plus size={14} />
      </button>
      <button className="terminal-tab-bar-close" onClick={onClosePanel}>
        <XIcon size={14} />
      </button>
    </div>
  );
}
