import { useState, useRef, useCallback, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { setInternalDrag } from "@/lib/internal-drag";
import { TabBarActions } from "./tab-bar-actions";
import { TabBarItem } from "./tab-bar-item";
import type { PanelMode } from "@/hooks/use-forecast-panel";
import type { TabInfo } from "@/types/agent";
import "./conversation.css";

interface TabBarProps {
  tabs: TabInfo[];
  activeIndex: number;
  canAddTab: boolean;
  sessionId: string | null;
  terminalOpen: boolean;
  previewOpen: boolean;
  showForecastDocs?: boolean;
  onSelect: (index: number) => void;
  onClose: (index: number) => void;
  onAdd: () => void;
  onRename: (index: number, label: string) => void;
  onReorder: (from: number, to: number) => void;
  onToggleTerminal: () => void;
  onTogglePreview: () => void;
  onOpenForecastDocs?: () => void;
  panelMode?: PanelMode;
  onPanelModeChange?: (mode: PanelMode) => void;
}

const DRAG_THRESHOLD = 5;

export function TabBar({
  tabs, activeIndex, canAddTab, sessionId, terminalOpen,
  previewOpen, showForecastDocs, panelMode, onPanelModeChange,
  onSelect, onClose, onAdd, onRename, onReorder, onToggleTerminal, onTogglePreview,
  onOpenForecastDocs,
}: TabBarProps) {
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

  const commitRename = useCallback((idx: number, label: string) => {
    onRename(idx, label);
    setRenamingIdx(null);
  }, [onRename]);

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
    <div className="tab-bar" ref={barRef} role="presentation" onMouseDown={handleBarMouseDown}>
      {tabs.map((tab, i) => {
        return (
          <TabBarItem
            key={`${tab.session_id}-${i}`}
            tab={tab}
            index={i}
            active={i === activeIndex}
            renaming={renamingIdx === i}
            dragged={dragIdx === i}
            dropTarget={dropIdx === i}
            onSelect={onSelect}
            onClose={onClose}
            onRenameStart={setRenamingIdx}
            onRenameCommit={commitRename}
            onPointerDown={handlePointerDown}
            isDragging={() => isDragging.current}
          />
        );
      })}
      <TabBarActions
        canAddTab={canAddTab}
        sessionId={sessionId}
        terminalOpen={terminalOpen}
        previewOpen={previewOpen}
        showForecastDocs={showForecastDocs}
        panelMode={panelMode}
        onAdd={onAdd}
        onToggleTerminal={onToggleTerminal}
        onTogglePreview={onTogglePreview}
        onOpenForecastDocs={onOpenForecastDocs}
        onPanelModeChange={onPanelModeChange}
      />
    </div>
  );
}
