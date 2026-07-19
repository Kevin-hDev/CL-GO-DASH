export const RESOLVED_THEME_OPTIONS = [
  { id: "light", labelKey: "settings.light", colorScheme: "light" },
  { id: "dark", labelKey: "settings.dark", colorScheme: "dark" },
  { id: "emerald-night", labelKey: "settings.emeraldNight", colorScheme: "dark" },
  { id: "cobalt-frost", labelKey: "settings.cobaltFrost", colorScheme: "light" },
] as const;

export const THEME_OPTIONS = [
  ...RESOLVED_THEME_OPTIONS,
  { id: "system", labelKey: "settings.system", colorScheme: "system" },
] as const;

export type ResolvedTheme = (typeof RESOLVED_THEME_OPTIONS)[number]["id"];
export type ThemeChoice = (typeof THEME_OPTIONS)[number]["id"];
export type ThemeColorScheme = "light" | "dark";

export function isThemeChoice(value: string | null): value is ThemeChoice {
  return THEME_OPTIONS.some((option) => option.id === value);
}

export function resolveTheme(choice: ThemeChoice, prefersDark: boolean): ResolvedTheme {
  if (choice !== "system") return choice;
  return prefersDark ? "dark" : "light";
}

export function getThemeColorScheme(theme: ResolvedTheme): ThemeColorScheme {
  return theme === "dark" || theme === "emerald-night" ? "dark" : "light";
}

export function getNextThemeChoice(choice: ThemeChoice): ThemeChoice {
  const currentIndex = THEME_OPTIONS.findIndex((option) => option.id === choice);
  const nextIndex = (currentIndex + 1) % THEME_OPTIONS.length;
  return THEME_OPTIONS[nextIndex].id;
}
