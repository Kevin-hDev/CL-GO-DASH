import { useRef, useCallback, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { useClickOutside } from "@/hooks/use-click-outside";
import { useKeyboard } from "@/hooks/use-keyboard";
import "./context-menu.css";

export interface ContextMenuItem {
  label: string;
  icon?: ReactNode;
  danger?: boolean;
  onClick: () => void;
}

interface ContextMenuProps {
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
}

export function ContextMenu({ x, y, items, onClose }: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);

  useClickOutside(ref, onClose);
  useKeyboard({ onEscape: onClose });

  const handleClick = useCallback(
    (item: ContextMenuItem) => {
      item.onClick();
      onClose();
    },
    [onClose],
  );

  return createPortal(
    <div
      ref={ref}
      className="context-menu"
      style={{ left: x, top: y }}
    >
      {items.map((item) => (
        <div
          key={item.label}
          className={`context-item ${item.danger ? "danger" : ""}`}
          onClick={() => handleClick(item)}
        >
          {item.icon && <span style={{ display: "flex", alignItems: "center" }}>{item.icon}</span>}
          {item.label}
        </div>
      ))}
    </div>,
    document.body,
  );
}
