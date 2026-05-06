import { createLowlight, common } from "lowlight";
import { toHtml } from "hast-util-to-html";
import { languageFromPath } from "./code-language";

const lowlight = createLowlight(common);

const LANG_ALIASES: Record<string, string> = {
  js: "javascript",
  jsx: "javascript",
  ts: "typescript",
  tsx: "typescript",
  toml: "ini",
  html: "xml",
  sh: "bash",
  py: "python",
  rs: "rust",
  yml: "yaml",
};

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export function highlightCode(code: string, language: string): string {
  const resolved = LANG_ALIASES[language] ?? language;
  if (!resolved || !lowlight.registered(resolved)) return escapeHtml(code);
  const tree = lowlight.highlight(resolved, code);
  return toHtml(tree);
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
