import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Check } from "@/components/ui/icons";
import type { RegistryModel } from "@/types/agent";
import "./ollama.css";

interface ModelSearchProps {
  onSelectModel: (model: RegistryModel) => void;
}

export function ModelSearch({ onSelectModel }: ModelSearchProps) {
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<RegistryModel[]>([]);
  const [searching, setSearching] = useState(false);

  const handleSearch = useCallback(async () => {
    if (!query.trim()) return;
    setSearching(true);
    try {
      const list = await invoke<RegistryModel[]>("search_ollama_models", { query: query.trim() });
      setResults(list);
    } catch (e: unknown) {
      console.warn("Erreur recherche:", e);
      setResults([]);
    } finally {
      setSearching(false);
    }
  }, [query]);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div style={{ padding: "var(--space-sm)" }}>
        <input
          className="ollama-search-input"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => { if (e.key.startsWith("Ent")) handleSearch(); }}
          placeholder={t("ollama.searchPlaceholder")}
        />
      </div>
      <div style={{ flex: 1, overflowY: "auto", padding: "var(--space-sm)" }}>
        {searching && (
          <div style={{ padding: "var(--space-md)", fontSize: "var(--text-sm)", color: "var(--ink-faint)" }}>
            {t("history.loading")}
          </div>
        )}
        {results.map((m) => (
          <div
            key={m.name}
            className="ollama-model-item"
            onClick={() => onSelectModel(m)}
            style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}
          >
            <div>
              <div style={{ fontSize: "var(--text-sm)", color: "var(--ink)" }}>{m.name}</div>
              <div style={{
                fontSize: "var(--text-xs)", color: "var(--ink-faint)",
                maxWidth: 200, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
              }}>
                {m.description}
              </div>
            </div>
            {m.is_installed && <Check size={14} style={{ color: "var(--pulse)" }} />}
          </div>
        ))}
      </div>
    </div>
  );
}
