import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Check } from "@/components/ui/icons";
import { useOllamaModels } from "@/hooks/use-ollama-models";
import type { RegistryTag, OllamaModel } from "@/types/agent";
import "./ollama.css";

interface ModelVariantsListProps {
  familyName: string;
  selectedVariant: string | null;
  onSelectVariant: (fullName: string) => void;
  onBack: () => void;
}

export function ModelVariantsList({
  familyName, selectedVariant, onSelectVariant, onBack,
}: ModelVariantsListProps) {
  const { t } = useTranslation();
  const { models: localModels } = useOllamaModels();
  const [tags, setTags] = useState<RegistryTag[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    setError(null);
    invoke<RegistryTag[]>("list_registry_tags", { name: familyName })
      .then((list) => setTags(list))
      .catch((e: unknown) => setError(String(e)))
      .finally(() => setLoading(false));
  }, [familyName]);

  const findLocal = (tagName: string): OllamaModel | undefined => {
    const fullName = `${familyName}:${tagName}`;
    return localModels.find((m) => m.name === fullName);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", flex: 1, minHeight: 0 }}>
      <button
        onClick={onBack}
        className="ollama-btn"
        style={{
          margin: "var(--space-sm)",
          display: "flex", alignItems: "center", gap: 6,
          justifyContent: "flex-start",
        }}
      >
        ← {familyName}
      </button>

      {loading && (
        <div style={{
          padding: "var(--space-md)", fontSize: "var(--text-sm)",
          color: "var(--ink-faint)",
        }}>
          {t("ollama.loadingVariants")}
        </div>
      )}

      {error && (
        <div style={{
          padding: "var(--space-md)", fontSize: "var(--text-xs)",
          color: "#e66",
        }}>
          {error}
        </div>
      )}

      <div style={{ flex: 1, overflowY: "auto", padding: "var(--space-sm)", paddingBottom: 20 }}>
        {tags.map((tag) => {
          const local = findLocal(tag.name);
          const installed = Boolean(local);
          const hasUpdate = installed && !local?.is_customized && local?.digest_short !== tag.digest_short;
          const fullName = `${familyName}:${tag.name}`;
          const isActive = selectedVariant ? selectedVariant === fullName : false;

          return (
            <div
              key={tag.name}
              className={`ollama-model-item ${isActive ? "active" : ""}`}
              onClick={() => onSelectVariant(fullName)}
              style={{
                display: "flex", justifyContent: "space-between",
                alignItems: "center", cursor: "pointer",
              }}
            >
              <div style={{ minWidth: 0, flex: 1 }}>
                <div style={{ fontSize: "var(--text-sm)", color: "var(--ink)" }}>
                  {tag.name}
                </div>
                <div style={{
                  fontSize: "var(--text-xs)", color: "var(--ink-faint)",
                }}>
                  {tag.size_gb ? `${tag.size_gb} GB` : "—"}
                  {tag.context_length ? ` · ${(tag.context_length / 1024).toFixed(0)}K ctx` : ""}
                </div>
              </div>
              {hasUpdate && (
                <span style={{
                  fontSize: "var(--text-xs)",
                  color: "#ea580c",
                  fontWeight: 600,
                }}>
                  {t("ollama.update")}
                </span>
              )}
              {installed && !hasUpdate && (
                <Check size={14} style={{ color: "var(--select-text)" }} />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
