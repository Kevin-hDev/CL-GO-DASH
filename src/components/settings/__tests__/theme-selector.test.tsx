import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ThemeSelector } from "../theme-selector";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key === "settings.emeraldNight" ? "Émeraude nocturne" : key,
  }),
}));

describe("ThemeSelector", () => {
  it("affiche et sélectionne Émeraude nocturne", () => {
    const onChange = vi.fn();
    const { container } = render(<ThemeSelector value="dark" onChange={onChange} />);

    fireEvent.click(screen.getByRole("button", { name: "Émeraude nocturne" }));

    expect(onChange).toHaveBeenCalledWith("emerald-night");
    expect(container.querySelector('[data-palette="emerald-night"]')).toHaveAttribute("data-theme", "dark");
  });
});
