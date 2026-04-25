import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import { ModelInstallButton } from "./model-install-button";
import { TranslationControls } from "./translation-controls";
import { SettingsCard } from "@/components/settings/settings-card";
import { useOllamaModels } from "@/hooks/use-ollama-models";
import type { RegistryModelDetails, RegistryTag, ModelInfo } from "@/types/agent";
import "./ollama.css";
import "./ollama-details.css";

interface ModelProfileProps {
  familyName: string;
  variantFullName: string | null;
}

export function ModelProfile({ familyName, variantFullName }: ModelProfileProps) {
  const { t } = useTranslation();
  const { models: localModels } = useOllamaModels();
  const [details, setDetails] = useState<RegistryModelDetails | null>(null);
  const [tags, setTags] = useState<RegistryTag[]>([]);
  const [localInfo, setLocalInfo] = useState<ModelInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [translated, setTranslated] = useState<string | null>(null);
  const [translatedLang, setTranslatedLang] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    setDetails(null);
    setLocalInfo(null);
    setTranslated(null);
    setTranslatedLang(null);
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
    return tags.find((tg) => tg.name === tagName) ?? null;
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
        {t("ollama.loadingProfile")}
      </div>
    );
  }

  const rows = buildSpecRows(t, details, currentTag, localInfo);

  return (
    <div style={{ overflowY: "auto", flex: 1 }}>
      <div style={{ padding: 24, maxWidth: 600, width: "100%", margin: "0 auto" }}>
        <div style={{
          display: "flex", alignItems: "center",
          justifyContent: "space-between", marginBottom: 8,
        }}>
          <h2 style={{ fontSize: "var(--text-xl)", fontWeight: 700, color: "var(--ink)", margin: 0 }}>
            {displayName}
          </h2>
          {variantFullName && (
            <ModelInstallButton
              fullName={variantFullName}
              isInstalled={Boolean(installedLocal)}
              hasUpdate={hasUpdate}
            />
          )}
        </div>

        {details?.description_short && (
          <div style={{
            fontSize: "var(--text-sm)", color: "var(--ink-muted)",
            lineHeight: 1.5, marginBottom: 28,
          }}>
            {details.description_short}
          </div>
        )}

        <SettingsCard>
          {rows.map((r, i) => (
            <div key={i} style={{
              display: "flex", justifyContent: "space-between", alignItems: "center",
              padding: "10px 20px",
              borderBottom: i < rows.length - 1 ? "1px solid var(--edge)" : "none",
              fontSize: "var(--text-sm)",
            }}>
              <span style={{ color: "var(--ink-muted)", fontSize: "var(--text-xs)" }}>{r.label}</span>
              <span style={{ color: "var(--ink)", fontFamily: r.mono ? "var(--font-mono)" : undefined, fontSize: r.mono ? "var(--text-xs)" : undefined }}>
                {r.value}
              </span>
            </div>
          ))}
        </SettingsCard>

        {details?.description_long_markdown && (
          <div style={{ marginTop: 24 }}>
            <TranslationControls
              modelName={familyName}
              originalText={details.description_long_markdown}
              currentLang={translatedLang}
              onChange={(lang, text) => { setTranslatedLang(lang); setTranslated(text); }}
            />
          </div>
        )}
      </div>

      {details?.description_long_markdown && (
        <div
          className="ollama-readme"
          style={{
            padding: "0 24px 24px",
            fontSize: "var(--text-sm)", color: "var(--ink)",
            lineHeight: 1.6,
          }}
        >
          <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeRaw, rehypeSanitize]}>
            {translated ?? details.description_long_markdown}
          </ReactMarkdown>
        </div>
      )}
    </div>
  );
}

interface SpecRow { label: string; value: string; mono?: boolean }

function buildSpecRows(
  t: (k: string) => string,
  details: RegistryModelDetails | null,
  tag: RegistryTag | null,
  info: ModelInfo | null,
): SpecRow[] {
  const rows: SpecRow[] = [];
  if (details?.capabilities?.length) rows.push({ label: t("ollama.capabilities"), value: details.capabilities.join(", ") });
  if (tag?.size_gb) rows.push({ label: t("ollama.fileSize"), value: `${tag.size_gb} GB` });
  if (info?.parameter_size) rows.push({ label: t("ollama.paramsLabel"), value: info.parameter_size });
  const ctx = tag?.context_length ?? details?.context_length;
  if (ctx) rows.push({ label: t("ollama.context"), value: `${(ctx / 1024).toFixed(0)}${t("ollama.contextTokens")}` });
  if (info?.quantization) rows.push({ label: t("ollama.quantization"), value: info.quantization });
  if (info?.architecture) rows.push({ label: t("ollama.architecture"), value: info.architecture });
  if (info) rows.push({ label: t("ollama.moe"), value: info.is_moe ? t("ollama.yes") : t("ollama.no") });
  if (tag?.digest_short) rows.push({ label: t("ollama.digest"), value: tag.digest_short, mono: true });
  return rows;
}
