import { createLowlight, common } from "lowlight";
import { toHtml } from "hast-util-to-html";
import { languageFromPath } from "./code-language";

const lowlight = createLowlight(common);

const LANG_ALIASES: Record<string, string> = {
  jsx: "javascript",
  tsx: "typescript",
  toml: "ini",
  html: "xml",
};

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export function highlightLines(code: string, path: string): string[] {
  const lang = languageFromPath(path);
  const resolved = LANG_ALIASES[lang] ?? lang;
  const registered = lowlight.registered(resolved);

  const lines = code.split("\n");
  if (!registered) return lines.map(escapeHtml);

  const tree = lowlight.highlight(resolved, code);
  const html = toHtml(tree);
  return html.split("\n");
}
