import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ForecastWorkbenchWindow } from "./forecast-workbench-window";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("./use-forecast-workbench-context", () => ({
  useForecastWorkbenchContext: () => ({
    snapshot: {
      context: {
        session_id: "550e8400-e29b-41d4-a716-446655440000",
        analysis_id: "123e4567-e89b-12d3-a456-426614174000",
        revision: 2,
      },
      draft: { section: "data", revision: 1 },
      analysis_name: "Forecast RSAFS",
    },
    loading: false,
    failed: false,
  }),
}));

vi.mock("./forecast-workbench-model-control", () => ({
  ForecastWorkbenchModelControl: () => <button type="button">Auto</button>,
}));

vi.mock("./forecast-workbench-section", () => ({
  ForecastWorkbenchSectionContent: () => <div>Contenu</div>,
}));

describe("ForecastWorkbenchWindow", () => {
  it("keeps only the analysis name and model selector in the header", () => {
    const { container } = render(<ForecastWorkbenchWindow />);

    const header = container.querySelector(".fcw-header");
    expect(header).not.toBeNull();
    expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent(
      "Forecast RSAFS",
    );
    expect(header?.querySelector(".fcw-kicker")).toBeNull();
    expect(header?.querySelector(".fcw-session")).toBeNull();
    expect(screen.getByRole("button", { name: "Auto" })).toBeInTheDocument();
  });
});
