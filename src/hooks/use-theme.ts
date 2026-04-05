import { useState, useCallback, useEffect } from "react";

export type Theme = "light" | "dark" | "orange";

const THEMES: Theme[] = ["light", "dark", "orange"];

function getInitialTheme(): Theme {
  const saved = localStorage.getItem("clgo-theme");
  if (saved && THEMES.includes(saved as Theme)) return saved as Theme;
  return "dark";
}

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(getInitialTheme);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("clgo-theme", theme);
  }, [theme]);

  const setTheme = useCallback((t: Theme) => {
    setThemeState(t);
  }, []);

  const toggle = useCallback(() => {
    setThemeState((t) => {
      const idx = THEMES.indexOf(t);
      return THEMES[(idx + 1) % THEMES.length];
    });
  }, []);

  return { theme, setTheme, toggle } as const;
}
