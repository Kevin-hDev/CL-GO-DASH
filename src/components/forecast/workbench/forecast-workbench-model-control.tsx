import { useForecastSelectionPolicy } from "../model-selection/use-forecast-selection-policy";
import { ForecastModelSelector } from "../widgets/forecast-model-selector";

export function ForecastWorkbenchModelControl() {
  const { policy, selectedModelId, selectModel, setMode, ready } =
    useForecastSelectionPolicy();
  return (
    <ForecastModelSelector
      selectedModelId={selectedModelId}
      selectionMode={policy.mode}
      selectionReady={ready}
      onSelectModel={selectModel}
      onModeChange={setMode}
      align="right"
    />
  );
}
