import { afterEach, describe, expect, it } from "vitest";
import { applyStoredSettings } from "../use-settings";

describe("applyStoredSettings", () => {
  afterEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-code-theme");
    document.documentElement.style.fontSize = "";
    document.documentElement.style.removeProperty("--font-sans");
  });

  it("applique les réglages visuels stockés avant le rendu des settings", () => {
    localStorage.setItem("clgo-font-size", "125");
    localStorage.setItem("clgo-font-family", "menlo");
    localStorage.setItem("clgo-code-theme", "tokyo-night");

    applyStoredSettings();

    expect(document.documentElement.style.fontSize).toBe("22.5px");
    expect(document.documentElement.style.getPropertyValue("--font-sans")).toBe(
      'Menlo, "SF Mono", Consolas, monospace',
    );
    expect(document.documentElement.getAttribute("data-code-theme")).toBe("tokyo-night");
  });

  it("retombe sur les valeurs par défaut si le stockage contient des valeurs invalides", () => {
    localStorage.setItem("clgo-font-size", "999");
    localStorage.setItem("clgo-font-family", "unknown");
    localStorage.setItem("clgo-code-theme", "unknown");

    applyStoredSettings();

    expect(document.documentElement.style.fontSize).toBe("18px");
    expect(document.documentElement.style.getPropertyValue("--font-sans")).toBe(
      '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    );
    expect(document.documentElement.getAttribute("data-code-theme")).toBe("default");
  });
});
