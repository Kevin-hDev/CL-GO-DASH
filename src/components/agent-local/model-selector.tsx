import { useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useClickOutside } from "@/hooks/use-click-outside";
import { useKeyboard } from "@/hooks/use-keyboard";
import { MagnifyingGlass } from "@/components/ui/icons";
import {
  useAvailableModels,
  type AvailableModel,
} from "@/hooks/use-available-models";
import { useFavoriteModels } from "@/hooks/use-favorite-models";
import { ModelSelectorList } from "./model-selector-list";
import "./model-selector.css";

interface ModelSelectorProps {
  selectedModel: string;
  selectedProvider: string;
  onSelect: (model: string, provider: string) => void;
  thinkingEnabled: boolean;
  onToggleThinking: () => void;
}

export function ModelSelector({
  selectedModel,
  selectedProvider,
  onSelect,
  thinkingEnabled,
  onToggleThinking,
}: ModelSelectorProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const ref = useRef<HTMLDivElement>(null);
  const { groups } = useAvailableModels();
  const { favorites, isFavorite, toggle: toggleFav } = useFavoriteModels();

  useClickOutside(ref, () => setOpen(false));
  useKeyboard({ onEscape: () => setOpen(false) });

  const selectedEntry = useMemo(() => {
    const list = groups.get(selectedProvider);
    return list?.find((m) => m.id === selectedModel) ?? null;
  }, [groups, selectedProvider, selectedModel]);

  const showThinkingToggle = selectedEntry?.supports_thinking ?? false;

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

  return (
    <div ref={ref} style={{ position: "relative" }}>
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="ms-trigger"
      >
        {selectedModel}
        {thinkingEnabled && showThinkingToggle && (
          <span style={{ marginLeft: 4, color: "var(--select-text)" }}>{t("agentLocal.thinkingToggle")}</span>
        )}
      </button>
      {open && (
        <div className="ms-dropdown">
          <div className="ms-search">
            <MagnifyingGlass size={14} className="ms-search-icon" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t("agentLocal.modelSearch")}
              className="ms-search-input"
              autoFocus
            />
          </div>

          {showThinkingToggle && (
            <div className="ms-thinking" onClick={onToggleThinking}>
              <span>{t("agentLocal.thinkingToggle")}</span>
              <span
                style={{
                  color: thinkingEnabled ? "var(--pulse)" : "var(--ink-faint)",
                }}
              >
                {thinkingEnabled ? "ON" : "OFF"}
              </span>
            </div>
          )}

          <div className="ms-list">
            <ModelSelectorList
              groups={filteredGroups}
              favorites={favorites}
              isFavorite={isFavorite}
              onToggleFavorite={toggleFav}
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
      )}
    </div>
  );
}
