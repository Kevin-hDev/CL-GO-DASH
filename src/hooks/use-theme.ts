import { useState, useCallback, useEffect } from "react";

type Theme = "dark" | "light";

function getInitialTheme(): Theme {
  const saved = localStorage.getItem("clgo-theme");
  return saved === "light" ? "light" : "dark";
}

export function useTheme() {
  const [theme, setTheme] = useState<Theme>(getInitialTheme);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("clgo-theme", theme);
  }, [theme]);

  const toggle = useCallback(() => {
    setTheme((t) => (t === "dark" ? "light" : "dark"));
  }, []);

  return { theme, toggle } as const;
}
