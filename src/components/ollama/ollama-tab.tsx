import { useState } from "react";
import { useTranslation } from "react-i18next";
import { EmptyState } from "@/components/ui/empty-state";
import { ModelfileList } from "./modelfile-list";
import { ModelfileViewer } from "./modelfile-viewer";
import { ModelSearch } from "./model-search";
import { ModelProfile } from "./model-profile";
import type { RegistryModel } from "@/types/agent";
import "./ollama.css";

type SubTab = "modelfile" | "models";

export function OllamaTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const [subTab, setSubTab] = useState<SubTab>("modelfile");
  const [selectedModel, setSelectedModel] = useState<string | null>(null);
  const [profileModel, setProfileModel] = useState<string | null>(null);

  const list = (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div className="ollama-subtabs">
        {(["modelfile", "models"] as const).map((tab) => (
          <button
            key={tab}
            className={`ollama-subtab ${subTab === tab ? "active" : ""}`}
            onClick={() => setSubTab(tab)}
          >
            {tab === "modelfile" ? "Modelfile" : "Models"}
          </button>
        ))}
      </div>
      {subTab === "modelfile" ? (
        <ModelfileList selectedModel={selectedModel} onSelect={setSelectedModel} />
      ) : (
        <ModelSearch onSelectModel={(m: RegistryModel) => setProfileModel(m.name)} />
      )}
    </div>
  );

  const detail = (() => {
    if (subTab === "modelfile" && selectedModel) return <ModelfileViewer modelName={selectedModel} />;
    if (subTab === "models" && profileModel) return <ModelProfile modelName={profileModel} />;
    return (
      <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center" }}>
        <EmptyState message={t("ollama.selectModel")} />
      </div>
    );
  })();

  return { list, detail };
}
