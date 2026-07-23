import { describe, expect, it } from "vitest";
import { buildForecastLayerGroups } from "./forecast-layer-matrix";

describe("buildForecastLayerGroups", () => {
  it("keeps a generated ensemble in the comparison layers", () => {
    const groups = buildForecastLayerGroups({
      scenarioLayers: [],
      comparisonLayers: [{ id: "scenario-ensemble", label: "Ensemble", interactive: true }],
      covariateNames: [],
    }, (key) => key);

    const comparisons = groups.find((group) => group.id === "comparisons");
    expect(comparisons?.items).toEqual([
      { id: "scenario-ensemble", label: "Ensemble", interactive: true },
    ]);
  });
});
