import { useTranslation } from "react-i18next";
import { EmptyState } from "@/components/ui/empty-state";
import {
  type ForecastModelEntry,
  type ForecastModelGroup,
} from "../forecast-model-meta";
import { ModelCard } from "./model-card";
import "./forecast-models.css";

interface ForecastModelsProps {
  group: ForecastModelGroup | null;
  onSelectModel: (model: ForecastModelEntry) => void;
  onRefresh: () => void;
}

export function ForecastModels({
  group,
  onSelectModel,
  onRefresh,
}: ForecastModelsProps) {
  const { t } = useTranslation();

  return (
    <div className="fmb-root">
      <div className="fmb-header">
        <span className="fmb-title">
          {group ? t(group.titleKey) : t("forecast.models.title")}
        </span>
      </div>
      <div className="fmb-list">
        {group && group.models.length > 0 ? (
          group.models.map((model) => (
            <ModelCard
              key={model.id}
              model={model}
              onSelect={() => onSelectModel(model)}
              onRefresh={onRefresh}
            />
          ))
        ) : (
          <div className="fmb-empty">
            <EmptyState message={t("forecast.models.noneAvailable")} />
          </div>
        )}
      </div>
    </div>
  );
}
