import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { getVersion, getTauriVersion } from "@tauri-apps/api/app";
import { open } from "@tauri-apps/plugin-shell";
import { ArrowSquareOut } from "@/components/ui/icons";
import { ThemedIcon } from "@/components/ui/themed-icon";
import { SettingsCard } from "./settings-card";
import logoDark from "@/assets/logo-dark.png";
import logoLight from "@/assets/logo-light.png";

const GITHUB_URL = "https://github.com/Kevin-hDev/CL-GO-DASH";

export function AboutSettings() {
  const { t } = useTranslation();
  const [appVersion, setAppVersion] = useState("");
  const [tauriVersion, setTauriVersion] = useState("");

  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => {});
    getTauriVersion().then(setTauriVersion).catch(() => {});
  }, []);

  const ua = navigator.userAgent;
  const platform = ua.includes("Mac") ? "macOS"
    : ua.includes("Windows") ? "Windows"
    : ua.includes("Linux") ? "Linux" : "—";

  return (
    <div style={{ padding: 24, overflowY: "auto", flex: 1 }}>
      <div style={{ maxWidth: 460, width: "100%", margin: "0 auto" }}>
        <div style={{
          display: "flex", flexDirection: "column",
          alignItems: "center", gap: 8, marginBottom: 32,
        }}>
          <ThemedIcon darkSrc={logoDark} lightSrc={logoLight} size="4rem" />
          <h2 style={{
            fontSize: "var(--text-xl)", fontWeight: 700,
            color: "var(--ink)", margin: 0,
          }}>
            CL-GO
          </h2>
          <span style={{
            fontSize: "var(--text-sm)", color: "var(--ink-muted)",
            textAlign: "center", maxWidth: 300,
          }}>
            {t("about.description")}
          </span>
        </div>

        <SettingsCard>
          <InfoRow label={t("about.version")} value={appVersion || "—"} />
          <InfoRow label={t("about.tauri")} value={tauriVersion || "—"} />
          <InfoRow label={t("about.os")} value={platform} last />
        </SettingsCard>

        <div style={{ marginTop: 16 }}>
          <button
            type="button"
            onClick={() => open(GITHUB_URL)}
            style={{
              width: "100%",
              display: "flex", alignItems: "center", justifyContent: "center",
              gap: 8, padding: "10px 16px",
              background: "var(--surface)", border: "1px solid var(--edge)",
              borderRadius: "var(--radius-sm)", color: "var(--ink)",
              fontSize: "var(--text-sm)", fontWeight: 500,
              cursor: "pointer", transition: "all 150ms ease",
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.borderColor = "var(--ink-faint)";
              e.currentTarget.style.background = "var(--surface-hover)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.borderColor = "var(--edge)";
              e.currentTarget.style.background = "var(--surface)";
            }}
          >
            {t("about.viewOnGithub")} <ArrowSquareOut size={14} />
          </button>
        </div>
      </div>
    </div>
  );
}

function InfoRow({ label, value, last }: { label: string; value: string; last?: boolean }) {
  return (
    <div style={{
      display: "flex", justifyContent: "space-between", alignItems: "center",
      padding: "10px 20px",
      borderBottom: last ? "none" : "1px solid var(--edge)",
      fontSize: "var(--text-sm)",
    }}>
      <span style={{
        color: "var(--ink-muted)", fontSize: "var(--text-xs)",
        fontWeight: 500, textTransform: "uppercase", letterSpacing: "0.5px",
      }}>
        {label}
      </span>
      <span style={{ color: "var(--ink)", fontFamily: "var(--font-mono)" }}>{value}</span>
    </div>
  );
}
