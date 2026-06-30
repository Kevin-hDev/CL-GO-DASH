import { useState, useCallback, useEffect, useMemo } from "react";

export const FONT_SIZES = [100, 112, 125, 137, 150] as const;
export type FontSize = (typeof FONT_SIZES)[number];

/**
 * Base de la taille de police en pixels pour 100%.
 * Le navigateur utilise 16px par défaut ; on monte à 18px (+2px) pour
 * grossir légèrement toutes les polices sans changer les paliers affichés.
 */
const FONT_BASE_PX = 18;

export const FONT_FAMILIES = [
  { id: "system", label: "System Default", value: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif' },
  { id: "jetbrains", label: "JetBrains Mono", value: '"JetBrains Mono", "Fira Code", monospace' },
  { id: "helvetica", label: "Helvetica Neue", value: '"Helvetica Neue", Helvetica, Arial, sans-serif' },
  { id: "menlo", label: "Menlo", value: 'Menlo, "SF Mono", Consolas, monospace' },
  { id: "ui-mono", label: "UI Monospace", value: 'ui-monospace, "SF Mono", "Cascadia Code", monospace' },
  { id: "pacifico", label: "Pacifico", value: '"Pacifico", cursive' },
  { id: "rancho", label: "Rancho", value: '"Rancho", cursive' },
] as const;

export type FontFamilyId = (typeof FONT_FAMILIES)[number]["id"];

/**
 * Thèmes de coloration de code disponibles dans /settings.
 * Chaque thème possède une variante dark + light (sélectionnée
 * automatiquement selon le mode app via l'attribut data-theme).
 */
export const CODE_THEMES = [
  { id: "default", label: "Défaut" },
  { id: "github", label: "GitHub" },
  { id: "one-dark-pro", label: "One Dark Pro" },
  { id: "tokyo-night", label: "Tokyo Night" },
  { id: "catppuccin", label: "Catppuccin" },
] as const;

export type CodeThemeId = (typeof CODE_THEMES)[number]["id"];

function loadFontSize(): FontSize {
  const saved = Number(localStorage.getItem("clgo-font-size"));
  return FONT_SIZES.includes(saved as FontSize) ? (saved as FontSize) : 100;
}

function loadFontFamily(): FontFamilyId {
  const saved = localStorage.getItem("clgo-font-family");
  if (saved && FONT_FAMILIES.some((f) => f.id === saved)) return saved as FontFamilyId;
  return "system";
}

function loadCodeTheme(): CodeThemeId {
  const saved = localStorage.getItem("clgo-code-theme");
  if (saved && CODE_THEMES.some((c) => c.id === saved)) return saved as CodeThemeId;
  return "default";
}

function loadSidebarExpand(): boolean {
  const saved = localStorage.getItem("clgo-sidebar-expand");
  return saved === null ? true : saved === "true";
}

export function useSettings() {
  const [fontSize, setFontSizeState] = useState<FontSize>(loadFontSize);
  const [fontFamilyId, setFontFamilyIdState] = useState<FontFamilyId>(loadFontFamily);
  const [codeThemeId, setCodeThemeIdState] = useState<CodeThemeId>(loadCodeTheme);
  const [sidebarExpand, setSidebarExpandState] = useState(loadSidebarExpand);

  const fontFamily = FONT_FAMILIES.find((f) => f.id === fontFamilyId)!;
  const codeTheme = CODE_THEMES.find((c) => c.id === codeThemeId)!;

  useEffect(() => {
    // Base 18px (au lieu des 16px navigateur) × pourcentage du réglage.
    document.documentElement.style.fontSize = `${(fontSize / 100) * FONT_BASE_PX}px`;
    localStorage.setItem("clgo-font-size", String(fontSize));
  }, [fontSize]);

  useEffect(() => {
    document.documentElement.style.setProperty("--font-sans", fontFamily.value);
    localStorage.setItem("clgo-font-family", fontFamilyId);
  }, [fontFamilyId, fontFamily.value]);

  useEffect(() => {
    document.documentElement.setAttribute("data-code-theme", codeThemeId);
    localStorage.setItem("clgo-code-theme", codeThemeId);
  }, [codeThemeId]);

  const setFontSize = useCallback((size: FontSize) => setFontSizeState(size), []);
  const setFontFamily = useCallback((id: FontFamilyId) => setFontFamilyIdState(id), []);
  const setCodeTheme = useCallback((id: CodeThemeId) => setCodeThemeIdState(id), []);
  const setSidebarExpand = useCallback((v: boolean) => {
    setSidebarExpandState(v);
    localStorage.setItem("clgo-sidebar-expand", String(v));
    window.dispatchEvent(new CustomEvent("clgo-sidebar-expand", { detail: v }));
  }, []);

  const decreaseFont = useCallback(() => {
    setFontSizeState((cur) => {
      const idx = FONT_SIZES.indexOf(cur);
      return idx > 0 ? FONT_SIZES[idx - 1] : cur;
    });
  }, []);

  const increaseFont = useCallback(() => {
    setFontSizeState((cur) => {
      const idx = FONT_SIZES.indexOf(cur);
      return idx < FONT_SIZES.length - 1 ? FONT_SIZES[idx + 1] : cur;
    });
  }, []);

  return useMemo(() => ({
    fontSize, setFontSize, decreaseFont, increaseFont,
    fontFamilyId, fontFamily, setFontFamily,
    codeThemeId, codeTheme, setCodeTheme,
    sidebarExpand, setSidebarExpand,
  }) as const, [
    fontSize,
    setFontSize,
    decreaseFont,
    increaseFont,
    fontFamilyId,
    fontFamily,
    setFontFamily,
    codeThemeId,
    codeTheme,
    setCodeTheme,
    sidebarExpand,
    setSidebarExpand,
  ]);
}
