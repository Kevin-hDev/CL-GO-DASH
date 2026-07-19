import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const darkCss = readFileSync("src/styles/themes/dark.css", "utf8");
const emeraldCss = readFileSync("src/styles/themes/emerald-night.css", "utf8");

function tokenNames(css: string): string[] {
  return [...css.matchAll(/(--[a-z0-9-]+)\s*:/g)]
    .map((match) => match[1])
    .sort();
}

function rgb(hex: string): [number, number, number] {
  const normalized = hex.replace("#", "");
  return [0, 2, 4].map((offset) => Number.parseInt(normalized.slice(offset, offset + 2), 16)) as [number, number, number];
}

function luminance(hex: string): number {
  const channels = rgb(hex).map((channel) => {
    const value = channel / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrast(first: string, second: string): number {
  const light = Math.max(luminance(first), luminance(second));
  const dark = Math.min(luminance(first), luminance(second));
  return (light + 0.05) / (dark + 0.05);
}

describe("Emerald Night palette", () => {
  it("définit tous les tokens du thème sombre", () => {
    expect(tokenNames(emeraldCss)).toEqual(tokenNames(darkCss));
  });

  it("maintient des contrastes lisibles pour les textes et actions", () => {
    expect(contrast("#f1e2c3", "#031f1c")).toBeGreaterThanOrEqual(7);
    expect(contrast("#bdc7b4", "#031f1c")).toBeGreaterThanOrEqual(4.5);
    expect(contrast("#4ee07a", "#062318")).toBeGreaterThanOrEqual(4.5);
  });

  it("conserve un orange distinct et lisible pour les anciennes lignes", () => {
    expect(emeraldCss).toContain("--diff-old: #f97316;");
    expect(contrast("#f97316", "#031f1c")).toBeGreaterThanOrEqual(4.5);
  });
});
