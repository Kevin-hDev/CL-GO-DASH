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
import "./model-profile.css";

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
      <div className="mp-loading">
        {t("ollama.loadingProfile")}
      </div>
    );
  }

  const rows = buildSpecRows(t, details, currentTag, localInfo);

  return (
    <div className="mp-root">
      <div className="mp-inner">
        <div className="mp-header">
          <h2 className="mp-title">
            {displayName}
          </h2>
          {variantFullName && (
            <ModelInstallButton
              fullName={variantFullName}
              isInstalled={Boolean(installedLocal)}
              hasUpdate={hasUpdate}
              sizeGb={currentTag?.size_gb ?? undefined}
            />
          )}
        </div>

        {details?.description_short && (
          <div className="mp-description">
            {details.description_short}
          </div>
        )}

        <SettingsCard>
          {rows.map((r, i) => (
            <div
              key={i}
              className={`mp-spec-row${i < rows.length - 1 ? " mp-spec-row-border" : ""}`}
            >
              <span className="mp-spec-label">{r.label}</span>
              <span className={r.mono ? "mp-spec-value-mono" : "mp-spec-value"}>
                {r.value}
              </span>
            </div>
          ))}
        </SettingsCard>

        {details?.description_long_markdown && (
          <div className="mp-translation-wrapper">
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
        <div className="ollama-readme mp-readme">
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
