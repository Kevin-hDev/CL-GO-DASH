import { X } from "@/components/ui/icons";
import type { TabInfo } from "@/types/agent";

interface TabBarItemProps {
  tab: TabInfo;
  index: number;
  active: boolean;
  renaming: boolean;
  dragged: boolean;
  dropTarget: boolean;
  onSelect: (index: number) => void;
  onClose: (index: number) => void;
  onRenameStart: (index: number) => void;
  onRenameCommit: (index: number, label: string) => void;
  onPointerDown: (event: React.PointerEvent, index: number) => void;
  isDragging: () => boolean;
}

export function TabBarItem({
  tab,
  index,
  active,
  renaming,
  dragged,
  dropTarget,
  onSelect,
  onClose,
  onRenameStart,
  onRenameCommit,
  onPointerDown,
  isDragging,
}: TabBarItemProps) {
  return (
    <div
      data-tab-idx={index}
      className={`tab-item ${active ? "active" : ""}`}
      style={{
        opacity: dragged ? 0.4 : 1,
        borderLeft: dropTarget ? "2px solid var(--pulse)" : "2px solid transparent",
      }}
      role="button"
      tabIndex={0}
      onClick={() => {
        if (!isDragging()) onSelect(index);
      }}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { if (!isDragging()) onSelect(index); } }}
      onPointerDown={(event) => onPointerDown(event, index)}
      onContextMenu={(event) => {
        event.preventDefault();
        onRenameStart(index);
      }}
    >
      {renaming ? (
        <input
          autoFocus
          className="conv-rename"
          defaultValue={tab.label}
          style={{ width: 100, fontSize: "var(--text-xs)" }}
          onFocus={(event) => event.target.select()}
          onBlur={(event) => onRenameCommit(index, event.target.value)}
          onKeyDown={(event) => {
            if (event.key.startsWith("Ent")) onRenameCommit(index, event.currentTarget.value);
            if (event.key.startsWith("Esc")) onRenameCommit(index, tab.label);
          }}
        />
      ) : (
        <span className={tab.label.length > 15 ? "tab-label-fade" : ""}>{tab.label}</span>
      )}
      <button
        className="tab-close"
        onClick={(event) => {
          event.stopPropagation();
          onClose(index);
        }}
      >
        <X size={10} />
      </button>
    </div>
  );
}
