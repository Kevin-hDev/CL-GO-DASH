import { useTranslation } from "react-i18next";
import { Check } from "@/components/ui/icons";

const LANGUAGES = [
  { id: "en", label: "English", flag: "🇬🇧" },
  { id: "fr", label: "Français", flag: "🇫🇷" },
] as const;

export function LanguagePicker() {
  const { i18n } = useTranslation();
  const current = i18n.language;

  function changeLang(id: string) {
    i18n.changeLanguage(id);
    localStorage.setItem("clgo-language", id);
  }

  return (
    <div>
      <label style={{
        display: "block", fontSize: "var(--text-xs)", color: "var(--ink-muted)",
        textTransform: "uppercase", letterSpacing: "0.5px", marginBottom: 12,
      }}>
        Language
      </label>
      <div style={{ display: "flex", gap: 12 }}>
        {LANGUAGES.map((lang) => {
          const active = lang.id === current;
          return (
            <div
              key={lang.id}
              onClick={() => changeLang(lang.id)}
              style={{
                flex: 1, padding: "14px 16px",
                borderRadius: "var(--radius-md)", cursor: "pointer",
                display: "flex", alignItems: "center", justifyContent: "center",
                gap: 8, position: "relative",
                border: active ? "2px solid var(--pulse)" : "1px solid var(--edge)",
                transition: "all 200ms ease-out",
              }}
            >
              <span style={{ fontSize: "1.2rem" }}>{lang.flag}</span>
              <span style={{
                fontSize: "var(--text-sm)",
                fontWeight: active ? 600 : 400,
                color: active ? "var(--pulse)" : "var(--ink)",
              }}>
                {lang.label}
              </span>
              {active && (
                <div style={{ position: "absolute", top: 6, right: 6, color: "var(--pulse)" }}>
                  <Check size={14} weight="bold" />
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
