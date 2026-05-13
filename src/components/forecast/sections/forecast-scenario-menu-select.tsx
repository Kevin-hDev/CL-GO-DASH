import { CaretDown } from "@/components/ui/icons";
import { useEffect, useRef, useState } from "react";

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
  const selected = options.find((option) => option.value === value);

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

  return (
    <div ref={rootRef} className={`fcs-menu ${className ?? ""}`.trim()}>
      <button
        className={`fcs-menu-trigger ${open ? "is-open" : ""}`}
        type="button"
        onClick={() => setOpen((current) => !current)}
      >
        <span className="fcs-menu-label">{selected?.label ?? placeholder ?? ""}</span>
        <CaretDown size={13} className={`fcs-menu-caret ${open ? "is-open" : ""}`} />
      </button>
      <div className={`fcs-menu-panel ${open ? "is-open" : ""}`}>
        {options.map((option) => (
          <button
            key={option.value}
            className={`fcs-menu-option ${option.value === value ? "is-selected" : ""}`}
            type="button"
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
