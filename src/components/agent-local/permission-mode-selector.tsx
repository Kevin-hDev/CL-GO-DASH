import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, Check } from "@/components/ui/icons";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import "./permission-mode-selector.css";

interface Props {
  mode: PermissionMode;
  onChange: (mode: PermissionMode) => void;
}

const MODES: PermissionMode[] = ["manual", "auto"];

export function PermissionModeSelector({ mode, onChange }: Props) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDoc = (e: MouseEvent) => {
      if (!rootRef.current?.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      const pressed = e.key;
      if (pressed.startsWith("Esc")) {
        setOpen(false);
        return;
      }
      switch (pressed) {
        case "1":
          onChange("manual");
          setOpen(false);
          break;
        case "2":
          onChange("auto");
          setOpen(false);
          break;
      }
    };
    document.addEventListener("mousedown", onDoc);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDoc);
      document.removeEventListener("keydown", onKey);
    };
  }, [open, onChange]);

  const label = t(`permissionMode.${mode}Label`);

  return (
    <div className="perm-mode-root" ref={rootRef}>
      <button
        type="button"
        className="perm-mode-trigger"
        onClick={() => setOpen((v) => !v)}
        title={t("permissionMode.toggleHint")}
      >
        <span className={`perm-mode-text perm-mode-${mode}`}>{label}</span>
        <CaretDown size={10} className="perm-mode-caret" />
      </button>

      {open && (
        <div className="perm-mode-dropdown" role="menu">
          <div className="perm-mode-dropdown-header">{t("permissionMode.title")}</div>
          {MODES.map((m, i) => (
            <button
              key={m}
              type="button"
              role="menuitem"
              className="perm-mode-option"
              onClick={() => { onChange(m); setOpen(false); }}
            >
              <span className={`perm-mode-option-label perm-mode-${m}`}>
                {t(`permissionMode.${m}Label`)}
              </span>
              <span className="perm-mode-option-right">
                {m === mode && <Check size={12} />}
                <span className="perm-mode-option-num">{i + 1}</span>
              </span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
