import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Check } from "@/components/ui/icons";
import { useOllamaModels } from "@/hooks/use-ollama-models";
import type { RegistryModel } from "@/types/agent";
import "./ollama.css";

interface ModelSearchProps {
  query: string;
  setQuery: (q: string) => void;
  results: RegistryModel[];
  setResults: (list: RegistryModel[]) => void;
  searching: boolean;
  setSearching: (b: boolean) => void;
  onSelectFamily: (familyName: string) => void;
  selectedFamily: string | null;
}

export function ModelSearch({
  query, setQuery, results, setResults,
  searching, setSearching,
  onSelectFamily, selectedFamily,
}: ModelSearchProps) {
  const { t } = useTranslation();
  const { models: localModels } = useOllamaModels();

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
  }, [query, setSearching, setResults]);

  const isFamilyInstalled = (familyName: string): boolean => {
    return localModels.some((m) => m.name.startsWith(`${familyName}:`));
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", flex: 1, minHeight: 0 }}>
      <div style={{ padding: "6px var(--space-sm) 0" }}>
        <input
          className="ollama-search-input"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => { if (e.code === "Enter") handleSearch(); }}
          placeholder={t("ollama.searchPlaceholder")}
        />
      </div>
      <div style={{ flex: 1, overflowY: "auto", padding: "2px var(--space-sm) 20px" }}>
        {searching && (
          <div style={{
            padding: "var(--space-md)", fontSize: "var(--text-sm)",
            color: "var(--ink-faint)",
          }}>
            {t("history.loading")}
          </div>
        )}
        {!searching && results.length === 0 && query.trim() === "" && (
          <div style={{
            padding: "var(--space-md)", fontSize: "var(--text-xs)",
            color: "var(--ink-faint)", fontStyle: "italic",
          }}>
            {t("ollama.searchHint")}
          </div>
        )}
        {results.map((m) => {
          const installed = isFamilyInstalled(m.name);
          const isActive = selectedFamily ? selectedFamily === m.name : false;
          return (
            <div
              key={m.name}
              className={`ollama-model-item ${isActive ? "active" : ""}`}
              onClick={() => onSelectFamily(m.name)}
              style={{
                display: "flex", justifyContent: "space-between", alignItems: "center",
                cursor: "pointer",
              }}
            >
              <div style={{ minWidth: 0, flex: 1 }}>
                <div style={{ fontSize: "var(--text-sm)", color: "var(--ink)" }}>
                  {m.name}
                </div>
                {m.description && (
                  <div style={{
                    fontSize: "var(--text-xs)", color: "var(--ink-faint)",
                    maxWidth: 200, overflow: "hidden",
                    textOverflow: "ellipsis", whiteSpace: "nowrap",
                  }}>
                    {m.description}
                  </div>
                )}
              </div>
              {installed && <Check size={14} style={{ color: "var(--select-text)" }} />}
            </div>
          );
        })}
      </div>
    </div>
  );
}
