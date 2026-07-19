import { describe, expect, it } from "vitest";
import {
  getNextThemeChoice,
  getThemeColorScheme,
  isThemeChoice,
  resolveTheme,
  THEME_OPTIONS,
} from "./app-themes";

describe("app themes", () => {
  it("déclare Émeraude nocturne comme une palette sombre", () => {
    expect(THEME_OPTIONS).toContainEqual({
      id: "emerald-night",
      labelKey: "settings.emeraldNight",
      colorScheme: "dark",
    });
    expect(getThemeColorScheme("emerald-night")).toBe("dark");
  });

  it("valide uniquement les choix de thème connus", () => {
    expect(isThemeChoice("emerald-night")).toBe(true);
    expect(isThemeChoice("unknown-theme")).toBe(false);
    expect(isThemeChoice(null)).toBe(false);
  });

  it("résout le mode système selon la préférence du système", () => {
    expect(resolveTheme("system", true)).toBe("dark");
    expect(resolveTheme("system", false)).toBe("light");
    expect(resolveTheme("emerald-night", false)).toBe("emerald-night");
  });

  it("inclut Émeraude nocturne dans le cycle des thèmes", () => {
    expect(getNextThemeChoice("dark")).toBe("emerald-night");
    expect(getNextThemeChoice("emerald-night")).toBe("system");
    expect(getNextThemeChoice("system")).toBe("light");
  });
});
