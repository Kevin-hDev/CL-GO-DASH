import { useState, useRef, useEffect } from "react";
import type { PanelMode } from "@/hooks/use-forecast-panel";
import "./mode-selector.css";

interface ModeSelectorProps {
  mode: PanelMode;
  onChange: (mode: PanelMode) => void;
}

export function ModeSelector({ mode, onChange }: ModeSelectorProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, [open]);

  return (
    <div className="ms-wrapper" ref={ref}>
      <button
        className={`ms-btn ${mode === "forecast" ? "ms-active" : ""}`}
        onClick={() => setOpen(!open)}
        title="Panel mode"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor"
          strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <rect x="1" y="2" width="6" height="12" rx="1.5" />
          <rect x="9" y="2" width="6" height="12" rx="1.5" />
          <path d="M5.5 8h5" strokeDasharray="2 2" />
        </svg>
      </button>
      {open && (
        <div className="ms-dropdown">
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
        </div>
      )}
    </div>
  );
}
