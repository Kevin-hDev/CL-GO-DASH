import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ModeSelector } from "../mode-selector";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => ({
      "forecast.panelMode.title": "Changer de mode",
      "forecast.panelMode.preview": "Aperçu",
      "forecast.panelMode.forecast": "Forecast",
      "browser.title": "Navigateur",
      "browser.unavailable": "Navigateur sécurisé indisponible",
    }[key] ?? key),
  }),
}));

describe("ModeSelector browser mode", () => {
  it("affiche le navigateur prêt avec un préfixe CSS réservé", () => {
    render(
      <ModeSelector
        mode="preview"
        browserStatus="ready"
        onChange={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button"));

    expect(screen.getByText("Navigateur")).toBeTruthy();
    expect(document.body.querySelector(".asp-mode-menu")).toBeTruthy();
    expect(document.body.querySelector(".ms-menu")).toBeNull();
  });

  it("montre le navigateur indisponible au lieu de le masquer", () => {
    const onChange = vi.fn();
    render(
      <ModeSelector
        mode="preview"
        browserStatus="unavailable"
        onChange={onChange}
      />,
    );

    fireEvent.click(screen.getByRole("button"));

    const browser = screen.getByRole("button", { name: "Navigateur" });
    expect(browser).toBeDisabled();
    expect(browser).toHaveAttribute("title", "Navigateur sécurisé indisponible");
    expect(onChange).not.toHaveBeenCalled();
  });
});
