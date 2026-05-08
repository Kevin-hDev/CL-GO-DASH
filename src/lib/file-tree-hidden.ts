const HIDDEN_ENTRIES = new Set([
  ".git",
  ".DS_Store",
  ".next",
  ".turbo",
  "__pycache__",
  "dist",
  "target",
  "build",
  ".cache",
]);

export function isHiddenEntry(name: string): boolean {
  return HIDDEN_ENTRIES.has(name);
}
