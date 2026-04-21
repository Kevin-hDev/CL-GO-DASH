import { useTranslation } from "react-i18next";
import type { PersonalityFile } from "@/types/personality";
import { FileText } from "@/components/ui/icons";
import { RoundToggle } from "@/components/heartbeat/round-toggle";
import "./personality-list.css";

interface PersonalityListProps {
  files: PersonalityFile[];
  selectedPath: string | null;
  injectionState: Record<string, boolean>;
  selectedFileName: string;
  onSelect: (path: string) => void;
  onToggleInjection: (enabled: boolean) => void;
}

export function PersonalityList({
  files,
  selectedPath,
  injectionState,
  selectedFileName,
  onSelect,
  onToggleInjection,
}: PersonalityListProps) {
  const { t } = useTranslation();

  const isAgentMd = selectedFileName === "AGENT.md";
  const isInjected = isAgentMd || (injectionState[selectedFileName] ?? false);

  return (
    <>
      <div className="pers-header">
        <span className="pers-title">{t("personality.files")}</span>
        <div style={{ visibility: isAgentMd || !selectedFileName ? "hidden" : "visible" }}>
          <RoundToggle
            checked={isInjected}
            onChange={onToggleInjection}
            title={isInjected ? t("personality.injected") : t("personality.inject")}
          />
        </div>
      </div>
      <div className="pers-content">
        {files.map((f) => {
          const name = f.name;
          const injected = name === "AGENT.md" || (injectionState[name] ?? false);
          return (
            <div
              key={f.name}
              className={`pers-item ${selectedPath === f.path ? "active" : ""}`}
              onClick={() => onSelect(f.path)}
            >
              <div className="pers-icon"><FileText size={16} weight="duotone" /></div>
              <div className="pers-item-body">
                <div className="pers-item-name">{f.name}</div>
                <div className="pers-item-desc">{f.description}</div>
              </div>
              {injected && <div className="pers-inject-dot" title={t("personality.injected")} />}
            </div>
          );
        })}
      </div>
    </>
  );
}
