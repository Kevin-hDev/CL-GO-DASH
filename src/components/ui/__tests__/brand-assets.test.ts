import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const root = process.cwd();

function path(relativePath: string): string {
  return resolve(root, relativePath);
}

function fileExists(relativePath: string): boolean {
  // Les chemins sont tous des constantes internes déclarées dans ce test.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  return existsSync(path(relativePath));
}

function readText(relativePath: string): string {
  // Les chemins sont tous des constantes internes déclarées dans ce test.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  return readFileSync(path(relativePath), "utf8");
}

function pngInfo(relativePath: string): { width: number; height: number; hasAlpha: boolean } {
  // Les chemins sont tous des constantes internes déclarées dans ce test.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  const bytes = readFileSync(path(relativePath));
  expect(bytes.subarray(1, 4).toString("ascii")).toBe("PNG");
  return {
    width: bytes.readUInt32BE(16),
    height: bytes.readUInt32BE(20),
    hasAlpha: [4, 6].includes(bytes.readUInt8(25)),
  };
}

function pngSize(relativePath: string): { width: number; height: number } {
  const { width, height } = pngInfo(relativePath);
  return { width, height };
}

describe("assets de marque", () => {
  it("conserve uniquement les deux sources visuelles actives", () => {
    expect(fileExists("public/castor.svg")).toBe(true);
    expect(pngInfo("src/assets/logo.png")).toEqual({
      width: 1024,
      height: 1024,
      hasAlpha: true,
    });

    for (const obsolete of [
      "src/assets/logo-dark.png",
      "src/assets/logo-light.png",
      "src/assets/icone-app.png",
      "public/splash-icon.png",
      "public/splash-icon-light.png",
    ]) {
      expect(fileExists(obsolete)).toBe(false);
    }
  });

  it("colore le castor selon le thème aux tailles prévues", () => {
    const splash = readText("index.html");
    const onboarding = readText("src/components/onboarding/onboarding.css");

    expect(splash).toContain("width: 170px");
    expect(splash).toContain("height: 170px");
    expect(splash).toContain("--splash-mark: #c8c8ce");
    expect(splash).toContain("--splash-mark: #1a1a1a");
    expect(splash).toContain('mask: url("/castor.svg")');
    expect(onboarding).toContain("width: 4.5rem");
    expect(onboarding).toContain("background: var(--ink)");
  });

  it("fournit toutes les icônes desktop requises", () => {
    expect(pngSize("src-tauri/icons/32x32.png")).toEqual({ width: 32, height: 32 });
    expect(pngSize("src-tauri/icons/128x128.png")).toEqual({ width: 128, height: 128 });
    expect(pngSize("src-tauri/icons/128x128@2x.png")).toEqual({ width: 256, height: 256 });
    expect(pngSize("src-tauri/icons/tray.png")).toEqual({ width: 64, height: 64 });
    expect(pngInfo("src-tauri/icons/tray.png").hasAlpha).toBe(true);
    expect(fileExists("src-tauri/icons/icon.icns")).toBe(true);
    expect(fileExists("src-tauri/icons/icon.ico")).toBe(true);
  });
});
