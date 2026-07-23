import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { CaretDown, MagnifyingGlass } from "@/components/ui/icons";
import { useFavoriteModels } from "@/hooks/use-favorite-models";
import {
  floatingMenuPortalRoot,
  useFloatingMenuPosition,
} from "@/hooks/use-floating-menu-position";
import { useKeyboard } from "@/hooks/use-keyboard";
import { focusLocalListItem } from "@/hooks/use-local-list-navigation";
import type { AvailableModel } from "@/hooks/use-available-models";
import { ModelSelectorList } from "@/components/agent-local/model-selector-list";
import {
  getForecastFamilyId,
  getForecastFamilyKey,
  groupForecastModels,
} from "../forecast-model-meta";
import { useAvailableForecastModels } from "../use-available-forecast-models";
import type { ForecastSelectionMode } from "../model-selection/forecast-selection-types";
import "@/components/agent-local/model-selector.css";
import "./export-dropdown.css";
import "./forecast-model-selector.css";

interface ForecastModelSelectorProps {
  selectedModelId: string;
  selectionMode: ForecastSelectionMode;
  selectionReady: boolean;
  onSelectModel: (modelId: string) => void;
  onModeChange: (mode: ForecastSelectionMode) => void;
  align?: "left" | "right";
}

export function ForecastModelSelector({
  selectedModelId,
  selectionMode,
  selectionReady,
  onSelectModel,
  onModeChange,
  align = "left",
}: ForecastModelSelectorProps) {
  const { t } = useTranslation();
  const ref = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const { anchorRef, floatingRef, floatingStyle, updateFloatingPosition } =
    useFloatingMenuPosition(open, align, 6, "auto");
  const { models } = useAvailableForecastModels();
  const { favorites, isFavorite, toggle } = useFavoriteModels();

  useKeyboard({ onEscape: () => setOpen(false) });

  useEffect(() => {
    if (!open) return;
    const close = (event: MouseEvent) => {
      const target = event.target as Node;
      if (ref.current?.contains(target) || floatingRef.current?.contains(target)) return;
      setOpen(false);
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, [floatingRef, open]);

  const selectedModel = useMemo(
    () => models.find((model) => model.id === selectedModelId) ?? null,
    [models, selectedModelId],
  );

  useEffect(() => {
    if (!selectionReady || selectionMode !== "manual") return;
    if (models.length === 0) return;
    if (!selectedModelId || !models.some((model) => model.id === selectedModelId)) {
      onSelectModel(models[0].id);
    }
  }, [models, selectedModelId, selectionMode, selectionReady, onSelectModel]);

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
          display_name: model.display_name,
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
  const focusDropdownList = (direction: 1 | -1) => {
    focusLocalListItem(floatingRef.current, direction);
  };

  const dropdown = open ? (
    <div
      ref={floatingRef}
      style={floatingStyle}
      className="ms-dropdown fmsel-dropdown"
      data-keyboard-scope="local"
    >
      <div className="ms-search">
        <MagnifyingGlass size="var(--icon-sm)" className="ms-search-icon" />
        <input
          type="text"
          value={query}
          onChange={(event) => {
            setQuery(event.target.value);
            requestAnimationFrame(updateFloatingPosition);
          }}
          onKeyDown={(event) => {
            if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
            event.preventDefault();
            focusDropdownList(event.key === "ArrowDown" ? 1 : -1);
          }}
          placeholder={t("agentLocal.modelSearch")}
          className="ms-search-input"
          autoFocus
        />
      </div>
      <div className="fmsel-mode" aria-label={t("forecast.selection.label")}>
        <span className="fmsel-mode-label">{t("forecast.selection.label")}</span>
        <div className="fmsel-mode-options">
          {(["manual", "auto"] as const).map((mode) => (
            <button
              key={mode}
              type="button"
              disabled={!selectionReady}
              className={`fmsel-mode-option ${selectionMode === mode ? "is-active" : ""}`}
              aria-pressed={selectionMode === mode}
              onClick={() => onModeChange(mode)}
            >
              {t(`forecast.selection.${mode}`)}
            </button>
          ))}
        </div>
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
  ) : null;

  return (
    <div className="fmsel-wrapper" ref={ref} data-keyboard-scope={open ? "local" : undefined}>
      <button
        ref={(node) => { anchorRef.current = node; }}
        className={`exd-trigger fmsel-trigger${selectedModel ? "" : " fmsel-trigger-empty"}`}
        type="button"
        disabled={!selectionReady}
        aria-busy={!selectionReady}
        onClick={() => setOpen((current) => !current)}
        onKeyDown={(event) => {
          if (!open && (event.key === "ArrowDown" || event.key === "ArrowUp")) {
            setOpen(true);
            requestAnimationFrame(() => focusDropdownList(event.key === "ArrowDown" ? 1 : -1));
            return;
          }
          if (open && (event.key === "ArrowDown" || event.key === "ArrowUp")) {
            event.preventDefault();
            focusDropdownList(event.key === "ArrowDown" ? 1 : -1);
          }
        }}
      >
        <span className="fmsel-trigger-label">
          {selectionMode === "auto"
            ? t("forecast.selection.auto")
            : selectedModel?.display_name ?? t("forecast.config.model")}
        </span>
        <CaretDown size="var(--icon-sm)" className={`fmsel-caret ${open ? "open" : ""}`} />
      </button>
      {dropdown ? createPortal(dropdown, floatingMenuPortalRoot()) : null}
    </div>
  );
}
