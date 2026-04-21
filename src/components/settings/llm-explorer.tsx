import { useState, useCallback, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { LlmFamilyGrid } from "./llm-family-grid";
import { LlmModelList } from "./llm-model-list";
import { LlmModelDetail } from "./llm-model-detail";
import type { RegistryModelInfo, FamilyGroup } from "./llm-types";
import "./llm-explorer.css";

type View =
  | { kind: "idle" }
  | { kind: "search"; query: string; results: RegistryModelInfo[] }
  | { kind: "family-models"; family: string; models: RegistryModelInfo[] }
  | { kind: "detail"; model: RegistryModelInfo; prev: View };

function viewIs(v: View, k: string): boolean {
  return v.kind.startsWith(k);
}

export function LlmExplorer() {
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const [view, setView] = useState<View>({ kind: "idle" });
  const [families, setFamilies] = useState<FamilyGroup[]>([]);
  const [showFamilies, setShowFamilies] = useState(false);
  const prevShowFamilies = useRef(false);

  useEffect(() => {
    invoke<FamilyGroup[]>("list_registry_families")
      .then(setFamilies)
      .catch(() => {});
  }, []);

  const handleSearch = useCallback(async () => {
    if (!query.trim()) return;
    try {
      const results = await invoke<RegistryModelInfo[]>("search_registry", { query: query.trim() });
      setView({ kind: "search", query: query.trim(), results });
      setShowFamilies(false);
    } catch (e) {
      console.warn("[llm] search error:", e);
    }
  }, [query]);

  const handleFamilyClick = useCallback(async (family: string) => {
    try {
      const models = await invoke<RegistryModelInfo[]>("list_family_models", { family });
      setView({ kind: "family-models", family, models });
      setShowFamilies(false);
    } catch (e) {
      console.warn("[llm] family error:", e);
    }
  }, []);

  const handleModelClick = useCallback((model: RegistryModelInfo) => {
    prevShowFamilies.current = showFamilies;
    setView((prev) => ({ kind: "detail", model, prev }));
  }, [showFamilies]);

  const handleBack = useCallback(() => {
    if (viewIs(view, "detail")) {
      const prev = (view as View & { kind: "detail" }).prev;
      setView(prev);
      if (prev.kind.startsWith("family")) setShowFamilies(false);
      else if (prev.kind.startsWith("idle")) setShowFamilies(prevShowFamilies.current);
    } else {
      setView({ kind: "idle" });
      setShowFamilies(true);
    }
  }, [view]);

  const toggleFamilies = useCallback(() => {
    setShowFamilies((prev) => !prev);
    if (!viewIs(view, "idle")) setView({ kind: "idle" });
  }, [view]);

  return (
    <div style={{ padding: 24, overflowY: "auto", flex: 1 }}>
      <div style={{ maxWidth: 700, width: "100%", margin: "0 auto" }}>
        <h2 style={{ fontSize: "var(--text-xl)", fontWeight: 700, color: "var(--ink)", marginBottom: 20 }}>
          LLM
        </h2>

        <div className="llm-search-bar">
          <input
            className="llm-search-input"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => { if (e.key.startsWith("Ent")) handleSearch(); }}
            placeholder={t("settings.llm.searchPlaceholder")}
          />
          <button
            className={`llm-family-btn ${showFamilies ? "active" : ""}`}
            onClick={toggleFamilies}
          >
            {t("settings.llm.families")}
          </button>
        </div>

        {showFamilies && viewIs(view, "idle") && (
          <LlmFamilyGrid families={families} onSelect={handleFamilyClick} />
        )}

        {viewIs(view, "search") && (
          <LlmModelList
            models={(view as View & { kind: "search" }).results}
            title={(view as View & { kind: "search" }).query}
            onSelect={handleModelClick}
          />
        )}

        {viewIs(view, "family") && (
          <LlmModelList
            models={(view as View & { kind: "family-models" }).models}
            title={(view as View & { kind: "family-models" }).family}
            onSelect={handleModelClick}
            onBack={() => { setView({ kind: "idle" }); setShowFamilies(true); }}
          />
        )}

        {viewIs(view, "detail") && (
          <LlmModelDetail
            model={(view as View & { kind: "detail" }).model}
            onBack={handleBack}
          />
        )}
      </div>
    </div>
  );
}
