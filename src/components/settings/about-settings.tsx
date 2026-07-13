import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { getVersion, getTauriVersion } from "@tauri-apps/api/app";
import { open } from "@tauri-apps/plugin-shell";
import { ArrowSquareOut } from "@/components/ui/icons";
import { ThemedIcon } from "@/components/ui/themed-icon";
import { cn } from "@/lib/utils";
import { IS_LINUX, IS_MAC, IS_WINDOWS } from "@/lib/platform";
import { SettingsCard } from "./settings-card";
import logoDark from "@/assets/logo-dark.png";
import logoLight from "@/assets/logo-light.png";
import "./about-settings.css";

const GITHUB_URL = "https://github.com/Kevin-hDev/CL-GO-DASH";

export function AboutSettings() {
  const { t } = useTranslation();
  const [appVersion, setAppVersion] = useState("");
  const [tauriVersion, setTauriVersion] = useState("");

  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => {});
    getTauriVersion().then(setTauriVersion).catch(() => {});
  }, []);

  const platform = IS_MAC ? "macOS"
    : IS_WINDOWS ? "Windows"
    : IS_LINUX ? "Linux" : "—";

  return (
    <div className="as-root">
      <div className="as-inner">
        <div className="as-hero">
          <ThemedIcon darkSrc={logoDark} lightSrc={logoLight} size="4rem" />
          <h2 className="as-title">
            CL-GO
          </h2>
          <span className="as-subtitle">
            {t("about.description")}
          </span>
        </div>

        <SettingsCard>
          <InfoRow label={t("about.version")} value={appVersion || "—"} />
          <InfoRow label={t("about.tauri")} value={tauriVersion || "—"} />
          <InfoRow label={t("about.os")} value={platform} last />
        </SettingsCard>

        <div className="as-links">
          <button
            type="button"
            onClick={() => void open(GITHUB_URL)}
            className="as-github-btn"
          >
            {t("about.viewOnGithub")} <ArrowSquareOut size="var(--icon-sm)" />
          </button>
        </div>
      </div>
    </div>
  );
}

function InfoRow({ label, value, last }: { label: string; value: string; last?: boolean }) {
  return (
    <div className={cn("as-info-row", !last && "as-info-row-border")}>
      <span className="as-info-label">
        {label}
      </span>
      <span className="as-info-value">{value}</span>
    </div>
  );
}
