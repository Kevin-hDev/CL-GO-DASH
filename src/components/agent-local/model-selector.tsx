import { useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useClickOutside } from "@/hooks/use-click-outside";
import { useKeyboard } from "@/hooks/use-keyboard";
import { Check, MagnifyingGlass } from "@/components/ui/icons";
import {
  useAvailableModels,
  type AvailableModel,
} from "@/hooks/use-available-models";
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
          <span style={{ marginLeft: 4, color: "var(--pulse)" }}>Étendue</span>
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
            {filteredGroups.size === 0 ? (
              <div className="ms-empty">{t("agentLocal.modelEmpty")}</div>
            ) : (
              Array.from(filteredGroups.entries()).map(([providerId, models]) => (
                <div key={providerId}>
                  <div className="ms-section">
                    {models[0]?.provider_name ?? providerId}
                  </div>
                  {models.map((m) => {
                    const isSelected =
                      m.id === selectedModel && m.provider_id === selectedProvider;
                    return (
                      <div
                        key={`${m.provider_id}:${m.id}`}
                        className={`ms-item ${isSelected ? "active" : ""}`}
                        onClick={() => {
                          onSelect(m.id, m.provider_id);
                          setOpen(false);
                          setQuery("");
                        }}
                      >
                        <span className="ms-item-name">{m.id}</span>
                        <span className="ms-item-right">
                          {m.hint && <span className="ms-hint">{m.hint}</span>}
                          {isSelected && <Check size={12} />}
                        </span>
                      </div>
                    );
                  })}
                </div>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  );
}
