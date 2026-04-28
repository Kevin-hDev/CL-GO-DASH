import { useState, useCallback, useEffect } from "react";

export type ThemeChoice = "light" | "dark" | "system";
export type Theme = "light" | "dark";

function getSystemTheme(): Theme {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function getInitialChoice(): ThemeChoice {
  const saved = localStorage.getItem("clgo-theme");
  if (saved === "light" || saved === "dark" || saved === "system") return saved;
  return "system";
}

function resolveTheme(choice: ThemeChoice): Theme {
  return choice === "system" ? getSystemTheme() : choice;
}

export function useTheme() {
  const [choice, setChoiceState] = useState<ThemeChoice>(getInitialChoice);
  const [resolved, setResolved] = useState<Theme>(() => resolveTheme(getInitialChoice()));

  useEffect(() => {
    const r = resolveTheme(choice);
    setResolved(r);
    document.documentElement.setAttribute("data-theme", r);
    localStorage.setItem("clgo-theme", choice);
  }, [choice]);

  useEffect(() => {
    if (choice !== "system") return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => {
      const r = getSystemTheme();
      setResolved(r);
      document.documentElement.setAttribute("data-theme", r);
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [choice]);

  const setTheme = useCallback((t: ThemeChoice) => {
    setChoiceState(t);
  }, []);

  const toggle = useCallback(() => {
    setChoiceState((c) => {
      if (c === "light") return "dark";
      if (c === "dark") return "system";
      return "light";
    });
  }, []);

  return { theme: resolved, choice, setTheme, toggle } as const;
}
