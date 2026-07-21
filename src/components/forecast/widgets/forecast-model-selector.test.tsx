import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ForecastModelEntry } from "../forecast-model-meta";
import { ForecastModelSelector } from "./forecast-model-selector";

const MODEL: ForecastModelEntry = {
  id: "chronos-bolt-small",
  provider_id: "amazon",
  family_id: "chronos-bolt",
  display_name: "Chronos Bolt Small",
  params: "21M",
  size_mb: 40,
  ram_mb: 350,
  vram_mb: 120,
  cpu_supported: true,
  gpu_supported: true,
  multivariate: false,
  covariates: false,
  horizon_max: 64,
  frequencies: "D,W,M",
  is_cloud: false,
  installed: true,
  runnable: true,
};

vi.mock("../use-available-forecast-models", () => ({
  useAvailableForecastModels: () => ({ models: [MODEL] }),
}));

vi.mock("@/hooks/use-favorite-models", () => ({
  useFavoriteModels: () => ({
    favorites: [],
    isFavorite: () => false,
    toggle: vi.fn(),
  }),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => ({
      "forecast.selection.label": "Sélection",
      "forecast.selection.manual": "Manuel",
      "forecast.selection.auto": "Auto",
      "agentLocal.modelSearch": "Rechercher un modèle…",
      "forecast.models.families.chronos-bolt": "Chronos Bolt",
    })[key] ?? key,
  }),
}));

afterEach(() => vi.clearAllMocks());

describe("ForecastModelSelector", () => {
  it("blocks changes until the persisted policy is loaded", () => {
    render(
      <ForecastModelSelector
        selectedModelId=""
        selectionMode="manual"
        selectionReady={false}
        onSelectModel={vi.fn()}
        onModeChange={vi.fn()}
      />,
    );

    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("keeps Manuel and Auto visible below the search filter", () => {
    const onModeChange = vi.fn();
    render(
      <ForecastModelSelector
        selectedModelId={MODEL.id}
        selectionMode="auto"
        selectionReady
        onSelectModel={vi.fn()}
        onModeChange={onModeChange}
      />,
    );

    fireEvent.click(screen.getByText("Auto"));
    fireEvent.change(screen.getByPlaceholderText("Rechercher un modèle…"), {
      target: { value: "absent" },
    });

    expect(screen.getByText("Sélection")).toBeVisible();
    expect(screen.getByText("Manuel")).toBeVisible();
    expect(screen.getAllByText("Auto").length).toBeGreaterThan(0);
    fireEvent.click(screen.getByText("Manuel"));
    expect(onModeChange).toHaveBeenCalledWith("manual");
  });

  it("selecting a model delegates the switch back to Manual", () => {
    const onSelectModel = vi.fn();
    render(
      <ForecastModelSelector
        selectedModelId={MODEL.id}
        selectionMode="manual"
        selectionReady
        onSelectModel={onSelectModel}
        onModeChange={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByText(MODEL.display_name));
    fireEvent.click(screen.getAllByText(MODEL.display_name)[1]);

    expect(onSelectModel).toHaveBeenCalledWith(MODEL.id);
  });
});
