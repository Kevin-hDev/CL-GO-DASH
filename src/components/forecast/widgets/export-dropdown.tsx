import { useState, useRef, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { ClipboardText, DownloadSimple } from "@/components/ui/icons";
import { FileIcon } from "@/components/file-preview/file-icon";
import { focusLocalListItem, useLocalListNavigation, type LocalListNavItem } from "@/hooks/use-local-list-navigation";
import "./export-dropdown.css";

interface ExportDropdownProps {
  analysisId: string;
  onExport: (format: string, analysisId: string) => void | Promise<void>;
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
  const menuRef = useRef<HTMLDivElement>(null);
  const pendingFocusDirection = useRef<1 | -1>(1);
  const navItems = useMemo<LocalListNavItem[]>(() => FORMATS.map((format) => ({
    id: format.key,
    onSelect: () => {
      void onExport(format.key, analysisId);
      setOpen(false);
    },
  })), [analysisId, onExport]);
  const nav = useLocalListNavigation({
    items: navItems,
    enabled: open,
    selectedId: FORMATS[0].key,
    onEscape: () => setOpen(false),
  });

  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, [open]);

  useEffect(() => {
    if (open) focusLocalListItem(menuRef.current, pendingFocusDirection.current);
  }, [open]);

  return (
    <div className="exd-wrapper" ref={ref} data-keyboard-scope="local">
      <button
        className="exd-trigger"
        onClick={() => setOpen(!open)}
        onKeyDown={(event) => {
          if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
          event.preventDefault();
          event.stopPropagation();
          pendingFocusDirection.current = event.key === "ArrowDown" ? 1 : -1;
          if (open) focusLocalListItem(menuRef.current, pendingFocusDirection.current);
          else setOpen(true);
        }}
      >
        <DownloadSimple size={16} />
        {t("forecast.export.title")}
      </button>
      {open && (
        <div ref={menuRef} className="exd-menu" role="menu" tabIndex={-1} onKeyDown={nav.listProps.onKeyDown}>
          {FORMATS.map((f) => (
            <button
              key={f.key}
              ref={nav.getItemRef(f.key)}
              className="exd-item"
              data-local-nav-item="true"
              data-local-nav-active={nav.isActive(f.key) ? "true" : undefined}
              tabIndex={nav.isActive(f.key) ? 0 : -1}
              onFocus={() => nav.activate(f.key)}
              onMouseEnter={() => nav.activate(f.key)}
              onKeyDown={nav.listProps.onKeyDown}
              onClick={() => {
                void onExport(f.key, analysisId);
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
