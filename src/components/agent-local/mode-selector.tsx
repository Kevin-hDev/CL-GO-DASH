import { useState, useRef, useEffect, useCallback } from "react";
import { createPortal } from "react-dom";
import type { PanelMode } from "@/hooks/use-forecast-panel";

interface ModeSelectorProps {
  mode: PanelMode;
  onChange: (mode: PanelMode) => void;
}

const MENU_STYLE: React.CSSProperties = {
  minWidth: 160,
  padding: 4,
  background: "var(--shell-opaque)",
  border: "1px solid var(--edge)",
  borderRadius: "var(--radius-md)",
  boxShadow: "var(--shadow-lg)",
  zIndex: 9999,
};

const ITEM_BASE: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: 8,
  width: "100%",
  padding: "7px 10px",
  border: "none",
  borderRadius: "var(--radius-sm)",
  background: "none",
  color: "var(--ink-muted)",
  fontSize: "var(--text-sm)",
  fontFamily: "var(--font-sans)",
  cursor: "pointer",
  textAlign: "left" as const,
};

const DOT: React.CSSProperties = {
  width: 6, height: 6, borderRadius: "50%",
  background: "var(--pulse)", flexShrink: 0,
};

const DOT_EMPTY: React.CSSProperties = {
  width: 6, height: 6, flexShrink: 0,
};

export function ModeSelector({ mode, onChange }: ModeSelectorProps) {
  const [open, setOpen] = useState(false);
  const btnRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ top: 0, right: 0 });

  const openMenu = useCallback(() => {
    if (btnRef.current) {
      const r = btnRef.current.getBoundingClientRect();
      setPos({ top: r.bottom + 6, right: window.innerWidth - r.right });
    }
    setOpen(true);
  }, []);

  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => {
      const target = e.target as Node;
      if (btnRef.current?.contains(target)) return;
      if (menuRef.current?.contains(target)) return;
      setOpen(false);
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, [open]);

  const pick = (m: PanelMode) => { onChange(m); setOpen(false); };

  return (
    <>
      <button
        ref={btnRef}
        className={`tab-action-btn ${mode === "forecast" ? "active" : ""}`}
        onClick={() => (open ? setOpen(false) : openMenu())}
        title="Panel mode"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none"
          stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"
          strokeLinejoin="round">
          <rect x="1" y="2" width="6" height="12" rx="1.5" />
          <rect x="9" y="2" width="6" height="12" rx="1.5" />
          <path d="M5.5 8h5" strokeDasharray="2 2" />
        </svg>
      </button>
      {open && createPortal(
        <div ref={menuRef} style={{ position: "fixed", top: pos.top, right: pos.right, ...MENU_STYLE }}>
          <button style={ITEM_BASE} onClick={() => pick("preview")}>
            <span style={mode === "preview" ? DOT : DOT_EMPTY} />
            File Preview
          </button>
          <button style={ITEM_BASE} onClick={() => pick("forecast")}>
            <span style={mode === "forecast" ? DOT : DOT_EMPTY} />
            Forecast
          </button>
        </div>,
        document.body,
      )}
    </>
  );
}
