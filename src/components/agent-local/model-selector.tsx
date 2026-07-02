import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { useKeyboard } from "@/hooks/use-keyboard";
import { floatingMenuPortalRoot, useFloatingMenuPosition } from "@/hooks/use-floating-menu-position";
import { focusLocalListItem } from "@/hooks/use-local-list-navigation";
import { Brain, CaretDown, MagnifyingGlass } from "@/components/ui/icons";
import {
  useAvailableModels,
  type AvailableModel,
} from "@/hooks/use-available-models";
import { useFavoriteModels } from "@/hooks/use-favorite-models";
import {
  normalizeReasoningMode,
  reasoningModeOptions,
  type ReasoningMode,
} from "@/lib/reasoning-modes";
import { ModelSelectorList } from "./model-selector-list";
import "./model-selector.css";

interface ModelSelectorProps {
  selectedModel: string;
  selectedProvider: string;
  onSelect: (model: string, provider: string) => void;
  reasoningMode?: string | null;
  onReasoningModeChange: (mode: ReasoningMode) => void;
  align?: "left" | "right";
}

export function ModelSelector({
  selectedModel,
  selectedProvider,
  onSelect,
  reasoningMode,
  onReasoningModeChange,
  align = "left",
}: ModelSelectorProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const ref = useRef<HTMLDivElement>(null);
  const { anchorRef, floatingRef, floatingStyle, updateFloatingPosition } =
    useFloatingMenuPosition(open, align, 4);
  const { groups } = useAvailableModels();
  const { favorites, isFavorite, toggle: toggleFav } = useFavoriteModels();

  useKeyboard({ onEscape: () => setOpen(false) });

  useEffect(() => {
    if (!open) return;
    const onDoc = (event: MouseEvent) => {
      const target = event.target as Node;
      if (ref.current?.contains(target) || floatingRef.current?.contains(target)) return;
      setOpen(false);
    };
    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, [floatingRef, open]);

  const selectedEntry = useMemo(() => {
    const list = groups.get(selectedProvider);
    return list?.find((m) => m.id === selectedModel) ?? null;
  }, [groups, selectedProvider, selectedModel]);

  const modeOptions = useMemo(() => reasoningModeOptions(selectedEntry), [selectedEntry]);
  const selectedReasoningMode = normalizeReasoningMode(reasoningMode, modeOptions);
  const selectedReasoningLabel = modeOptions.find((option) => option.mode === selectedReasoningMode)?.labelKey;
  const simpleReasoningToggle = selectedReasoningMode === "auto"
    && modeOptions.every((option) => option.mode === "off" || option.mode === "auto");
  const showReasoningModes = modeOptions.length > 0;

  const filteredGroups = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return groups;
    const out = new Map<string, AvailableModel[]>();
    for (const [key, models] of groups.entries()) {
      const kept = models.filter((m) => m.id.toLowerCase().includes(q));
      if (kept.length > 0) out.set(key, kept);
    }
    return out;
  }, [groups, query]);
  const focusDropdownList = (direction: 1 | -1) => {
    focusLocalListItem(floatingRef.current, direction);
  };

  const dropdown = open ? (
    <div
      ref={floatingRef}
      style={floatingStyle}
      data-keyboard-scope="local"
      className={`ms-dropdown${showReasoningModes ? " ms-dropdown-with-reasoning" : ""}`}
    >
      <div className="ms-main">
        <div className="ms-search">
          <MagnifyingGlass size="var(--icon-sm)" className="ms-search-icon" />
          <input
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
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

        <div className="ms-list">
          <ModelSelectorList
            groups={filteredGroups}
            favorites={favorites}
            isFavorite={isFavorite}
            onToggleFavorite={(p, m) => void toggleFav(p, m)}
            selectedModel={selectedModel}
            selectedProvider={selectedProvider}
            onSelect={(model, provider) => {
              onSelect(model, provider);
              setOpen(false);
              setQuery("");
            }}
          />
        </div>
      </div>
      {showReasoningModes && (
        <aside className="ms-reasoning-panel" aria-label={t("agentLocal.reasoningTitle")}>
          <div className="ms-reasoning-title">{t("agentLocal.reasoningTitle")}</div>
          {modeOptions.map((option) => (
            <button
              key={option.mode}
              type="button"
              className={`ms-reasoning-option${selectedReasoningMode === option.mode ? " ms-reasoning-option-active" : ""}`}
              onClick={() => onReasoningModeChange(option.mode)}
            >
              {t(option.labelKey)}
            </button>
          ))}
        </aside>
      )}
    </div>
  ) : null;
  const portalRoot = floatingMenuPortalRoot();

  return (
    <div
      ref={ref}
      className={`ms-root${align === "right" ? " ms-root-align-right" : ""}`}
      data-keyboard-scope={open ? "local" : undefined}
    >
      <button
        ref={(node) => { anchorRef.current = node; }}
        type="button"
        onClick={() => setOpen(!open)}
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
        className={`ms-trigger${selectedModel ? "" : " ms-trigger-empty"}`}
      >
        <span className="ms-trigger-label">{selectedModel || t("agentLocal.selectModel")}</span>
        {selectedReasoningLabel && selectedReasoningMode !== "off" && (
          <span className="ms-trigger-reasoning" title={t(selectedReasoningLabel)}>
            <Brain size="var(--icon-xs)" className="ms-trigger-reasoning-icon" />
            {!simpleReasoningToggle && <span>{t(selectedReasoningLabel)}</span>}
          </span>
        )}
        <CaretDown size="var(--icon-2xs)" className="ms-trigger-caret" />
      </button>
      {dropdown ? createPortal(dropdown, portalRoot) : null}
    </div>
  );
}
