import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ModelInstallButton } from "./model-install-button";
import type { ModelInfo } from "@/types/agent";
import "./ollama.css";

interface ModelProfileProps {
  modelName: string;
}

export function ModelProfile({ modelName }: ModelProfileProps) {
  const [info, setInfo] = useState<ModelInfo | null>(null);

  useEffect(() => {
    invoke<ModelInfo>("show_ollama_model", { name: modelName })
      .then(setInfo)
      .catch((e: unknown) => console.warn("Erreur profil modèle:", e));
  }, [modelName]);

  if (!info) {
    return (
      <div style={{ padding: "var(--space-md)", fontSize: "var(--text-sm)", color: "var(--ink-faint)" }}>
        Chargement...
      </div>
    );
  }

  const rows: [string, string][] = [
    ["Paramètres", info.parameter_size],
    ["Architecture", info.architecture],
    ["MoE", info.is_moe ? "Oui" : "Non"],
    ["Contexte", `${(info.context_length / 1024).toFixed(0)}K tokens`],
    ["Famille", info.family],
    ["Quantization", info.quantization],
    ["Capabilities", info.capabilities.join(", ")],
    ["Audio natif", info.has_audio ? "Oui" : "Non"],
    ["Licence", info.license.slice(0, 80) || "—"],
  ];

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div className="ollama-detail-header">
        <span className="ollama-detail-name">{info.name}</span>
        <ModelInstallButton modelName={modelName} />
      </div>
      <div style={{ flex: 1, overflowY: "auto", padding: "var(--space-md)" }}>
        <table className="ollama-profile-table">
          <tbody>
            {rows.map(([label, value]) => (
              <tr key={label}>
                <td>{label}</td>
                <td>{value}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
