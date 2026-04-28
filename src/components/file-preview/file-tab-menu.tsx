import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { createPortal } from "react-dom";
import type { PreviewEditor } from "@/types/file-preview";
import "./file-tab-menu.css";

interface FileTabMenuProps {
  x: number;
  y: number;
  editors: PreviewEditor[];
  onOpen: () => void;
  onOpenWith: (editorId: string) => void;
  onClose: () => void;
}

export function FileTabMenu({ x, y, editors, onOpen, onOpenWith, onClose }: FileTabMenuProps) {
  const { t } = useTranslation();
  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [onClose]);

  return createPortal(
    <div
      className="fp-menu"
      style={{ left: x, top: y }}
      onPointerDown={(event) => event.stopPropagation()}
      onContextMenu={(event) => event.preventDefault()}
      role="menu"
    >
      <button className="fp-menu-item" onClick={onOpen}>{t("filePreview.open")}</button>
      <div className="fp-menu-sep" />
      <div className="fp-menu-label">{t("filePreview.openWith")}</div>
      {editors.length === 0 ? (
        <div className="fp-menu-empty">{t("filePreview.noEditorDetected")}</div>
      ) : editors.map((editor) => (
        <button
          key={editor.id}
          className="fp-menu-item"
          onClick={() => onOpenWith(editor.id)}
        >
          {editor.label}
        </button>
      ))}
    </div>,
    document.body,
  );
}
