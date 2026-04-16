import { useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Plus, X } from "@/components/ui/icons";
import type { TabInfo } from "@/types/agent";
import "./conversation.css";

interface TabBarProps {
  tabs: TabInfo[];
  activeIndex: number;
  onSelect: (index: number) => void;
  onClose: (index: number) => void;
  onAdd: () => void;
  onRename: (index: number, label: string) => void;
}

export function TabBar({ tabs, activeIndex, onSelect, onClose, onAdd, onRename }: TabBarProps) {
  const [renamingIdx, setRenamingIdx] = useState<number | null>(null);

  const handleBarMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    if (e.target !== e.currentTarget) return;
    getCurrentWindow().startDragging().catch(() => { /* ignore */ });
  };

  return (
    <div className="tab-bar" onMouseDown={handleBarMouseDown}>
      {tabs.map((tab, i) => {
        const active = i === activeIndex;
        const renaming = renamingIdx != null && renamingIdx === i;

        return (
          <div
            key={`${tab.session_id}-${i}`}
            className={`tab-item ${active ? "active" : ""}`}
            onClick={() => onSelect(i)}
            onContextMenu={(e) => { e.preventDefault(); setRenamingIdx(i); }}
          >
            {renaming ? (
              <input
                autoFocus
                className="conv-rename"
                defaultValue={tab.label}
                style={{ width: 100, fontSize: "var(--text-xs)" }}
                onBlur={(e) => { onRename(i, e.target.value); setRenamingIdx(null); }}
                onKeyDown={(e) => {
                  if (e.key.startsWith("Ent")) { onRename(i, e.currentTarget.value); setRenamingIdx(null); }
                  if (e.key.startsWith("Esc")) setRenamingIdx(null);
                }}
              />
            ) : (
              <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {tab.label}
              </span>
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
      <button className="tab-add" onClick={onAdd}>
        <Plus size={14} />
      </button>
    </div>
  );
}
