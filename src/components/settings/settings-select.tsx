import { useState, useRef, useCallback, useMemo } from "react";
import { CaretDown, CaretRight, Check, MagnifyingGlass } from "@/components/ui/icons";
import { useClickOutside } from "@/hooks/use-click-outside";
import { useKeyboard } from "@/hooks/use-keyboard";
import "./settings-select.css";

export interface SelectOption {
  value: string;
  label: string;
  icon?: React.ReactNode;
}

export interface SelectGroup {
  label: string;
  options: SelectOption[];
}

interface SettingsSelectProps {
  options?: SelectOption[];
  groups?: SelectGroup[];
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  searchable?: boolean;
  searchPlaceholder?: string;
}

export function SettingsSelect({
  options,
  groups,
  value,
  onChange,
  placeholder,
  searchable,
  searchPlaceholder,
}: SettingsSelectProps) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const ref = useRef<HTMLDivElement>(null);

  const defaultCollapsed = useMemo(() => {
    if (!groups) return {};
    const map: Record<string, boolean> = {};
    for (const g of groups) {
      map[g.label] = true;
    }
    return map;
  }, [groups]);

  const [collapsed, setCollapsed] = useState<Record<string, boolean>>(defaultCollapsed);

  const close = useCallback(() => {
    setOpen(false);
    setQuery("");
  }, []);

  useClickOutside(ref, close);
  useKeyboard({ onEscape: open ? close : undefined });

  const allOptions = useMemo(() => {
    if (options) return options;
    if (!groups) return [];
    return groups.flatMap((g) => g.options);
  }, [options, groups]);

  const filtered = useMemo(() => {
    if (!searchable || !query) return null;
    const q = query.toLowerCase();
    return allOptions.filter((o) => o.label.toLowerCase().includes(q));
  }, [allOptions, query, searchable]);

  const current = allOptions.find((o) => o.value === value);
  const displayLabel = current?.label ?? placeholder ?? "—";
  const isOverflowing = displayLabel.length > 20;

  const handleSelect = (val: string) => {
    onChange(val);
    close();
  };

  const toggleGroup = (label: string) => {
    setCollapsed((prev) => ({ ...prev, [label]: !prev[label] }));
  };

  const renderOption = (opt: SelectOption) => (
    <div
      key={opt.value}
      className={`ss-option ${opt.value === value ? "active" : ""}`}
      onClick={() => handleSelect(opt.value)}
    >
      <div className="ss-option-check">
        {opt.value === value && <Check size={14} weight="bold" />}
      </div>
      {opt.icon}
      <span className="ss-option-label">{opt.label}</span>
    </div>
  );

  const renderContent = () => {
    if (filtered) {
      if (filtered.length === 0) return <div className="ss-empty">--</div>;
      return filtered.map(renderOption);
    }

    if (groups) {
      return groups.map((g) => {
        const isCollapsed = collapsed[g.label] ?? true;
        return (
          <div key={g.label} className="ss-group">
            <div className="ss-group-header" onClick={() => toggleGroup(g.label)}>
              <CaretRight
                size={12}
                weight="bold"
                className={`ss-group-caret ${isCollapsed ? "" : "open"}`}
              />
              <span>{g.label}</span>
              <span className="ss-group-count">{g.options.length}</span>
            </div>
            {!isCollapsed && g.options.map(renderOption)}
          </div>
        );
      });
    }

    if (options) {
      if (options.length === 0) return <div className="ss-empty">--</div>;
      return options.map(renderOption);
    }

    return <div className="ss-empty">--</div>;
  };

  return (
    <div className={`ss-wrap ${open ? "open" : ""}`} ref={ref}>
      <div className="ss-trigger" onClick={() => setOpen(!open)} title={isOverflowing ? displayLabel : undefined}>
        <span className="ss-trigger-label">
          {displayLabel}
        </span>
        <CaretDown size={14} weight="bold" className="ss-trigger-icon" />
      </div>

      {open && (
        <div className={`ss-panel ${groups ? "ss-panel-fixed" : ""}`}>
          {searchable && (
            <div className="ss-search">
              <MagnifyingGlass size={14} />
              <input
                autoFocus
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder={searchPlaceholder ?? ""}
              />
            </div>
          )}
          {renderContent()}
        </div>
      )}
    </div>
  );
}
