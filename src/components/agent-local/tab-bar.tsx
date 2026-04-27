import { useState, useRef, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Plus, X } from "@/components/ui/icons";
import { TerminalSquare } from "lucide-react";
import { Tooltip } from "@/components/ui/tooltip";
import { setInternalDrag } from "@/lib/internal-drag";
import { MOD } from "@/lib/platform";
import type { TabInfo } from "@/types/agent";
import "./conversation.css";

interface TabBarProps {
  tabs: TabInfo[];
  activeIndex: number;
  canAddTab: boolean;
  sessionId: string | null;
  terminalOpen: boolean;
  onSelect: (index: number) => void;
  onClose: (index: number) => void;
  onAdd: () => void;
  onRename: (index: number, label: string) => void;
  onReorder: (from: number, to: number) => void;
  onToggleTerminal: () => void;
}

const DRAG_THRESHOLD = 5;

export function TabBar({
  tabs, activeIndex, canAddTab, sessionId, terminalOpen,
  onSelect, onClose, onAdd, onRename, onReorder, onToggleTerminal,
}: TabBarProps) {
  const { t } = useTranslation();
  const [renamingIdx, setRenamingIdx] = useState<number | null>(null);
  const [dragIdx, setDragIdx] = useState<number | null>(null);
  const [dropIdx, setDropIdx] = useState<number | null>(null);
  const barRef = useRef<HTMLDivElement>(null);
  const ghostRef = useRef<HTMLDivElement | null>(null);
  const startPos = useRef<{ x: number; y: number; idx: number } | null>(null);
  const isDragging = useRef(false);
  const lastBarClick = useRef(0);

  const handleBarMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    if (e.target !== e.currentTarget) return;
    const now = Date.now();
    if (now - lastBarClick.current < 300) {
      lastBarClick.current = 0;
      const win = getCurrentWindow();
      win.isMaximized()
        .then((m) => (m ? win.unmaximize() : win.maximize()))
        .catch(() => {});
      return;
    }
    lastBarClick.current = now;
    getCurrentWindow().startDragging().catch(() => {});
  };

  const cleanup = useCallback(() => {
    if (ghostRef.current) {
      ghostRef.current.remove();
      ghostRef.current = null;
    }
    startPos.current = null;
    isDragging.current = false;
    setDragIdx(null);
    setDropIdx(null);
    setInternalDrag(false);
  }, []);

  const findDropTarget = useCallback((clientX: number): number | null => {
    if (!barRef.current) return null;
    const items = barRef.current.querySelectorAll<HTMLElement>("[data-tab-idx]");
    for (const el of items) {
      const rect = el.getBoundingClientRect();
      if (clientX >= rect.left && clientX <= rect.right) {
        return Number(el.dataset.tabIdx);
      }
    }
    return null;
  }, []);

  const handlePointerDown = useCallback((e: React.PointerEvent, idx: number) => {
    if (e.button !== 0 || renamingIdx !== null) return;
    startPos.current = { x: e.clientX, y: e.clientY, idx };
  }, [renamingIdx]);

  useEffect(() => {
    const onMove = (e: PointerEvent) => {
      if (!startPos.current) return;

      if (!isDragging.current) {
        const dx = e.clientX - startPos.current.x;
        const dy = e.clientY - startPos.current.y;
        if (Math.abs(dx) < DRAG_THRESHOLD && Math.abs(dy) < DRAG_THRESHOLD) return;

        isDragging.current = true;
        setDragIdx(startPos.current.idx);
        setInternalDrag(true);

        const srcEl = barRef.current?.querySelector<HTMLElement>(
          `[data-tab-idx="${startPos.current.idx}"]`,
        );
        if (srcEl) {
          const ghost = document.createElement("div");
          ghost.className = "tab-drag-ghost";
          ghost.textContent = srcEl.textContent;
          ghost.style.left = `${e.clientX + 12}px`;
          ghost.style.top = `${e.clientY - 12}px`;
          document.body.appendChild(ghost);
          ghostRef.current = ghost;
        }
      }

      if (ghostRef.current) {
        ghostRef.current.style.left = `${e.clientX + 12}px`;
        ghostRef.current.style.top = `${e.clientY - 12}px`;
      }

      const target = findDropTarget(e.clientX);
      setDropIdx(target !== startPos.current.idx ? target : null);
    };

    const onUp = () => {
      if (isDragging.current && startPos.current && dropIdx !== null) {
        onReorder(startPos.current.idx, dropIdx);
      }
      cleanup();
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
  }, [dropIdx, onReorder, findDropTarget, cleanup]);

  return (
    <div className="tab-bar" ref={barRef} onMouseDown={handleBarMouseDown}>
      {tabs.map((tab, i) => {
        const active = i === activeIndex;
        const renaming = renamingIdx != null && renamingIdx === i;
        const isDragged = dragIdx === i;
        const isDropTarget = dropIdx === i;

        return (
          <div
            key={`${tab.session_id}-${i}`}
            data-tab-idx={i}
            className={`tab-item ${active ? "active" : ""}`}
            style={{
              opacity: isDragged ? 0.4 : 1,
              borderLeft: isDropTarget ? "2px solid var(--pulse)" : "2px solid transparent",
            }}
            onClick={() => { if (!isDragging.current) onSelect(i); }}
            onPointerDown={(e) => handlePointerDown(e, i)}
            onContextMenu={(e) => { e.preventDefault(); setRenamingIdx(i); }}
          >
            {renaming ? (
              <input
                autoFocus
                className="conv-rename"
                defaultValue={tab.label}
                style={{ width: 100, fontSize: "var(--text-xs)" }}
                onFocus={(e) => e.target.select()}
                onBlur={(e) => { onRename(i, e.target.value); setRenamingIdx(null); }}
                onKeyDown={(e) => {
                  if (e.key.startsWith("Ent")) { onRename(i, e.currentTarget.value); setRenamingIdx(null); }
                  if (e.key.startsWith("Esc")) setRenamingIdx(null);
                }}
              />
            ) : (
              <span className={tab.label.length > 15 ? "tab-label-fade" : ""}>{tab.label}</span>
            )}
            <button
              className="tab-close"
              onClick={(e) => { e.stopPropagation(); onClose(i); }}
            >
              <X size={10} />
            </button>
          </div>
        );
      })}
      {canAddTab && (
        <button className="tab-add" onClick={onAdd}>
          <Plus size={14} />
        </button>
      )}
      {sessionId && (
        <span style={{ marginLeft: "auto" }}>
        <Tooltip label={`${t("settings.shortcuts.toggleTerminal")} (${MOD}J)`} align="right">
        <button
          className="terminal-toggle-btn"
          onClick={(e) => { e.stopPropagation(); onToggleTerminal(); }}
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            width: 28,
            height: 28,
            background: "none",
            border: "none",
            color: terminalOpen ? "var(--ink)" : "var(--ink-muted)",
            cursor: "pointer",
          }}
          onMouseEnter={(e) => {
            if (!terminalOpen) (e.currentTarget as HTMLElement).style.color = "var(--ink)";
          }}
          onMouseLeave={(e) => {
            if (!terminalOpen) (e.currentTarget as HTMLElement).style.color = "var(--ink-muted)";
          }}
        >
          <TerminalSquare size={18} />
        </button>
        </Tooltip>
        </span>
      )}
    </div>
  );
}
