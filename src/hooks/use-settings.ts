import { useState, useCallback, useEffect } from "react";

export const FONT_SIZES = [100, 112, 125, 137, 150] as const;
export type FontSize = (typeof FONT_SIZES)[number];

export const FONT_FAMILIES = [
  { id: "system", label: "System Default", value: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif' },
  { id: "sf-pro", label: "SF Pro", value: '"SF Pro Display", "SF Pro", -apple-system, sans-serif' },
  { id: "helvetica", label: "Helvetica Neue", value: '"Helvetica Neue", Helvetica, Arial, sans-serif' },
  { id: "menlo", label: "Menlo", value: 'Menlo, "SF Mono", Consolas, monospace' },
  { id: "georgia", label: "Georgia", value: 'Georgia, "Times New Roman", serif' },
] as const;

export type FontFamilyId = (typeof FONT_FAMILIES)[number]["id"];

function loadFontSize(): FontSize {
  const saved = Number(localStorage.getItem("clgo-font-size"));
  return FONT_SIZES.includes(saved as FontSize) ? (saved as FontSize) : 100;
}

function loadFontFamily(): FontFamilyId {
  const saved = localStorage.getItem("clgo-font-family");
  if (saved && FONT_FAMILIES.some((f) => f.id === saved)) return saved as FontFamilyId;
  return "system";
}

export function useSettings() {
  const [fontSize, setFontSizeState] = useState<FontSize>(loadFontSize);
  const [fontFamilyId, setFontFamilyIdState] = useState<FontFamilyId>(loadFontFamily);

  const fontFamily = FONT_FAMILIES.find((f) => f.id === fontFamilyId)!;

  useEffect(() => {
    document.documentElement.style.fontSize = `${fontSize}%`;
    localStorage.setItem("clgo-font-size", String(fontSize));
  }, [fontSize]);

  useEffect(() => {
    document.documentElement.style.setProperty("--font-sans", fontFamily.value);
    localStorage.setItem("clgo-font-family", fontFamilyId);
  }, [fontFamilyId, fontFamily.value]);

  const setFontSize = useCallback((size: FontSize) => setFontSizeState(size), []);
  const setFontFamily = useCallback((id: FontFamilyId) => setFontFamilyIdState(id), []);

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

  return {
    fontSize, setFontSize, decreaseFont, increaseFont,
    fontFamilyId, fontFamily, setFontFamily,
  } as const;
}
