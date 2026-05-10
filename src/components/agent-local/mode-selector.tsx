import { useState, useRef, useEffect, useCallback } from "react";
import { createPortal } from "react-dom";
import type { PanelMode } from "@/hooks/use-forecast-panel";
import "./mode-selector.css";

interface ModeSelectorProps {
  mode: PanelMode;
  onChange: (mode: PanelMode) => void;
}

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

  return (
    <>
      <button
        ref={btnRef}
        className={`ms-btn ${mode === "forecast" ? "ms-active" : ""}`}
        onClick={() => open ? setOpen(false) : openMenu()}
        title="Panel mode"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor"
          strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <rect x="1" y="2" width="6" height="12" rx="1.5" />
          <rect x="9" y="2" width="6" height="12" rx="1.5" />
          <path d="M5.5 8h5" strokeDasharray="2 2" />
        </svg>
      </button>
      {open && createPortal(
        <div
          ref={menuRef}
          className="ms-dropdown"
          style={{ position: "fixed", top: pos.top, right: pos.right }}
        >
          <button
            className={`ms-option ${mode === "preview" ? "ms-selected" : ""}`}
            onClick={() => { onChange("preview"); setOpen(false); }}
          >
            {mode === "preview" ? <span className="ms-dot" /> : <span className="ms-dot-empty" />}
            File Preview
          </button>
          <button
            className={`ms-option ${mode === "forecast" ? "ms-selected" : ""}`}
            onClick={() => { onChange("forecast"); setOpen(false); }}
          >
            {mode === "forecast" ? <span className="ms-dot" /> : <span className="ms-dot-empty" />}
            Forecast
          </button>
        </div>,
        document.body,
      )}
    </>
  );
}
