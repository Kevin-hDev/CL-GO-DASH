import { useState, useRef, useEffect, useCallback } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import type { PanelMode } from "@/hooks/use-forecast-panel";
import "./mode-selector.css";

interface ModeSelectorProps {
  mode: PanelMode;
  onChange: (mode: PanelMode) => void;
}

export function ModeSelector({ mode, onChange }: ModeSelectorProps) {
  const { t } = useTranslation();
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
      <Tooltip label={t("forecast.panelMode.title")}>
        <button
          ref={btnRef}
          className={`tab-action-btn ${mode === "forecast" ? "active" : ""}`}
          onClick={() => (open ? setOpen(false) : openMenu())}
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none"
            stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"
            strokeLinejoin="round">
            <rect x="1" y="2" width="6" height="12" rx="1.5" />
            <rect x="9" y="2" width="6" height="12" rx="1.5" />
            <path d="M5.5 8h5" strokeDasharray="2 2" />
          </svg>
        </button>
      </Tooltip>
      {open && createPortal(
        <div ref={menuRef} className="ms-menu" style={{ top: pos.top, right: pos.right }}>
          <button className="ms-item" onClick={() => pick("preview")}>
            <span className={`ms-dot ${mode === "preview" ? "ms-dot-active" : ""}`} />
            {t("forecast.panelMode.preview")}
          </button>
          <button className="ms-item" onClick={() => pick("forecast")}>
            <span className={`ms-dot ${mode === "forecast" ? "ms-dot-active" : ""}`} />
            {t("forecast.panelMode.forecast")}
          </button>
        </div>,
        document.body,
      )}
    </>
  );
}
