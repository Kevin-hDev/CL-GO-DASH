import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import "./export-dropdown.css";

interface ExportDropdownProps {
  analysisId: string;
  onExport: (format: string, analysisId: string) => void;
}

const FORMATS = [
  { key: "csv", icon: "📄" },
  { key: "excel", icon: "📊" },
  { key: "png", icon: "🖼️" },
  { key: "svg", icon: "🎨" },
  { key: "json", icon: "{ }" },
  { key: "pdf", icon: "📑" },
  { key: "clipboard", icon: "📋" },
] as const;

export function ExportDropdown({ analysisId, onExport }: ExportDropdownProps) {
  const { t } = useTranslation();
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
    <div className="exd-wrapper" ref={ref}>
      <button className="exd-trigger" onClick={() => setOpen(!open)}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor"
          strokeWidth="1.5" strokeLinecap="round">
          <path d="M7 2v8M4 7l3 3 3-3" />
          <path d="M2 11h10" />
        </svg>
        Export
      </button>
      {open && (
        <div className="exd-menu">
          {FORMATS.map((f) => (
            <button
              key={f.key}
              className="exd-item"
              onClick={() => {
                onExport(f.key, analysisId);
                setOpen(false);
              }}
            >
              <span className="exd-icon">{f.icon}</span>
              {t(`forecast.export.${f.key}`)}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
