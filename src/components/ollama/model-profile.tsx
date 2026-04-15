import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import ReactMarkdown from "react-markdown";
import { ModelInstallButton } from "./model-install-button";
import { useOllamaModels } from "@/hooks/use-ollama-models";
import type { RegistryModelDetails, RegistryTag, ModelInfo } from "@/types/agent";
import "./ollama.css";

interface ModelProfileProps {
  familyName: string;
  variantFullName: string | null;
}

export function ModelProfile({ familyName, variantFullName }: ModelProfileProps) {
  const { models: localModels } = useOllamaModels();
  const [details, setDetails] = useState<RegistryModelDetails | null>(null);
  const [tags, setTags] = useState<RegistryTag[]>([]);
  const [localInfo, setLocalInfo] = useState<ModelInfo | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    setDetails(null);
    setLocalInfo(null);
    Promise.all([
      invoke<RegistryModelDetails>("get_registry_model_details", { name: familyName }),
      invoke<RegistryTag[]>("list_registry_tags", { name: familyName }),
    ])
      .then(([d, ts]) => { setDetails(d); setTags(ts); })
      .catch((e) => console.warn("Fiche modèle:", e))
      .finally(() => setLoading(false));
  }, [familyName]);

  useEffect(() => {
    if (!variantFullName) { setLocalInfo(null); return; }
    const installed = localModels.find((m) => m.name === variantFullName);
    if (!installed) { setLocalInfo(null); return; }
    invoke<ModelInfo>("show_ollama_model", { name: variantFullName })
      .then(setLocalInfo)
      .catch(() => setLocalInfo(null));
  }, [variantFullName, localModels]);

  const currentTag = useMemo(() => {
    if (!variantFullName) return null;
    const tagName = variantFullName.split(":")[1];
    return tags.find((t) => t.name === tagName) ?? null;
  }, [variantFullName, tags]);

  const displayName = variantFullName ?? familyName;
  const installedLocal = variantFullName
    ? localModels.find((m) => m.name === variantFullName)
    : null;
  const hasUpdate = Boolean(
    installedLocal && currentTag && installedLocal.digest_short !== currentTag.digest_short,
  );

  if (loading) {
    return (
      <div style={{ padding: "var(--space-md)", fontSize: "var(--text-sm)", color: "var(--ink-faint)" }}>
        Chargement de la fiche…
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div className="ollama-detail-header">
        <span className="ollama-detail-name">{displayName}</span>
        {variantFullName && (
          <ModelInstallButton
            fullName={variantFullName}
            isInstalled={Boolean(installedLocal)}
            hasUpdate={hasUpdate}
          />
        )}
      </div>

      {details && (
        <div style={{
          padding: "var(--space-md)",
          borderBottom: "1px solid var(--edge)",
          fontSize: "var(--text-sm)", color: "var(--ink)",
          lineHeight: 1.5,
        }}>
          {details.description_short}
        </div>
      )}

      <div style={{ padding: "var(--space-md)", borderBottom: "1px solid var(--edge)" }}>
        <table className="ollama-profile-table">
          <tbody>
            {details && details.capabilities.length > 0 && (
              <tr><td>Capabilities</td><td>{details.capabilities.join(", ")}</td></tr>
            )}
            {currentTag?.size_gb && (
              <tr><td>Taille fichier</td><td>{currentTag.size_gb} GB</td></tr>
            )}
            {localInfo?.parameter_size && (
              <tr><td>Paramètres</td><td>{localInfo.parameter_size}</td></tr>
            )}
            {(currentTag?.context_length ?? details?.context_length) && (
              <tr>
                <td>Contexte</td>
                <td>{((currentTag?.context_length ?? details?.context_length ?? 0) / 1024).toFixed(0)}K tokens</td>
              </tr>
            )}
            {localInfo?.quantization && (
              <tr><td>Quantization</td><td>{localInfo.quantization}</td></tr>
            )}
            {localInfo?.architecture && (
              <tr><td>Architecture</td><td>{localInfo.architecture}</td></tr>
            )}
            {localInfo && (
              <tr><td>MoE</td><td>{localInfo.is_moe ? "Oui" : "Non"}</td></tr>
            )}
            {currentTag?.digest_short && (
              <tr><td>Digest</td><td style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)" }}>{currentTag.digest_short}</td></tr>
            )}
          </tbody>
        </table>
      </div>

      {details?.description_long_markdown && (
        <div
          className="ollama-readme"
          style={{
            flex: 1, overflow: "auto",
            padding: "var(--space-md)",
            fontSize: "var(--text-sm)", color: "var(--ink)",
            lineHeight: 1.6,
          }}
        >
          <ReactMarkdown>{details.description_long_markdown}</ReactMarkdown>
        </div>
      )}
    </div>
  );
}
