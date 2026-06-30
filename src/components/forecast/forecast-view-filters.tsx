import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ChevronDown } from "lucide-react";
import { useTranslation } from "react-i18next";
import { focusLocalListItem, useLocalListNavigation, type LocalListNavItem } from "@/hooks/use-local-list-navigation";
import type {
  ForecastLayerGroup,
  ForecastLayerState,
} from "./forecast-layer-matrix";
import { FilterGroup, FilterItem } from "./forecast-view-filter-items";
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
  const panelRef = useRef<HTMLDivElement | null>(null);
  const pendingFocusDirection = useRef<1 | -1>(1);

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

  const toggleGroup = useCallback((groupId: string) => {
    setOpenGroups((current) =>
      current.includes(groupId)
        ? current.filter((id) => id !== groupId)
        : [...current, groupId]
    );
  }, []);

  const navItems = useMemo<LocalListNavItem[]>(() => groups.flatMap((group) => {
    const expanded = openGroups.includes(group.id);
    const groupNavId = `group:${group.id}`;
    const children = expanded ? group.items.map((item) => ({
      id: `item:${item.id}`,
      disabled: !item.interactive,
      onSelect: () => onChange({ ...layers, [item.id]: !layers[item.id] }),
    })) : [];
    return [
      {
        id: groupNavId,
        onSelect: () => toggleGroup(group.id),
        onArrowRight: () => setOpenGroups((current) => current.includes(group.id) ? current : [...current, group.id]),
        onArrowLeft: () => setOpenGroups((current) => current.filter((id) => id !== group.id)),
      },
      ...children,
    ];
  }), [groups, layers, onChange, openGroups, toggleGroup]);

  const nav = useLocalListNavigation({
    items: navItems,
    enabled: open,
    onEscape: () => {
      setOpen(false);
      setOpenGroups([]);
    },
  });

  useEffect(() => {
    if (open) focusLocalListItem(panelRef.current, pendingFocusDirection.current);
  }, [open]);

  return (
    <div ref={rootRef} className="fcf-root" data-keyboard-scope="local">
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
        onKeyDown={(event) => {
          if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
          event.preventDefault();
          event.stopPropagation();
          pendingFocusDirection.current = event.key === "ArrowDown" ? 1 : -1;
          if (open) focusLocalListItem(panelRef.current, pendingFocusDirection.current);
          else setOpen(true);
        }}
      >
        <span>{t("forecast.view.filters.button")}</span>
        <ChevronDown size="var(--icon-sm)" className={`fcf-chevron ${open ? "is-open" : ""}`} />
      </button>
      <div ref={panelRef} className={`fcf-panel ${open ? "is-open" : ""}`} role="menu" tabIndex={-1} onKeyDown={nav.listProps.onKeyDown}>
        {groups.map((group) => (
          <FilterGroup
            key={group.id}
            groupId={group.id}
            open={openGroups.includes(group.id)}
            title={t(group.titleKey)}
            onToggle={toggleGroup}
            nav={nav}
          >
            {group.items.length > 0 ? (
              <>
                {group.items.map((item) => (
                  <FilterItem
                    key={item.id}
                    navId={`item:${item.id}`}
                    label={item.label}
                    checked={Boolean(layers[item.id])}
                    disabled={!item.interactive}
                    nav={nav}
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
