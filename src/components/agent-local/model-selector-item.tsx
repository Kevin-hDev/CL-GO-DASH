import type { KeyboardEvent } from "react";
import { Check, Star } from "@/components/ui/icons";
import type { AvailableModel } from "@/hooks/use-available-models";
import type { useLocalListNavigation } from "@/hooks/use-local-list-navigation";

type NavApi = ReturnType<typeof useLocalListNavigation>;

interface ModelSelectorItemProps {
  model: AvailableModel;
  isSelected: boolean;
  isFav: boolean;
  navId: string;
  activate: NavApi["activate"];
  getItemRef: NavApi["getItemRef"];
  isNavActive: NavApi["isActive"];
  onItemKeyDown: (event: KeyboardEvent) => void;
  onSelect: (model: string, provider: string) => void;
  onToggleFav: (provider: string, model: string) => void;
}

export function ModelSelectorItem({
  model: m,
  isSelected,
  isFav,
  navId,
  activate,
  getItemRef,
  isNavActive,
  onItemKeyDown,
  onSelect,
  onToggleFav,
}: ModelSelectorItemProps) {
  const isPaid = !m.is_free && !m.is_local;
  return (
    <div
      className={`ms-item ${isSelected ? "active" : ""} ${isPaid ? "ms-item-paid" : ""}`}
      role="button"
      ref={getItemRef(navId)}
      tabIndex={isNavActive(navId) ? 0 : -1}
      data-local-nav-item="true"
      data-local-nav-active={isNavActive(navId) ? "true" : undefined}
      onFocus={() => activate(navId)}
      onMouseEnter={() => activate(navId)}
      onKeyDown={onItemKeyDown}
      onClick={() => onSelect(m.id, m.provider_id)}
    >
      <span
        className={`ms-star ${isFav ? "ms-star-on" : ""}`}
        role="button"
        tabIndex={0}
        onClick={(event) => {
          event.stopPropagation();
          onToggleFav(m.provider_id, m.id);
        }}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.stopPropagation();
            onToggleFav(m.provider_id, m.id);
          }
        }}
      >
        <Star size="var(--icon-xs)" weight={isFav ? "fill" : "regular"} />
      </span>
      <span className="ms-item-name">{m.id}</span>
      <span className="ms-item-right">
        {m.supports_vision && <span className="ms-badge-vision" title="Vision">V</span>}
        {m.supports_tools && <span className="ms-badge-tools" title="Tools">T</span>}
        {m.hint && <span className="ms-hint">{m.hint}</span>}
        {isSelected && <Check size="var(--icon-xs)" />}
      </span>
    </div>
  );
}
