import { CaretDown } from "@/components/ui/icons";
import { useEffect, useMemo, useRef, useState } from "react";
import { focusLocalListItem, useLocalListNavigation, type LocalListNavItem } from "@/hooks/use-local-list-navigation";

interface ForecastScenarioMenuSelectProps {
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
}

export function ForecastScenarioMenuSelect({
  value,
  options,
  onChange,
  placeholder,
  className,
}: ForecastScenarioMenuSelectProps) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement | null>(null);
  const panelRef = useRef<HTMLDivElement | null>(null);
  const pendingFocusDirection = useRef<1 | -1>(1);
  const selected = options.find((option) => option.value === value);
  const navItems = useMemo<LocalListNavItem[]>(() => options.map((option) => ({
    id: option.value,
    onSelect: () => {
      onChange(option.value);
      setOpen(false);
    },
  })), [onChange, options]);
  const nav = useLocalListNavigation({
    items: navItems,
    enabled: open,
    selectedId: value,
    onEscape: () => setOpen(false),
  });

  useEffect(() => {
    if (!open) return;
    const handlePointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    window.addEventListener("mousedown", handlePointerDown);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("mousedown", handlePointerDown);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [open]);

  useEffect(() => {
    if (open) focusLocalListItem(panelRef.current, pendingFocusDirection.current);
  }, [open]);

  return (
    <div ref={rootRef} className={`fcs-menu ${className ?? ""}`.trim()} data-keyboard-scope="local">
      <button
        className={`fcs-menu-trigger ${open ? "is-open" : ""}`}
        type="button"
        onClick={() => setOpen((current) => !current)}
        onKeyDown={(event) => {
          if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
          event.preventDefault();
          event.stopPropagation();
          pendingFocusDirection.current = event.key === "ArrowDown" ? 1 : -1;
          if (open) focusLocalListItem(panelRef.current, pendingFocusDirection.current);
          else setOpen(true);
        }}
      >
        <span className="fcs-menu-label">{selected?.label ?? placeholder ?? ""}</span>
        <CaretDown size={13} className={`fcs-menu-caret ${open ? "is-open" : ""}`} />
      </button>
      <div ref={panelRef} className={`fcs-menu-panel ${open ? "is-open" : ""}`} role="menu" tabIndex={-1} onKeyDown={nav.listProps.onKeyDown}>
        {options.map((option) => (
          <button
            key={option.value}
            ref={nav.getItemRef(option.value)}
            className={`fcs-menu-option ${option.value === value ? "is-selected" : ""}`}
            type="button"
            data-local-nav-item="true"
            data-local-nav-active={nav.isActive(option.value) ? "true" : undefined}
            tabIndex={open && nav.isActive(option.value) ? 0 : -1}
            onFocus={() => nav.activate(option.value)}
            onMouseEnter={() => nav.activate(option.value)}
            onKeyDown={nav.listProps.onKeyDown}
            onClick={() => {
              onChange(option.value);
              setOpen(false);
            }}
          >
            <span className="fcs-menu-option-mark">
              {option.value === value ? <span className="fcs-menu-option-dot" /> : null}
            </span>
            <span>{option.label}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
