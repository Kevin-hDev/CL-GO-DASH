import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Check, CaretDown, CaretRight, Star } from "@/components/ui/icons";
import type { AvailableModel } from "@/hooks/use-available-models";
import type { FavoriteModel } from "@/hooks/use-favorite-models";

interface Props {
  groups: Map<string, AvailableModel[]>;
  favorites: FavoriteModel[];
  isFavorite: (provider: string, model: string) => boolean;
  onToggleFavorite: (provider: string, model: string) => void;
  selectedModel: string;
  selectedProvider: string;
  onSelect: (model: string, provider: string) => void;
}

export function ModelSelectorList({
  groups,
  favorites,
  isFavorite,
  onToggleFavorite,
  selectedModel,
  selectedProvider,
  onSelect,
}: Props) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  const toggle = (key: string) =>
    setExpanded((s) => {
      const n = new Set(s);
      n.has(key) ? n.delete(key) : n.add(key);
      return n;
    });

  // Section favoris : on retrouve les AvailableModel correspondants.
  const favModels: AvailableModel[] = [];
  for (const fav of favorites) {
    const list = groups.get(fav.provider);
    const m = list?.find((x) => x.id === fav.model);
    if (m) favModels.push(m);
  }

  if (groups.size === 0 && favModels.length === 0) {
    return <div className="ms-empty">{t("agentLocal.modelEmpty")}</div>;
  }

  return (
    <>
      {favModels.length > 0 && (
        <div>
          <div className="ms-section ms-section-fav">★ Favoris</div>
          {favModels.map((m) => (
            <ModelItem
              key={`fav:${m.provider_id}:${m.id}`}
              model={m}
              isSelected={m.id === selectedModel && m.provider_id === selectedProvider}
              isFav
              onSelect={onSelect}
              onToggleFav={onToggleFavorite}
            />
          ))}
        </div>
      )}

      {Array.from(groups.entries()).map(([providerId, models]) => {
        const isOpen = expanded.has(providerId);
        const name = models[0]?.provider_name ?? providerId;
        const freeCount = models.filter((m) => m.is_free || m.is_local).length;
        return (
          <div key={providerId}>
            <div className="ms-provider" onClick={() => toggle(providerId)}>
              <span className="ms-provider-caret">
                {isOpen ? <CaretDown size={12} /> : <CaretRight size={12} />}
              </span>
              <span className="ms-provider-name">{name}</span>
              <span className="ms-provider-count">
                {freeCount > 0 && freeCount < models.length
                  ? `${freeCount} free / ${models.length}`
                  : `${models.length}`}
              </span>
            </div>
            {isOpen && sortedModels(models).map((m) => (
              <ModelItem
                key={`${m.provider_id}:${m.id}`}
                model={m}
                isSelected={m.id === selectedModel && m.provider_id === selectedProvider}
                isFav={isFavorite(m.provider_id, m.id)}
                onSelect={onSelect}
                onToggleFav={onToggleFavorite}
              />
            ))}
          </div>
        );
      })}
    </>
  );
}

function sortedModels(models: AvailableModel[]): AvailableModel[] {
  return [...models].sort((a, b) => {
    const af = a.is_free || a.is_local ? 0 : 1;
    const bf = b.is_free || b.is_local ? 0 : 1;
    if (af !== bf) return af - bf;
    return a.id.localeCompare(b.id);
  });
}

function ModelItem({
  model: m,
  isSelected,
  isFav,
  onSelect,
  onToggleFav,
}: {
  model: AvailableModel;
  isSelected: boolean;
  isFav: boolean;
  onSelect: (model: string, provider: string) => void;
  onToggleFav: (provider: string, model: string) => void;
}) {
  const isPaid = !m.is_free && !m.is_local;
  return (
    <div
      className={`ms-item ${isSelected ? "active" : ""} ${isPaid ? "ms-item-paid" : ""}`}
      onClick={() => onSelect(m.id, m.provider_id)}
    >
      <span
        className={`ms-star ${isFav ? "ms-star-on" : ""}`}
        onClick={(e) => {
          e.stopPropagation();
          onToggleFav(m.provider_id, m.id);
        }}
      >
        <Star size={12} weight={isFav ? "fill" : "regular"} />
      </span>
      <span className="ms-item-name">{m.id}</span>
      <span className="ms-item-right">
        {m.supports_vision && <span className="ms-badge-vision" title="Vision">V</span>}
        {m.supports_tools && <span className="ms-badge-tools" title="Tools">T</span>}
        {m.hint && <span className="ms-hint">{m.hint}</span>}
        {isSelected && <Check size={12} />}
      </span>
    </div>
  );
}
