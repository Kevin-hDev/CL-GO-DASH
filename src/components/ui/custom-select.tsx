import { useCallback, useRef, useState } from "react";
import { useClickOutside } from "@/hooks/use-click-outside";
import "./custom-select.css";

export interface SelectOption {
  value: string;
  label: string;
}

interface CustomSelectProps {
  options: SelectOption[];
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
}

export function CustomSelect({
  options,
  value,
  onChange,
  placeholder,
  disabled,
}: CustomSelectProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const close = useCallback(() => setOpen(false), []);
  useClickOutside(ref, close);

  const selected = options.find((o) => o.value === value);

  return (
    <div ref={ref} className="cs-wrapper" data-keyboard-scope={open ? "local" : undefined}>
      <button
        type="button"
        className="cs-trigger"
        onClick={() => !disabled && setOpen(!open)}
        disabled={disabled}
      >
        <span className={selected ? "cs-trigger-label" : "cs-trigger-label cs-placeholder"}>
          {selected?.label ?? placeholder ?? "—"}
        </span>
        <span className="cs-trigger-caret">▾</span>
      </button>
      {open && (
        <div className="cs-dropdown" role="listbox">
          {options.map((opt) => (
            <div
              key={opt.value}
              className={`cs-option ${opt.value === value ? "active" : ""}`}
              role="option"
              tabIndex={0}
              aria-selected={opt.value === value}
              onClick={() => {
                onChange(opt.value);
                setOpen(false);
              }}
              onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { onChange(opt.value); setOpen(false); } }}
            >
              {opt.label}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
