import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useOllamaModels } from "@/hooks/use-ollama-models";
import { EmptyState } from "@/components/ui/empty-state";
import { ThemedIcon } from "@/components/ui/themed-icon";
import { ModelfileList } from "./modelfile-list";
import { ModelfileViewer } from "./modelfile-viewer";
import { ModelSearch } from "./model-search";
import { ModelVariantsList } from "./model-variants-list";
import { ModelProfile } from "./model-profile";
import modelfileDark from "@/assets/modelfile.png";
import modelfileLight from "@/assets/modelfile-light.png";
import modelsDark from "@/assets/models.png";
import modelsLight from "@/assets/models-light.png";
import type { RegistryModel } from "@/types/agent";
import "./ollama.css";

type SubTab = "modelfile" | "models";

export function OllamaTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const ollamaModels = useOllamaModels();
  const [subTab, setSubTab] = useState<SubTab>("modelfile");
  const [selectedInstalled, setSelectedInstalled] = useState<string | null>(null);
  const [selectedFamily, setSelectedFamily] = useState<string | null>(null);
  const [selectedVariant, setSelectedVariant] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<RegistryModel[]>([]);
  const [searching, setSearching] = useState(false);

  useEffect(() => {
    if (!selectedInstalled && ollamaModels.models.length > 0) {
      setSelectedInstalled(ollamaModels.models[0].name);
    }
  }, [ollamaModels.models, selectedInstalled]);

  const list = (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div className="ollama-subtabs">
        {(["modelfile", "models"] as const).map((tab) => (
          <button
            key={tab}
            className={`ollama-subtab ${subTab === tab ? "active" : ""}`}
            onClick={() => setSubTab(tab)}
          >
            {tab === "modelfile" ? (
              <><ThemedIcon darkSrc={modelfileDark} lightSrc={modelfileLight} size="1.6rem" /> Modelfile</>
            ) : (
              <><ThemedIcon darkSrc={modelsDark} lightSrc={modelsLight} size="1.6rem" /> Models</>
            )}
          </button>
        ))}
      </div>
      {subTab === "modelfile" ? (
        <ModelfileList
          models={ollamaModels.models}
          selectedModel={selectedInstalled}
          onSelect={setSelectedInstalled}
        />
      ) : selectedFamily ? (
        <ModelVariantsList
          familyName={selectedFamily}
          selectedVariant={selectedVariant}
          onSelectVariant={setSelectedVariant}
          onBack={() => { setSelectedFamily(null); setSelectedVariant(null); }}
        />
      ) : (
        <ModelSearch
          query={searchQuery}
          setQuery={setSearchQuery}
          results={searchResults}
          setResults={setSearchResults}
          searching={searching}
          setSearching={setSearching}
          onSelectFamily={(f) => { setSelectedFamily(f); setSelectedVariant(null); }}
          selectedFamily={selectedFamily}
        />
      )}
    </div>
  );

  const detail = (() => {
    if (subTab === "modelfile" && selectedInstalled) {
      return (
        <ModelfileViewer
          modelName={selectedInstalled}
          onDeleted={() => setSelectedInstalled(null)}
        />
      );
    }
    if (subTab === "models" && selectedFamily) {
      return (
        <ModelProfile
          familyName={selectedFamily}
          variantFullName={selectedVariant}
        />
      );
    }
    return (
      <div style={{
        flex: 1, display: "flex",
        alignItems: "center", justifyContent: "center",
      }}>
        <EmptyState message={t("ollama.selectModel")} />
      </div>
    );
  })();

  return { list, detail };
}
