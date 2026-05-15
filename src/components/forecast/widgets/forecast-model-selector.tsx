import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, MagnifyingGlass } from "@/components/ui/icons";
import { useClickOutside } from "@/hooks/use-click-outside";
import { useFavoriteModels } from "@/hooks/use-favorite-models";
import { useKeyboard } from "@/hooks/use-keyboard";
import type { AvailableModel } from "@/hooks/use-available-models";
import { ModelSelectorList } from "@/components/agent-local/model-selector-list";
import {
  getForecastFamilyId,
  getForecastFamilyKey,
  groupForecastModels,
} from "../forecast-model-meta";
import { useAvailableForecastModels } from "../use-available-forecast-models";
import "@/components/agent-local/model-selector.css";
import "./export-dropdown.css";
import "./forecast-model-selector.css";

interface ForecastModelSelectorProps {
  selectedModelId: string;
  selectionReady: boolean;
  onSelectModel: (modelId: string) => void;
}

export function ForecastModelSelector({
  selectedModelId,
  selectionReady,
  onSelectModel,
}: ForecastModelSelectorProps) {
  const { t } = useTranslation();
  const ref = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const { models } = useAvailableForecastModels();
  const { favorites, isFavorite, toggle } = useFavoriteModels();

  useClickOutside(ref, () => setOpen(false));
  useKeyboard({ onEscape: () => setOpen(false) });

  const selectedModel = useMemo(
    () => models.find((model) => model.id === selectedModelId) ?? null,
    [models, selectedModelId],
  );

  useEffect(() => {
    if (!selectionReady) return;
    if (models.length === 0) return;
    if (!selectedModelId || !models.some((model) => model.id === selectedModelId)) {
      onSelectModel(models[0].id);
    }
  }, [models, selectedModelId, selectionReady, onSelectModel]);

  const groups = useMemo(() => {
    const lowered = query.trim().toLowerCase();
    const visible = lowered
      ? models.filter((model) => model.id.toLowerCase().includes(lowered))
      : models;
    const mapped = new Map<string, AvailableModel[]>();
    for (const group of groupForecastModels(visible)) {
      const familyName = t(group.titleKey);
      mapped.set(
        group.id,
        group.models.map((model) => ({
          id: model.id,
          provider_id: getForecastFamilyId(model),
          provider_name: familyName === getForecastFamilyKey(group.id) ? group.id : familyName,
          is_local: !model.is_cloud,
          supports_tools: false,
          supports_vision: false,
          is_free: true,
          hint: model.params,
        })),
      );
    }
    return mapped;
  }, [models, query, t]);

  return (
    <div className="fmsel-wrapper" ref={ref}>
      <button
        className={`exd-trigger fmsel-trigger${selectedModel ? "" : " fmsel-trigger-empty"}`}
        type="button"
        onClick={() => setOpen((current) => !current)}
      >
        <span className="fmsel-trigger-label">
          {selectedModel?.display_name ?? t("forecast.config.model")}
        </span>
        <CaretDown size={14} className={`fmsel-caret ${open ? "open" : ""}`} />
      </button>
      {open && (
        <div className="ms-dropdown fmsel-dropdown">
          <div className="ms-search">
            <MagnifyingGlass size={14} className="ms-search-icon" />
            <input
              type="text"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder={t("agentLocal.modelSearch")}
              className="ms-search-input"
              autoFocus
            />
          </div>
          <div className="ms-list">
            <ModelSelectorList
              groups={groups}
              favorites={favorites}
              isFavorite={isFavorite}
              onToggleFavorite={(provider, model) => void toggle(provider, model)}
              selectedModel={selectedModelId}
              selectedProvider={selectedModel ? getForecastFamilyId(selectedModel) : ""}
              onSelect={(model) => {
                onSelectModel(model);
                setOpen(false);
                setQuery("");
              }}
            />
          </div>
        </div>
      )}
    </div>
  );
}
