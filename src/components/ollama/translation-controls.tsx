import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import "./ollama.css";

const LANGUAGES: { code: string; label: string }[] = [
  { code: "fr", label: "Français" },
  { code: "es", label: "Español" },
  { code: "de", label: "Deutsch" },
  { code: "zh", label: "中文" },
];

interface TranslationControlsProps {
  modelName: string;
  originalText: string;
  currentLang: string | null;
  onChange: (lang: string | null, translatedText: string | null) => void;
}

export function TranslationControls({
  modelName, originalText, currentLang, onChange,
}: TranslationControlsProps) {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const translate = async (lang: string) => {
    setLoading(true);
    setError(null);
    try {
      const translated = await invoke<string>("translate_description", {
        modelName,
        text: originalText,
        targetLang: lang,
      });
      onChange(lang, translated);
    } catch (e: unknown) {
      setError(String(e));
      onChange(null, null);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{
      display: "flex", alignItems: "center", gap: 8,
      padding: "var(--space-sm) var(--space-md)",
      borderBottom: "1px solid var(--edge)",
      fontSize: "var(--text-xs)",
      color: "var(--ink-faint)",
    }}>
      <span style={{ textTransform: "uppercase", letterSpacing: "0.05em" }}>
        {t("ollama.description")}
      </span>

      <div style={{ flex: 1 }} />

      {loading && (
        <span style={{ color: "var(--ink-faint)", fontStyle: "italic" }}>
          {t("ollama.translating")}
        </span>
      )}

      {!loading && error && (
        <span
          style={{ color: "#e66", maxWidth: 280, overflow: "hidden", textOverflow: "ellipsis" }}
          title={error}
        >
          {error}
        </span>
      )}

      {!loading && currentLang && (
        <button className="ollama-btn" onClick={() => onChange(null, null)}>
          {t("ollama.original")}
        </button>
      )}

      <select
        value={currentLang ?? ""}
        onChange={(e) => {
          const lang = e.target.value;
          if (lang) translate(lang);
        }}
        disabled={loading}
        style={{
          padding: "4px 8px",
          background: "var(--void)",
          border: "1px solid var(--edge)",
          borderRadius: "var(--radius-sm)",
          color: "var(--ink)",
          fontSize: "var(--text-xs)",
          cursor: "pointer",
        }}
      >
        <option value="">{t("ollama.translate")}</option>
        {LANGUAGES.map((l) => (
          <option key={l.code} value={l.code}>{l.label}</option>
        ))}
      </select>
    </div>
  );
}
