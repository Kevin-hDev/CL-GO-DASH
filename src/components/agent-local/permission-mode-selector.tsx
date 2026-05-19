import { useMemo, useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, Check } from "@/components/ui/icons";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import { focusLocalListItem, useLocalListNavigation } from "@/hooks/use-local-list-navigation";
import "./permission-mode-selector.css";

interface Props {
  mode: PermissionMode;
  onChange: (mode: PermissionMode) => void;
}

const MODES: PermissionMode[] = ["chat", "manual", "auto"];

export function PermissionModeSelector({ mode, onChange }: Props) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const navItems = useMemo(() => MODES.map((m) => ({
    id: modeNavId(m),
    onSelect: () => {
      onChange(m);
      setOpen(false);
    },
  })), [onChange]);
  const { activate, getItemRef, isActive, listProps } = useLocalListNavigation({
    items: navItems,
    enabled: open,
    selectedId: modeNavId(mode),
    onEscape: () => setOpen(false),
  });

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
          onChange("chat");
          setOpen(false);
          break;
        case "2":
          onChange("manual");
          setOpen(false);
          break;
        case "3":
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

  useEffect(() => {
    if (!open) return;
    requestAnimationFrame(() => focusLocalListItem(rootRef.current, 1));
  }, [open]);

  const label = t(`permissionMode.${mode}Label`);

  return (
    <div className="perm-mode-root" ref={rootRef} data-keyboard-scope={open ? "local" : undefined}>
      <button
        type="button"
        className="perm-mode-trigger"
        onClick={() => setOpen((v) => !v)}
        onKeyDown={(event) => {
          if (!open && (event.key === "Enter" || event.key === " " || event.key === "ArrowDown")) {
            setOpen(true);
            return;
          }
          if (open) listProps.onKeyDown(event);
        }}
        title={t("permissionMode.toggleHint")}
      >
        <span className={`perm-mode-text perm-mode-${mode}`}>{label}</span>
        <CaretDown size={10} className="perm-mode-caret" />
      </button>

      {open && (
        <div className="perm-mode-dropdown" role="menu" tabIndex={-1} onKeyDown={listProps.onKeyDown}>
          {MODES.map((m, i) => {
            const navId = modeNavId(m);
            return (
            <button
              key={m}
              type="button"
              role="menuitem"
              className="perm-mode-option"
              ref={getItemRef(navId)}
              tabIndex={isActive(navId) ? 0 : -1}
              data-local-nav-item="true"
              data-local-nav-active={isActive(navId) ? "true" : undefined}
              onFocus={() => activate(navId)}
              onMouseEnter={() => activate(navId)}
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
          ); })}
        </div>
      )}
    </div>
  );
}

function modeNavId(mode: PermissionMode) {
  return `mode:${mode}`;
}
