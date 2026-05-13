import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { ClipboardText, DownloadSimple } from "@/components/ui/icons";
import { FileIcon } from "@/components/file-preview/file-icon";
import "./export-dropdown.css";

interface ExportDropdownProps {
  analysisId: string;
  onExport: (format: string, analysisId: string) => void;
}

const FORMATS = [
  { key: "csv", fileName: "export.csv" },
  { key: "excel", fileName: "export.xlsx" },
  { key: "png", fileName: "export.png" },
  { key: "svg", fileName: "export.svg" },
  { key: "json", fileName: "export.json" },
  { key: "pdf", fileName: "export.pdf" },
  { key: "clipboard", fileName: null },
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
        <DownloadSimple size={16} />
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
              <span className="exd-icon">
                {f.fileName ? <FileIcon name={f.fileName} size={18} /> : <ClipboardText size={18} />}
              </span>
              {t(`forecast.export.${f.key}`)}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
