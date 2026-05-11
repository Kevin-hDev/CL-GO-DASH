import { useEffect, useRef, useState, type ReactNode } from "react";
import { ChevronDown, SlidersHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import type {
  ForecastLayerGroup,
  ForecastLayerState,
} from "./forecast-layer-matrix";
import "./forecast-view-filters.css";

interface ForecastViewFiltersProps {
  groups: ForecastLayerGroup[];
  layers: ForecastLayerState;
  onChange: (layers: ForecastLayerState) => void;
}

export function ForecastViewFilters({
  groups,
  layers,
  onChange,
}: ForecastViewFiltersProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [openGroups, setOpenGroups] = useState<string[]>([]);
  const rootRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!open) return;
    const handlePointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false);
        setOpenGroups([]);
      }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpen(false);
        setOpenGroups([]);
      }
    };
    window.addEventListener("mousedown", handlePointerDown);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("mousedown", handlePointerDown);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [open]);

  const toggleGroup = (groupId: string) => {
    setOpenGroups((current) =>
      current.includes(groupId)
        ? current.filter((id) => id !== groupId)
        : [...current, groupId]
    );
  };

  return (
    <div ref={rootRef} className="fcf-root">
      <button
        className={`fcf-trigger ${open ? "is-open" : ""}`}
        type="button"
        onClick={() => {
          setOpen((current) => {
            const next = !current;
            if (!next) setOpenGroups([]);
            return next;
          });
        }}
      >
        <SlidersHorizontal size={14} />
        <span>{t("forecast.view.filters.button")}</span>
        <ChevronDown size={14} className={`fcf-chevron ${open ? "is-open" : ""}`} />
      </button>
      <div className={`fcf-panel ${open ? "is-open" : ""}`}>
        {groups.map((group) => (
          <FilterGroup
            key={group.id}
            groupId={group.id}
            open={openGroups.includes(group.id)}
            title={t(group.titleKey)}
            onToggle={toggleGroup}
          >
            {group.items.length > 0 ? (
              <>
                {group.items.map((item) => (
                  <FilterItem
                    key={item.id}
                    label={item.label}
                    checked={Boolean(layers[item.id])}
                    disabled={!item.interactive}
                    onToggle={() =>
                      item.interactive
                        ? onChange({ ...layers, [item.id]: !layers[item.id] })
                        : undefined
                    }
                  />
                ))}
              </>
            ) : (
              <div className="fcf-empty">{group.emptyKey ? t(group.emptyKey) : ""}</div>
            )}
          </FilterGroup>
        ))}
      </div>
    </div>
  );
}

function FilterGroup({
  groupId,
  open,
  title,
  onToggle,
  children,
}: {
  groupId: string;
  open: boolean;
  title: string;
  onToggle: (groupId: string) => void;
  children: ReactNode;
}) {
  return (
    <div className="fcf-group">
      <button
        className="fcf-group-btn"
        type="button"
        onClick={() => onToggle(groupId)}
      >
        <span className="fcf-group-title">{title}</span>
        <ChevronDown size={14} className={`fcf-group-chevron ${open ? "is-open" : ""}`} />
      </button>
      <div className={`fcf-group-items ${open ? "is-open" : ""}`}>
        <div className="fcf-group-content">{children}</div>
      </div>
    </div>
  );
}

function FilterItem({
  label,
  checked,
  disabled,
  onToggle,
}: {
  label: string;
  checked: boolean;
  disabled?: boolean;
  onToggle?: () => void;
}) {
  return (
    <label className={`fcf-item ${disabled ? "is-disabled" : ""}`}>
      <input
        className="fcf-checkbox"
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={() => onToggle?.()}
      />
      <span className="fcf-item-label">{label}</span>
    </label>
  );
}
